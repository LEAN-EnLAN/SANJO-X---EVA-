-- Eva · RLS and integrity tests. Runs inside one transaction and rolls everything back.
--
--   local:  supabase/tests/local/run.sh          (plain PostgreSQL + tests/local/supabase-stub.sql)
--   hosted: supabase/scripts/sql.sh --rls-test   (Management API; see supabase/README.md)
--
-- Callers are the eva-test-* users (one per role, plus a second team member). On the hosted
-- project they must already exist (provision.sh creates them); the test never writes auth.users
-- there. Locally (eva.test_local=on) it creates them in the stub's auth.users.
-- Each check impersonates a caller the way PostgREST does: SET ROLE anon | authenticated |
-- service_role plus request.jwt.claims. In a query, {CL} {MC} {TEAM} {VIEW} {TEAM2} {NOPRO} {NEW}
-- stand for that caller's id. Lines starting with a backslash are psql-only; everything after
-- "-- @@results" is the psql report (the hosted runner replaces it).

\set QUIET on
\o /dev/null
begin;

create temp table t_results (n serial, label text, ok boolean, got text, expected text) on commit drop;

-- ---------------------------------------------------------------- impersonation helpers

create temp table t_users (k text primary key, id uuid not null, email text) on commit drop;
grant select on t_users to service_role;  -- the service-role fixture step reads it

create function pg_temp.become(who text) returns void language plpgsql as $$
declare u t_users;
begin
  select * into u from t_users where k = who;
  if found then
    perform set_config('request.jwt.claims', json_build_object('sub', u.id, 'role', 'authenticated', 'aud', 'authenticated', 'email', u.email)::text, true);
    perform set_config('role', 'authenticated', true);
  elsif who = 'owner' then
    return; -- stay the connecting role (table owner)
  elsif who = 'anon' then
    perform set_config('request.jwt.claims', '{"role":"anon"}', true);
    perform set_config('role', 'anon', true);
  elsif who = 'service' then
    perform set_config('request.jwt.claims', '{"role":"service_role"}', true);
    perform set_config('role', 'service_role', true);
  else
    raise exception 'unknown caller %', who;
  end if;
end $$;

-- {KEY} -> 'uuid'::uuid
create function pg_temp.subst(q text) returns text language plpgsql as $$
declare u t_users;
begin
  for u in select * from t_users loop
    q := replace(q, '{' || u.k || '}', quote_literal(u.id) || '::uuid');
  end loop;
  return q;
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
  q := pg_temp.subst(q);
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
  q := pg_temp.subst(q);
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

do $$
declare k text; e text; uid uuid;
begin
  foreach k in array array['crew-lead:CL', 'mission-control:MC', 'team:TEAM', 'viewer:VIEW', 'team-2:TEAM2', 'new:NEW'] loop
    e := 'eva-test-' || split_part(k, ':', 1) || '@example.com';
    select id into uid from auth.users where email = e;
    if uid is null then
      if coalesce(current_setting('eva.test_local', true), '') <> 'on' then
        raise exception 'test user % missing: run provision.sh (step 5) first', e;
      end if;
      insert into auth.users (email) values (e) returning id into uid;   -- local stub only
    end if;
    insert into t_users values (split_part(k, ':', 2), uid, e);
  end loop;
  insert into t_users values ('NOPRO', gen_random_uuid(), 'eva-test-no-profile@example.com');  -- no auth user, no profile
end $$;

-- profiles and roles as the service role would set them; clean rate-limit buckets
select pg_temp.become('service');
insert into eva.profiles (id, email, role)
  select id, email, (case k when 'CL' then 'crew_lead' when 'MC' then 'mission_control' when 'VIEW' then 'viewer' else 'team' end)::eva.app_role
    from t_users where k in ('CL', 'MC', 'TEAM', 'VIEW', 'TEAM2')
  on conflict (id) do update set role = excluded.role;
delete from eva.profiles where id in (select id from t_users where k in ('NOPRO', 'NEW'));
delete from eva.rate_limits where user_id in (select id from t_users);

