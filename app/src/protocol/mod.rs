//! **Eva Link**: the super-light protocol between wristbands, PWA clients,
//! the ship hub and the ground. Spec: `docs/app/PROTOCOL.md`.
//!
//! Layers, bottom up:
//! * [`wire`]: varints, zigzag, strings, CRC-16
//! * [`frame`]: versioned frame header, message types, priority classes
//! * [`payload`]: typed payloads (hello, ack, alarm, record, presence…)
//! * [`delta`]: keyframe + delta encoding of vitals
//! * [`queue`]: store-and-forward priority queue, ACK/NACK, retransmit, RTT
//! * [`profile`]: simulated link profiles (LAN, LoRa-like, near-space, outage)
//! * [`stats`]: byte accounting per message type
//! * [`Endpoint`] (here): glues the above into one side of a link.

pub mod delta;
pub mod frame;
pub mod payload;
pub mod profile;
pub mod queue;
pub mod stats;
pub mod wire;

use delta::{DeltaDecoder, DeltaEncoder, DeltaError, StreamSample, VitalsSample};
use frame::{Frame, MsgType, Priority};
use payload::NackReason;
use queue::{Receiver, SendQueue};
use stats::ByteStats;

/// Something the application must handle after a datagram arrived.
#[derive(Debug, Clone, PartialEq)]
pub enum Delivered {
    /// A new (deduplicated) application frame: alarm, guidance, record, text, hello, presence…
    Frame(Frame),
    /// A decoded vitals sample.
    Vitals { src: u32, sample: StreamSample },
    /// Our reliable frames that the peer has now acknowledged.
    Acked(Vec<u32>),
    /// A fresh RTT measurement from a ping/pong.
    Rtt(f64),
    /// A frame that failed to decode (counted, then ignored).
    Corrupt,
}

/// One side of an Eva Link: send queue + receive state + vitals codecs.
#[derive(Debug, Clone)]
pub struct Endpoint {
    pub queue: SendQueue,
    pub rx: Receiver,
    pub enc: DeltaEncoder,
    pub dec: DeltaDecoder,
    pub received: ByteStats,
    pub corrupt: u64,
    control: Vec<Frame>,
    /// time-to-live given to queued vitals frames
    pub vitals_ttl_ms: u64,
}

impl Endpoint {
    pub fn new(node_id: u32) -> Self {
        Self {
            queue: SendQueue::new(node_id),
            rx: Receiver::default(),
            enc: DeltaEncoder::new(10),
            dec: DeltaDecoder::new(),
            received: ByteStats::default(),
            corrupt: 0,
            control: Vec::new(),
            vitals_ttl_ms: 30_000,
        }
    }
    pub fn with_crc(mut self, crc: bool) -> Self {
        self.queue.crc = crc;
        self
    }

    /// Queue one vitals sample for `stream` (delta-encoded, latest-wins).
    pub fn send_vitals(&mut self, now_ms: u64, stream: u32, ts_s: u64, s: &VitalsSample) -> u32 {
        self.queue.purge_stale(Priority::Vitals, now_ms);
        let payload = self.enc.encode(stream, ts_s, s);
        self.queue.push_with(MsgType::Vitals, Priority::Vitals, payload, None, Some(now_ms + self.vitals_ttl_ms))
    }
    /// Queue any message with its default class.
    pub fn send(&mut self, kind: MsgType, payload: Vec<u8>, dst: Option<u32>) -> u32 {
        self.queue.push(kind, payload, dst)
    }
    /// Send a ping now (bypasses the queue).
    pub fn ping(&mut self, now_ms: u64) {
        let f = self.queue.control(MsgType::Ping, payload::encode_time(now_ms));
        self.control.push(f);
    }

    /// Datagrams to transmit now: pending control frames (acks, pongs) first,
    /// then queued frames by priority, packed up to `mtu` bytes each.
    pub fn outgoing(&mut self, now_ms: u64, budget: usize, mtu: usize) -> Vec<Vec<u8>> {
        let acks = self.rx.take_acks();
        if !acks.is_empty() {
            for chunk in acks.chunks(64) {
                let f = self.queue.control(MsgType::Ack, payload::encode_ack(chunk));
                self.control.push(f);
            }
        }
        let mut frames: Vec<Frame> = std::mem::take(&mut self.control);
        let used: usize = frames.iter().map(|f| f.wire_len()).sum();
        frames.extend(self.queue.poll(now_ms, budget.saturating_sub(used)));
        pack(&frames, mtu)
    }

