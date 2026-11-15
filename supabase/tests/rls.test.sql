-- Eva · RLS and integrity tests. Runs inside one transaction and rolls everything back.
--
--   psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -f supabase/tests/rls.test.sql
--
-- Works on the hosted project (connect as postgres, e.g. through the session pooler) and on a
-- plain local PostgreSQL with tests/local/supabase-stub.sql (tests/local/run.sh).
-- Each check impersonates a caller the way PostgREST does: SET ROLE anon | authenticated |
-- service_role plus request.jwt.claims. The script ends with a table of results and fails
-- (raises) if any check failed.

\set QUIET on
\o /dev/null
begin;

create temp table t_results (n serial, label text, ok boolean, got text, expected text) on commit drop;

-- ---------------------------------------------------------------- impersonation helpers

create function pg_temp.become(who text) returns void language plpgsql as $$
begin
  if who = 'owner' then
    return; -- stay the connecting role (table owner)
  elsif who = 'anon' then
    perform set_config('request.jwt.claims', '{"role":"anon"}', true);
    perform set_config('role', 'anon', true);
  elsif who = 'service' then
    perform set_config('request.jwt.claims', '{"role":"service_role"}', true);
    perform set_config('role', 'service_role', true);
  else
    perform set_config('request.jwt.claims', json_build_object('sub', who, 'role', 'authenticated', 'aud', 'authenticated')::text, true);
    perform set_config('role', 'authenticated', true);
  end if;
end $$;

create function pg_temp.unbecome() returns void language plpgsql as $$
begin
  perform set_config('role', session_user, true);
  perform set_config('request.jwt.claims', '', true);
end $$;

-- t_count: run a query returning one integer as `who`; compare with expected.
create function pg_temp.t_count(label text, who text, q text, expected int) returns void language plpgsql as $$
declare v int; err text;
begin
  perform pg_temp.become(who);
  begin
    execute q into v;
  exception when others then err := sqlstate || ' ' || sqlerrm;
  end;
  perform pg_temp.unbecome();
  insert into t_results (label, ok, got, expected)
  values (label, err is null and v = expected, coalesce(err, v::text), expected::text);
end $$;

-- t_exec: run a statement as `who`. expect = 'denied' (must raise) or 'rows:N' (rows affected).
create function pg_temp.t_exec(label text, who text, q text, expect text) returns void language plpgsql as $$
declare rc bigint; err text; got text;
begin
  perform pg_temp.become(who);
  begin
    execute q;
    get diagnostics rc = row_count;
  exception when others then err := sqlstate;
  end;
  perform pg_temp.unbecome();
  got := case when err is not null then 'denied(' || err || ')' else 'rows:' || rc end;
  insert into t_results (label, ok, got, expected)
  values (label, case when expect = 'denied' then err is not null else got = expect end, got, expect);
end $$;

-- ---------------------------------------------------------------- fixtures (rolled back)

insert into auth.users (id, email) values
  ('00000000-0000-4000-8000-0000000000a1', 'rls-crew-lead@eva.test'),
  ('00000000-0000-4000-8000-0000000000a2', 'rls-mission-control@eva.test'),
  ('00000000-0000-4000-8000-0000000000a3', 'rls-team@eva.test'),
  ('00000000-0000-4000-8000-0000000000a4', 'rls-viewer@eva.test'),
  ('00000000-0000-4000-8000-0000000000a5', 'rls-team-2@eva.test'),
  ('00000000-0000-4000-8000-0000000000a6', 'rls-no-profile@eva.test');

-- the sign-up trigger made a 'viewer' profile for each; promote as the service role would
select pg_temp.become('service');
update public.profiles set role = 'crew_lead'       where id = '00000000-0000-4000-8000-0000000000a1';
update public.profiles set role = 'mission_control' where id = '00000000-0000-4000-8000-0000000000a2';
update public.profiles set role = 'team'            where id in ('00000000-0000-4000-8000-0000000000a3', '00000000-0000-4000-8000-0000000000a5');
delete from public.profiles where id = '00000000-0000-4000-8000-0000000000a6';

insert into public.ships (id, display_name, demo, pubkey) values
  ('rls-demo', 'RLS demo ship', true,  repeat('ab', 32)),
  ('rls-real', 'RLS real ship', false, repeat('cd', 32));
