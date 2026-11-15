//! Typed payloads carried inside frames. Every encoder here is mirrored by a
//! test so the byte layout documented in PROTOCOL.md cannot drift silently.

use super::wire::{Reader, WireError, Writer};
use serde::Serialize;

/// What kind of node sits at the other end of a link.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
#[repr(u8)]
pub enum Role {
    Hub = 0,
    Ship = 1,
    Ground = 2,
    App = 3,
    Dash = 4,
    Band = 5,
    /// the ship's medical station (12 compartments, Eva screen, red button)
    Station = 6,
}

impl Role {
    pub fn from_u8(v: u8) -> Result<Self, WireError> {
        Ok(match v {
            0 => Role::Hub,
            1 => Role::Ship,
            2 => Role::Ground,
            3 => Role::App,
            4 => Role::Dash,
            5 => Role::Band,
            6 => Role::Station,
            _ => return Err(WireError::Invalid("role")),
        })
    }
    pub fn name(self) -> &'static str {
        match self {
            Role::Hub => "hub",
            Role::Ship => "ship",
            Role::Ground => "ground",
            Role::App => "app",
            Role::Dash => "dash",
            Role::Band => "band",
            Role::Station => "station",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hello {
    pub role: Role,
    pub name: String,
    pub client_ms: u64,
    /// optional trailing field: the sender's Ed25519 public key (32 bytes).
    /// Ships send it so the ground can check record signatures; old
    /// decoders ignore it, so the field is backward compatible.
    pub pubkey: Vec<u8>,
}
impl Hello {
    pub fn new(role: Role, name: impl Into<String>, client_ms: u64) -> Self {
        Hello { role, name: name.into(), client_ms, pubkey: Vec::new() }
    }
    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        w.u8(self.role as u8).str(&self.name).uvar(self.client_ms);
        if !self.pubkey.is_empty() {
            w.bytes(&self.pubkey);
        }
        w.finish()
    }
    pub fn decode(b: &[u8]) -> Result<Self, WireError> {
        let mut r = Reader::new(b);
        let role = Role::from_u8(r.u8()?)?;
        let name = r.str()?.to_string();
        let client_ms = r.uvar()?;
        let pubkey = if r.is_empty() { Vec::new() } else { r.bytes()?.to_vec() };
        if !pubkey.is_empty() && pubkey.len() != 32 {
            return Err(WireError::Invalid("public key length"));
        }
        Ok(Hello { role, name, client_ms, pubkey })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Welcome {
    pub node_id: u32,
    pub server_ms: u64,
}
impl Welcome {
    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        w.uvar(self.node_id as u64).uvar(self.server_ms);
        w.finish()
    }
    pub fn decode(b: &[u8]) -> Result<Self, WireError> {
        let mut r = Reader::new(b);
        Ok(Welcome { node_id: r.uvar()? as u32, server_ms: r.uvar()? })
    }
}

/// Ping and Pong carry the sender's clock; the pong echoes it back untouched,
/// so RTT = now - echoed value without needing synchronised clocks.
pub fn encode_time(t_ms: u64) -> Vec<u8> {
    let mut w = Writer::new();
    w.uvar(t_ms);
    w.finish()
}
pub fn decode_time(b: &[u8]) -> Result<u64, WireError> {
    Reader::new(b).uvar()
}

/// Selective ack: sorted seqs, first absolute then gaps. 10 consecutive seqs ≈ 12 bytes.
pub fn encode_ack(seqs: &[u32]) -> Vec<u8> {
    let mut s = seqs.to_vec();
    s.sort_unstable();
    s.dedup();
    let mut w = Writer::new();
    w.uvar(s.len() as u64);
    let mut prev = 0u32;
    for (i, v) in s.iter().enumerate() {
        w.uvar(if i == 0 { *v as u64 } else { (*v - prev) as u64 });
        prev = *v;
    }
    w.finish()
}
pub fn decode_ack(b: &[u8]) -> Result<Vec<u32>, WireError> {
    let mut r = Reader::new(b);
    let n = r.uvar()? as usize;
    if n > 4096 {
        return Err(WireError::Invalid("ack too long"));
    }
    let mut out = Vec::with_capacity(n);
    let mut prev = 0u32;
    for i in 0..n {
        let d = r.uvar()? as u32;
        prev = if i == 0 { d } else { prev.wrapping_add(d) };
        out.push(prev);
    }
    Ok(out)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[repr(u8)]
pub enum NackReason {
    /// Payload could not be decoded.
    Malformed = 0,
    /// A vitals delta referenced a keyframe the receiver does not have.
    NeedKeyframe = 1,
}
pub fn encode_nack(seq: u32, reason: NackReason, stream: u32) -> Vec<u8> {
    let mut w = Writer::new();
    w.uvar(seq as u64).u8(reason as u8).uvar(stream as u64);
    w.finish()
}
pub fn decode_nack(b: &[u8]) -> Result<(u32, NackReason, u32), WireError> {
    let mut r = Reader::new(b);
    let seq = r.uvar()? as u32;
    let reason = match r.u8()? {
        0 => NackReason::Malformed,
        1 => NackReason::NeedKeyframe,
        _ => return Err(WireError::Invalid("nack reason")),
    };
    Ok((seq, reason, r.uvar()? as u32))
}

/// Alarm codes are numbers on the wire; the text is optional extra context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Alarm {
    pub subject: String,
    pub code: u8,
    pub text: String,
}
pub mod alarm_code {
    pub const NO_PULSE: u8 = 1;
    pub const LOW_SPO2: u8 = 2;
    pub const SOLAR_STORM: u8 = 3;
    pub const HELP_REQUEST: u8 = 4;
    pub const BATTERY: u8 = 5;
    /// the red emergency button on the medical station
    pub const STATION_BUTTON: u8 = 6;
}
impl Alarm {
    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        w.str(&self.subject).u8(self.code).str(&self.text);
        w.finish()
    }
    pub fn decode(b: &[u8]) -> Result<Self, WireError> {
        let mut r = Reader::new(b);
        Ok(Alarm { subject: r.str()?.into(), code: r.u8()?, text: r.str()?.into() })
    }
}

/// Free text between nodes: mesh chat, guidance lines, messages to Earth.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Text {
    pub from: String,
    pub to: String,
    pub text: String,
}
impl Text {
    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        w.str(&self.from).str(&self.to).str(&self.text);
        w.finish()
    }
    pub fn decode(b: &[u8]) -> Result<Self, WireError> {
        let mut r = Reader::new(b);
        Ok(Text { from: r.str()?.into(), to: r.str()?.into(), text: r.str()?.into() })
    }
}

