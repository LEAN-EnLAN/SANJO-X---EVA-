//! Break tests for Eva Link: fuzzing, truncated / oversized / replayed frames,
//! wrong CRC, duplicate and out-of-order sequence numbers, a 10-minute total
//! outage, and clock skew between ship and ground. Results: docs/app/TESTING.md.

use eva::chain::Chain;
use eva::link::{Inserted, Mirror};
use eva::protocol::delta::{DeltaDecoder, DeltaEncoder, VitalsSample};
use eva::protocol::frame::{decode_batch, encode_batch, Frame, MsgType};
use eva::protocol::payload::{self, Alarm, Hello, RecordMsg, Role, Text, Welcome};
use eva::protocol::profile::{LinkProfile, LinkSim, Rng};
use eva::protocol::wire::WireError;
use eva::protocol::{Delivered, Endpoint};
use serde_json::json;

/// Every decoder in the crate, fed the same bytes. Must never panic.
fn feed_everything(b: &[u8], ep: &mut Endpoint, dec: &mut DeltaDecoder, mirror: &mut Mirror) {
    let _ = decode_batch(b);
    let _ = Frame::decode(b);
    let _ = Hello::decode(b);
    let _ = Welcome::decode(b);
    let _ = payload::decode_time(b);
    let _ = payload::decode_ack(b);
    let _ = payload::decode_nack(b);
    let _ = Alarm::decode(b);
    let _ = Text::decode(b);
    let _ = payload::decode_presence(b);
    let _ = dec.decode(b);
    let _ = ep.incoming(1_000, b);
    let _ = ep.outgoing(1_000, 10_000, 1_024);
    if let Ok(r) = RecordMsg::decode(b) {
        mirror.insert(r, "ship", 1);
    }
}

