-- 2/6 · ship data mirrored from the ground node: ships, records (signed chain), cases,
-- case notes, passengers (summary only) and the medical station state.
-- Written only by the service role (the ground ingest). Model: docs/app/ADR-014-supabase.md,
-- translated from the Firebase kit's DATA-MODEL.md and firestore.rules.

-- ---------------------------------------------------------------- helpers

create or replace function public.jsonb_key_count(j jsonb)
returns integer
language sql immutable strict parallel safe
set search_path = ''
as $$
  select case when jsonb_typeof(j) = 'object' then (select count(*)::int from jsonb_object_keys(j)) else 0 end
$$;

-- ---------------------------------------------------------------- ships

create table if not exists public.ships (
  id             text primary key check (id ~ '^[a-z0-9-]{1,40}$'),
  display_name   text not null check (char_length(display_name) between 1 and 120),
  demo           boolean not null default false,
  pubkey         text not null check (pubkey ~ '^[0-9a-f]{64}$'),   -- pinned Ed25519 public key
  epoch          integer not null default 1 check (epoch between 0 and 9999),
  head_seq       bigint check (head_seq >= 0),
  head_hash      text check (head_hash ~ '^[0-9a-f]{64}$'),
  last_ingest_at timestamptz,
  notes          text check (notes is null or char_length(notes) <= 2000),
  created_at     timestamptz not null default now()
);

create or replace function public.ships_guard()
returns trigger
language plpgsql
set search_path = ''
as $$
begin
  if new.id <> old.id then raise exception 'ships: id is immutable' using errcode = '42501'; end if;
  if new.pubkey <> old.pubkey then raise exception 'ships: the pinned public key never changes (new ship id or admin action)' using errcode = '42501'; end if;
  if new.demo <> old.demo then raise exception 'ships: demo is fixed at creation' using errcode = '42501'; end if;
  if new.epoch < old.epoch then raise exception 'ships: epoch only moves forward' using errcode = '42501'; end if;
  return new;
end $$;
drop trigger if exists ships_guard on public.ships;
create trigger ships_guard before update on public.ships for each row execute function public.ships_guard();

-- Health data (records, cases, passengers): mission control and crew lead always; the team only
-- for ships flagged demo (fictional seed data). Same rule as firestore.rules canReadHealth().
create or replace function public.can_read_health(ship_id text)
returns boolean
language sql stable security definer
set search_path = ''
as $$
  select public.has_role('mission_control', 'crew_lead')
      or (public.has_role('team') and exists (select 1 from public.ships s where s.id = ship_id and s.demo))
$$;
revoke all on function public.can_read_health(text) from public;
grant execute on function public.can_read_health(text) to authenticated, service_role;

-- ---------------------------------------------------------------- records: append-only signed chain mirror

create table if not exists public.records (
  ship               text not null references public.ships (id),
  epoch              integer not null check (epoch between 0 and 9999),
  seq                bigint not null check (seq >= 0),
  ts                 bigint not null check (ts >= 0),                         -- chain::Record.ts_ms (mission time)
  kind               text not null check (kind ~ '^[a-z][a-z_.]{0,31}$'),
  subject            text not null check (char_length(subject) between 1 and 32),
  hash               text not null check (hash ~ '^[0-9a-f]{64}$'),
  prev               text not null check (prev ~ '^[0-9a-f]{64}$'),
  signature          text not null check (signature ~ '^[0-9a-f]{128}$'),    -- Ed25519("eva-record-v1\0" || hash)
  body               jsonb not null default '{}'::jsonb
                     check (jsonb_typeof(body) = 'object' and public.jsonb_key_count(body) <= 24),
  ground_received_ms bigint check (ground_received_ms >= 0),
  ingested_at        timestamptz not null default now(),
  primary key (ship, epoch, seq),
  unique (ship, hash)
);
comment on table public.records is
  'Mirror of the ship''s Ed25519-signed hash chain, as verified and acknowledged by the ground. Append-only; insert by service role only. Every row is acknowledged by definition.';

create index if not exists records_kind_idx on public.records (ship, kind, epoch, seq desc);
create index if not exists records_subject_idx on public.records (ship, subject, epoch, seq desc);

-- No UPDATE, no DELETE, no TRUNCATE, for anyone (service role and table owner included).
create or replace function public.records_append_only()
returns trigger
language plpgsql
set search_path = ''
as $$
begin
  raise exception 'records are append-only: % refused', tg_op using errcode = '42501';