/// One record of the hash chain travelling to the ground. Only the first 8
/// bytes of the hash are sent; the ground recomputes the full hash from the
/// body and the previous record and compares the prefix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordMsg {
    pub chain_seq: u64,
    pub ts_ms: u64,
    pub kind: String,
    pub subject: String,
    pub hash8: [u8; 8],
    pub body: Vec<u8>,
    /// Ed25519 signature over the record's full hash (64 bytes; empty = unsigned).
    /// Trailing and optional on the wire, so v1 decoders without it still work.
    pub sig: Vec<u8>,
}
impl RecordMsg {
    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        w.uvar(self.chain_seq).uvar(self.ts_ms).str(&self.kind).str(&self.subject).raw(&self.hash8).bytes(&self.body);
        if !self.sig.is_empty() {
            w.bytes(&self.sig);
        }
        w.finish()
    }
    pub fn decode(b: &[u8]) -> Result<Self, WireError> {
        let mut r = Reader::new(b);
        let chain_seq = r.uvar()?;
        let ts_ms = r.uvar()?;
        let kind = r.str()?.into();
        let subject = r.str()?.into();
        let mut hash8 = [0u8; 8];
        hash8.copy_from_slice(r.take(8)?);
        let body = r.bytes()?.to_vec();
        let sig = if r.is_empty() { Vec::new() } else { r.bytes()?.to_vec() };
        if !sig.is_empty() && sig.len() != 64 {
            return Err(WireError::Invalid("signature length"));
        }
        Ok(RecordMsg { chain_seq, ts_ms, kind, subject, hash8, body, sig })
    }
}

