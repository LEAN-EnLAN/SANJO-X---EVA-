# ADR-014 · Supabase for the Earth / cloud side

- **Status:** accepted · 15/11/2026 · Claude (for Sanjo-X)
- **Supersedes:** the planned Firebase setup (Firebase kit, `ADR-013-firebase.md`, never merged into this repo)
- **Extends:** ADR-005 (data and chain), ADR-010 (Ed25519-signed chain), ADR-011 (station), ADR-012 (topic guard)
- **Code:** `supabase/` (migrations, RLS tests, Edge Function `eva-chat`, seed, provisioning); `supabase/README.md`

## Context

The ship runs offline-first: the hub, the wristbands, the station and the on-board PWA never depend on the internet, and every health event becomes a hash-chained record signed with the ship's Ed25519 key (ADR-010) that the ground node verifies, mirrors and acknowledges. On Earth the team still wants a cloud side: a copy of what the ground has verified (for mission control and the web demo), sign-in with roles, team files, and a server-side proxy so Eva's web chat never exposes the Groq key.

The Firebase kit designed all of this (data model, rules, ingest plan, `evaChat`). Setting it up proved cumbersome for a hackathon team: a Google Cloud project per environment, the Blaze plan with billing for any Cloud Function, Secret Manager and App Check wiring, an emulator suite to test rules, and a rules language that cannot check what we care about most (the chain's hash links).

## Decision

Use **Supabase** (project `eva-wearmeds`, `sa-east-1` São Paulo, free plan) for the Earth / cloud side.

- **Postgres + Row Level Security** replace Firestore + rules. The Firebase data model maps one to one: `ships`, `records`, `cases` (+ `case_notes`), `passengers`, `station_state`, plus `profiles`, `journal_entries` and `team_docs`. Roles (`crew_lead`, `mission_control`, `team`, `viewer`) live in `profiles.role`, assigned only by an admin RPC or the service role; the same read rules apply (health data for mission control and the crew lead; the team only on demo ships; journals only for their owner).
- **The signed chain mirror is append-only in the database itself**: triggers refuse UPDATE, DELETE and TRUNCATE for every role, inserts are reserved to the service role (the ground ingest), and an insert must link to the previous record's hash. Firestore rules could not check the link. Anyone can still re-verify the cloud copy with the app's own verifier (`supabase/scripts/export-chain.mjs` → `eva verify`).
- **Edge Function `eva-chat`** (Deno) replaces `evaChat`: the same topic guard as the site's Vercel function (ADR-012 rules, Spanish vocabulary, the stricter "help me" rule), a valid Supabase JWT required, a persistent per-user rate limit, `GROQ_API_KEY` as a function secret. Runs on the free plan.
- **Storage**: a private `team-assets` bucket and a public `public-media` bucket, with policies by role.
- **Everything is SQL in the repo** (migrations, an 80-check RLS test that runs on a plain local PostgreSQL without Docker), so the setup is reviewable and reproducible with one script (`supabase/scripts/provision.sh`).

## Invariants (unchanged from the Firebase plan)

1. **The ship stays offline-first.** No Supabase client, key or call in the ship role, the on-board PWA, the wristbands or the station. Losing Supabase (or the ground's internet) changes nothing on board.
2. **Acknowledgement comes only from the ground node's mirror** (PROTOCOL.md §7), never from the cloud. Supabase holds a copy of records that are already acknowledged.
3. **The cloud never rewrites history.** Records are write-once; a conflict on `(ship, epoch, seq)` with a different hash is an alarm, not an overwrite.
4. **Minimum health data.** Passengers are a triage summary (first name + initial, group, badge); no vitals series, ECG, journal, meals or chats leave the ship. Demo data is fictional.
5. **No secret in the repo, the browser or a log.** The anon key is public by design; the service-role key, the DB password and the Groq key stay in environment / secret stores.

## Consequences

- One free project, no billing account, no Google Cloud project. The org's free tier allows two active projects per owner: creating `eva-wearmeds` needs a free slot.
- Google sign-in needs an OAuth client from Google Cloud (manual, steps in `supabase/README.md`); email/password works now.
- Free-plan limits apply (built-in mailer rate limit, projects pause after a week idle, no PITR, no leaked-password protection). Fine for the demo; revisit for a real mission.
- The ground ingest (`app/src/cloud.rs`, INTEGRATION plan part A) will write with the service role over PostgREST (`insert … on conflict do nothing`, then compare hashes) instead of Firestore `batchWrite`. Not built yet; nothing in `app/` changes with this ADR.
- Data stays in São Paulo; the jurisdiction notes of the Firebase plan (Ley 25.326, LGPD, GDPR) still apply before any real passenger's data goes in.