insert into public.records (ship, epoch, seq, ts, kind, subject, hash, prev, signature, body) values
  ('rls-demo', 1, 0, 1000, 'log', 'ship', repeat('1', 64), repeat('0', 64), repeat('e', 128), '{"text":"demo 0"}'),
  ('rls-demo', 1, 1, 2000, 'alarm', '4B', repeat('2', 64), repeat('1', 64), repeat('e', 128), '{"text":"demo 1"}'),
  ('rls-real', 1, 0, 1000, 'log', 'ship', repeat('3', 64), repeat('0', 64), repeat('e', 128), '{"text":"real 0"}');
insert into public.cases (ship, epoch, case_no, seat, kind, open, opened_ms) values
  ('rls-demo', 1, 1, '4B', 'cardiac', true, 2000),
  ('rls-real', 1, 1, '2A', 'cardiac', false, 1000);
insert into public.passengers (ship, seat, label, role, trained, grp, status) values
  ('rls-demo', '4B', 'Ana R.', null, false, 'help', 'case'),
  ('rls-real', '2A', 'Yuki S.', null, true, 'ok', 'ok');
insert into public.station_state (ship, mode, compartments)
  select 'rls-demo', 'online', jsonb_agg(jsonb_build_object('n', g, 'state', 'idle')) from generate_series(1, 12) g;
insert into public.team_docs (id, title, author_id) values
  ('00000000-0000-4000-8000-00000000d0c1', 'Team note by team-1', '00000000-0000-4000-8000-0000000000a3');
insert into public.journal_entries (id, user_id, body) values
  ('00000000-0000-4000-8000-0000000001a4', '00000000-0000-4000-8000-0000000000a4', 'viewer private entry'),
  ('00000000-0000-4000-8000-0000000001a3', '00000000-0000-4000-8000-0000000000a3', 'team private entry');
select pg_temp.unbecome();

-- ---------------------------------------------------------------- checks
-- callers
\set CL   '00000000-0000-4000-8000-0000000000a1'
\set MC   '00000000-0000-4000-8000-0000000000a2'
\set TEAM '00000000-0000-4000-8000-0000000000a3'
\set VIEW '00000000-0000-4000-8000-0000000000a4'
\set TEAM2 '00000000-0000-4000-8000-0000000000a5'
\set NOPRO '00000000-0000-4000-8000-0000000000a6'

-- profiles
select pg_temp.t_count('profiles: viewer sees only own profile', :'VIEW', $q$select count(*)::int from public.profiles where id::text like '00000000-0000-4000-8000-0000000000a%'$q$, 1);
select pg_temp.t_count('profiles: mission control sees all test profiles', :'MC', $q$select count(*)::int from public.profiles where id::text like '00000000-0000-4000-8000-0000000000a%'$q$, 5);
select pg_temp.t_exec('profiles: anon sees none (denied, no grant)', 'anon', $q$select count(*)::int from public.profiles$q$, 'denied');
select pg_temp.t_exec('profiles: viewer cannot raise own role', :'VIEW', $q$update public.profiles set role = 'mission_control' where id = auth.uid()$q$, 'denied');
select pg_temp.t_exec('profiles: viewer edits own display_name', :'VIEW', $q$update public.profiles set display_name = 'Viewer' where id = auth.uid()$q$, 'rows:1');
select pg_temp.t_exec('profiles: viewer cannot edit another profile', :'VIEW', $q$update public.profiles set display_name = 'x' where id = '00000000-0000-4000-8000-0000000000a3'$q$, 'rows:0');
select pg_temp.t_exec('admin_set_role: viewer refused', :'VIEW', $q$select public.admin_set_role(auth.uid(), 'mission_control')$q$, 'denied');
select pg_temp.t_exec('admin_set_role: team refused', :'TEAM', $q$select public.admin_set_role('00000000-0000-4000-8000-0000000000a4', 'team')$q$, 'denied');
select pg_temp.t_exec('admin_set_role: anon refused', 'anon', $q$select public.admin_set_role('00000000-0000-4000-8000-0000000000a4', 'team')$q$, 'denied');
select pg_temp.t_exec('admin_set_role: mission control promotes viewer', :'MC', $q$select public.admin_set_role('00000000-0000-4000-8000-0000000000a4', 'team')$q$, 'rows:1');
select pg_temp.t_exec('admin_set_role: service role demotes back', 'service', $q$select public.admin_set_role('00000000-0000-4000-8000-0000000000a4', 'viewer')$q$, 'rows:1');
select pg_temp.t_count('role_changes: audited (2 rows), MC reads', :'MC', $q$select count(*)::int from public.role_changes where target = '00000000-0000-4000-8000-0000000000a4'$q$, 2);
select pg_temp.t_count('role_changes: viewer reads none', :'VIEW', $q$select count(*)::int from public.role_changes$q$, 0);