/// Compartments on the medical station (numbered 1..=12 on the cabinet).
pub const STATION_COMPARTMENTS: u8 = 12;
/// Bit mask with one bit per compartment (bit 0 = compartment 1).
pub const STATION_MASK: u16 = (1 << STATION_COMPARTMENTS) - 1;

/// Operations the hub asks of the medical station.
pub mod station_op {
    /// lights off on every compartment (doors and counts unchanged)
    pub const CLEAR_ALL: u8 = 0;
    /// light one compartment and show `text` on the Eva screen
    pub const LIGHT: u8 = 1;
    /// light off on one compartment
    pub const CLEAR: u8 = 2;
    /// demo reset: every compartment idle and full again
    pub const RESTOCK: u8 = 3;
}

/// Hub → station. `case` ties the request to an emergency case (0 = none).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StationCmd {
    pub op: u8,
    /// 1..=12; 0 with CLEAR_ALL / RESTOCK
    pub compartment: u8,
    pub case: u64,
    pub text: String,
}
impl StationCmd {
    pub fn light(compartment: u8, text: impl Into<String>, case: u64) -> Self {
        StationCmd { op: station_op::LIGHT, compartment, case, text: text.into() }
    }
    pub fn clear(compartment: u8) -> Self {
        StationCmd { op: station_op::CLEAR, compartment, case: 0, text: String::new() }
    }
    pub fn all(op: u8) -> Self {
        StationCmd { op, compartment: 0, case: 0, text: String::new() }
    }
    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        w.u8(self.op).u8(self.compartment).uvar(self.case).str(&self.text);
        w.finish()
    }
    pub fn decode(b: &[u8]) -> Result<Self, WireError> {
        let mut r = Reader::new(b);
        let op = r.u8()?;
        let compartment = r.u8()?;
        let case = r.uvar()?;
        let text = r.str()?.to_string();
        if op > station_op::RESTOCK {
            return Err(WireError::Invalid("station op"));
        }
        let needs_one = matches!(op, station_op::LIGHT | station_op::CLEAR);
        if compartment > STATION_COMPARTMENTS || (needs_one && compartment == 0) {
            return Err(WireError::Invalid("station compartment"));
        }
        Ok(StationCmd { op, compartment, case, text })
    }
}

/// What the station reports.
pub mod station_ev {
    pub const DOOR_OPENED: u8 = 1;
    pub const DOOR_CLOSED: u8 = 2;
    /// the item sensor (weight / reed switch) saw an item leave
    pub const ITEM_TAKEN: u8 = 3;
    /// the red emergency button
    pub const BUTTON: u8 = 4;
    /// the station hears the hub again (count = events it buffered meanwhile)
    pub const ONLINE: u8 = 5;
    /// a wristband's no-pulse alarm heard directly over LoRa (subject = seat)
    pub const ALARM_HEARD: u8 = 6;
    /// the station lit a compartment by itself (standalone fallback)
    pub const SELF_LIT: u8 = 7;
    pub const LAST: u8 = SELF_LIT;
}

/// Station → hub. `t_s` is the station's own clock (mission seconds), so a
/// buffered event keeps the time it happened, not the time it was uploaded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StationEvent {
    pub event: u8,
    /// 1..=12, or 0 when the event is not about one compartment
    pub compartment: u8,
    pub t_s: u64,
    /// items left (item events), events buffered (ONLINE), else 0
    pub count: u32,
    /// seat for ALARM_HEARD, else empty
    pub subject: String,
}
impl StationEvent {
    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        w.u8(self.event).u8(self.compartment).uvar(self.t_s).uvar(self.count as u64).str(&self.subject);
        w.finish()
    }
    pub fn decode(b: &[u8]) -> Result<Self, WireError> {
        let mut r = Reader::new(b);
        let event = r.u8()?;
        let compartment = r.u8()?;
        let t_s = r.uvar()?;
        let count = r.uvar()?;
        let subject = r.str()?.to_string();
        if event == 0 || event > station_ev::LAST {
            return Err(WireError::Invalid("station event"));
        }
        if compartment > STATION_COMPARTMENTS {
            return Err(WireError::Invalid("station compartment"));
        }
        if count > u32::MAX as u64 {
            return Err(WireError::Invalid("station count"));
        }
        Ok(StationEvent { event, compartment, t_s, count: count as u32, subject })
    }
}

