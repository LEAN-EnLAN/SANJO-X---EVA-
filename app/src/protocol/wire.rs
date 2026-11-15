//! Low-level byte helpers for Eva Link: LEB128 varints, zigzag signed ints,
//! length-prefixed strings and a CRC-16/CCITT-FALSE checksum.
//!
//! Everything is hand-written so the exact bytes on the wire are documented
//! in `docs/app/PROTOCOL.md` and can be mirrored in the browser (`static/js/link.js`).

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WireError {
    Truncated,
    VarintOverflow,
    BadUtf8,
    BadVersion(u8),
    UnknownType(u8),
    BadCrc,
    Invalid(&'static str),
}

impl fmt::Display for WireError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for WireError {}

/// Append-only byte writer.
#[derive(Default, Debug, Clone)]
pub struct Writer {
    pub buf: Vec<u8>,
}

impl Writer {
    pub fn new() -> Self {
        Self { buf: Vec::with_capacity(32) }
    }
    pub fn u8(&mut self, v: u8) -> &mut Self {
        self.buf.push(v);
        self
    }
    /// Unsigned LEB128: 7 bits per byte, high bit = "more bytes follow".
    pub fn uvar(&mut self, mut v: u64) -> &mut Self {
        loop {
            let byte = (v & 0x7f) as u8;
            v >>= 7;
            if v == 0 {
                self.buf.push(byte);
                return self;
            }
            self.buf.push(byte | 0x80);
        }
    }
    /// Signed varint via zigzag (0,-1,1,-2,2 → 0,1,2,3,4) so small negatives stay 1 byte.
    pub fn ivar(&mut self, v: i64) -> &mut Self {
        self.uvar(zigzag(v))
    }
    pub fn bytes(&mut self, b: &[u8]) -> &mut Self {
        self.uvar(b.len() as u64);
        self.buf.extend_from_slice(b);
        self
    }
    pub fn str(&mut self, s: &str) -> &mut Self {
        self.bytes(s.as_bytes())
    }
    pub fn raw(&mut self, b: &[u8]) -> &mut Self {
        self.buf.extend_from_slice(b);
        self
    }
    pub fn finish(self) -> Vec<u8> {
        self.buf
    }
}

/// Cursor over a byte slice.
pub struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }
    pub fn pos(&self) -> usize {
        self.pos
    }
    pub fn remaining(&self) -> usize {
        self.buf.len() - self.pos
    }
    pub fn is_empty(&self) -> bool {
        self.remaining() == 0
    }
    pub fn u8(&mut self) -> Result<u8, WireError> {
        let b = *self.buf.get(self.pos).ok_or(WireError::Truncated)?;
        self.pos += 1;
        Ok(b)
    }
    pub fn uvar(&mut self) -> Result<u64, WireError> {
        let mut out: u64 = 0;
        let mut shift = 0u32;
        loop {
            let b = self.u8()?;
            if shift >= 64 {
                return Err(WireError::VarintOverflow);
            }
            out |= ((b & 0x7f) as u64) << shift;
            if b & 0x80 == 0 {
                return Ok(out);
            }
            shift += 7;
        }
    }
    pub fn ivar(&mut self) -> Result<i64, WireError> {
        Ok(unzigzag(self.uvar()?))
    }
    pub fn take(&mut self, n: usize) -> Result<&'a [u8], WireError> {
        if self.remaining() < n {
            return Err(WireError::Truncated);
        }
        let s = &self.buf[self.pos..self.pos + n];
        self.pos += n;
        Ok(s)
    }
    pub fn bytes(&mut self) -> Result<&'a [u8], WireError> {
        let n = self.uvar()? as usize;
        self.take(n)
    }
    pub fn str(&mut self) -> Result<&'a str, WireError> {
        std::str::from_utf8(self.bytes()?).map_err(|_| WireError::BadUtf8)
    }
}

pub fn zigzag(v: i64) -> u64 {
    ((v << 1) ^ (v >> 63)) as u64
}
pub fn unzigzag(v: u64) -> i64 {
    ((v >> 1) as i64) ^ -((v & 1) as i64)
}

/// Size in bytes of `v` as an unsigned varint.
pub fn uvar_len(mut v: u64) -> usize {
    let mut n = 1;
    while v >= 0x80 {
        v >>= 7;
        n += 1;
    }
    n
}

/// CRC-16/CCITT-FALSE (poly 0x1021, init 0xFFFF). Used only on links without
/// their own integrity check (the LoRa-like profile).
pub fn crc16(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xffff;
    for &b in data {
        crc ^= (b as u16) << 8;
        for _ in 0..8 {
            crc = if crc & 0x8000 != 0 { (crc << 1) ^ 0x1021 } else { crc << 1 };
        }
    }
    crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn varint_roundtrip_and_sizes() {
        for v in [0u64, 1, 127, 128, 300, 16_383, 16_384, u32::MAX as u64, u64::MAX] {
            let mut w = Writer::new();
            w.uvar(v);
            assert_eq!(w.buf.len(), uvar_len(v), "len for {v}");
            assert_eq!(Reader::new(&w.buf).uvar().unwrap(), v);
        }
        let mut w = Writer::new();
        w.uvar(127);
        assert_eq!(w.buf, vec![0x7f]);
    }

    #[test]
    fn zigzag_keeps_small_negatives_small() {
        for v in [-3i64, -1, 0, 1, 2, -64, 63, i64::MIN, i64::MAX] {
            assert_eq!(unzigzag(zigzag(v)), v);
        }
        let mut w = Writer::new();
        w.ivar(-1);
        assert_eq!(w.buf.len(), 1);
    }

    #[test]
    fn crc_known_vector() {
        // "123456789" → 0x29B1 for CRC-16/CCITT-FALSE
        assert_eq!(crc16(b"123456789"), 0x29b1);
    }

    #[test]
    fn truncated_inputs_error() {
        assert_eq!(Reader::new(&[0x80]).uvar(), Err(WireError::Truncated));
        assert_eq!(Reader::new(&[5, b'a']).str(), Err(WireError::Truncated));
    }
}
