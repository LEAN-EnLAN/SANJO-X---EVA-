-- 5/6 · per-user rate limit for the eva-chat Edge Function.
-- The table is server-only: RLS on, no policies, no client grants. The function calls
-- eva_chat_consume() with the caller's own JWT; it can only count against the caller's bucket.

create table if not exists eva.rate_limits (
  user_id      uuid primary key references auth.users (id) on delete cascade,
  minute_start timestamptz not null,
  minute_count integer not null default 0,
  day_start    timestamptz not null,
  day_count    integer not null default 0,
  updated_at   timestamptz not null default now()
);
alter table eva.rate_limits enable row level security;
revoke all on eva.rate_limits from anon, authenticated;
grant all on eva.rate_limits to service_role;

-- Check-and-count in one statement (row lock). Returns the caller's role and, when refused,
-- the seconds to wait. role is null when the caller has no profile (the function refuses them).
create or replace function eva.eva_chat_consume(per_minute integer default 6, per_day integer default 80)
returns table (role eva.app_role, allowed boolean, retry_after integer)
language plpgsql volatile security definer
set search_path = ''
as $$
declare
  uid uuid := auth.uid();
  r   eva.app_role;
  b   eva.rate_limits;
  t   timestamptz := clock_timestamp();
begin
  per_minute := least(greatest(per_minute, 1), 6);   -- callers may tighten, never loosen
  per_day    := least(greatest(per_day, 1), 80);
  if uid is null then
    return query select null::eva.app_role, false, 0; return;
  end if;
  select p.role into r from eva.profiles p where p.id = uid;
  if r is null then
    return query select null::eva.app_role, false, 0; return;
  end if;

  insert into eva.rate_limits as rl (user_id, minute_start, minute_count, day_start, day_count)
  values (uid, t, 0, t, 0)
  on conflict (user_id) do nothing;
  select * into b from eva.rate_limits where user_id = uid for update;

  if t - b.minute_start >= interval '1 minute' then b.minute_start := t; b.minute_count := 0; end if;
  if t - b.day_start >= interval '1 day' then b.day_start := t; b.day_count := 0; end if;

  if b.day_count >= per_day then
    update eva.rate_limits set minute_start = b.minute_start, minute_count = b.minute_count,
           day_start = b.day_start, day_count = b.day_count, updated_at = t where user_id = uid;
    return query select r, false, ceil(extract(epoch from (b.day_start + interval '1 day' - t)))::int; return;
  end if;
  if b.minute_count >= per_minute then
    update eva.rate_limits set minute_start = b.minute_start, minute_count = b.minute_count,
           day_start = b.day_start, day_count = b.day_count, updated_at = t where user_id = uid;
    return query select r, false, ceil(extract(epoch from (b.minute_start + interval '1 minute' - t)))::int; return;
  end if;

  update eva.rate_limits set minute_start = b.minute_start, minute_count = b.minute_count + 1,
         day_start = b.day_start, day_count = b.day_count + 1, updated_at = t where user_id = uid;
  return query select r, true, 0;
end $$;

revoke all on function eva.eva_chat_consume(integer, integer) from public, anon;
grant execute on function eva.eva_chat_consume(integer, integer) to authenticated, service_role;

-- Housekeeping: buckets idle for two days (call from pg_cron if enabled, or by hand).
create or replace function eva.cleanup_rate_limits()
returns integer
language sql volatile security definer
set search_path = ''
as $$
  with d as (delete from eva.rate_limits where updated_at < now() - interval '2 days' returning 1)
  select count(*)::int from d
$$;
revoke all on function eva.cleanup_rate_limits() from public, anon, authenticated;
grant execute on function eva.cleanup_rate_limits() to service_role;