pub mod station_flag {
    /// the station cannot hear the hub and runs on its own rules
    pub const STANDALONE: u8 = 1;
    /// running on its backup battery (no ship power)
    pub const ON_BATTERY: u8 = 2;
}

/// Station → hub, latest-wins (vitals class, never retransmitted).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StationStatus {
    pub battery: u8,
    pub flags: u8,
    pub lit: u16,
    pub open: u16,
    pub taken: u16,
    pub buffered: u32,
    /// items per compartment, compartment 1 first (up to 12 bytes)
    pub items: Vec<u8>,
}
impl StationStatus {
    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        w.u8(self.battery).u8(self.flags).uvar(self.lit as u64).uvar(self.open as u64).uvar(self.taken as u64).uvar(self.buffered as u64).bytes(&self.items);
        w.finish()
    }
    pub fn decode(b: &[u8]) -> Result<Self, WireError> {
        let mut r = Reader::new(b);
        let battery = r.u8()?;
        let flags = r.u8()?;
        let mut mask = || -> Result<u16, WireError> {
            let m = r.uvar()?;
            if m > STATION_MASK as u64 {
                return Err(WireError::Invalid("station mask"));
            }
            Ok(m as u16)
        };
        let (lit, open, taken) = (mask()?, mask()?, mask()?);
        let buffered = r.uvar()?;
        let items = r.bytes()?.to_vec();
        if battery > 100 || buffered > u32::MAX as u64 || items.len() > STATION_COMPARTMENTS as usize {
            return Err(WireError::Invalid("station status"));
        }
        Ok(StationStatus { battery, flags, lit, open, taken, buffered: buffered as u32, items })
    }
}