insert into eva.ships (id, display_name, demo, pubkey) values
  ('rls-demo', 'RLS demo ship', true,  repeat('ab', 32)),
  ('rls-real', 'RLS real ship', false, repeat('cd', 32));
insert into eva.records (ship, epoch, seq, ts, kind, subject, hash, prev, signature, body) values
  ('rls-demo', 1, 0, 1000, 'log', 'ship', repeat('1', 64), repeat('0', 64), repeat('e', 128), '{"text":"demo 0"}'),
  ('rls-demo', 1, 1, 2000, 'alarm', '4B', repeat('2', 64), repeat('1', 64), repeat('e', 128), '{"text":"demo 1"}'),
  ('rls-real', 1, 0, 1000, 'log', 'ship', repeat('3', 64), repeat('0', 64), repeat('e', 128), '{"text":"real 0"}');
insert into eva.cases (ship, epoch, case_no, seat, kind, open, opened_ms) values
  ('rls-demo', 1, 1, '4B', 'cardiac', true, 2000),
  ('rls-real', 1, 1, '2A', 'cardiac', false, 1000);
insert into eva.passengers (ship, seat, label, role, trained, grp, status) values
  ('rls-demo', '4B', 'Ana R.', null, false, 'help', 'case'),
  ('rls-real', '2A', 'Yuki S.', null, true, 'ok', 'ok');
insert into eva.station_state (ship, mode, compartments)
  select 'rls-demo', 'online', jsonb_agg(jsonb_build_object('n', g, 'state', 'idle')) from generate_series(1, 12) g;
insert into eva.team_docs (id, title, author_id) values
  ('00000000-0000-4000-8000-00000000d0c1', 'Team note by team-1', (select id from t_users where k = 'TEAM'));
insert into eva.journal_entries (id, user_id, body) values
  ('00000000-0000-4000-8000-0000000001a4', (select id from t_users where k = 'VIEW'), 'viewer private entry'),
  ('00000000-0000-4000-8000-0000000001a3', (select id from t_users where k = 'TEAM'), 'team private entry');
select pg_temp.unbecome();

-- ---------------------------------------------------------------- checks
-- profiles
select pg_temp.t_count('profiles: viewer sees only own profile', 'VIEW', $q$select count(*)::int from eva.profiles where id in ({CL}, {MC}, {TEAM}, {VIEW}, {TEAM2})$q$, 1);
select pg_temp.t_count('profiles: mission control sees all test profiles', 'MC', $q$select count(*)::int from eva.profiles where id in ({CL}, {MC}, {TEAM}, {VIEW}, {TEAM2})$q$, 5);
select pg_temp.t_exec('profiles: anon sees none (denied, no grant)', 'anon', $q$select count(*)::int from eva.profiles$q$, 'denied');
select pg_temp.t_exec('profiles: viewer cannot raise own role', 'VIEW', $q$update eva.profiles set role = 'mission_control' where id = auth.uid()$q$, 'denied');
select pg_temp.t_exec('profiles: viewer edits own display_name', 'VIEW', $q$update eva.profiles set display_name = 'Viewer' where id = auth.uid()$q$, 'rows:1');
select pg_temp.t_exec('profiles: viewer cannot edit another profile', 'VIEW', $q$update eva.profiles set display_name = 'x' where id = {TEAM}$q$, 'rows:0');
select pg_temp.t_exec('admin_set_role: viewer refused', 'VIEW', $q$select eva.admin_set_role(auth.uid(), 'mission_control')$q$, 'denied');
select pg_temp.t_exec('admin_set_role: team refused', 'TEAM', $q$select eva.admin_set_role({VIEW}, 'team')$q$, 'denied');
select pg_temp.t_exec('admin_set_role: anon refused', 'anon', $q$select eva.admin_set_role({VIEW}, 'team')$q$, 'denied');
select pg_temp.t_exec('admin_set_role: mission control promotes viewer', 'MC', $q$select eva.admin_set_role({VIEW}, 'team')$q$, 'rows:1');
select pg_temp.t_exec('admin_set_role: service role demotes back', 'service', $q$select eva.admin_set_role({VIEW}, 'viewer')$q$, 'rows:1');
select pg_temp.t_count('role_changes: audited (2 rows), MC reads', 'MC', $q$select count(*)::int from eva.role_changes where target = {VIEW}$q$, 2);
select pg_temp.t_count('role_changes: viewer reads none', 'VIEW', $q$select count(*)::int from eva.role_changes$q$, 0);

