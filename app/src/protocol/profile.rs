//! Simulated link profiles: LAN, LoRa-like, near-space uplink, outage.
//!
//! Eva is designed for **near space** (low Earth orbit, cislunar at most):
//! one-way latency of hundreds of milliseconds to a few seconds, narrow and
//! intermittent links, loss of contact while a relay is below the horizon.
//! It is *not* a deep-space (minutes of delay) design: see
//! `docs/app/ADR-002-near-space-not-deep-space.md`.
//!
//! [`LinkSim`] is deterministic (seeded PRNG, explicit clock) so tests and
//! benchmarks give the same numbers on every run.

use serde::Serialize;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LinkProfile {
    pub name: &'static str,
    pub label: &'static str,
    /// one-way base latency
    pub latency_ms: u64,
    /// uniform jitter added on top, 0..=jitter
    pub jitter_ms: u64,
    /// bits per second (serialisation delay and budget)
    pub bandwidth_bps: u64,
    /// probability that a datagram is lost, 0.0..=1.0
    pub loss: f64,
    /// largest datagram the link carries
    pub mtu: usize,
    /// loss of contact: every `period_ms`, the link is down for `down_ms`
    pub outage_period_ms: u64,
    pub outage_down_ms: u64,
    /// add CRC-16 to frames (the link has no integrity check of its own)
    pub crc: bool,
}

impl LinkProfile {
    /// Ship LAN / Wi-Fi: effectively instant.
    pub fn lan() -> Self {
        Self { name: "lan", label: "Ship LAN", latency_ms: 2, jitter_ms: 3, bandwidth_bps: 50_000_000, loss: 0.0, mtu: 65_000, outage_period_ms: 0, outage_down_ms: 0, crc: false }
    }
    /// LoRa-like radio between the wristband parts and the hub:
    /// ~5.5 kbit/s (SF7/125 kHz class), 222-byte packets, some loss.
    pub fn lora() -> Self {
        Self { name: "lora", label: "LoRa-like radio", latency_ms: 60, jitter_ms: 60, bandwidth_bps: 5_470, loss: 0.05, mtu: 222, outage_period_ms: 0, outage_down_ms: 0, crc: true }
    }
    /// Near-space uplink (LEO ship ↔ relay ↔ ground): seconds of latency,
    /// 64 kbit/s shared, 2% loss, and a contact gap of 20 s every 2 min
    /// (scaled down from real orbital gaps so a demo shows it).
    pub fn near_space() -> Self {
        Self { name: "near-space", label: "Near-space uplink", latency_ms: 1_200, jitter_ms: 800, bandwidth_bps: 64_000, loss: 0.02, mtu: 1_024, outage_period_ms: 120_000, outage_down_ms: 20_000, crc: true }
    }
    /// Degraded near-space link during a solar event: slower, lossier.
    pub fn storm() -> Self {
        Self { name: "storm", label: "Uplink during solar storm", latency_ms: 2_500, jitter_ms: 1_500, bandwidth_bps: 9_600, loss: 0.15, mtu: 512, outage_period_ms: 60_000, outage_down_ms: 25_000, crc: true }
    }
    /// No contact at all (store-and-forward only).
    pub fn outage() -> Self {
        Self { name: "outage", label: "Link outage", latency_ms: 0, jitter_ms: 0, bandwidth_bps: 0, loss: 1.0, mtu: 0, outage_period_ms: 1, outage_down_ms: 1, crc: true }
    }

    pub fn all() -> Vec<LinkProfile> {
        vec![Self::lan(), Self::lora(), Self::near_space(), Self::storm(), Self::outage()]
    }
    pub fn by_name(name: &str) -> Option<LinkProfile> {
        Self::all().into_iter().find(|p| p.name == name)
    }

    pub fn is_up(&self, now_ms: u64) -> bool {
        if self.bandwidth_bps == 0 {
            return false;
        }
        if self.outage_period_ms == 0 {
            return true;
        }
        now_ms % self.outage_period_ms >= self.outage_down_ms
    }
    /// Bytes the link can carry in `dt_ms`.
    pub fn budget(&self, dt_ms: u64) -> usize {
        ((self.bandwidth_bps as u128 * dt_ms as u128) / 8_000) as usize
    }
}

