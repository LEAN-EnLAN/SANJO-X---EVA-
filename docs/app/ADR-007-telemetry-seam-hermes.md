# ADR-007 · Seam for NASA dictionaries and Hermes (later, not now)

- **Status:** accepted (seam only) · 15/11/2026 · Claude

## Context

The team wants to adopt NASA telemetry dictionaries and the pattern of NASA's Hermes framework (JPL, Apache-2.0): *"the dictionary is the contract"* (XTCE). Not for this skeleton; we must not block it either. Team rule: patterns only, no copied code.

## Decision

Keep every telemetry path behind two narrow, documented entry points, so a dictionary-driven adapter can sit in front of them later:

1. **Eva Link `Vitals` frames** (`protocol/delta.rs`): fixed-point fields with names, units and order defined in one place (`VitalsSample`, PROTOCOL.md §6). That table is our proto-dictionary; an XTCE/Hermes dictionary would generate it (field ids, units, ranges) instead of hand-writing it.
2. **`POST /api/ingest/vitals`** (JSON by seat, `web/api.rs`): the adapter boundary for any external gateway. A Hermes-style ground station or a real band gateway posts here; overlays and Eva's rules apply unchanged.

Records already carry `kind` and `subject` strings; mapping them to dictionary packet/command names is a lookup table, not a redesign.

## Next steps (when the team picks this up)

- Write the WearMeds dictionary (XTCE or Hermes YAML) for the band's two parts: sensor base (pulse, SpO₂, breathing estimate, skin temperature, EDA, motion, signal quality, battery, LoRa) and interface (screen, battery, LoRa, 30 s ECG).
- Generate `VitalsSample` field order and ranges from it at build time (`build.rs`), keeping the wire format versioned (frame version byte).
- Validate `/api/ingest/vitals` against the dictionary's ranges.
