//! Delta-encoded vitals.
//!
//! Each wristband is one *stream*. Every `kf_interval` samples (or on demand)
//! the encoder sends a full **keyframe**; in between it sends **deltas against
//! that keyframe** (not against the previous sample). Losing a delta therefore
//! never corrupts later ones; losing a keyframe only blanks that stream until
//! the next keyframe, and the receiver can ask for one early with a
//! `Nack(NeedKeyframe)`. This is what makes the scheme safe on lossy,
//! intermittent links without making vitals "reliable" (they are latest-wins).

use super::wire::{Reader, WireError, Writer};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// One wristband sample in fixed-point units (no floats on the wire).
/// A value of 0 for `hr`, `spo2` or `resp` means "no reading".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct VitalsSample {
    /// pulse, beats per minute (0 = no pulse found)
    pub hr: u16,
    /// blood oxygen saturation, % (0 = no reading)
    pub spo2: u8,
    /// breathing estimate, breaths per minute
    pub resp: u8,
    /// skin temperature, tenths of °C
    pub skin_c10: i16,
    /// electrodermal activity, hundredths of µS
    pub eda_c: u16,
    /// motion / activity index 0..=255
    pub motion: u8,
    /// optical signal quality 0..=100
    pub quality: u8,
    /// sensor base battery %
    pub base_batt: u8,
    /// interface (screen) battery %
    pub screen_batt: u8,
    /// bit flags, see `vflag`
    pub flags: u8,
}

pub mod vflag {
    /// The suit is between the two parts; the base runs on its own battery.
    pub const SUIT: u8 = 1;
    /// A 30 s on-demand ECG is being recorded on the interface pads.
    pub const ECG: u8 = 2;
    /// The wearer did not answer the wristband's "are you OK?" prompt.
    pub const NO_ANSWER: u8 = 4;
    /// Base ↔ interface LoRa link is up.
    pub const LORA: u8 = 8;
}

const NFIELDS: usize = 10;

impl VitalsSample {
    fn fields(&self) -> [i64; NFIELDS] {
        [
            self.hr as i64,
            self.spo2 as i64,
            self.resp as i64,
            self.skin_c10 as i64,
            self.eda_c as i64,
            self.motion as i64,
            self.quality as i64,
            self.base_batt as i64,
            self.screen_batt as i64,
            self.flags as i64,
        ]
    }
    fn from_fields(f: [i64; NFIELDS]) -> Self {
        VitalsSample {
            hr: f[0].clamp(0, u16::MAX as i64) as u16,
            spo2: f[1].clamp(0, 255) as u8,
            resp: f[2].clamp(0, 255) as u8,
            skin_c10: f[3].clamp(i16::MIN as i64, i16::MAX as i64) as i16,
            eda_c: f[4].clamp(0, u16::MAX as i64) as u16,
            motion: f[5].clamp(0, 255) as u8,
            quality: f[6].clamp(0, 255) as u8,
            base_batt: f[7].clamp(0, 255) as u8,
            screen_batt: f[8].clamp(0, 255) as u8,
            flags: f[9].clamp(0, 255) as u8,
        }
    }
    pub fn skin_c(&self) -> f32 {
        self.skin_c10 as f32 / 10.0
    }
    pub fn has(&self, flag: u8) -> bool {
        self.flags & flag != 0
    }
}

/// A decoded sample with its stream id and timestamp (seconds).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamSample {
    pub stream: u32,
    pub ts_s: u64,
    pub sample: VitalsSample,
    pub keyframe: bool,
}

#[derive(Debug, Clone)]
struct EncState {
    kf: VitalsSample,
    kf_ts: u64,
    kf_id: u8,
    since_kf: u32,
}

/// Sender side. One encoder handles many streams.
#[derive(Debug, Clone)]
pub struct DeltaEncoder {
    pub kf_interval: u32,
    streams: HashMap<u32, EncState>,
}