-- ships
select pg_temp.t_count('ships: viewer reads ships', :'VIEW', $q$select count(*)::int from public.ships where id like 'rls-%'$q$, 2);
select pg_temp.t_exec('ships: anon reads none (denied, no grant)', 'anon', $q$select count(*)::int from public.ships$q$, 'denied');
select pg_temp.t_count('ships: user without profile reads none', :'NOPRO', $q$select count(*)::int from public.ships$q$, 0);
select pg_temp.t_exec('ships: mission control edits notes', :'MC', $q$update public.ships set notes = 'ok' where id = 'rls-demo'$q$, 'rows:1');
select pg_temp.t_exec('ships: mission control cannot change pubkey', :'MC', $q$update public.ships set pubkey = repeat('ff', 32) where id = 'rls-demo'$q$, 'denied');
select pg_temp.t_exec('ships: service role cannot change pinned pubkey', 'service', $q$update public.ships set pubkey = repeat('ff', 32) where id = 'rls-demo'$q$, 'denied');
select pg_temp.t_exec('ships: team cannot edit notes', :'TEAM', $q$update public.ships set notes = 'x' where id = 'rls-demo'$q$, 'rows:0');

-- records (signed chain mirror)
select pg_temp.t_count('records: crew lead reads all', :'CL', $q$select count(*)::int from public.records where ship like 'rls-%'$q$, 3);
select pg_temp.t_count('records: mission control reads all', :'MC', $q$select count(*)::int from public.records where ship like 'rls-%'$q$, 3);
select pg_temp.t_count('records: team reads demo ship only', :'TEAM', $q$select count(*)::int from public.records where ship like 'rls-%'$q$, 2);
select pg_temp.t_count('records: viewer reads none', :'VIEW', $q$select count(*)::int from public.records$q$, 0);
select pg_temp.t_exec('records: anon reads none (denied, no grant)', 'anon', $q$select count(*)::int from public.records$q$, 'denied');
select pg_temp.t_exec('records: mission control cannot insert', :'MC', $q$insert into public.records (ship, epoch, seq, ts, kind, subject, hash, prev, signature) values ('rls-demo', 1, 2, 3000, 'log', 'ship', repeat('4', 64), repeat('2', 64), repeat('e', 128))$q$, 'denied');
select pg_temp.t_exec('records: anon cannot insert', 'anon', $q$insert into public.records (ship, epoch, seq, ts, kind, subject, hash, prev, signature) values ('rls-demo', 1, 2, 3000, 'log', 'ship', repeat('4', 64), repeat('2', 64), repeat('e', 128))$q$, 'denied');
select pg_temp.t_exec('records: service role inserts next link', 'service', $q$insert into public.records (ship, epoch, seq, ts, kind, subject, hash, prev, signature) values ('rls-demo', 1, 2, 3000, 'log', 'ship', repeat('4', 64), repeat('2', 64), repeat('e', 128))$q$, 'rows:1');
select pg_temp.t_exec('records: broken prev link refused', 'service', $q$insert into public.records (ship, epoch, seq, ts, kind, subject, hash, prev, signature) values ('rls-demo', 1, 3, 4000, 'log', 'ship', repeat('5', 64), repeat('9', 64), repeat('e', 128))$q$, 'denied');
select pg_temp.t_exec('records: duplicate (ship, epoch, seq) refused', 'service', $q$insert into public.records (ship, epoch, seq, ts, kind, subject, hash, prev, signature) values ('rls-demo', 1, 2, 3000, 'log', 'ship', repeat('6', 64), repeat('2', 64), repeat('e', 128))$q$, 'denied');
select pg_temp.t_exec('records: service role cannot UPDATE', 'service', $q$update public.records set kind = 'note' where ship = 'rls-demo'$q$, 'denied');
select pg_temp.t_exec('records: service role cannot DELETE', 'service', $q$delete from public.records where ship = 'rls-demo'$q$, 'denied');
select pg_temp.t_exec('records: table owner cannot DELETE', 'owner', $q$delete from public.records where ship = 'rls-demo'$q$, 'denied');
select pg_temp.t_exec('records: mission control cannot UPDATE', :'MC', $q$update public.records set kind = 'note' where ship = 'rls-demo'$q$, 'denied');
select pg_temp.t_exec('records: crew lead cannot DELETE', :'CL', $q$delete from public.records where ship = 'rls-demo'$q$, 'denied');

