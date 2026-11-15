# ADR-002 · Near space, not deep space

- **Status:** accepted · 15/11/2026 · decided by the team lead, written by Claude
- **Supersedes:** the deep-space framing of the earlier concept ("3 to 22 min per leg", "Earth delay 12 min 40 s" in the wireframes)

## Context

The team said it plainly: *"we will never reach deep space"*. Eva's market is commercial flights in **near space**: low Earth orbit, stations, at most cislunar. That changes the telecom problem completely:

| | Deep space (interplanetary) | **Near space (our case)** |
|---|---|---|
| One-way latency | 3 to 22 **minutes** | hundreds of ms to a few **seconds** (relay hops, ground station queues) |
| Link availability | scheduled passes, long blackouts | **intermittent**: contact gaps while a relay or station is below the horizon, handovers, solar events |
| Bandwidth | very low, very expensive | **constrained and shared** (tens of kbit/s for health data on a shared downlink) |
| Protocol family | DTN / Bundle Protocol, custody transfer over hours | ordinary request/ack with retransmit, plus store-and-forward for gaps |
| What "real time" means | impossible; autonomy is mandatory | the ground *can* follow a case live, seconds behind |

## Decision

1. Design **Eva Link** (ADR-003) for *seconds of latency, intermittent links, small bandwidth*: priority classes, selective acks, RFC 6298 retransmission timers (RTO adapts to measured RTT), store-and-forward queue that never drops records, latest-wins vitals with TTL.
2. **Do not** implement DTN/Bundle Protocol, custody transfer or hour-scale scheduling. They solve a problem we do not have and would cost complexity and bytes.
3. The simulated **near-space profile** is: 1.2 s one-way + up to 0.8 s jitter, 64 kbit/s, 2% loss, a 20 s contact gap every 2 min (real gaps are longer and rarer; scaled down so a 3-minute demo shows one). A **storm** profile degrades it (2.5 s, 9.6 kbit/s, 15% loss, longer gaps).
4. Eva still works **fully on board without the ground** (offline-first): layer 1 rules, the hash chain and the crew dashboard never depend on the link. The link only carries records and lets the ground follow along.
5. Product copy talks about *ground*, *seconds* and *contact gaps*, not *Earth delay of minutes*. The dashboard KPI is "Ground delay · 2.7 s one way" (measured), the ground view shows "Ship time" and "Ground received" side by side.

## Consequences

- The ground can answer during a case (flight surgeon note → reliable Text frame to the ship).
- Timers are seconds, not minutes: initial RTO 1 s, max 60 s backoff; vitals TTL 30 s.
- If a deep-space mission ever comes back on the table, the seams are the `Endpoint` and the record queue: a bundle layer could wrap them, but nothing in the current design blocks that.
- Wireframe copy that says "minutes behind" must be updated as screens are ported (open item).