fn valid_corpus() -> Vec<Vec<u8>> {
    let mut enc = DeltaEncoder::new(10);
    let v = VitalsSample { hr: 72, spo2: 98, resp: 14, skin_c10: 344, eda_c: 210, motion: 12, quality: 95, base_batt: 66, screen_batt: 71, flags: 8 };
    let kf = enc.encode(4, 1_000, &v);
    let d = enc.encode(4, 1_001, &VitalsSample { hr: 75, ..v });
    let rec = RecordMsg { chain_seq: 3, ts_ms: 9, kind: "alarm".into(), subject: "4B".into(), hash8: [1; 8], body: br#"{"body":{"text":"x"},"prev":"00"}"#.to_vec(), sig: vec![2; 64] };
    let frames = vec![
        Frame::new(MsgType::Hello, 1, 3, Hello { pubkey: vec![9; 32], ..Hello::new(Role::Ship, "ship · ship", 77) }.encode()),
        Frame::new(MsgType::Ping, 2, 3, payload::encode_time(123_456)),
        Frame::new(MsgType::Ack, 3, 0, payload::encode_ack(&[5, 6, 7, 9, 400])),
        Frame::new(MsgType::Vitals, 4, 3, kf).with_crc(),
        Frame::new(MsgType::Vitals, 5, 3, d),
        Frame::new(MsgType::Alarm, 6, 3, Alarm { subject: "4B".into(), code: 1, text: "no pulse".into() }.encode()).to(0),
        Frame::new(MsgType::Record, 7, 1, rec.encode()).with_crc(),
        Frame::new(MsgType::Presence, 8, 0, payload::encode_presence(&[payload::PresenceEntry { id: 1, role: Role::Dash, name: "Mara".into(), rtt_ms: 3 }])),
        Frame::new(MsgType::Nack, 9, 0, payload::encode_nack(4, payload::NackReason::NeedKeyframe, 4)),
    ];
    let mut out: Vec<Vec<u8>> = frames.iter().map(|f| f.encode()).collect();
    out.push(encode_batch(&frames));
    out
}

#[test]
fn fuzz_decoders_never_panic() {
    let mut ep = Endpoint::new(0).with_crc(true);
    let mut dec = DeltaDecoder::new();
    let mut mirror = Mirror::default();
    let corpus = valid_corpus();
    let seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(7);
    for (round, seed) in [0xE7A_F022u64, seed].into_iter().enumerate() {
        let mut rng = Rng::new(seed);
        let n = if round == 0 { 150_000 } else { 50_000 };
        for i in 0..n {
            let b: Vec<u8> = if i % 2 == 0 {
                // pure random bytes, biased towards plausible headers
                let len = rng.range(80) as usize;
                let mut b: Vec<u8> = (0..len).map(|_| rng.next_u64() as u8).collect();
                if !b.is_empty() && rng.range(1) == 0 {
                    b[0] = (1 << 5) | (b[0] & 0x1f);
                }
                if b.len() > 1 && rng.range(1) == 0 {
                    b[1] = (b[1] & 0x60) | (rng.range(12) as u8);
                }
                b
            } else {
                // mutate a valid datagram: flips, truncation, insertion, varint bombs
                let mut b = corpus[rng.range(corpus.len() as u64 - 1) as usize].clone();
                for _ in 0..=rng.range(4) {
                    if b.is_empty() {
                        break;
                    }
                    let at = rng.range(b.len() as u64 - 1) as usize;
                    match rng.range(4) {
                        0 => b[at] ^= 1 << rng.range(7),
                        1 => b.truncate(at),
                        2 => b.insert(at, rng.next_u64() as u8),
                        3 => b.splice(at..at, [0xff; 10]).for_each(drop),
                        _ => b[at] = 0x80,
                    }
                }
                b
            };
            feed_everything(&b, &mut ep, &mut dec, &mut mirror);
        }
        println!("fuzz round {round}: seed {seed:#x}, {n} inputs, {} corrupt datagrams counted", ep.corrupt);
    }
}

#[test]
fn every_truncation_of_a_valid_batch_is_rejected() {
    for b in valid_corpus() {
        for cut in 0..b.len() {
            let r = decode_batch(&b[..cut]);
            if cut == 0 {
                assert_eq!(r, Ok(vec![]), "empty datagram is an empty batch");
            } else if let Ok(frames) = r {
                // a cut exactly between two frames of a batch is a shorter, valid batch
                assert_eq!(encode_batch(&frames), b[..cut].to_vec());
            }
        }
    }
}

#[test]
fn oversized_lengths_are_rejected_without_allocating() {
    // header says the payload is 2^40 bytes long
    let mut b = vec![(1 << 5), MsgType::Text as u8, 1, 1];
    b.extend([0x80, 0x80, 0x80, 0x80, 0x80, 0x20]);
    assert_eq!(decode_batch(&b), Err(WireError::Truncated));
    // varint longer than 64 bits
    let mut b = vec![(1 << 5), MsgType::Ping as u8];
    b.extend([0xff; 11]);
    assert_eq!(decode_batch(&b), Err(WireError::VarintOverflow));
    // an ack claiming a million entries
    let mut w = eva::protocol::wire::Writer::new();
    w.uvar(1_000_000);
    assert!(payload::decode_ack(&w.buf).is_err());
    // a presence list claiming a billion nodes only reads what is there
    let mut w = eva::protocol::wire::Writer::new();
    w.uvar(1_000_000_000);
    assert!(payload::decode_presence(&w.buf).is_err());
}

#[test]
fn wrong_crc_is_always_detected() {
    let f = Frame::new(MsgType::Record, 812, 1, RecordMsg { chain_seq: 1, ts_ms: 2, kind: "alarm".into(), subject: "4B".into(), hash8: [7; 8], body: b"{}".to_vec(), sig: vec![1; 64] }.encode()).with_crc();
    let good = f.encode();
    let mut ep = Endpoint::new(0).with_crc(true);
    for i in 0..good.len() {
        for bit in 0..8 {
            let mut b = good.clone();
            b[i] ^= 1 << bit;
            assert!(decode_batch(&b).is_err(), "flip at byte {i} bit {bit} slipped through");
            assert_eq!(ep.incoming(0, &b), vec![Delivered::Corrupt]);
        }
    }
    assert_eq!(ep.corrupt as usize, good.len() * 8);
}

#[test]
fn replayed_duplicate_and_out_of_order_frames_are_delivered_once() {
    let mut ship = Endpoint::new(1);
    ship.queue.max_inflight = 1_000;
    let mut ground = Endpoint::new(0);
    let seqs: Vec<u32> = (0..200).map(|i| ship.send(MsgType::Text, Text { from: "ship".into(), to: "ground".into(), text: format!("m{i}") }.encode(), None)).collect();
    let mut datagrams = Vec::new();
    for f in ship.queue.poll(0, usize::MAX) {
        datagrams.push(f.encode());
    }
    // shuffle, duplicate every datagram 3 times, replay the whole lot again later
    let mut rng = Rng::new(99);
    let mut wire: Vec<Vec<u8>> = datagrams.iter().flat_map(|d| std::iter::repeat_n(d.clone(), 3)).collect();
    for i in (1..wire.len()).rev() {
        wire.swap(i, rng.range(i as u64) as usize);
    }
    wire.extend(datagrams.iter().cloned());
    let mut delivered = Vec::new();
    for d in &wire {
        for ev in ground.incoming(0, d) {
            if let Delivered::Frame(f) = ev {
                delivered.push(f.seq);
            }
        }
    }
    delivered.sort_unstable();
    assert_eq!(delivered, seqs, "each reliable frame delivered exactly once");
    assert_eq!(ground.rx.duplicates as usize, wire.len() - seqs.len());
    // every copy is acked (the first ack may be lost), and the sender settles
    let acks = ground.outgoing(1, usize::MAX, 65_000);
    for a in acks {
        ship.incoming(1, &a);
    }
    assert!(ship.queue.is_idle());
}

fn record_msgs(c: &Chain) -> Vec<RecordMsg> {
    c.records
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let mut w = json!({"body": r.body});
            if i == 0 {
                w["prev"] = json!(r.prev);
            }
            let mut h = [0u8; 8];
            h.copy_from_slice(&hex::decode(&r.hash).unwrap()[..8]);
            RecordMsg { chain_seq: r.seq, ts_ms: r.ts_ms, kind: r.kind.clone(), subject: r.subject.clone(), hash8: h, body: serde_json::to_vec(&w).unwrap(), sig: hex::decode(&r.sig).unwrap() }
        })
        .collect()
}

