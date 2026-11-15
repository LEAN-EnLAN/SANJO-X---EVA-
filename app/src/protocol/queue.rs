//! Store-and-forward send queue with priority classes, selective ACK/NACK and
//! retransmission.
//!
//! * Four FIFO queues, one per [`Priority`]. `poll` always drains the highest
//!   class first, so an alarm overtakes any backlog of vitals or log records.
//! * Reliable frames (alarm, guidance, text, record) move to an in-flight
//!   table when sent and stay there until acknowledged. Retransmission uses
//!   an RFC 6298-style RTO with exponential backoff and never gives up:
//!   during an outage records simply wait (store-and-forward).
//! * Unreliable frames (vitals) carry an optional time-to-live and are
//!   dropped when stale instead of clogging a link that just came back.
//! * `poll` takes a byte budget so the caller can model a slow link.

use super::frame::{Frame, MsgType, Priority, FLAG_RETX};
use super::stats::ByteStats;
use std::collections::{BTreeMap, HashSet, VecDeque};

/// Smoothed RTT estimator (RFC 6298 constants).
#[derive(Debug, Clone, Copy)]
pub struct RttEstimator {
    pub srtt_ms: f64,
    pub rttvar_ms: f64,
    pub samples: u64,
    pub last_ms: f64,
    pub min_rto_ms: u64,
    pub max_rto_ms: u64,
    /// RFC 6298 §5.5: after a retransmission timeout the RTO itself doubles
    /// (exponent here) and stays backed off until a valid sample arrives.
    /// Without this, a link slower than the initial 1 s RTO retransmits every
    /// frame, and Karn's rule then forbids every sample: the estimator never learns.
    pub backoff: u32,
}

impl Default for RttEstimator {
    fn default() -> Self {
        Self { srtt_ms: 0.0, rttvar_ms: 0.0, samples: 0, last_ms: 0.0, min_rto_ms: 200, max_rto_ms: 60_000, backoff: 0 }
    }
}

impl RttEstimator {
    pub fn sample(&mut self, rtt_ms: f64) {
        let rtt_ms = rtt_ms.max(0.0);
        self.last_ms = rtt_ms;
        if self.samples == 0 {
            self.srtt_ms = rtt_ms;
            self.rttvar_ms = rtt_ms / 2.0;
        } else {
            self.rttvar_ms = 0.75 * self.rttvar_ms + 0.25 * (self.srtt_ms - rtt_ms).abs();
            self.srtt_ms = 0.875 * self.srtt_ms + 0.125 * rtt_ms;
        }
        self.samples += 1;
        self.backoff = 0;
    }
    /// Retransmission timeout. 1 s before the first sample (RFC 6298 section 2.1),
    /// doubled for each backoff step.
    pub fn rto_ms(&self) -> u64 {
        let base = if self.samples == 0 { 1_000 } else { (self.srtt_ms + (4.0 * self.rttvar_ms).max(10.0)) as u64 };
        base.saturating_mul(1u64 << self.backoff.min(8)).clamp(self.min_rto_ms, self.max_rto_ms)
    }
    /// A retransmission timer expired.
    pub fn on_timeout(&mut self) {
        if self.rto_ms() < self.max_rto_ms {
            self.backoff += 1;
        }
    }
}

#[derive(Debug, Clone)]
struct Queued {
    frame: Frame,
    expires_ms: Option<u64>,
}

#[derive(Debug, Clone)]
struct Inflight {
    frame: Frame,
    first_sent_ms: u64,
    retries: u32,
    deadline_ms: u64,
}

/// Snapshot for dashboards and tests.
#[derive(Debug, Clone, Copy, Default, serde::Serialize)]
pub struct QueueDepth {
    pub alarm: usize,
    pub guidance: usize,
    pub vitals: usize,
    pub logs: usize,
    pub inflight: usize,
}

impl QueueDepth {
    pub fn queued(&self) -> usize {
        self.alarm + self.guidance + self.vitals + self.logs
    }
}

#[derive(Debug, Clone)]
pub struct SendQueue {
    pub src: u32,
    next_seq: u32,
    queues: [VecDeque<Queued>; 4],
    inflight: BTreeMap<u32, Inflight>,
    pub rtt: RttEstimator,
    pub max_inflight: usize,
    pub crc: bool,
    pub sent: ByteStats,
    pub dropped_stale: u64,
}

