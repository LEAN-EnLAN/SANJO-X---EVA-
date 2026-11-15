#!/usr/bin/env bash
# Apply the migrations to a throwaway local PostgreSQL (no Docker) and run the RLS tests.
#   supabase/tests/local/run.sh            # needs initdb/pg_ctl (PostgreSQL 15+) on PATH or in /usr/lib/postgresql/*/bin
# Uses a temporary data dir and a Unix socket only; nothing listens on TCP; everything is removed at the end.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../.." && pwd)
bin=$(dirname "$(command -v initdb 2>/dev/null || ls -d /usr/lib/postgresql/*/bin/initdb | sort -V | tail -1)")
work=$(mktemp -d "${TMPDIR:-/tmp}/eva-pg.XXXXXX")
cleanup() { "$bin/pg_ctl" -D "$work/data" -m immediate stop >/dev/null 2>&1 || true; rm -rf "$work"; }
trap cleanup EXIT

run_as=()
if [ "$(id -u)" = 0 ]; then chown -R nobody "$work" 2>/dev/null || true; run_as=(runuser -u nobody --); fi
"${run_as[@]}" "$bin/initdb" -D "$work/data" -U postgres -A trust >/dev/null
"${run_as[@]}" "$bin/pg_ctl" -D "$work/data" -o "-k $work -c listen_addresses=''" -l "$work/log" -w start >/dev/null

export PGHOST=$work PGUSER=postgres PGDATABASE=postgres PGOPTIONS="-c client_min_messages=warning -c eva.test_local=on"
psql -q -v ON_ERROR_STOP=1 -f "$here/supabase-stub.sql" >/dev/null
for f in "$root"/migrations/*.sql; do
  psql -q -v ON_ERROR_STOP=1 -f "$f" >/dev/null
  echo "migrated $(basename "$f")"
done
if [ "${1:-}" = "--seed" ] && [ -f "$root/seed.sql" ]; then
  psql -q -v ON_ERROR_STOP=1 -f "$root/seed.sql" >/dev/null && echo "seeded"
  psql -q -c "select ship, acknowledged_records, open_cases, passengers, station_mode from eva.public_stats"
fi
psql -v ON_ERROR_STOP=1 -f "$root/tests/rls.test.sql"