    /// Feed one received datagram; returns what the application must handle.
    pub fn incoming(&mut self, now_ms: u64, datagram: &[u8]) -> Vec<Delivered> {
        let mut out = Vec::new();
        let frames = match frame::decode_batch(datagram) {
            Ok(f) => f,
            Err(_) => {
                self.corrupt += 1;
                return vec![Delivered::Corrupt];
            }
        };
        for f in frames {
            self.received.record(&f, f.wire_len());
            match f.kind {
                MsgType::Ack => {
                    if let Ok(seqs) = payload::decode_ack(&f.payload) {
                        let done = self.queue.on_ack(&seqs, now_ms);
                        if !done.is_empty() {
                            out.push(Delivered::Acked(done));
                        }
                    }
                }
                MsgType::Nack => {
                    if let Ok((seq, reason, stream)) = payload::decode_nack(&f.payload) {
                        match reason {
                            NackReason::NeedKeyframe => self.enc.force_keyframe(stream),
                            NackReason::Malformed => self.queue.on_nack(seq, now_ms),
                        }
                    }
                }
                MsgType::Ping => {
                    let pong = self.queue.control(MsgType::Pong, f.payload.clone());
                    self.control.push(pong);
                }
                MsgType::Pong => {
                    if let Ok(t) = payload::decode_time(&f.payload) {
                        let rtt = now_ms.saturating_sub(t) as f64;
                        self.queue.rtt.sample(rtt);
                        out.push(Delivered::Rtt(rtt));
                    }
                }
                MsgType::Vitals => match self.dec.decode(&f.payload) {
                    Ok(sample) => out.push(Delivered::Vitals { src: f.src, sample }),
                    Err(DeltaError::NeedKeyframe(stream)) => {
                        let n = self.queue.control(MsgType::Nack, payload::encode_nack(f.seq, NackReason::NeedKeyframe, stream));
                        self.control.push(n);
                    }
                    Err(DeltaError::Wire(_)) => self.corrupt += 1,
                },
                _ => {
                    if self.rx.accept(&f) {
                        out.push(Delivered::Frame(f));
                    }
                }
            }
        }
        out
    }
}

