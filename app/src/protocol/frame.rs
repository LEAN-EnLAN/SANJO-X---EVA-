//! Eva Link frame: the unit that crosses a link.
//!
//! ```text
//! byte 0   : VVV FFFFF   version (3 bits) | flags (5 bits)
//! byte 1   : PPP TTTTT   priority class (3 bits) | message type (5 bits)
//! varint   : seq         per-sender sequence number
//! varint   : src         sender node id (0 = server / hub)
//! [varint] : dst         only if FLAG_DST (absent = to the hub / broadcast)
//! [varint] : ack         only if FLAG_ACK (piggybacked cumulative ack)
//! varint   : len         payload length (lets several frames share one datagram)
//! bytes    : payload
//! [u16 BE] : crc16       only if FLAG_CRC, over every byte above
//! ```
//! Smallest frame (ping with a 1-byte payload) is 6 bytes.

use super::wire::{crc16, Reader, WireError, Writer};
use serde::Serialize;

pub const VERSION: u8 = 1;

pub const FLAG_DST: u8 = 0b00001;
pub const FLAG_ACK: u8 = 0b00010;
pub const FLAG_RETX: u8 = 0b00100;
pub const FLAG_CRC: u8 = 0b01000;
/// Reliable: the receiver must answer with an Ack frame listing this seq.
pub const FLAG_REL: u8 = 0b10000;

/// Priority classes, lower value wins. A queued alarm always leaves before
/// anything else, even if thousands of log records are waiting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[repr(u8)]
pub enum Priority {
    Alarm = 0,
    Guidance = 1,
    Vitals = 2,
    Logs = 3,
}

impl Priority {
    pub const ALL: [Priority; 4] = [Priority::Alarm, Priority::Guidance, Priority::Vitals, Priority::Logs];
    pub fn from_u8(v: u8) -> Result<Self, WireError> {
        Ok(match v {
            0 => Priority::Alarm,
            1 => Priority::Guidance,
            2 => Priority::Vitals,
            3 => Priority::Logs,
            _ => return Err(WireError::Invalid("priority")),
        })
    }
    pub fn name(self) -> &'static str {
        match self {
            Priority::Alarm => "alarm",
            Priority::Guidance => "guidance",
            Priority::Vitals => "vitals",
            Priority::Logs => "logs",
        }
    }
}

/// Message types. Control types (Hello..Nack) bypass the priority queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[repr(u8)]
pub enum MsgType {
    Hello = 0,
    Welcome = 1,
    Ping = 2,
    Pong = 3,
    Ack = 4,
    Nack = 5,
    Vitals = 6,
    Alarm = 7,
    Guidance = 8,
    Record = 9,
    Text = 10,
    Presence = 11,
    Bye = 12,
    /// hub → medical station: light / clear a compartment, show a message
    StationCmd = 13,
    /// medical station → hub: door opened, item taken, red button, back online…
    StationEvent = 14,
    /// medical station → hub: battery, lit/open/taken masks, buffered events (latest-wins)
    StationStatus = 15,
}