-- ships
select pg_temp.t_count('ships: viewer reads ships', 'VIEW', $q$select count(*)::int from eva.ships where id like 'rls-%'$q$, 2);
select pg_temp.t_exec('ships: anon reads none (denied, no grant)', 'anon', $q$select count(*)::int from eva.ships$q$, 'denied');
select pg_temp.t_count('ships: user without profile reads none', 'NOPRO', $q$select count(*)::int from eva.ships$q$, 0);
select pg_temp.t_exec('ships: mission control edits notes', 'MC', $q$update eva.ships set notes = 'ok' where id = 'rls-demo'$q$, 'rows:1');
select pg_temp.t_exec('ships: mission control cannot change pubkey', 'MC', $q$update eva.ships set pubkey = repeat('ff', 32) where id = 'rls-demo'$q$, 'denied');
select pg_temp.t_exec('ships: service role cannot change pinned pubkey', 'service', $q$update eva.ships set pubkey = repeat('ff', 32) where id = 'rls-demo'$q$, 'denied');
select pg_temp.t_exec('ships: team cannot edit notes', 'TEAM', $q$update eva.ships set notes = 'x' where id = 'rls-demo'$q$, 'rows:0');

-- records (signed chain mirror)
select pg_temp.t_count('records: crew lead reads all', 'CL', $q$select count(*)::int from eva.records where ship like 'rls-%'$q$, 3);
select pg_temp.t_count('records: mission control reads all', 'MC', $q$select count(*)::int from eva.records where ship like 'rls-%'$q$, 3);
select pg_temp.t_count('records: team reads demo ship only', 'TEAM', $q$select count(*)::int from eva.records where ship like 'rls-%'$q$, 2);
select pg_temp.t_count('records: viewer reads none', 'VIEW', $q$select count(*)::int from eva.records$q$, 0);
select pg_temp.t_exec('records: anon reads none (denied, no grant)', 'anon', $q$select count(*)::int from eva.records$q$, 'denied');
select pg_temp.t_exec('records: mission control cannot insert', 'MC', $q$insert into eva.records (ship, epoch, seq, ts, kind, subject, hash, prev, signature) values ('rls-demo', 1, 2, 3000, 'log', 'ship', repeat('4', 64), repeat('2', 64), repeat('e', 128))$q$, 'denied');
select pg_temp.t_exec('records: anon cannot insert', 'anon', $q$insert into eva.records (ship, epoch, seq, ts, kind, subject, hash, prev, signature) values ('rls-demo', 1, 2, 3000, 'log', 'ship', repeat('4', 64), repeat('2', 64), repeat('e', 128))$q$, 'denied');
select pg_temp.t_exec('records: service role inserts next link', 'service', $q$insert into eva.records (ship, epoch, seq, ts, kind, subject, hash, prev, signature) values ('rls-demo', 1, 2, 3000, 'log', 'ship', repeat('4', 64), repeat('2', 64), repeat('e', 128))$q$, 'rows:1');
select pg_temp.t_exec('records: broken prev link refused', 'service', $q$insert into eva.records (ship, epoch, seq, ts, kind, subject, hash, prev, signature) values ('rls-demo', 1, 3, 4000, 'log', 'ship', repeat('5', 64), repeat('9', 64), repeat('e', 128))$q$, 'denied');
select pg_temp.t_exec('records: duplicate (ship, epoch, seq) refused', 'service', $q$insert into eva.records (ship, epoch, seq, ts, kind, subject, hash, prev, signature) values ('rls-demo', 1, 2, 3000, 'log', 'ship', repeat('6', 64), repeat('2', 64), repeat('e', 128))$q$, 'denied');
select pg_temp.t_exec('records: service role cannot UPDATE', 'service', $q$update eva.records set kind = 'note' where ship = 'rls-demo'$q$, 'denied');
select pg_temp.t_exec('records: service role cannot DELETE', 'service', $q$delete from eva.records where ship = 'rls-demo'$q$, 'denied');
select pg_temp.t_exec('records: table owner cannot DELETE', 'owner', $q$delete from eva.records where ship = 'rls-demo'$q$, 'denied');
select pg_temp.t_exec('records: mission control cannot UPDATE', 'MC', $q$update eva.records set kind = 'note' where ship = 'rls-demo'$q$, 'denied');
select pg_temp.t_exec('records: crew lead cannot DELETE', 'CL', $q$delete from eva.records where ship = 'rls-demo'$q$, 'denied');