end $$;

drop trigger if exists records_no_update_delete on public.records;
create trigger records_no_update_delete before update or delete on public.records
  for each row execute function public.records_append_only();
drop trigger if exists records_no_truncate on public.records;
create trigger records_no_truncate before truncate on public.records
  for each statement execute function public.records_append_only();

-- Inserts: service role only, and the chain link must hold where it can be checked here.
-- (The hash and the signature are verified by the ground before pushing; anyone can re-verify
-- the mirror with `eva verify`. See supabase/README.md.)
create or replace function public.records_insert_guard()
returns trigger
language plpgsql
set search_path = ''
as $$
declare
  pred text;
begin
  if not public.is_service() then
    raise exception 'records: insert is reserved to the service role (ground ingest)' using errcode = '42501';
  end if;
  if new.seq = 0 and new.prev <> repeat('0', 64) then
    raise exception 'records: seq 0 must link to the genesis hash' using errcode = '23514';
  end if;
  select r.hash into pred from public.records r
   where r.ship = new.ship and r.epoch = new.epoch and r.seq = new.seq - 1;
  if found and pred <> new.prev then
    raise exception 'records: prev does not match the hash of seq %', new.seq - 1 using errcode = '23514';
  end if;
  new.ingested_at := now();
  return new;
end $$;
drop trigger if exists records_insert_guard on public.records;
create trigger records_insert_guard before insert on public.records
  for each row execute function public.records_insert_guard();

-- ---------------------------------------------------------------- cases (rebuilt by the ground from records)

create table if not exists public.cases (
  ship       text not null references public.ships (id),
  epoch      integer not null check (epoch between 0 and 9999),
  case_no    integer not null check (case_no >= 0),
  seat       text not null check (seat ~ '^[0-9]{1,2}[A-Z]$'),
  kind       text not null check (kind ~ '^[a-z][a-z_]{0,31}$'),
  open       boolean not null,
  opened_ms  bigint not null check (opened_ms >= 0),
  closed_ms  bigint check (closed_ms >= 0),
  helpers    jsonb not null default '[]'::jsonb check (jsonb_typeof(helpers) = 'array' and jsonb_array_length(helpers) <= 20),
  steps      jsonb not null default '[]'::jsonb check (jsonb_typeof(steps) = 'array' and jsonb_array_length(steps) <= 200),
  now        text check (char_length(now) <= 500),
  why        text check (char_length(why) <= 500),
  how        text check (char_length(how) <= 500),
  next       text check (char_length(next) <= 500),
  last_seq   bigint,
  updated_at timestamptz not null default now(),
  primary key (ship, epoch, case_no)
);
comment on column public.cases.helpers is 'Helpers by seat and task only ([{seat, task}]), never names.';

-- Flight-surgeon / crew-lead annotations: append-only, authored by the caller.
create table if not exists public.case_notes (
  id         bigint generated always as identity primary key,
  ship       text not null,
  epoch      integer not null,
  case_no    integer not null,
  author_id  uuid not null default auth.uid() references auth.users (id),
  text       text not null check (char_length(text) between 1 and 2000),
  created_at timestamptz not null default now(),
  foreign key (ship, epoch, case_no) references public.cases (ship, epoch, case_no)
);

create or replace function public.no_update_delete()
returns trigger
language plpgsql
set search_path = ''
as $$
begin
  raise exception '%: % refused (append-only)', tg_table_name, tg_op using errcode = '42501';
end $$;
drop trigger if exists case_notes_append_only on public.case_notes;
create trigger case_notes_append_only before update or delete on public.case_notes
  for each row execute function public.no_update_delete();

-- ---------------------------------------------------------------- passengers: summary only

-- No vitals, ECG, journal, meals or chats: those stay on board. The table has a fixed set of
-- summary columns, and checks keep free text from smuggling readings in:
--   label is "First I." (first name + initial), status is a badge from a closed list,
--   and a structural check (tests/rls.test.sql) fails if any vitals-like column is ever added.
create table if not exists public.passengers (
  ship       text not null references public.ships (id),
  seat       text not null check (seat ~ '^[0-9]{1,2}[A-Z]$'),
  label      text not null check (label ~ '^[^[:space:][:digit:].]{1,24}( [^[:space:][:digit:].]\.)?$'),
  role       text check (role is null or role in ('Crew lead', 'Flight engineer')),
  trained    boolean not null default false,
  grp        text not null check (grp in ('ok', 'watch', 'help')),
  status     text not null check (status in ('ok', 'ecg', 'battery', 'warm', 'suit', 'help', 'case', 'offline', 'sleep')),
  updated_at timestamptz not null default now(),
  primary key (ship, seat)
);
comment on table public.passengers is 'Triage summary only. Raw vitals never leave the ship; no vitals columns (checked by tests/rls.test.sql and the passengers_summary_only event trigger where permitted).';

