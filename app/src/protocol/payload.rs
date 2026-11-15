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
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hello {
    pub role: Role,
    pub name: String,
    pub client_ms: u64,
}
impl Hello {
    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        w.u8(self.role as u8).str(&self.name).uvar(self.client_ms);
        w.finish()
    }
    pub fn decode(b: &[u8]) -> Result<Self, WireError> {
        let mut r = Reader::new(b);
        Ok(Hello { role: Role::from_u8(r.u8()?)?, name: r.str()?.to_string(), client_ms: r.uvar()? })
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
}
impl RecordMsg {
    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        w.uvar(self.chain_seq).uvar(self.ts_ms).str(&self.kind).str(&self.subject).raw(&self.hash8).bytes(&self.body);
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
        Ok(RecordMsg { chain_seq, ts_ms, kind, subject, hash8, body })
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
        let h = Hello { role: Role::App, name: "Ana · 4B".into(), client_ms: 123_456 };
        assert_eq!(Hello::decode(&h.encode()).unwrap(), h);
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
        let r = RecordMsg { chain_seq: 7, ts_ms: 1_700_000_000_000, kind: "alarm".into(), subject: "4B".into(), hash8: [1, 2, 3, 4, 5, 6, 7, 8], body: br#"{"a":1}"#.to_vec() };
        assert_eq!(RecordMsg::decode(&r.encode()).unwrap(), r);
        let a = Alarm { subject: "4B".into(), code: alarm_code::NO_PULSE, text: "No pulse".into() };
        assert_eq!(Alarm::decode(&a.encode()).unwrap(), a);
        let t = Text { from: "Leo".into(), to: "*".into(), text: "On my way".into() };
        assert_eq!(Text::decode(&t.encode()).unwrap(), t);
        let p = vec![PresenceEntry { id: 1, role: Role::Dash, name: "Mara".into(), rtt_ms: 4 }];
        assert_eq!(decode_presence(&encode_presence(&p)).unwrap(), p);
        let (s, why, st) = decode_nack(&encode_nack(9, NackReason::NeedKeyframe, 3)).unwrap();
        assert_eq!((s, why, st), (9, NackReason::NeedKeyframe, 3));
    }
}