#[test]
fn replay_beyond_the_receiver_window_is_still_idempotent_at_the_mirror() {
    let mut c = Chain::in_memory("ship");
    c.append(1, "alarm", "4B", json!({"text": "No pulse"}));
    let msg = record_msgs(&c).remove(0);
    let mut m = Mirror::default();
    m.pin_key("ship", c.public_key()).unwrap();
    assert_eq!(m.insert(msg.clone(), "ship", 1), Inserted::New);
    // the same record replayed as a brand-new frame (new seq, e.g. a captured datagram re-sent days later)
    for _ in 0..5 {
        assert_eq!(m.insert(msg.clone(), "ship", 2), Inserted::Duplicate);
    }
    assert_eq!(m.records.len(), 1);
    assert!(m.is_verified(0));
}

/// The headline store-and-forward test: 10 minutes (mission time) of a 100%
/// outage while the ship keeps signing one record a second and streaming
/// vitals, then contact returns on the near-space profile. Every record must
/// reach the ground exactly once, verified (hash chain + Ed25519).
#[test]
fn ten_minute_total_outage_then_recovery_loses_no_record() {
    let mut chain = Chain::in_memory("ship");
    let mut mirror = Mirror::default();
    mirror.pin_key("ship", chain.public_key()).unwrap();
    let outage = LinkProfile::outage();
    let near = LinkProfile::near_space();
    let mut up = LinkSim::new(outage.clone(), 1);
    let mut down = LinkSim::new(outage.clone(), 2);
    let mut ship = Endpoint::new(1).with_crc(true);
    let mut ground = Endpoint::new(0).with_crc(true);
    let mut frame_to_chain = std::collections::HashMap::new();
    let mut acked = std::collections::HashSet::new();
    let mut max_depth = 0;
    let tick = 100u64;
    let outage_ms = 600_000u64;
    let mut anchor = true;
    let mut t = 0u64;
    let mut tokens = 0usize;
    let v = VitalsSample { hr: 70, spo2: 98, resp: 14, skin_c10: 340, eda_c: 200, motion: 10, quality: 95, base_batt: 70, screen_batt: 70, flags: 8 };
    loop {
        if t == outage_ms {
            up.set_profile(near.clone());
            down.set_profile(near.clone());
        }
        let p = if t < outage_ms { &outage } else { &near };
        // the ship keeps working during the outage: 1 record/s for 10 min, vitals for 15 bands
        if t % 1000 == 0 && t < outage_ms {
            let seq = chain.append(t, if t == 300_000 { "alarm" } else { "case" }, "4B", json!({"text": format!("step at {t}")}));
            let r = &chain.records[seq as usize];
            let mut w = json!({"body": r.body});
            if anchor {
                w["prev"] = json!(r.prev);
                anchor = false;
            }
            let mut h = [0u8; 8];
            h.copy_from_slice(&hex::decode(&r.hash).unwrap()[..8]);
            let msg = RecordMsg { chain_seq: r.seq, ts_ms: r.ts_ms, kind: r.kind.clone(), subject: r.subject.clone(), hash8: h, body: serde_json::to_vec(&w).unwrap(), sig: hex::decode(&r.sig).unwrap() };
            let prio = eva::uplink::priority_for(&r.kind);
            let fs = ship.queue.push_with(MsgType::Record, prio, msg.encode(), None, None);
            frame_to_chain.insert(fs, seq);
            for band in 1..=15 {
                ship.send_vitals(t, band, t / 1000, &v);
            }
        }
        if p.is_up(t) {
            tokens = (tokens + p.budget(tick)).min(p.mtu.max(256) * 4);
            for d in ship.outgoing(t, tokens, p.mtu.max(64)) {
                tokens = tokens.saturating_sub(d.len());
                up.send(t, d);
            }
            for d in ground.outgoing(t, usize::MAX, p.mtu.max(64)) {
                down.send(t, d);
            }
        } else {
            tokens = 0;
        }
        max_depth = max_depth.max(ship.queue.depth().queued() + ship.queue.depth().inflight);
        for d in up.recv(t) {
            for ev in ground.incoming(t, &d) {
                if let Delivered::Frame(f) = ev {
                    if let Ok(r) = RecordMsg::decode(&f.payload) {
                        mirror.insert(r, "ship", t);
                    }
                }
            }
        }
        for d in down.recv(t) {
            for ev in ship.incoming(t, &d) {
                if let Delivered::Acked(s) = ev {
                    for x in s {
                        if let Some(c) = frame_to_chain.get(&x) {
                            acked.insert(*c);
                        }
                    }
                }
            }
        }
        if t < outage_ms {
            assert!(mirror.records.is_empty(), "nothing crosses a 100% outage");
        }
        t += tick;
        if acked.len() == chain.len() || t > outage_ms + 900_000 {
            break;
        }
    }
    let n = chain.len();
    assert_eq!(n, 600);
    assert_eq!(acked.len(), n, "every record acknowledged after recovery");
    assert_eq!(mirror.records.len(), n, "every record mirrored on the ground");
    assert_eq!(mirror.verified.len(), n, "every record verified: hash chain + signature");
    assert!(mirror.failed.is_empty());
    assert!(ship.queue.dropped_stale > 0, "stale vitals were dropped instead of clogging the recovered link");
    println!("outage: queue peaked at {max_depth} frames; drained {:.0} s after contact returned; {} stale vitals dropped", (t - outage_ms) as f64 / 1000.0, ship.queue.dropped_stale);
}