-- cases, notes, passengers
select pg_temp.t_count('cases: crew lead reads all', 'CL', $q$select count(*)::int from eva.cases where ship like 'rls-%'$q$, 2);
select pg_temp.t_count('cases: team reads demo only', 'TEAM', $q$select count(*)::int from eva.cases where ship like 'rls-%'$q$, 1);
select pg_temp.t_count('cases: viewer reads none', 'VIEW', $q$select count(*)::int from eva.cases$q$, 0);
select pg_temp.t_exec('case_notes: mission control adds a note', 'MC', $q$insert into eva.case_notes (ship, epoch, case_no, text) values ('rls-demo', 1, 1, 'Reviewed')$q$, 'rows:1');
select pg_temp.t_exec('case_notes: team cannot add a note', 'TEAM', $q$insert into eva.case_notes (ship, epoch, case_no, text) values ('rls-demo', 1, 1, 'x')$q$, 'denied');
select pg_temp.t_exec('case_notes: author cannot edit (append-only)', 'MC', $q$update eva.case_notes set text = 'changed' where ship = 'rls-demo'$q$, 'denied');
select pg_temp.t_count('passengers: mission control reads all', 'MC', $q$select count(*)::int from eva.passengers where ship like 'rls-%'$q$, 2);
select pg_temp.t_count('passengers: team reads demo only', 'TEAM', $q$select count(*)::int from eva.passengers where ship like 'rls-%'$q$, 1);
select pg_temp.t_count('passengers: viewer reads none', 'VIEW', $q$select count(*)::int from eva.passengers$q$, 0);
select pg_temp.t_exec('passengers: crew lead cannot write', 'CL', $q$update eva.passengers set grp = 'ok' where ship = 'rls-demo'$q$, 'denied');
select pg_temp.t_exec('passengers: full name refused by check', 'service', $q$insert into eva.passengers (ship, seat, label, trained, grp, status) values ('rls-demo', '5A', 'Diego Luna', false, 'ok', 'ok')$q$, 'denied');
select pg_temp.t_exec('passengers: vitals in status refused by check', 'service', $q$insert into eva.passengers (ship, seat, label, trained, grp, status) values ('rls-demo', '5A', 'Diego L.', false, 'ok', 'hr 142')$q$, 'denied');
select pg_temp.t_count('passengers: no vitals-like columns (summary only)', 'service', $q$select count(*)::int from information_schema.columns where table_schema = 'public' and table_name = 'passengers' and column_name !~ '^(ship|seat|label|role|trained|grp|status|updated_at)$'$q$, 0);

-- station state
select pg_temp.t_count('station_state: viewer reads', 'VIEW', $q$select count(*)::int from eva.station_state where ship = 'rls-demo'$q$, 1);
select pg_temp.t_exec('station_state: anon reads none (denied, no grant)', 'anon', $q$select count(*)::int from eva.station_state$q$, 'denied');