impl SendQueue {
    pub fn new(src: u32) -> Self {
        Self {
            src,
            next_seq: 1,
            queues: Default::default(),
            inflight: BTreeMap::new(),
            rtt: RttEstimator::default(),
            max_inflight: 64,
            crc: false,
            sent: ByteStats::default(),
            dropped_stale: 0,
        }
    }

    pub fn next_seq(&mut self) -> u32 {
        let s = self.next_seq;
        self.next_seq = self.next_seq.wrapping_add(1).max(1);
        s
    }

    /// Build a control frame (ack, ping, ...) that bypasses the queue.
    pub fn control(&mut self, kind: MsgType, payload: Vec<u8>) -> Frame {
        let seq = self.next_seq();
        let mut f = Frame::new(kind, seq, self.src, payload);
        if self.crc {
            f = f.with_crc();
        }
        self.sent.record(&f, f.wire_len());
        f
    }

    /// Queue a message with its default priority. Returns its sequence number.
    pub fn push(&mut self, kind: MsgType, payload: Vec<u8>, dst: Option<u32>) -> u32 {
        self.push_with(kind, kind.default_priority(), payload, dst, None)
    }

    pub fn push_with(&mut self, kind: MsgType, prio: Priority, payload: Vec<u8>, dst: Option<u32>, expires_ms: Option<u64>) -> u32 {
        let seq = self.next_seq();
        let mut f = Frame::new(kind, seq, self.src, payload);
        f.prio = prio;
        f.dst = dst;
        if self.crc {
            f = f.with_crc();
        }
        self.queues[prio as usize].push_back(Queued { frame: f, expires_ms });
        seq
    }

    /// Drop expired frames at the head of one class. During a contact gap
    /// nothing is polled, so the sender calls this when it queues a new
    /// sample: a long outage keeps at most one TTL's worth of vitals in memory.
    /// O(expired) because frames of a class are queued in expiry order.
    pub fn purge_stale(&mut self, prio: Priority, now_ms: u64) {
        let q = &mut self.queues[prio as usize];
        while q.front().is_some_and(|x| x.expires_ms.is_some_and(|e| e <= now_ms)) {
            q.pop_front();
            self.dropped_stale += 1;
        }
    }

    /// Frames to put on the wire now, highest priority first, within `budget` bytes.
    /// Due retransmissions of a class go before new frames of the same class.
    pub fn poll(&mut self, now_ms: u64, budget: usize) -> Vec<Frame> {
        let mut out = Vec::new();
        let mut left = budget;
        for prio in Priority::ALL {
            // 1. due retransmits of this class
            let due: Vec<u32> = self
                .inflight
                .iter()
                .filter(|(_, i)| i.frame.prio == prio && i.deadline_ms <= now_ms)
                .map(|(s, _)| *s)
                .collect();
            if !due.is_empty() {
                self.rtt.on_timeout();
            }
            let rto = self.rtt.rto_ms();
            for seq in due {
                let inf = self.inflight.get_mut(&seq).expect("inflight");
                let len = inf.frame.wire_len();
                if len > left {
                    return out;
                }
                left -= len;
                inf.retries += 1;
                inf.frame.flags |= FLAG_RETX;
                // the estimator is already backed off; each retry of this frame doubles again
                let backoff = rto.saturating_mul(1u64 << inf.retries.saturating_sub(1).min(6)).min(self.rtt.max_rto_ms);
                inf.deadline_ms = now_ms + backoff;
                self.sent.record(&inf.frame, len);
                out.push(inf.frame.clone());
            }
            // 2. new frames of this class
            while let Some(q) = self.queues[prio as usize].front() {
                if q.expires_ms.is_some_and(|e| e <= now_ms) {
                    self.queues[prio as usize].pop_front();
                    self.dropped_stale += 1;
                    continue;
                }
                let reliable = q.frame.is_reliable();
                if reliable && self.inflight.len() >= self.max_inflight {
                    break;
                }
                let len = q.frame.wire_len();
                if len > left {
                    return out;
                }
                left -= len;
                let q = self.queues[prio as usize].pop_front().expect("front");
                self.sent.record(&q.frame, len);
                if reliable {
                    self.inflight.insert(q.frame.seq, Inflight { frame: q.frame.clone(), first_sent_ms: now_ms, retries: 0, deadline_ms: now_ms + rto });
                }
                out.push(q.frame);
            }
        }
        out
    }

