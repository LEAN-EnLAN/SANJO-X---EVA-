# ADR-014 · Supabase for the Earth / cloud side

- **Status:** accepted · 15/11/2026 · Claude (for Sanjo-X)
- **Supersedes:** the planned Firebase setup (Firebase kit, `ADR-013-firebase.md`, never merged into this repo)
- **Extends:** ADR-005 (data and chain), ADR-010 (Ed25519-signed chain), ADR-011 (station), ADR-012 (topic guard)
- **Code:** `supabase/` (migrations, RLS tests, Edge Function `eva-chat`, seed, provisioning); `supabase/README.md`

## Context

The ship runs offline-first: the hub, the wristbands, the station and the on-board PWA never depend on the internet, and every health event becomes a hash-chained record signed with the ship's Ed25519 key (ADR-010) that the ground node verifies, mirrors and acknowledges. On Earth the team still wants a cloud side: a copy of what the ground has verified (for mission control and the web demo), sign-in with roles, team files, and a server-side proxy so Eva's web chat never exposes the Groq key.

The Firebase kit designed all of this (data model, rules, ingest plan, `evaChat`). Setting it up proved cumbersome for a hackathon team: a Google Cloud project per environment, the Blaze plan with billing for any Cloud Function, Secret Manager and App Check wiring, an emulator suite to test rules, and a rules language that cannot check what we care about most (the chain's hash links).

## Decision

Use **Supabase** for the Earth / cloud side, in the existing free project **`changuito`** (ref `rcxmvcxyrwufmoylbrrz`, us-east-1), isolated in an **`eva` schema** (see "Shared project" below).

- **Postgres + Row Level Security** replace Firestore + rules. The Firebase data model maps one to one: `ships`, `records`, `cases` (+ `case_notes`), `passengers`, `station_state`, plus `profiles`, `journal_entries` and `team_docs`. Roles (`crew_lead`, `mission_control`, `team`, `viewer`) live in `profiles.role`, assigned only by an admin RPC or the service role; the same read rules apply (health data for mission control and the crew lead; the team only on demo ships; journals only for their owner).
- **The signed chain mirror is append-only in the database itself**: triggers refuse UPDATE, DELETE and TRUNCATE for every role, inserts are reserved to the service role (the ground ingest), and an insert must link to the previous record's hash. Firestore rules could not check the link. Anyone can still re-verify the cloud copy with the app's own verifier (`supabase/scripts/export-chain.mjs` → `eva verify`).
- **Edge Function `eva-chat`** (Deno) replaces `evaChat`: the same topic guard as the site's Vercel function (ADR-012 rules, Spanish vocabulary, the stricter "help me" rule), a valid Supabase JWT required, a persistent per-user rate limit, `GROQ_API_KEY` as a function secret. Runs on the free plan.
- **Storage**: a private `eva-team-assets` bucket and a public `eva-public-media` bucket, with policies by role.
- **Everything is SQL in the repo** (migrations, an 80-check RLS test that runs on a plain local PostgreSQL without Docker), so the setup is reviewable and reproducible with one script (`supabase/scripts/provision.sh`).

## Shared project (amended 15/11/2026)

A dedicated project `eva-wearmeds` (sa-east-1) was the first choice, but the org owner's free tier was full (two active free projects; Supabase refused with `free_project_limit`). The team chose to put Eva inside `changuito` rather than pause a project or pay. Rules for living there:

- **Everything Eva owns is in the `eva` schema** and named `eva-…` outside it: buckets `eva-team-assets` / `eva-public-media`, storage policies `eva-…` (each scoped to an Eva bucket), Edge Function `eva-chat`, function secret `EVA_GROQ_API_KEY`. Migrations are logged in `eva.schema_migrations`, not in the project's `supabase_migrations`.
- **No hooks on shared objects**: no trigger on `auth.users` (profiles come from `eva.ensure_profile()` on first use), no event trigger. No Eva foreign key or trigger can block deleting a changuito user.
- **Project-wide settings are only appended to**: `eva` joins the Data API's exposed schemas; the Eva URLs join the auth redirect allow-list. Site URL, providers, sign-ups, SMTP and JWT settings stay changuito's. Anything else project-wide needs the team's decision first.
- **Shared fate**: both apps share the auth user pool, the JWT secret, quotas, pausing and the project owner. A changuito user can sign in to Eva (and gets only a `viewer` profile); an Eva test user exists in changuito's user list (`eva-test-*@example.com`).
- `provision.sh` inspects the project first and stops on any name collision.

### Moving Eva to a dedicated project later

1. Create the project (free slot or paid), then run `provision.sh` with `EVA_SB_REF=<new ref>`. The migrations are self-contained, so the `eva` schema can stay as is.
2. Copy the data: `pg_dump --schema=eva --data-only` from changuito, restore into the new project as the service role (records keep their hashes and signatures; `eva verify` on an export proves nothing changed). Recreate users (or ask them to sign up again) and remap `eva.profiles.id` by email; move bucket objects with the Storage API.
3. Point clients at the new URL and anon key (Vercel env `SUPABASE_URL`, `SUPABASE_ANON_KEY`), redeploy `eva-chat` there.
4. Clean up changuito: delete the `eva-chat` function and the `EVA_GROQ_API_KEY` secret, empty and delete the `eva-*` buckets, drop the `eva-*` storage policies, `drop schema eva cascade`, remove `eva` from the exposed schemas and the Eva URLs from the redirect list, delete the `eva-test-*` users.

## Invariants (unchanged from the Firebase plan)

1. **The ship stays offline-first.** No Supabase client, key or call in the ship role, the on-board PWA, the wristbands or the station. Losing Supabase (or the ground's internet) changes nothing on board.
2. **Acknowledgement comes only from the ground node's mirror** (PROTOCOL.md §7), never from the cloud. Supabase holds a copy of records that are already acknowledged.
3. **The cloud never rewrites history.** Records are write-once; a conflict on `(ship, epoch, seq)` with a different hash is an alarm, not an overwrite.
4. **Minimum health data.** Passengers are a triage summary (first name + initial, group, badge); no vitals series, ECG, journal, meals or chats leave the ship. Demo data is fictional.
5. **No secret in the repo, the browser or a log.** The anon key is public by design; the service-role key, the DB password and the Groq key stay in environment / secret stores.

## Consequences

- No new project, no billing account, no Google Cloud project; Eva rides on changuito's free project and shares its limits.
- Sign-in uses changuito's auth configuration (email/password and Google are already enabled there).
- Free-plan limits apply (built-in mailer rate limit, projects pause after a week idle, no PITR, no leaked-password protection). Fine for the demo; revisit for a real mission.
- The ground ingest (`app/src/cloud.rs`, INTEGRATION plan part A) will write with the service role over PostgREST (`insert … on conflict do nothing`, then compare hashes) instead of Firestore `batchWrite`. Not built yet; nothing in `app/` changes with this ADR.
- Data sits in us-east-1 while Eva shares changuito; the jurisdiction notes of the Firebase plan (Ley 25.326, LGPD, GDPR) still apply before any real passenger's data goes in.