impl DeltaEncoder {
    pub fn new(kf_interval: u32) -> Self {
        Self { kf_interval: kf_interval.max(1), streams: HashMap::new() }
    }
    /// Make the next sample of `stream` a keyframe (after a NeedKeyframe nack).
    pub fn force_keyframe(&mut self, stream: u32) {
        self.streams.remove(&stream);
    }
    pub fn encode(&mut self, stream: u32, ts_s: u64, s: &VitalsSample) -> Vec<u8> {
        let mut w = Writer::new();
        let need_kf = match self.streams.get(&stream) {
            None => true,
            Some(st) => st.since_kf + 1 >= self.kf_interval || ts_s < st.kf_ts || ts_s - st.kf_ts > 120,
        };
        if need_kf {
            let kf_id = self.streams.get(&stream).map(|s| s.kf_id.wrapping_add(1) & 0x7f).unwrap_or(0);
            w.u8(0x80 | kf_id).uvar(stream as u64).uvar(ts_s);
            for v in s.fields() {
                w.ivar(v);
            }
            self.streams.insert(stream, EncState { kf: *s, kf_ts: ts_s, kf_id, since_kf: 0 });
        } else {
            let st = self.streams.get_mut(&stream).expect("stream exists");
            st.since_kf += 1;
            w.u8(st.kf_id).uvar(stream as u64).uvar(ts_s - st.kf_ts);
            let (a, b) = (st.kf.fields(), s.fields());
            let mut mask: u64 = 0;
            for i in 0..NFIELDS {
                if a[i] != b[i] {
                    mask |= 1 << i;
                }
            }
            w.uvar(mask);
            for i in 0..NFIELDS {
                if mask & (1 << i) != 0 {
                    w.ivar(b[i] - a[i]);
                }
            }
        }
        w.finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeltaError {
    Wire(WireError),
    /// The keyframe this delta refers to never arrived: ask for a new one.
    NeedKeyframe(u32),
}
impl From<WireError> for DeltaError {
    fn from(e: WireError) -> Self {
        DeltaError::Wire(e)
    }
}

/// Receiver side.
#[derive(Debug, Default, Clone)]
pub struct DeltaDecoder {
    /// stream → (kf_id, keyframe, kf timestamp)
    streams: HashMap<u32, (u8, VitalsSample, u64)>,
}

impl DeltaDecoder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn decode(&mut self, b: &[u8]) -> Result<StreamSample, DeltaError> {
        let mut r = Reader::new(b);
        let hdr = r.u8()?;
        let keyframe = hdr & 0x80 != 0;
        let kf_id = hdr & 0x7f;
        let stream = r.uvar()? as u32;
        if keyframe {
            let ts_s = r.uvar()?;
            let mut f = [0i64; NFIELDS];
            for v in f.iter_mut() {
                *v = r.ivar()?;
            }
            let s = VitalsSample::from_fields(f);
            self.streams.insert(stream, (kf_id, s, ts_s));
            Ok(StreamSample { stream, ts_s, sample: s, keyframe: true })
        } else {
            let (id, kf, kf_ts) = *self.streams.get(&stream).ok_or(DeltaError::NeedKeyframe(stream))?;
            if id != kf_id {
                return Err(DeltaError::NeedKeyframe(stream));
            }
            let ts_s = kf_ts + r.uvar()?;
            let mask = r.uvar()?;
            let mut f = kf.fields();
            for (i, v) in f.iter_mut().enumerate() {
                if mask & (1 << i) != 0 {
                    *v += r.ivar()?;
                }
            }
            Ok(StreamSample { stream, ts_s, sample: VitalsSample::from_fields(f), keyframe: false })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(hr: u16) -> VitalsSample {
        VitalsSample { hr, spo2: 98, resp: 14, skin_c10: 344, eda_c: 210, motion: 12, quality: 95, base_batt: 66, screen_batt: 71, flags: vflag::LORA }
    }

    #[test]
    fn keyframe_then_small_deltas() {
        let mut enc = DeltaEncoder::new(10);
        let mut dec = DeltaDecoder::new();
        let kf = enc.encode(3, 1_000, &sample(72));
        let d1 = enc.encode(3, 1_001, &sample(73));
        let d2 = enc.encode(3, 1_002, &sample(72));
        assert!(kf.len() <= 24, "keyframe {} bytes", kf.len());
        assert!(d1.len() <= 6, "delta {} bytes", d1.len());
        assert!(d2.len() <= 5, "unchanged delta {} bytes", d2.len());
        assert_eq!(dec.decode(&kf).unwrap().sample, sample(72));
        let s1 = dec.decode(&d1).unwrap();
        assert_eq!((s1.sample, s1.ts_s, s1.keyframe), (sample(73), 1_001, false));
        assert_eq!(dec.decode(&d2).unwrap().sample, sample(72));
        let json = serde_json::to_vec(&sample(73)).unwrap();
        assert!(json.len() > 8 * d1.len(), "json {} vs delta {}", json.len(), d1.len());
    }

    #[test]
    fn lost_delta_does_not_corrupt_next() {
        let mut enc = DeltaEncoder::new(10);
        let mut dec = DeltaDecoder::new();
        dec.decode(&enc.encode(1, 0, &sample(70))).unwrap();
        let _lost = enc.encode(1, 1, &sample(90));
        let d = enc.encode(1, 2, &sample(71));
        assert_eq!(dec.decode(&d).unwrap().sample, sample(71));
    }

    #[test]
    fn lost_keyframe_asks_for_a_new_one() {
        let mut enc = DeltaEncoder::new(3);
        let mut dec = DeltaDecoder::new();
        dec.decode(&enc.encode(5, 0, &sample(70))).unwrap();
        enc.encode(5, 1, &sample(70));
        enc.encode(5, 2, &sample(70));
        let _lost_kf = enc.encode(5, 3, &sample(70)); // interval 3 → keyframe
        let d = enc.encode(5, 4, &sample(75));
        assert_eq!(dec.decode(&d), Err(DeltaError::NeedKeyframe(5)));
        enc.force_keyframe(5);
        let kf = enc.encode(5, 5, &sample(76));
        assert!(dec.decode(&kf).unwrap().keyframe);
    }

    #[test]
    fn no_pulse_and_negative_changes_roundtrip() {
        let mut enc = DeltaEncoder::new(10);
        let mut dec = DeltaDecoder::new();
        dec.decode(&enc.encode(9, 10, &sample(80))).unwrap();
        let mut arrest = sample(0);
        arrest.spo2 = 0;
        arrest.motion = 0;
        arrest.skin_c10 = 329;
        arrest.flags |= vflag::NO_ANSWER;
        assert_eq!(dec.decode(&enc.encode(9, 11, &arrest)).unwrap().sample, arrest);
    }
}