-- cases, notes, passengers
select pg_temp.t_count('cases: crew lead reads all', :'CL', $q$select count(*)::int from public.cases where ship like 'rls-%'$q$, 2);
select pg_temp.t_count('cases: team reads demo only', :'TEAM', $q$select count(*)::int from public.cases where ship like 'rls-%'$q$, 1);
select pg_temp.t_count('cases: viewer reads none', :'VIEW', $q$select count(*)::int from public.cases$q$, 0);
select pg_temp.t_exec('case_notes: mission control adds a note', :'MC', $q$insert into public.case_notes (ship, epoch, case_no, text) values ('rls-demo', 1, 1, 'Reviewed')$q$, 'rows:1');
select pg_temp.t_exec('case_notes: team cannot add a note', :'TEAM', $q$insert into public.case_notes (ship, epoch, case_no, text) values ('rls-demo', 1, 1, 'x')$q$, 'denied');
select pg_temp.t_exec('case_notes: author cannot edit (append-only)', :'MC', $q$update public.case_notes set text = 'changed' where ship = 'rls-demo'$q$, 'denied');
select pg_temp.t_count('passengers: mission control reads all', :'MC', $q$select count(*)::int from public.passengers where ship like 'rls-%'$q$, 2);
select pg_temp.t_count('passengers: team reads demo only', :'TEAM', $q$select count(*)::int from public.passengers where ship like 'rls-%'$q$, 1);
select pg_temp.t_count('passengers: viewer reads none', :'VIEW', $q$select count(*)::int from public.passengers$q$, 0);
select pg_temp.t_exec('passengers: crew lead cannot write', :'CL', $q$update public.passengers set grp = 'ok' where ship = 'rls-demo'$q$, 'denied');
select pg_temp.t_exec('passengers: full name refused by check', 'service', $q$insert into public.passengers (ship, seat, label, trained, grp, status) values ('rls-demo', '5A', 'Diego Luna', false, 'ok', 'ok')$q$, 'denied');
select pg_temp.t_exec('passengers: vitals in status refused by check', 'service', $q$insert into public.passengers (ship, seat, label, trained, grp, status) values ('rls-demo', '5A', 'Diego L.', false, 'ok', 'hr 142')$q$, 'denied');
select pg_temp.t_count('passengers: no vitals-like columns (summary only)', 'service', $q$select count(*)::int from information_schema.columns where table_schema = 'public' and table_name = 'passengers' and column_name !~ '^(ship|seat|label|role|trained|grp|status|updated_at)$'$q$, 0);

-- station state
select pg_temp.t_count('station_state: viewer reads', :'VIEW', $q$select count(*)::int from public.station_state where ship = 'rls-demo'$q$, 1);
select pg_temp.t_exec('station_state: anon reads none (denied, no grant)', 'anon', $q$select count(*)::int from public.station_state$q$, 'denied');

-- journal entries: private per user
select pg_temp.t_count('journal: owner reads own entry', :'VIEW', $q$select count(*)::int from public.journal_entries$q$, 1);
select pg_temp.t_count('journal: mission control reads none of others', :'MC', $q$select count(*)::int from public.journal_entries$q$, 0);
select pg_temp.t_count('journal: crew lead reads none of others', :'CL', $q$select count(*)::int from public.journal_entries$q$, 0);
select pg_temp.t_exec('journal: owner writes an entry', :'TEAM', $q$insert into public.journal_entries (body) values ('slept 7 h')$q$, 'rows:1');
select pg_temp.t_exec('journal: cannot write into another user''s journal', :'TEAM', $q$insert into public.journal_entries (user_id, body) values ('00000000-0000-4000-8000-0000000000a4', 'x')$q$, 'denied');
select pg_temp.t_exec('journal: cannot edit another user''s entry', :'MC', $q$update public.journal_entries set body = 'x' where id = '00000000-0000-4000-8000-0000000001a4'$q$, 'rows:0');
select pg_temp.t_exec('journal: cannot delete another user''s entry', :'TEAM2', $q$delete from public.journal_entries where id = '00000000-0000-4000-8000-0000000001a3'$q$, 'rows:0');
select pg_temp.t_exec('journal: anon reads none (denied, no grant)', 'anon', $q$select count(*)::int from public.journal_entries$q$, 'denied');