-- journal entries: private per user
select pg_temp.t_count('journal: owner reads own entry', 'VIEW', $q$select count(*)::int from eva.journal_entries$q$, 1);
select pg_temp.t_count('journal: mission control reads none of others', 'MC', $q$select count(*)::int from eva.journal_entries$q$, 0);
select pg_temp.t_count('journal: crew lead reads none of others', 'CL', $q$select count(*)::int from eva.journal_entries$q$, 0);
select pg_temp.t_exec('journal: owner writes an entry', 'TEAM', $q$insert into eva.journal_entries (body) values ('slept 7 h')$q$, 'rows:1');
select pg_temp.t_exec('journal: cannot write into another user''s journal', 'TEAM', $q$insert into eva.journal_entries (user_id, body) values ({VIEW}, 'x')$q$, 'denied');
select pg_temp.t_exec('journal: cannot edit another user''s entry', 'MC', $q$update eva.journal_entries set body = 'x' where id = '00000000-0000-4000-8000-0000000001a4'$q$, 'rows:0');
select pg_temp.t_exec('journal: cannot delete another user''s entry', 'TEAM2', $q$delete from eva.journal_entries where id = '00000000-0000-4000-8000-0000000001a3'$q$, 'rows:0');
select pg_temp.t_exec('journal: anon reads none (denied, no grant)', 'anon', $q$select count(*)::int from eva.journal_entries$q$, 'denied');

-- team docs
select pg_temp.t_count('team_docs: crew lead reads', 'CL', $q$select count(*)::int from eva.team_docs where id = '00000000-0000-4000-8000-00000000d0c1'$q$, 1);
select pg_temp.t_count('team_docs: viewer reads none', 'VIEW', $q$select count(*)::int from eva.team_docs$q$, 0);
select pg_temp.t_exec('team_docs: team creates a doc', 'TEAM2', $q$insert into eva.team_docs (title) values ('Pitch notes')$q$, 'rows:1');
select pg_temp.t_exec('team_docs: crew lead cannot create', 'CL', $q$insert into eva.team_docs (title) values ('x')$q$, 'denied');
select pg_temp.t_exec('team_docs: other team member cannot edit', 'TEAM2', $q$update eva.team_docs set title = 'x' where id = '00000000-0000-4000-8000-00000000d0c1'$q$, 'rows:0');
select pg_temp.t_exec('team_docs: author edits', 'TEAM', $q$update eva.team_docs set title = 'Edited' where id = '00000000-0000-4000-8000-00000000d0c1'$q$, 'rows:1');
select pg_temp.t_exec('team_docs: mission control deletes', 'MC', $q$delete from eva.team_docs where id = '00000000-0000-4000-8000-00000000d0c1'$q$, 'rows:1');

-- rate limits (server-only table; the RPC counts against the caller only)
select pg_temp.t_exec('rate_limits: authenticated cannot read the table', 'VIEW', $q$select count(*) from eva.rate_limits$q$, 'denied');
select pg_temp.t_count('eva_chat_consume: viewer allowed', 'VIEW', $q$select count(*)::int from eva.eva_chat_consume() where allowed and role = 'viewer'$q$, 1);
select pg_temp.t_count('eva_chat_consume: 7th call in a minute refused', 'VIEW', $q$select (select count(*) from generate_series(1, 6) g, lateral eva.eva_chat_consume(6 + 0 * g) c where not c.allowed)::int$q$, 1);
select pg_temp.t_count('eva_chat_consume: user without profile refused', 'NOPRO', $q$select count(*)::int from eva.eva_chat_consume() where not allowed and role is null$q$, 1);
select pg_temp.t_exec('eva_chat_consume: anon cannot call', 'anon', $q$select * from eva.eva_chat_consume()$q$, 'denied');