/// Server → clients: who is on the mesh right now, with measured RTT.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PresenceEntry {
    pub id: u32,
    pub role: Role,
    pub name: String,
    pub rtt_ms: u32,
}
pub fn encode_presence(nodes: &[PresenceEntry]) -> Vec<u8> {
    let mut w = Writer::new();
    w.uvar(nodes.len() as u64);
    for n in nodes {
        w.uvar(n.id as u64).u8(n.role as u8).str(&n.name).uvar(n.rtt_ms as u64);
    }
    w.finish()
}
pub fn decode_presence(b: &[u8]) -> Result<Vec<PresenceEntry>, WireError> {
    let mut r = Reader::new(b);
    let n = r.uvar()? as usize;
    let mut out = Vec::with_capacity(n.min(256));
    for _ in 0..n {
        out.push(PresenceEntry { id: r.uvar()? as u32, role: Role::from_u8(r.u8()?)?, name: r.str()?.into(), rtt_ms: r.uvar()? as u32 });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_welcome_roundtrip() {
        let h = Hello::new(Role::App, "Ana · 4B", 123_456);
        assert_eq!(Hello::decode(&h.encode()).unwrap(), h);
        let k = Hello { pubkey: vec![7; 32], ..Hello::new(Role::Ship, "ship", 9) };
        assert_eq!(Hello::decode(&k.encode()).unwrap(), k);
        assert_eq!(k.encode().len(), Hello::new(Role::Ship, "ship", 9).encode().len() + 33, "the key adds 33 bytes");
        let w = Welcome { node_id: 42, server_ms: 9_999_999 };
        assert_eq!(Welcome::decode(&w.encode()).unwrap(), w);
    }

    #[test]
    fn ack_is_compact() {
        let seqs: Vec<u32> = (1000..1010).collect();
        let b = encode_ack(&seqs);
        assert_eq!(decode_ack(&b).unwrap(), seqs);
        assert!(b.len() <= 12, "10 consecutive acks in {} bytes", b.len());
    }

    #[test]
    fn record_alarm_text_presence_roundtrip() {
        let r = RecordMsg { chain_seq: 7, ts_ms: 1_700_000_000_000, kind: "alarm".into(), subject: "4B".into(), hash8: [1, 2, 3, 4, 5, 6, 7, 8], body: br#"{"a":1}"#.to_vec(), sig: vec![] };
        assert_eq!(RecordMsg::decode(&r.encode()).unwrap(), r);
        let signed = RecordMsg { sig: vec![9; 64], ..r.clone() };
        assert_eq!(RecordMsg::decode(&signed.encode()).unwrap(), signed);
        assert_eq!(signed.encode().len(), r.encode().len() + 65);
        let a = Alarm { subject: "4B".into(), code: alarm_code::NO_PULSE, text: "No pulse".into() };
        assert_eq!(Alarm::decode(&a.encode()).unwrap(), a);
        let t = Text { from: "Leo".into(), to: "*".into(), text: "On my way".into() };
        assert_eq!(Text::decode(&t.encode()).unwrap(), t);
        let p = vec![PresenceEntry { id: 1, role: Role::Dash, name: "Mara".into(), rtt_ms: 4 }];
        assert_eq!(decode_presence(&encode_presence(&p)).unwrap(), p);
        let (s, why, st) = decode_nack(&encode_nack(9, NackReason::NeedKeyframe, 3)).unwrap();
        assert_eq!((s, why, st), (9, NackReason::NeedKeyframe, 3));
    }
    #[test]
    fn station_messages_roundtrip_and_stay_small() {
        let c = StationCmd::light(1, "Take the AED", 101);
        assert_eq!(StationCmd::decode(&c.encode()).unwrap(), c);
        // 01 op · 01 compartment · 65 case · 0c "Take the AED"
        assert_eq!(&c.encode()[..4], &[1, 1, 101, 12]);
        assert_eq!(c.encode().len(), 16);
        for c in [StationCmd::clear(1), StationCmd::all(station_op::CLEAR_ALL), StationCmd::all(station_op::RESTOCK)] {
            assert_eq!(StationCmd::decode(&c.encode()).unwrap(), c);
        }
        let e = StationEvent { event: station_ev::ITEM_TAKEN, compartment: 1, t_s: 8_344_727, count: 0, subject: String::new() };
        assert_eq!(StationEvent::decode(&e.encode()).unwrap(), e);
        assert_eq!(e.encode().len(), 8, "AED taken: 8 bytes of payload");
        let h = StationEvent { event: station_ev::ALARM_HEARD, compartment: 1, t_s: 9, count: 0, subject: "4B".into() };
        assert_eq!(StationEvent::decode(&h.encode()).unwrap(), h);
        let s = StationStatus { battery: 96, flags: station_flag::STANDALONE, lit: 1, open: 1, taken: 1, buffered: 3, items: vec![0, 1, 1, 1, 1, 1, 6, 4, 2, 2, 2, 3] };
        assert_eq!(StationStatus::decode(&s.encode()).unwrap(), s);
        assert_eq!(s.encode().len(), 19);
    }

    #[test]
    fn station_messages_reject_nonsense() {
        assert!(StationCmd::decode(&StationCmd::light(13, "x", 0).encode()).is_err(), "no compartment 13");
        assert!(StationCmd::decode(&StationCmd::light(0, "x", 0).encode()).is_err(), "light needs a compartment");
        assert!(StationCmd::decode(&StationCmd { op: 9, compartment: 1, case: 0, text: String::new() }.encode()).is_err());
        assert!(StationEvent::decode(&StationEvent { event: 0, compartment: 1, t_s: 1, count: 0, subject: String::new() }.encode()).is_err());
        assert!(StationEvent::decode(&StationEvent { event: 99, compartment: 1, t_s: 1, count: 0, subject: String::new() }.encode()).is_err());
        let mut w = Writer::new();
        w.u8(50).u8(0).uvar(1 << 12).uvar(0).uvar(0).uvar(0).bytes(&[]);
        assert!(StationStatus::decode(&w.finish()).is_err(), "mask wider than 12 compartments");
        let s = StationStatus { battery: 101, ..Default::default() };
        assert!(StationStatus::decode(&s.encode()).is_err());
        let s = StationStatus { items: vec![1; 13], ..Default::default() };
        assert!(StationStatus::decode(&s.encode()).is_err());
        assert_eq!(Role::from_u8(6).unwrap(), Role::Station);
    }
}