-- team docs
select pg_temp.t_count('team_docs: crew lead reads', :'CL', $q$select count(*)::int from public.team_docs where id = '00000000-0000-4000-8000-00000000d0c1'$q$, 1);
select pg_temp.t_count('team_docs: viewer reads none', :'VIEW', $q$select count(*)::int from public.team_docs$q$, 0);
select pg_temp.t_exec('team_docs: team creates a doc', :'TEAM2', $q$insert into public.team_docs (title) values ('Pitch notes')$q$, 'rows:1');
select pg_temp.t_exec('team_docs: crew lead cannot create', :'CL', $q$insert into public.team_docs (title) values ('x')$q$, 'denied');
select pg_temp.t_exec('team_docs: other team member cannot edit', :'TEAM2', $q$update public.team_docs set title = 'x' where id = '00000000-0000-4000-8000-00000000d0c1'$q$, 'rows:0');
select pg_temp.t_exec('team_docs: author edits', :'TEAM', $q$update public.team_docs set title = 'Edited' where id = '00000000-0000-4000-8000-00000000d0c1'$q$, 'rows:1');
select pg_temp.t_exec('team_docs: mission control deletes', :'MC', $q$delete from public.team_docs where id = '00000000-0000-4000-8000-00000000d0c1'$q$, 'rows:1');

-- rate limits (server-only table; the RPC counts against the caller only)
select pg_temp.t_exec('rate_limits: authenticated cannot read the table', :'VIEW', $q$select count(*) from public.rate_limits$q$, 'denied');
select pg_temp.t_count('eva_chat_consume: viewer allowed', :'VIEW', $q$select count(*)::int from public.eva_chat_consume() where allowed and role = 'viewer'$q$, 1);
select pg_temp.t_count('eva_chat_consume: 7th call in a minute refused', :'VIEW', $q$select (select count(*) from generate_series(1, 6) g, lateral public.eva_chat_consume(6 + 0 * g) c where not c.allowed)::int$q$, 1);
select pg_temp.t_count('eva_chat_consume: user without profile refused', :'NOPRO', $q$select count(*)::int from public.eva_chat_consume() where not allowed and role is null$q$, 1);
select pg_temp.t_exec('eva_chat_consume: anon cannot call', 'anon', $q$select * from public.eva_chat_consume()$q$, 'denied');

-- storage
select pg_temp.t_exec('storage: team uploads to team-assets', :'TEAM', $q$insert into storage.objects (bucket_id, name, owner_id) values ('team-assets', 'reports/r1.pdf', auth.uid()::text)$q$, 'rows:1');
select pg_temp.t_exec('storage: viewer cannot upload to team-assets', :'VIEW', $q$insert into storage.objects (bucket_id, name, owner_id) values ('team-assets', 'x.pdf', auth.uid()::text)$q$, 'denied');
select pg_temp.t_exec('storage: viewer cannot upload to public-media', :'VIEW', $q$insert into storage.objects (bucket_id, name, owner_id) values ('public-media', 'x.png', auth.uid()::text)$q$, 'denied');
select pg_temp.t_exec('storage: mission control uploads to public-media', :'MC', $q$insert into storage.objects (bucket_id, name, owner_id) values ('public-media', 'hero.webp', auth.uid()::text)$q$, 'rows:1');
select pg_temp.t_count('storage: crew lead reads team-assets', :'CL', $q$select count(*)::int from storage.objects where bucket_id = 'team-assets' and name = 'reports/r1.pdf'$q$, 1);
select pg_temp.t_count('storage: viewer cannot read team-assets', :'VIEW', $q$select count(*)::int from storage.objects where bucket_id = 'team-assets'$q$, 0);
select pg_temp.t_count('storage: anon cannot list team-assets', 'anon', $q$select count(*)::int from storage.objects where bucket_id = 'team-assets'$q$, 0);
select pg_temp.t_count('storage: team-assets is private, public-media is public', 'service', $q$select count(*)::int from storage.buckets where (id = 'team-assets' and not public) or (id = 'public-media' and public)$q$, 2);

-- public_stats: aggregates for anon, nothing sensitive
select pg_temp.t_count('public_stats: anon reads aggregates', 'anon', $q$select acknowledged_records::int from public.public_stats where ship = 'rls-demo'$q$, 3);
select pg_temp.t_count('public_stats: only aggregate columns', 'service', $q$select count(*)::int from information_schema.columns where table_schema = 'public' and table_name = 'public_stats' and column_name !~ '^(ship|display_name|demo|epoch|acknowledged_records|last_ingest_at|open_cases|total_cases|passengers|station_mode)$'$q$, 0);

-- ---------------------------------------------------------------- results

\o
\set QUIET off
select n, case when ok then 'PASS' else 'FAIL' end as result, label, got, expected from t_results order by n;
select count(*) filter (where ok) as passed, count(*) filter (where not ok) as failed, count(*) as total from t_results;
do $$ declare f int; begin
  select count(*) into f from t_results where not ok;
  if f > 0 then raise exception '% RLS check(s) failed', f; end if;
end $$;
rollback;