-- storage
select pg_temp.t_exec('storage: team uploads to team-assets', 'TEAM', $q$insert into storage.objects (bucket_id, name, owner_id) values ('eva-team-assets', 'reports/r1.pdf', auth.uid()::text)$q$, 'rows:1');
select pg_temp.t_exec('storage: viewer cannot upload to team-assets', 'VIEW', $q$insert into storage.objects (bucket_id, name, owner_id) values ('eva-team-assets', 'x.pdf', auth.uid()::text)$q$, 'denied');
select pg_temp.t_exec('storage: viewer cannot upload to public-media', 'VIEW', $q$insert into storage.objects (bucket_id, name, owner_id) values ('eva-public-media', 'x.png', auth.uid()::text)$q$, 'denied');
select pg_temp.t_exec('storage: mission control uploads to public-media', 'MC', $q$insert into storage.objects (bucket_id, name, owner_id) values ('eva-public-media', 'hero.webp', auth.uid()::text)$q$, 'rows:1');
select pg_temp.t_count('storage: crew lead reads team-assets', 'CL', $q$select count(*)::int from storage.objects where bucket_id = 'eva-team-assets' and name = 'reports/r1.pdf'$q$, 1);
select pg_temp.t_count('storage: viewer cannot read team-assets', 'VIEW', $q$select count(*)::int from storage.objects where bucket_id = 'eva-team-assets'$q$, 0);
select pg_temp.t_count('storage: anon cannot list team-assets', 'anon', $q$select count(*)::int from storage.objects where bucket_id = 'eva-team-assets'$q$, 0);
select pg_temp.t_count('storage: team-assets is private, public-media is public', 'service', $q$select count(*)::int from storage.buckets where (id = 'eva-team-assets' and not public) or (id = 'eva-public-media' and public)$q$, 2);

-- public_stats: aggregates for anon, nothing sensitive
select pg_temp.t_count('public_stats: anon reads aggregates', 'anon', $q$select acknowledged_records::int from eva.public_stats where ship = 'rls-demo'$q$, 3);
select pg_temp.t_count('public_stats: only aggregate columns', 'service', $q$select count(*)::int from information_schema.columns where table_schema = 'public' and table_name = 'public_stats' and column_name !~ '^(ship|display_name|demo|epoch|acknowledged_records|last_ingest_at|open_cases|total_cases|passengers|station_mode)$'$q$, 0);

-- first use: ensure_profile
select pg_temp.t_exec('ensure_profile: anon refused', 'anon', $q$select eva.ensure_profile()$q$, 'denied');
select pg_temp.t_count('ensure_profile: new user gets viewer', 'NEW', $q$select count(*)::int from (select eva.ensure_profile() r) x where r = 'viewer'$q$, 1);
select pg_temp.t_count('ensure_profile: idempotent, role kept', 'MC', $q$select count(*)::int from (select eva.ensure_profile() r) x where r = 'mission_control'$q$, 1);
select pg_temp.t_count('ensure_profile: new profile is visible to its owner only', 'NEW', $q$select count(*)::int from eva.profiles$q$, 1);

-- shared-project isolation: nothing of Eva outside the eva schema (bar its storage policies)
select pg_temp.t_count('isolation: no Eva trigger on auth.users', 'owner', $q$select count(*)::int from pg_trigger where tgrelid = 'auth.users'::regclass and tgfoid::regproc::text like 'eva.%'$q$, 0);
select pg_temp.t_count('isolation: storage policies only touch eva- buckets', 'owner', $q$select count(*)::int from pg_policies where schemaname = 'storage' and policyname like 'eva-%' and coalesce(qual, '') || coalesce(with_check, '') !~ 'eva-(team-assets|public-media)'$q$, 0);

-- ---------------------------------------------------------------- results
-- @@results

\o
\set QUIET off
select n, case when ok then 'PASS' else 'FAIL' end as result, label, got, expected from t_results order by n;
select count(*) filter (where ok) as passed, count(*) filter (where not ok) as failed, count(*) as total from t_results;
do $$ declare f int; begin
  select count(*) into f from t_results where not ok;
  if f > 0 then raise exception '% RLS check(s) failed', f; end if;
end $$;
rollback;
