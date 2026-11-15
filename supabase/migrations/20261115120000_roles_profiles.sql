-- Eva by WearMeds · Supabase (Earth / cloud side only; the ship never talks to Supabase). ADR-014.
-- 1/6 · schema, roles and profiles.
--
-- Everything Eva owns lives in the `eva` schema, so it can share a Supabase project with another
-- app without touching that app's tables, functions or auth hooks (ADR-014, "Shared project").
-- No trigger on auth.users: an Eva profile is created on first use by eva.ensure_profile(), so
-- users of the other app get nothing in `eva` unless they use Eva.
--
-- Roles live in eva.profiles.role, default 'viewer'. A user can never change their own role: the
-- column is not granted for UPDATE to clients, and the only ways to change it are
-- eva.admin_set_role() (mission control) or the service role.

create schema if not exists eva;
comment on schema eva is 'Eva by WearMeds (Earth side). Self-contained; see supabase/README.md.';
revoke all on schema eva from public;
grant usage on schema eva to anon, authenticated, service_role;

-- Applied-migration log for this schema only (the project's supabase_migrations belongs to the host app).
create table if not exists eva.schema_migrations (
  version    text primary key,
  name       text not null,
  applied_at timestamptz not null default now()
);
revoke all on eva.schema_migrations from anon, authenticated;

do $$ begin
  create type eva.app_role as enum ('crew_lead', 'mission_control', 'team', 'viewer');
exception when duplicate_object then null; end $$;

create table if not exists eva.profiles (
  id           uuid primary key references auth.users (id) on delete cascade,
  display_name text check (display_name is null or char_length(display_name) <= 80),
  email        text check (email is null or char_length(email) <= 320),
  role         eva.app_role not null default 'viewer',
  prefs        jsonb not null default '{}'::jsonb check (jsonb_typeof(prefs) = 'object' and pg_column_size(prefs) <= 8192),
  created_at   timestamptz not null default now(),
  updated_at   timestamptz not null default now()
);
comment on table eva.profiles is 'One per Eva user (created by eva.ensure_profile() on first use). role is assigned only by admin_set_role() or the service role.';

-- ---------------------------------------------------------------- role helpers
-- SECURITY DEFINER so policies can read the caller's role without a policy on profiles recursing.

create or replace function eva.my_role()
returns eva.app_role
language sql stable security definer
set search_path = ''
as $$
  select p.role from eva.profiles p where p.id = auth.uid()
$$;

create or replace function eva.has_role(variadic roles eva.app_role[])
returns boolean
language sql stable security definer
set search_path = ''
as $$
  select coalesce((select p.role = any (roles) from eva.profiles p where p.id = auth.uid()), false)
$$;

-- True for the service role (Edge Functions with the service key, the ground ingest, scripts).
create or replace function eva.is_service()
returns boolean
language sql stable
set search_path = ''
as $$
  select current_user = 'service_role' or coalesce(auth.role(), '') = 'service_role'
$$;

revoke all on function eva.my_role() from public;
revoke all on function eva.has_role(eva.app_role[]) from public;
grant execute on function eva.my_role() to authenticated, service_role;
grant execute on function eva.has_role(eva.app_role[]) to anon, authenticated, service_role;
grant execute on function eva.is_service() to anon, authenticated, service_role;

-- ---------------------------------------------------------------- first use -> profile

-- Called by the signed-in user (the web app after sign-in, and eva-chat before each message).
-- Creates their Eva profile with the default role if it does not exist; returns their role.
-- Name and email come from the caller's own JWT; sign-up metadata can never set the role.
create or replace function eva.ensure_profile()
returns eva.app_role
language plpgsql volatile security definer
set search_path = ''
as $$
declare
  uid uuid := auth.uid();
  claims jsonb := coalesce(auth.jwt(), '{}'::jsonb);
  r eva.app_role;
begin
  if uid is null or coalesce(claims ->> 'role', '') <> 'authenticated' then
    raise exception 'ensure_profile: sign in first' using errcode = '42501';
  end if;
  insert into eva.profiles (id, email, display_name)
  values (
    uid,
    left(claims ->> 'email', 320),
    left(coalesce(claims -> 'user_metadata' ->> 'full_name', claims -> 'user_metadata' ->> 'name',
                  split_part(coalesce(claims ->> 'email', ''), '@', 1)), 80)
  )
  on conflict (id) do nothing;
  select p.role into r from eva.profiles p where p.id = uid;
  return r;
end $$;
revoke all on function eva.ensure_profile() from public, anon;
grant execute on function eva.ensure_profile() to authenticated;

-- Keep id, email, role and created_at out of reach of a client UPDATE (belt and braces with
-- the column grants below), and stamp updated_at.
create or replace function eva.profiles_guard()
returns trigger
language plpgsql
set search_path = ''
as $$
begin
  if new.id <> old.id or new.created_at <> old.created_at then
    raise exception 'profiles: id and created_at are immutable' using errcode = '42501';
  end if;
  if new.role is distinct from old.role
     and not eva.is_service()
     and coalesce(current_setting('eva.role_change', true), '') <> 'admin_set_role' then
    raise exception 'profiles: role is assigned only by admin_set_role() or the service role' using errcode = '42501';
  end if;
  new.updated_at := now();
  return new;
end $$;

drop trigger if exists profiles_guard on eva.profiles;
create trigger profiles_guard before update on eva.profiles
  for each row execute function eva.profiles_guard();

-- ---------------------------------------------------------------- admin RPC

-- Mission control (or the service role) assigns roles. Audited in eva.role_changes.
create table if not exists eva.role_changes (
  id          bigint generated always as identity primary key,
  target      uuid not null,
  old_role    eva.app_role,
  new_role    eva.app_role not null,
  changed_by  uuid,             -- null = service role
  changed_at  timestamptz not null default now()
);

create or replace function eva.admin_set_role(target uuid, new_role eva.app_role)
returns eva.app_role
language plpgsql security definer
set search_path = ''
as $$
declare
  prior eva.app_role;
begin
  if not (eva.is_service() or eva.has_role('mission_control')) then
    raise exception 'admin_set_role: mission control or service role only' using errcode = '42501';
  end if;
  select role into prior from eva.profiles where id = target for update;
  if not found then
    raise exception 'admin_set_role: no profile for %', target using errcode = 'P0002';
  end if;
  perform set_config('eva.role_change', 'admin_set_role', true);
  update eva.profiles set role = new_role where id = target;
  perform set_config('eva.role_change', '', true);
  insert into eva.role_changes (target, old_role, new_role, changed_by)
  values (target, prior, new_role, case when eva.is_service() then null else auth.uid() end);
  return new_role;
end $$;

revoke all on function eva.admin_set_role(uuid, eva.app_role) from public, anon;
grant execute on function eva.admin_set_role(uuid, eva.app_role) to authenticated, service_role;

-- ---------------------------------------------------------------- grants + RLS

alter table eva.profiles enable row level security;
alter table eva.role_changes enable row level security;

revoke all on eva.profiles from anon, authenticated;
grant select on eva.profiles to authenticated;
grant update (display_name, prefs) on eva.profiles to authenticated;
grant all on eva.profiles to service_role;

revoke all on eva.role_changes from anon, authenticated;
grant select on eva.role_changes to authenticated;
grant all on eva.role_changes to service_role;

drop policy if exists profiles_read on eva.profiles;
create policy profiles_read on eva.profiles for select to authenticated
  using (id = (select auth.uid()) or (select eva.has_role('mission_control')));

drop policy if exists profiles_update_own on eva.profiles;
create policy profiles_update_own on eva.profiles for update to authenticated
  using (id = (select auth.uid()))
  with check (id = (select auth.uid()));

drop policy if exists role_changes_read on eva.role_changes;
create policy role_changes_read on eva.role_changes for select to authenticated
  using ((select eva.has_role('mission_control')));