-- Defence in depth: refuse any DDL that gives passengers a vitals-like column.
create or replace function public.passengers_summary_only()
returns event_trigger
language plpgsql
set search_path = ''
as $$
declare bad text;
begin
  select string_agg(column_name, ', ') into bad
    from information_schema.columns
   where table_schema = 'public' and table_name = 'passengers'
     and column_name ~* '(^|_)(hr|bpm|heart|spo2|sat|ecg|ekg|resp|rr|temp|bp|sys|dia|glucose|vital|eda|hrv|pulse)(_|$)';
  if bad is not null then
    raise exception 'passengers is summary-only; vitals-like column(s) refused: %', bad using errcode = '42501';
  end if;
end $$;

do $$ begin
  drop event trigger if exists passengers_summary_only;
  create event trigger passengers_summary_only on ddl_command_end
    when tag in ('ALTER TABLE', 'CREATE TABLE')
    execute function public.passengers_summary_only();
exception when insufficient_privilege then
  raise notice 'event trigger not permitted here; tests/rls.test.sql still checks the passengers columns';
end $$;

-- ---------------------------------------------------------------- medical station (no health data)

create table if not exists public.station_state (
  ship         text primary key references public.ships (id),
  mode         text not null check (mode in ('online', 'standalone', 'unknown')),
  compartments jsonb not null check (jsonb_typeof(compartments) = 'array' and jsonb_array_length(compartments) = 12),
  screen       text check (char_length(screen) <= 200),
  last_event   text check (char_length(last_event) <= 200),
  updated_at   timestamptz not null default now()
);

-- ---------------------------------------------------------------- grants

alter table public.ships         enable row level security;
alter table public.records       enable row level security;
alter table public.cases         enable row level security;
alter table public.case_notes    enable row level security;
alter table public.passengers    enable row level security;
alter table public.station_state enable row level security;

revoke all on public.ships, public.records, public.cases, public.case_notes, public.passengers, public.station_state
  from anon, authenticated;
grant all on public.ships, public.records, public.cases, public.case_notes, public.passengers, public.station_state
  to service_role;

grant select on public.ships, public.records, public.cases, public.case_notes, public.passengers, public.station_state
  to authenticated;
grant update (display_name, notes) on public.ships to authenticated;           -- mission control only (policy)
grant insert (ship, epoch, case_no, text) on public.case_notes to authenticated; -- MC / crew lead (policy)
-- records: no INSERT/UPDATE/DELETE grant to any client role, ever.

-- ---------------------------------------------------------------- policies

-- ships: any role reads; mission control edits display_name and notes only (column grant).
drop policy if exists ships_read on public.ships;
create policy ships_read on public.ships for select to authenticated
  using ((select public.my_role()) is not null);
drop policy if exists ships_mc_update on public.ships;
create policy ships_mc_update on public.ships for update to authenticated
  using ((select public.has_role('mission_control')))
  with check ((select public.has_role('mission_control')));

-- records, cases, notes, passengers: health readers (MC, crew lead; team on demo ships).
drop policy if exists records_read on public.records;
create policy records_read on public.records for select to authenticated
  using (public.can_read_health(ship));
drop policy if exists cases_read on public.cases;
create policy cases_read on public.cases for select to authenticated
  using (public.can_read_health(ship));
drop policy if exists case_notes_read on public.case_notes;
create policy case_notes_read on public.case_notes for select to authenticated
  using (public.can_read_health(ship));
drop policy if exists case_notes_create on public.case_notes;
create policy case_notes_create on public.case_notes for insert to authenticated
  with check ((select public.has_role('mission_control', 'crew_lead')) and author_id = (select auth.uid()));
drop policy if exists passengers_read on public.passengers;
create policy passengers_read on public.passengers for select to authenticated
  using (public.can_read_health(ship));

-- station state: no health data, every role reads.
drop policy if exists station_read on public.station_state;
create policy station_read on public.station_state for select to authenticated
  using ((select public.my_role()) is not null);