    /// Process a selective ack. Returns the seqs that were in flight (newly acknowledged).
    /// RTT is sampled only from frames never retransmitted (Karn's rule).
    pub fn on_ack(&mut self, seqs: &[u32], now_ms: u64) -> Vec<u32> {
        let mut done = Vec::new();
        for s in seqs {
            if let Some(inf) = self.inflight.remove(s) {
                if inf.retries == 0 {
                    self.rtt.sample(now_ms.saturating_sub(inf.first_sent_ms) as f64);
                }
                done.push(*s);
            }
        }
        done
    }

    /// The receiver could not use `seq`: retransmit it on the next poll.
    pub fn on_nack(&mut self, seq: u32, now_ms: u64) {
        if let Some(inf) = self.inflight.get_mut(&seq) {
            inf.deadline_ms = now_ms;
        }
    }

    pub fn depth(&self) -> QueueDepth {
        QueueDepth {
            alarm: self.queues[0].len(),
            guidance: self.queues[1].len(),
            vitals: self.queues[2].len(),
            logs: self.queues[3].len(),
            inflight: self.inflight.len(),
        }
    }
    pub fn is_idle(&self) -> bool {
        self.inflight.is_empty() && self.queues.iter().all(|q| q.is_empty())
    }
    pub fn inflight_seqs(&self) -> Vec<u32> {
        self.inflight.keys().copied().collect()
    }
}

/// How many delivered reliable seqs a receiver remembers for dedup. A replay
/// older than this is caught one level up (the ground's mirror drops a record
/// it already has by chain seq + hash).
pub const DEDUP_WINDOW: usize = 65_536;

/// Receiver-side helper: remembers which reliable seqs were already delivered
/// (retransmits after a lost ack, or replayed frames, must not create
/// duplicates) and batches acks. O(1) per frame.
#[derive(Debug, Default, Clone)]
pub struct Receiver {
    seen: HashSet<u32>,
    order: VecDeque<u32>,
    pending_acks: Vec<u32>,
    pub duplicates: u64,
}

