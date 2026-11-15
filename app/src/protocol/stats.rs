//! Byte accounting: how many frames and bytes each message type costs.
//! Benchmarks and the dashboard read these counters to show "bytes per message".

use super::frame::{Frame, MsgType};
use serde::Serialize;

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct TypeStats {
    pub frames: u64,
    pub bytes: u64,
    pub payload_bytes: u64,
    pub retransmits: u64,
}

impl TypeStats {
    pub fn avg_bytes(&self) -> f64 {
        if self.frames == 0 {
            0.0
        } else {
            self.bytes as f64 / self.frames as f64
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ByteStats {
    by_type: [TypeStats; MsgType::ALL.len()],
}

#[derive(Debug, Clone, Serialize)]
pub struct StatsRow {
    pub kind: &'static str,
    pub frames: u64,
    pub bytes: u64,
    pub payload_bytes: u64,
    pub retransmits: u64,
    pub avg_bytes: f64,
}

impl ByteStats {
    pub fn record(&mut self, f: &Frame, wire_len: usize) {
        let t = &mut self.by_type[f.kind as usize];
        t.frames += 1;
        t.bytes += wire_len as u64;
        t.payload_bytes += f.payload.len() as u64;
        if f.is_retransmit() {
            t.retransmits += 1;
        }
    }
    pub fn get(&self, kind: MsgType) -> TypeStats {
        self.by_type[kind as usize]
    }
    pub fn total_bytes(&self) -> u64 {
        self.by_type.iter().map(|t| t.bytes).sum()
    }
    pub fn total_frames(&self) -> u64 {
        self.by_type.iter().map(|t| t.frames).sum()
    }
    /// Non-empty rows, for JSON and the dashboard.
    pub fn rows(&self) -> Vec<StatsRow> {
        MsgType::ALL
            .iter()
            .filter_map(|k| {
                let t = self.by_type[*k as usize];
                (t.frames > 0).then(|| StatsRow {
                    kind: k.name(),
                    frames: t.frames,
                    bytes: t.bytes,
                    payload_bytes: t.payload_bytes,
                    retransmits: t.retransmits,
                    avg_bytes: (t.avg_bytes() * 10.0).round() / 10.0,
                })
            })
            .collect()
    }
}