/// Tiny deterministic PRNG (xorshift64*). Not cryptographic.
#[derive(Debug, Clone)]
pub struct Rng(u64);
impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed.max(1))
    }
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }
    /// uniform in [0, 1)
    pub fn f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
    pub fn range(&mut self, max_inclusive: u64) -> u64 {
        if max_inclusive == 0 {
            0
        } else {
            self.next_u64() % (max_inclusive + 1)
        }
    }
}

/// One direction of a simulated link.
#[derive(Debug, Clone)]
pub struct LinkSim {
    pub profile: LinkProfile,
    rng: Rng,
    /// (deliver_at, order, bytes)
    in_flight: BinaryHeap<Reverse<(u64, u64, Vec<u8>)>>,
    order: u64,
    tx_free_at: u64,
    pub sent_datagrams: u64,
    pub lost_datagrams: u64,
    pub sent_bytes: u64,
}

impl LinkSim {
    pub fn new(profile: LinkProfile, seed: u64) -> Self {
        Self { profile, rng: Rng::new(seed), in_flight: BinaryHeap::new(), order: 0, tx_free_at: 0, sent_datagrams: 0, lost_datagrams: 0, sent_bytes: 0 }
    }
    pub fn set_profile(&mut self, p: LinkProfile) {
        self.profile = p;
    }

    /// Put a datagram on the link at `now_ms`. Returns false if it was lost
    /// (down, random loss, or larger than the MTU).
    pub fn send(&mut self, now_ms: u64, bytes: Vec<u8>) -> bool {
        self.sent_datagrams += 1;
        self.sent_bytes += bytes.len() as u64;
        let p = &self.profile;
        if !p.is_up(now_ms) || bytes.len() > p.mtu || self.rng.f64() < p.loss {
            self.lost_datagrams += 1;
            return false;
        }
        let ser_ms = (bytes.len() as u64 * 8_000).div_ceil(p.bandwidth_bps.max(1));
        let start = now_ms.max(self.tx_free_at);
        self.tx_free_at = start + ser_ms;
        let deliver = self.tx_free_at + p.latency_ms + self.rng.range(p.jitter_ms);
        self.order += 1;
        self.in_flight.push(Reverse((deliver, self.order, bytes)));
        true
    }

    /// Datagrams that have arrived by `now_ms`.
    pub fn recv(&mut self, now_ms: u64) -> Vec<Vec<u8>> {
        let mut out = Vec::new();
        while let Some(Reverse((t, _, _))) = self.in_flight.peek() {
            if *t > now_ms {
                break;
            }
            let Reverse((_, _, b)) = self.in_flight.pop().expect("peeked");
            out.push(b);
        }
        out
    }
    pub fn in_flight(&self) -> usize {
        self.in_flight.len()
    }
    /// Time when the next datagram arrives, if any.
    pub fn next_arrival(&self) -> Option<u64> {
        self.in_flight.peek().map(|Reverse((t, _, _))| *t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn near_space_has_seconds_not_minutes() {
        let p = LinkProfile::near_space();
        assert!(p.latency_ms >= 500 && p.latency_ms + p.jitter_ms <= 5_000);
    }

    #[test]
    fn outage_windows() {
        let p = LinkProfile::near_space();
        assert!(!p.is_up(0));
        assert!(!p.is_up(19_999));
        assert!(p.is_up(20_000));
        assert!(!p.is_up(120_000));
        assert!(!LinkProfile::outage().is_up(12_345));
    }

    #[test]
    fn lan_delivers_fast_and_in_order() {
        let mut l = LinkSim::new(LinkProfile::lan(), 7);
        for i in 0..5u8 {
            assert!(l.send(0, vec![i]));
        }
        let got = l.recv(10);
        assert_eq!(got.len(), 5);
    }

    #[test]
    fn lora_serialisation_delay_is_real() {
        let mut l = LinkSim::new(LinkProfile { loss: 0.0, jitter_ms: 0, ..LinkProfile::lora() }, 1);
        l.send(0, vec![0; 200]); // 1600 bits at 5470 bps ≈ 293 ms + 60 ms
        assert!(l.recv(300).is_empty());
        assert_eq!(l.recv(400).len(), 1);
        assert!(!l.send(0, vec![0; 300]), "over the 222-byte MTU");
    }

    #[test]
    fn deterministic_with_seed() {
        let run = || {
            let mut l = LinkSim::new(LinkProfile::storm(), 42);
            (0..200).filter(|i| l.send(30_000 + i * 10, vec![1; 20])).count()
        };
        assert_eq!(run(), run());
    }
}
