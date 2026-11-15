# ADR-008 · Mesh through the hub ("triangulation" server-side)

- **Status:** accepted · 15/11/2026 · Claude

## Context

The team asked for real-time, mesh-style ping and communication between app instances, with the binary in local-server mode: every client with the PWA talks to the others **through it**, fast and optimised, with server-side presence, routing, relay and RTT per client.

## Options

1. **WebRTC peer-to-peer**: true mesh, but needs signalling, STUN/ICE and much more code; browsers on a ship LAN gain little.
2. **Hub-and-spoke over one WebSocket per client** ✔: the hub is always on the ship anyway (it hosts the app).

## Decision

- Each open `/app` or `/dash` holds **one WebSocket to `/link`** speaking Eva Link (ADR-003). `Hello` registers it in the **mesh registry** (`mesh.rs`) with a node id, role, name and seat.
- **Presence:** the hub broadcasts the node list (id, role, name, RTT) every 2 s.
- **RTT per client:** the hub pings every node every 2 s and keeps an RFC 6298 estimator per node (shown on the dashboard's mesh card and in the dev panel). Clients also ping the hub every 5 s (shown in the app header).
- **Routing and relay:** a frame with `dst` ≠ 0 is relayed verbatim to that node (no re-encoding). End-to-end RTT between two apps = ping to their node id; the dev panel's "Ping every client end to end" does it for all.
- **Broadcast:** a Text frame without `dst` goes to every app and dashboard and is kept as a mesh message.
- The same `/link` also serves band gateways (`eva sim`, role Band) and the ship's uplink on a ground node (role Ship).

## Consequences

- Relay cost is one memory copy per frame; measured hub ↔ client RTT on localhost ≈ 1–2 ms.
- The hub is a single point of failure by design (it is also the app server); the apps keep working offline from the SW cache and queue posts.
- If true peer-to-peer is ever needed (two ships, no hub), node ids and `dst` routing already fit a multi-hop design.