impl Receiver {
    /// Returns true if this frame is new and should be processed.
    pub fn accept(&mut self, f: &Frame) -> bool {
        if !f.is_reliable() {
            return true;
        }
        if self.pending_acks.len() < 4_096 {
            self.pending_acks.push(f.seq);
        }
        if !self.seen.insert(f.seq) {
            self.duplicates += 1;
            return false;
        }
        self.order.push_back(f.seq);
        if self.order.len() > DEDUP_WINDOW {
            if let Some(old) = self.order.pop_front() {
                self.seen.remove(&old);
            }
        }
        true
    }
    pub fn take_acks(&mut self) -> Vec<u32> {
        std::mem::take(&mut self.pending_acks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alarm_overtakes_backlog() {
        let mut q = SendQueue::new(1);
        for i in 0..50 {
            q.push(MsgType::Record, vec![i; 20], None);
        }
        q.push(MsgType::Vitals, vec![1; 5], None);
        let alarm = q.push(MsgType::Alarm, b"no pulse".to_vec(), None);
        let out = q.poll(0, 40);
        assert_eq!(out[0].seq, alarm, "alarm goes first");
        assert_eq!(out[0].kind, MsgType::Alarm);
    }

    #[test]
    fn retransmits_until_acked_with_backoff() {
        let mut q = SendQueue::new(1);
        let s = q.push(MsgType::Record, vec![0; 10], None);
        assert_eq!(q.poll(0, 1_000).len(), 1);
        assert!(q.poll(500, 1_000).is_empty(), "not due before RTO (1 s default)");
        let r = q.poll(1_000, 1_000);
        assert_eq!(r.len(), 1);
        assert!(r[0].is_retransmit());
        assert!(q.poll(2_000, 1_000).is_empty(), "backoff doubled");
        assert_eq!(q.on_ack(&[s], 2_100), vec![s]);
        assert!(q.is_idle());
        assert_eq!(q.rtt.samples, 0, "Karn: no RTT sample from a retransmitted frame");
        assert_eq!(q.rtt.rto_ms(), 2_000, "RFC 6298 5.5: the RTO stays backed off until a valid sample");
    }

    #[test]
    fn slow_link_learns_its_rtt_instead_of_retransmitting_forever() {
        // RTT 2.5 s, longer than the initial 1 s RTO; a new frame every second
        let mut q = SendQueue::new(1);
        let rtt = 2_500u64;
        let mut first_sent: std::collections::HashMap<u32, u64> = Default::default();
        let mut retx = 0;
        for t in (0..120_000u64).step_by(100) {
            if t % 1_000 == 0 {
                q.push(MsgType::Record, vec![0; 20], None);
            }
            for f in q.poll(t, 10_000) {
                if f.is_retransmit() {
                    retx += 1;
                }
                first_sent.entry(f.seq).or_insert(t);
            }
            let due: Vec<u32> = first_sent.iter().filter(|(_, at)| **at + rtt == t).map(|(s, _)| *s).collect();
            q.on_ack(&due, t);
        }
        assert!(q.rtt.samples > 50, "clean samples once the RTO backed off: {}", q.rtt.samples);
        assert!(retx < 20, "{retx} retransmissions for 120 frames");
        assert!(q.rtt.rto_ms() >= rtt);
    }

    #[test]
    fn rtt_sampled_from_clean_ack() {
        let mut q = SendQueue::new(1);
        let s = q.push(MsgType::Text, b"hi".to_vec(), None);
        q.poll(10, 1_000);
        q.on_ack(&[s], 260);
        assert_eq!(q.rtt.samples, 1);
        assert!((q.rtt.srtt_ms - 250.0).abs() < 0.01);
        assert!(q.rtt.rto_ms() >= 250);
    }

    #[test]
    fn budget_zero_stores_and_forwards() {
        let mut q = SendQueue::new(1);
        for _ in 0..10 {
            q.push(MsgType::Record, vec![0; 30], None);
        }
        assert!(q.poll(0, 0).is_empty(), "link down: nothing leaves");
        assert_eq!(q.depth().logs, 10);
        let out = q.poll(1, 10_000);
        assert_eq!(out.len(), 10, "link back: everything leaves in order");
        assert!(out.windows(2).all(|w| w[0].seq < w[1].seq));
    }

    #[test]
    fn stale_vitals_are_dropped_not_sent() {
        let mut q = SendQueue::new(1);
        q.push_with(MsgType::Vitals, Priority::Vitals, vec![1; 6], None, Some(5_000));
        assert!(q.poll(6_000, 1_000).is_empty());
        assert_eq!(q.dropped_stale, 1);
    }

    #[test]
    fn long_outage_keeps_at_most_one_ttl_of_vitals() {
        let mut q = SendQueue::new(1);
        // one sample a second for an hour, nothing polled (link down)
        for t in (0..3_600_000u64).step_by(1_000) {
            q.purge_stale(Priority::Vitals, t);
            q.push_with(MsgType::Vitals, Priority::Vitals, vec![1; 6], None, Some(t + 30_000));
        }
        assert!(q.depth().vitals <= 31, "{} vitals kept", q.depth().vitals);
        assert_eq!(q.dropped_stale as usize, 3_600 - q.depth().vitals);
        q.purge_stale(Priority::Vitals, 3_700_000);
        assert_eq!(q.depth().vitals, 0);
    }

    #[test]
    fn nack_triggers_immediate_retransmit() {
        let mut q = SendQueue::new(1);
        let s = q.push(MsgType::Guidance, b"CPR".to_vec(), None);
        q.poll(0, 100);
        q.on_nack(s, 10);
        assert_eq!(q.poll(10, 100).len(), 1);
    }

    #[test]
    fn receiver_dedups_and_acks_duplicates() {
        let mut r = Receiver::default();
        let f = Frame::new(MsgType::Record, 7, 2, vec![]);
        assert!(r.accept(&f));
        assert!(!r.accept(&f), "duplicate is not processed twice");
        assert_eq!(r.take_acks(), vec![7, 7], "but is acked again (the first ack may be lost)");
    }
}
