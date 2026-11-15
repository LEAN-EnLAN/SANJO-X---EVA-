-- 6/6 · public_stats: counts only, safe for the anon key (the /status/ page on the public site).
-- No health data, no names, no seats, no hashes, no keys: per ship, how many signed records the
-- ground has acknowledged and mirrored, the latest ingest time, open cases and the station mode.
--
-- The view runs with its owner's rights (security_invoker = false) on purpose, so anon can see the
-- aggregates without any read access to the underlying tables. Only aggregates are selected.

create or replace view eva.public_stats
with (security_invoker = false, security_barrier = true)
as
select
  s.id                                                         as ship,
  s.display_name,
  s.demo,
  s.epoch,
  (select count(*) from eva.records r where r.ship = s.id)   as acknowledged_records,
  (select max(r.ingested_at) from eva.records r where r.ship = s.id) as last_ingest_at,
  (select count(*) from eva.cases c where c.ship = s.id and c.open) as open_cases,
  (select count(*) from eva.cases c where c.ship = s.id)     as total_cases,
  (select count(*) from eva.passengers p where p.ship = s.id) as passengers,
  (select st.mode from eva.station_state st where st.ship = s.id) as station_mode
from eva.ships s;

comment on view eva.public_stats is 'Public aggregates for the /status/ page. Exposes nothing sensitive.';

revoke all on eva.public_stats from anon, authenticated;
grant select on eva.public_stats to anon, authenticated, service_role;