impl MsgType {
    pub const ALL: [MsgType; 16] = [
        MsgType::Hello,
        MsgType::Welcome,
        MsgType::Ping,
        MsgType::Pong,
        MsgType::Ack,
        MsgType::Nack,
        MsgType::Vitals,
        MsgType::Alarm,
        MsgType::Guidance,
        MsgType::Record,
        MsgType::Text,
        MsgType::Presence,
        MsgType::Bye,
        MsgType::StationCmd,
        MsgType::StationEvent,
        MsgType::StationStatus,
    ];
    pub fn from_u8(v: u8) -> Result<Self, WireError> {
        Self::ALL.get(v as usize).copied().ok_or(WireError::UnknownType(v))
    }
    pub fn name(self) -> &'static str {
        match self {
            MsgType::Hello => "hello",
            MsgType::Welcome => "welcome",
            MsgType::Ping => "ping",
            MsgType::Pong => "pong",
            MsgType::Ack => "ack",
            MsgType::Nack => "nack",
            MsgType::Vitals => "vitals",
            MsgType::Alarm => "alarm",
            MsgType::Guidance => "guidance",
            MsgType::Record => "record",
            MsgType::Text => "text",
            MsgType::Presence => "presence",
            MsgType::Bye => "bye",
            MsgType::StationCmd => "station_cmd",
            MsgType::StationEvent => "station_event",
            MsgType::StationStatus => "station_status",
        }
    }
    /// Default priority class for a message type.
    pub fn default_priority(self) -> Priority {
        match self {
            MsgType::Alarm => Priority::Alarm,
            MsgType::Guidance | MsgType::Text | MsgType::StationCmd | MsgType::StationEvent => Priority::Guidance,
            MsgType::Vitals | MsgType::StationStatus => Priority::Vitals,
            MsgType::Record => Priority::Logs,
            // control frames are sent immediately; class is only informative
            _ => Priority::Alarm,
        }
    }
    /// Whether frames of this type are acknowledged and retransmitted.
    pub fn reliable(self) -> bool {
        matches!(self, MsgType::Alarm | MsgType::Guidance | MsgType::Record | MsgType::Text | MsgType::StationCmd | MsgType::StationEvent)
    }
    pub fn is_control(self) -> bool {
        matches!(
            self,
            MsgType::Hello | MsgType::Welcome | MsgType::Ping | MsgType::Pong | MsgType::Ack | MsgType::Nack | MsgType::Presence | MsgType::Bye
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub flags: u8,
    pub prio: Priority,
    pub kind: MsgType,
    pub seq: u32,
    pub src: u32,
    pub dst: Option<u32>,
    pub ack: Option<u32>,
    pub payload: Vec<u8>,
}

impl Frame {
    pub fn new(kind: MsgType, seq: u32, src: u32, payload: Vec<u8>) -> Self {
        let mut flags = 0;
        if kind.reliable() {
            flags |= FLAG_REL;
        }
        Frame { flags, prio: kind.default_priority(), kind, seq, src, dst: None, ack: None, payload }
    }
    pub fn to(mut self, dst: u32) -> Self {
        self.dst = Some(dst);
        self
    }
    pub fn with_crc(mut self) -> Self {
        self.flags |= FLAG_CRC;
        self
    }
    pub fn is_reliable(&self) -> bool {
        self.flags & FLAG_REL != 0
    }
    pub fn is_retransmit(&self) -> bool {
        self.flags & FLAG_RETX != 0
    }

    pub fn encode_into(&self, w: &mut Writer) {
        let start = w.buf.len();
        let mut flags = self.flags & !(FLAG_DST | FLAG_ACK);
        if self.dst.is_some() {
            flags |= FLAG_DST;
        }
        if self.ack.is_some() {
            flags |= FLAG_ACK;
        }
        w.u8((VERSION << 5) | (flags & 0x1f));
        w.u8(((self.prio as u8) << 5) | (self.kind as u8 & 0x1f));
        w.uvar(self.seq as u64);
        w.uvar(self.src as u64);
        if let Some(d) = self.dst {
            w.uvar(d as u64);
        }
        if let Some(a) = self.ack {
            w.uvar(a as u64);
        }
        w.bytes(&self.payload);
        if flags & FLAG_CRC != 0 {
            let c = crc16(&w.buf[start..]);
            w.raw(&c.to_be_bytes());
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        self.encode_into(&mut w);
        w.finish()
    }

    /// Decode one frame from the reader (several may follow in one datagram).
    pub fn decode_from(r: &mut Reader<'_>, whole: &[u8]) -> Result<Frame, WireError> {
        let start = r.pos();
        let b0 = r.u8()?;
        let version = b0 >> 5;
        if version != VERSION {
            return Err(WireError::BadVersion(version));
        }
        let flags = b0 & 0x1f;
        let b1 = r.u8()?;
        let prio = Priority::from_u8(b1 >> 5)?;
        let kind = MsgType::from_u8(b1 & 0x1f)?;
        let seq = r.uvar()? as u32;
        let src = r.uvar()? as u32;
        let dst = if flags & FLAG_DST != 0 { Some(r.uvar()? as u32) } else { None };
        let ack = if flags & FLAG_ACK != 0 { Some(r.uvar()? as u32) } else { None };
        let payload = r.bytes()?.to_vec();
        if flags & FLAG_CRC != 0 {
            let end = r.pos();
            let got = r.take(2)?;
            let want = crc16(&whole[start..end]).to_be_bytes();
            if got != want {
                return Err(WireError::BadCrc);
            }
        }
        Ok(Frame { flags, prio, kind, seq, src, dst, ack, payload })
    }

    pub fn decode(buf: &[u8]) -> Result<Frame, WireError> {
        let mut r = Reader::new(buf);
        Self::decode_from(&mut r, buf)
    }

    /// Size of this frame on the wire.
    pub fn wire_len(&self) -> usize {
        self.encode().len()
    }
    /// Header + trailer bytes (everything that is not payload).
    pub fn overhead(&self) -> usize {
        self.wire_len() - self.payload.len()
    }
}

/// Pack several frames into one datagram (LoRa packets are expensive to start).
pub fn encode_batch(frames: &[Frame]) -> Vec<u8> {
    let mut w = Writer::new();
    for f in frames {
        f.encode_into(&mut w);
    }
    w.finish()
}

pub fn decode_batch(buf: &[u8]) -> Result<Vec<Frame>, WireError> {
    let mut r = Reader::new(buf);
    let mut out = Vec::new();
    while !r.is_empty() {
        out.push(Frame::decode_from(&mut r, buf)?);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_minimal_and_full() {
        let f = Frame::new(MsgType::Ping, 1, 3, vec![7]);
        let b = f.encode();
        assert_eq!(b.len(), 6, "minimal frame is 6 bytes");
        assert_eq!(Frame::decode(&b).unwrap(), f);

        let mut g = Frame::new(MsgType::Alarm, 70_000, 12, b"no pulse".to_vec()).to(0).with_crc();
        g.ack = Some(69_999);
        let b = g.encode();
        let back = Frame::decode(&b).unwrap();
        assert_eq!(back.kind, MsgType::Alarm);
        assert_eq!(back.prio, Priority::Alarm);
        assert!(back.is_reliable());
        assert_eq!(back.dst, Some(0));
        assert_eq!(back.ack, Some(69_999));
        assert_eq!(back.payload, b"no pulse");
    }

    #[test]
    fn crc_detects_corruption() {
        let f = Frame::new(MsgType::Text, 5, 1, b"hello".to_vec()).with_crc();
        let mut b = f.encode();
        b[6] ^= 0x01;
        assert_eq!(Frame::decode(&b), Err(WireError::BadCrc));
    }

    #[test]
    fn rejects_other_versions_and_types() {
        let mut b = Frame::new(MsgType::Ping, 1, 1, vec![]).encode();
        b[0] = (2 << 5) | (b[0] & 0x1f);
        assert_eq!(Frame::decode(&b), Err(WireError::BadVersion(2)));
        let mut b = Frame::new(MsgType::Ping, 1, 1, vec![]).encode();
        b[1] = 31;
        assert_eq!(Frame::decode(&b), Err(WireError::UnknownType(31)));
    }

    #[test]
    fn batch_roundtrip() {
        let fs = vec![
            Frame::new(MsgType::Vitals, 1, 2, vec![1, 2, 3]),
            Frame::new(MsgType::Record, 2, 2, vec![9; 40]).with_crc(),
            Frame::new(MsgType::Ack, 3, 2, vec![]),
        ];
        let b = encode_batch(&fs);
        assert_eq!(decode_batch(&b).unwrap(), fs);
    }
}
