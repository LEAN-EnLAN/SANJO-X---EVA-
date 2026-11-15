-- Eva by WearMeds · Supabase (Earth / cloud side only; the ship never talks to Supabase). ADR-014.
-- 1/6 · roles and profiles.
--
-- Roles live in public.profiles.role, one profile per auth user, default 'viewer'.
-- A user can never change their own role: the column is not granted for UPDATE to clients,
-- and the only ways to change it are public.admin_set_role() (mission control) or the service role.

create extension if not exists pgcrypto with schema extensions;

do $$ begin
  create type public.app_role as enum ('crew_lead', 'mission_control', 'team', 'viewer');
exception when duplicate_object then null; end $$;

create table if not exists public.profiles (
  id           uuid primary key references auth.users (id) on delete cascade,
  display_name text check (display_name is null or char_length(display_name) <= 80),
  email        text check (email is null or char_length(email) <= 320),
  role         public.app_role not null default 'viewer',
  prefs        jsonb not null default '{}'::jsonb check (jsonb_typeof(prefs) = 'object' and pg_column_size(prefs) <= 8192),
  created_at   timestamptz not null default now(),
  updated_at   timestamptz not null default now()
);
comment on table public.profiles is 'One per auth user. role is assigned only by admin_set_role() or the service role.';

-- ---------------------------------------------------------------- role helpers
-- SECURITY DEFINER so policies can read the caller's role without a policy on profiles recursing.

create or replace function public.my_role()
returns public.app_role
language sql stable security definer
set search_path = ''
as $$
  select p.role from public.profiles p where p.id = auth.uid()
$$;

create or replace function public.has_role(variadic roles public.app_role[])
returns boolean
language sql stable security definer
set search_path = ''
as $$
  select coalesce((select p.role = any (roles) from public.profiles p where p.id = auth.uid()), false)
$$;

-- True for the service role (Edge Functions with the service key, the ground ingest, scripts).
create or replace function public.is_service()
returns boolean
language sql stable
set search_path = ''
as $$
  select current_user = 'service_role' or coalesce(auth.role(), '') = 'service_role'
$$;

revoke all on function public.my_role() from public;
revoke all on function public.has_role(public.app_role[]) from public;
grant execute on function public.my_role() to authenticated, service_role;
grant execute on function public.has_role(public.app_role[]) to anon, authenticated, service_role;
grant execute on function public.is_service() to anon, authenticated, service_role;

-- ---------------------------------------------------------------- new user -> profile

create or replace function public.handle_new_user()
returns trigger
language plpgsql security definer
set search_path = ''
as $$
begin
  insert into public.profiles (id, email, display_name)
  values (
    new.id,
    new.email,
    left(coalesce(new.raw_user_meta_data ->> 'full_name', new.raw_user_meta_data ->> 'name', split_part(coalesce(new.email, ''), '@', 1)), 80)
  )
  on conflict (id) do nothing;
  return new; -- role is always the default ('viewer'); sign-up metadata can never set it
end $$;

drop trigger if exists on_auth_user_created on auth.users;
create trigger on_auth_user_created
  after insert on auth.users
  for each row execute function public.handle_new_user();

-- Keep id, email, role and created_at out of reach of a client UPDATE (belt and braces with
-- the column grants below), and stamp updated_at.
create or replace function public.profiles_guard()
returns trigger
language plpgsql
set search_path = ''
as $$
begin
  if new.id <> old.id or new.created_at <> old.created_at then
    raise exception 'profiles: id and created_at are immutable' using errcode = '42501';
  end if;
  if new.role is distinct from old.role
     and not public.is_service()
     and coalesce(current_setting('eva.role_change', true), '') <> 'admin_set_role' then
    raise exception 'profiles: role is assigned only by admin_set_role() or the service role' using errcode = '42501';
  end if;
  new.updated_at := now();
  return new;
end $$;

drop trigger if exists profiles_guard on public.profiles;
create trigger profiles_guard before update on public.profiles
  for each row execute function public.profiles_guard();

-- ---------------------------------------------------------------- admin RPC

-- Mission control (or the service role) assigns roles. Audited in public.role_changes.
create table if not exists public.role_changes (
  id          bigint generated always as identity primary key,
  target      uuid not null,
  old_role    public.app_role,
  new_role    public.app_role not null,
  changed_by  uuid,             -- null = service role
  changed_at  timestamptz not null default now()
);

create or replace function public.admin_set_role(target uuid, new_role public.app_role)
returns public.app_role
language plpgsql security definer
set search_path = ''
as $$
declare
  prior public.app_role;
begin
  if not (public.is_service() or public.has_role('mission_control')) then
    raise exception 'admin_set_role: mission control or service role only' using errcode = '42501';
  end if;
  select role into prior from public.profiles where id = target for update;
  if not found then
    raise exception 'admin_set_role: no profile for %', target using errcode = 'P0002';
  end if;
  perform set_config('eva.role_change', 'admin_set_role', true);
  update public.profiles set role = new_role where id = target;
  perform set_config('eva.role_change', '', true);
  insert into public.role_changes (target, old_role, new_role, changed_by)
  values (target, prior, new_role, case when public.is_service() then null else auth.uid() end);
  return new_role;
end $$;

revoke all on function public.admin_set_role(uuid, public.app_role) from public, anon;
grant execute on function public.admin_set_role(uuid, public.app_role) to authenticated, service_role;

-- ---------------------------------------------------------------- grants + RLS

alter table public.profiles enable row level security;
alter table public.role_changes enable row level security;

revoke all on public.profiles from anon, authenticated;
grant select on public.profiles to authenticated;
grant update (display_name, prefs) on public.profiles to authenticated;
grant all on public.profiles to service_role;

revoke all on public.role_changes from anon, authenticated;
grant select on public.role_changes to authenticated;
grant all on public.role_changes to service_role;

drop policy if exists profiles_read on public.profiles;
create policy profiles_read on public.profiles for select to authenticated
  using (id = (select auth.uid()) or (select public.has_role('mission_control')));

drop policy if exists profiles_update_own on public.profiles;
create policy profiles_update_own on public.profiles for update to authenticated
  using (id = (select auth.uid()))
  with check (id = (select auth.uid()));

drop policy if exists role_changes_read on public.role_changes;
create policy role_changes_read on public.role_changes for select to authenticated
  using ((select public.has_role('mission_control')));