/// Pack frames into datagrams of at most `mtu` bytes (a frame bigger than the
/// MTU travels alone and the link decides whether it survives).
pub fn pack(frames: &[Frame], mtu: usize) -> Vec<Vec<u8>> {
    let mut out: Vec<Vec<u8>> = Vec::new();
    let mut cur: Vec<u8> = Vec::new();
    for f in frames {
        let b = f.encode();
        if !cur.is_empty() && cur.len() + b.len() > mtu {
            out.push(std::mem::take(&mut cur));
        }
        cur.extend_from_slice(&b);
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// Byte sizes of typical messages, for docs and `eva bench`.
pub fn message_size_table() -> Vec<(&'static str, usize, usize)> {
    use payload::*;
    let mut enc = DeltaEncoder::new(10);
    let v = VitalsSample { hr: 72, spo2: 98, resp: 14, skin_c10: 344, eda_c: 210, motion: 12, quality: 95, base_batt: 66, screen_batt: 71, flags: delta::vflag::LORA };
    let kf = enc.encode(4, 1_731_650_000, &v);
    let mut v2 = v;
    v2.hr = 74;
    v2.motion = 15;
    let d = enc.encode(4, 1_731_650_001, &v2);
    let json = serde_json::to_vec(&v2).map(|j| j.len()).unwrap_or(0);
    let rows: Vec<(&'static str, Frame)> = vec![
        ("ping", Frame::new(MsgType::Ping, 120, 3, encode_time(1_731_650_000_123))),
        ("ack (10 seqs)", Frame::new(MsgType::Ack, 121, 0, encode_ack(&(500..510).collect::<Vec<_>>()))),
        ("vitals keyframe", Frame::new(MsgType::Vitals, 122, 3, kf)),
        ("vitals delta", Frame::new(MsgType::Vitals, 123, 3, d)),
        ("alarm: no pulse", Frame::new(MsgType::Alarm, 124, 3, Alarm { subject: "4B".into(), code: alarm_code::NO_PULSE, text: String::new() }.encode())),
        ("guidance line", Frame::new(MsgType::Guidance, 125, 0, Text { from: "Eva".into(), to: "3B".into(), text: "Push hard and fast in the middle of the chest".into() }.encode())),
        (
            "record (chain, unsigned)",
            Frame::new(MsgType::Record, 126, 1, RecordMsg { chain_seq: 812, ts_ms: 1_731_650_000_000, kind: "alarm".into(), subject: "4B".into(), hash8: [0xab; 8], body: br#"{"code":"no_pulse"}"#.to_vec(), sig: vec![] }.encode()).with_crc(),
        ),
        (
            "record (chain, Ed25519 signed)",
            Frame::new(MsgType::Record, 127, 1, RecordMsg { chain_seq: 812, ts_ms: 1_731_650_000_000, kind: "alarm".into(), subject: "4B".into(), hash8: [0xab; 8], body: br#"{"code":"no_pulse"}"#.to_vec(), sig: vec![0xcd; 64] }.encode()).with_crc(),
        ),
    ];
    let mut out: Vec<(&'static str, usize, usize)> = rows.into_iter().map(|(n, f)| (n, f.wire_len(), f.payload.len())).collect();
    out.push(("vitals as JSON (for comparison)", json, json));
    out
}

#[cfg(test)]
mod tests {
    use super::profile::{LinkProfile, LinkSim};
    use super::*;

    /// Ship → ground over the near-space profile, with the return path for
    /// acks over the same profile. Starts inside an outage window: everything
    /// queues, then flows when contact returns; every record is acknowledged
    /// exactly once and the alarm arrives before the backlog of records.
    #[test]
    fn near_space_store_and_forward_end_to_end() {
        let profile = LinkProfile::near_space();
        let mut up = LinkSim::new(profile.clone(), 11);
        let mut down = LinkSim::new(profile.clone(), 12);
        let mut ship = Endpoint::new(1).with_crc(true);
        let mut ground = Endpoint::new(2).with_crc(true);

        let mut record_seqs = Vec::new();
        for i in 0..40u64 {
            let body = format!(r#"{{"i":{i}}}"#).into_bytes();
            let r = payload::RecordMsg { chain_seq: i, ts_ms: i, kind: "log".into(), subject: "ship".into(), hash8: [0; 8], body, sig: vec![] };
            record_seqs.push(ship.send(MsgType::Record, r.encode(), None));
        }
        let alarm = ship.send(MsgType::Alarm, payload::Alarm { subject: "4B".into(), code: payload::alarm_code::NO_PULSE, text: String::new() }.encode(), None);

        let mut acked = Vec::new();
        let mut delivered_kinds = Vec::new();
        let mut first_datagram: Option<Vec<u8>> = None;
        let tick = 100;
        let mut now = 0;
        while now < 240_000 && acked.len() < 41 {
            if profile.is_up(now) {
                for d in ship.outgoing(now, profile.budget(tick), profile.mtu) {
                    first_datagram.get_or_insert_with(|| d.clone());
                    up.send(now, d);
                }
                for d in ground.outgoing(now, profile.budget(tick), profile.mtu) {
                    down.send(now, d);
                }
            }
            for d in up.recv(now) {
                for ev in ground.incoming(now, &d) {
                    if let Delivered::Frame(f) = ev {
                        delivered_kinds.push((f.kind, f.seq));
                    }
                }
            }
            for d in down.recv(now) {
                for ev in ship.incoming(now, &d) {
                    if let Delivered::Acked(s) = ev {
                        acked.extend(s);
                    }
                }
            }
            now += tick;
        }
        assert!(now > 20_000, "nothing could flow during the initial outage");
        acked.sort_unstable();
        let mut want = record_seqs.clone();
        want.push(alarm);
        want.sort_unstable();
        assert_eq!(acked, want, "every reliable frame acknowledged exactly once");
        let first = frame::decode_batch(&first_datagram.expect("sent something")).unwrap();
        assert_eq!(first[0].kind, MsgType::Alarm, "the alarm leaves first, ahead of the backlog");
        let uniq: std::collections::HashSet<_> = delivered_kinds.iter().map(|k| k.1).collect();
        assert_eq!(uniq.len(), delivered_kinds.len(), "no duplicates delivered to the app");
        assert!(ship.queue.is_idle());
    }

    #[test]
    fn vitals_flow_and_ping_measures_rtt() {
        let p = LinkProfile { loss: 0.0, jitter_ms: 0, ..LinkProfile::lan() };
        let mut a = Endpoint::new(1);
        let mut b = Endpoint::new(0);
        let mut ab = LinkSim::new(p.clone(), 1);
        let mut ba = LinkSim::new(p.clone(), 2);
        let s = VitalsSample { hr: 70, spo2: 98, ..Default::default() };
        a.send_vitals(0, 4, 100, &s);
        a.ping(0);
        let mut got = None;
        let mut rtt = None;
        for now in 0..50 {
            for d in a.outgoing(now, 10_000, 1_500) {
                ab.send(now, d);
            }
            for d in b.outgoing(now, 10_000, 1_500) {
                ba.send(now, d);
            }
            for d in ab.recv(now) {
                for ev in b.incoming(now, &d) {
                    if let Delivered::Vitals { sample, .. } = ev {
                        got = Some(sample.sample);
                    }
                }
            }
            for d in ba.recv(now) {
                for ev in a.incoming(now, &d) {
                    if let Delivered::Rtt(r) = ev {
                        rtt = Some(r);
                    }
                }
            }
        }
        assert_eq!(got, Some(s));
        let r = rtt.expect("pong arrived");
        assert!((3.0..=10.0).contains(&r), "LAN rtt {r}");
    }

    #[test]
    fn size_table_is_small() {
        for (name, bytes, _) in message_size_table() {
            if name.contains("signed)") {
                assert!(bytes <= 128, "{name} is {bytes} bytes");
            } else if !name.contains("JSON") {
                assert!(bytes <= 64, "{name} is {bytes} bytes");
            }
        }
    }
}