/// Ship and ground clocks disagree (each process has its own mission clock;
/// here the ground is 1 h ahead, then the ship's clock jumps backwards).
/// Nothing in the link depends on synchronised clocks.
#[test]
fn clock_skew_between_ship_and_ground_is_harmless() {
    let skew = 3_600_000u64;
    let p = LinkProfile { loss: 0.0, jitter_ms: 0, ..LinkProfile::near_space() };
    let mut up = LinkSim::new(p.clone(), 3);
    let mut down = LinkSim::new(p.clone(), 4);
    let mut ship = Endpoint::new(1).with_crc(true);
    let mut ground = Endpoint::new(0).with_crc(true);
    let mut chain = Chain::in_memory("ship");
    let mut mirror = Mirror::default();
    mirror.pin_key("ship", chain.public_key()).unwrap();
    let mut rtts = Vec::new();
    let mut vitals = Vec::new();
    let mut enc_ts = 10_000u64;
    for t in (20_000u64..80_000).step_by(100) {
        let ship_now = t;
        let ground_now = t + skew;
        if t % 2_000 == 0 {
            ship.ping(ship_now);
            ground.ping(ground_now);
            // ship time jumps back 30 s halfway (clock reset): vitals ts goes backwards
            enc_ts = if t == 50_000 { enc_ts - 30 } else { enc_ts + 2 };
            ship.send_vitals(ship_now, 4, enc_ts, &VitalsSample { hr: 70 + (t / 2_000 % 5) as u16, spo2: 98, ..Default::default() });
            let seq = chain.append(ship_now, "case", "4B", json!({"t": t}));
            let msg = record_msgs(&chain).remove(seq as usize);
            ship.send(MsgType::Record, msg.encode(), None);
        }
        if p.is_up(t) {
            for d in ship.outgoing(ship_now, usize::MAX, 1024) {
                up.send(t, d);
            }
            for d in ground.outgoing(ground_now, usize::MAX, 1024) {
                down.send(t, d);
            }
        }
        for d in up.recv(t) {
            for ev in ground.incoming(ground_now, &d) {
                match ev {
                    Delivered::Frame(f) => {
                        mirror.insert(RecordMsg::decode(&f.payload).unwrap(), "ship", ground_now);
                    }
                    Delivered::Vitals { sample, .. } => vitals.push(sample.ts_s),
                    Delivered::Rtt(r) => rtts.push(("ground", r)),
                    _ => {}
                }
            }
        }
        for d in down.recv(t) {
            for ev in ship.incoming(ship_now, &d) {
                if let Delivered::Rtt(r) = ev {
                    rtts.push(("ship", r));
                }
            }
        }
    }
    assert!(!rtts.is_empty());
    for (side, r) in &rtts {
        assert!((2_400.0..3_000.0).contains(r), "{side} RTT {r} ms: measured on its own clock, skew does not leak in");
    }
    assert_eq!(mirror.verified.len(), chain.len(), "records verify whatever the clocks say");
    assert!(vitals.windows(2).any(|w| w[1] < w[0]), "a backwards ship clock still decodes (new keyframe)");
    assert!(ship.queue.rtt.rto_ms() < 10_000);
}
