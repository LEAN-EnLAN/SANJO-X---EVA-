-- 3/6 · per-user journal (private) and the team space.

-- ---------------------------------------------------------------- journal_entries: private per user
-- Only the owner can read or write their entries. No role (not even mission control) can read
-- another user's journal. The ship's on-board journal never enters the chain (ADR-005) and is not
-- synced here; this table is for the web demo / ground-side users' own notes.

create table if not exists eva.journal_entries (
  id         uuid primary key default gen_random_uuid(),
  user_id    uuid not null default auth.uid() references auth.users (id) on delete cascade,
  mood       text check (mood is null or mood in ('great', 'good', 'ok', 'low', 'bad')),
  body       text not null check (char_length(body) between 1 and 10000),
  tags       text[] not null default '{}' check (cardinality(tags) <= 20),
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);
create index if not exists journal_entries_user_idx on eva.journal_entries (user_id, created_at desc);

create or replace function eva.journal_guard()
returns trigger
language plpgsql
set search_path = ''
as $$
begin
  if new.user_id <> old.user_id or new.created_at <> old.created_at then
    raise exception 'journal_entries: user_id and created_at are immutable' using errcode = '42501';
  end if;
  new.updated_at := now();
  return new;
end $$;
drop trigger if exists journal_guard on eva.journal_entries;
create trigger journal_guard before update on eva.journal_entries
  for each row execute function eva.journal_guard();

alter table eva.journal_entries enable row level security;
revoke all on eva.journal_entries from anon, authenticated;
grant select, delete on eva.journal_entries to authenticated;
grant insert (mood, body, tags) on eva.journal_entries to authenticated;
grant update (mood, body, tags) on eva.journal_entries to authenticated;
grant all on eva.journal_entries to service_role;

drop policy if exists journal_owner_select on eva.journal_entries;
create policy journal_owner_select on eva.journal_entries for select to authenticated
  using (user_id = (select auth.uid()));
drop policy if exists journal_owner_insert on eva.journal_entries;
create policy journal_owner_insert on eva.journal_entries for insert to authenticated
  with check (user_id = (select auth.uid()) and (select eva.my_role()) is not null);
drop policy if exists journal_owner_update on eva.journal_entries;
create policy journal_owner_update on eva.journal_entries for update to authenticated
  using (user_id = (select auth.uid())) with check (user_id = (select auth.uid()));
drop policy if exists journal_owner_delete on eva.journal_entries;
create policy journal_owner_delete on eva.journal_entries for delete to authenticated
  using (user_id = (select auth.uid()));

-- ---------------------------------------------------------------- team_docs: reports, notes, asset metadata
-- Read: team, mission control, crew lead. Create: team, mission control. Edit/delete: the author
-- or mission control. Health data never goes here (it lives under the ship tables).

create table if not exists eva.team_docs (
  id           uuid primary key default gen_random_uuid(),
  title        text not null check (char_length(title) between 1 and 200),
  body         text not null default '' check (char_length(body) <= 50000),
  kind         text not null default 'note' check (kind in ('report', 'note', 'asset')),
  tags         text[] not null default '{}' check (cardinality(tags) <= 20),
  storage_path text check (storage_path is null or storage_path ~ '^eva-team-assets/'),
  author_id    uuid default auth.uid() references auth.users (id) on delete set null,
  created_at   timestamptz not null default now(),
  updated_at   timestamptz not null default now()
);
create index if not exists team_docs_kind_idx on eva.team_docs (kind, updated_at desc);

create or replace function eva.team_docs_guard()
returns trigger
language plpgsql
set search_path = ''
as $$
begin
  -- author_id may only become null (the author's auth account was deleted: on delete set null)
  if (new.author_id is not null and new.author_id is distinct from old.author_id) or new.created_at <> old.created_at then
    raise exception 'team_docs: author_id and created_at are immutable' using errcode = '42501';
  end if;
  new.updated_at := now();
  return new;
end $$;
drop trigger if exists team_docs_guard on eva.team_docs;
create trigger team_docs_guard before update on eva.team_docs
  for each row execute function eva.team_docs_guard();

alter table eva.team_docs enable row level security;
revoke all on eva.team_docs from anon, authenticated;
grant select, delete on eva.team_docs to authenticated;
grant insert (title, body, kind, tags, storage_path) on eva.team_docs to authenticated;
grant update (title, body, kind, tags, storage_path) on eva.team_docs to authenticated;
grant all on eva.team_docs to service_role;

drop policy if exists team_docs_read on eva.team_docs;
create policy team_docs_read on eva.team_docs for select to authenticated
  using ((select eva.has_role('team', 'mission_control', 'crew_lead')));
drop policy if exists team_docs_create on eva.team_docs;
create policy team_docs_create on eva.team_docs for insert to authenticated
  with check ((select eva.has_role('team', 'mission_control')) and author_id = (select auth.uid()));
drop policy if exists team_docs_update on eva.team_docs;
create policy team_docs_update on eva.team_docs for update to authenticated
  using ((select eva.has_role('team', 'mission_control'))
         and (author_id = (select auth.uid()) or (select eva.has_role('mission_control'))))
  with check ((select eva.has_role('team', 'mission_control')));
drop policy if exists team_docs_delete on eva.team_docs;
create policy team_docs_delete on eva.team_docs for delete to authenticated
  using ((select eva.has_role('team', 'mission_control'))
         and (author_id = (select auth.uid()) or (select eva.has_role('mission_control'))));
