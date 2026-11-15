#!/usr/bin/env bash
# Run SQL against the hosted project through the Management API (POST /v1/projects/{ref}/database/query),
# as postgres, without the database password. Used by provision.sh; handy by hand.
#
#   SUPABASE_ACCESS_TOKEN=… EVA_SB_REF=… supabase/scripts/sql.sh -c "select 1"
#   supabase/scripts/sql.sh -f supabase/migrations/20261115120000_roles_profiles.sql
#   supabase/scripts/sql.sh --rls-test     # tests/rls.test.sql, always rolled back; prints PASS/FAIL lines
#
# Prints the JSON result of the last statement. Exit 1 on an SQL error (message on stderr).
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
: "${SUPABASE_ACCESS_TOKEN:?set SUPABASE_ACCESS_TOKEN}"
REF=${EVA_SB_REF:-rcxmvcxyrwufmoylbrrz}

post() { # SQL on stdin → response body; returns 1 on HTTP >= 400
  local out code
  out=$(mktemp "${TMPDIR:-/tmp}/eva-sql.XXXXXX")
  code=$(jq -Rs '{query: .}' | curl -sS -X POST -H "Authorization: Bearer $SUPABASE_ACCESS_TOKEN" -H 'content-type: application/json' \
    --data @- -o "$out" -w '%{http_code}' "https://api.supabase.com/v1/projects/$REF/database/query")
  cat "$out"; rm -f "$out"
  [ "$code" -lt 400 ]
}

case "${1:-}" in
  -c) printf '%s' "$2" | post || { echo >&2; exit 1; } ;;
  -f) post < "$2" || { echo >&2; exit 1; } ;;
  --rls-test)
    # psql meta-commands out; the psql report after "-- @@results" replaced by one RAISE that carries
    # the results and aborts the transaction, so every fixture and change is rolled back.
    body=$(sed -n '/^-- @@results/q;/^\\/!p' "$here/../tests/rls.test.sql")
    resp=$( { printf '%s\n' "$body"; cat <<'SQL'
do $$ begin
  raise exception 'EVA_RLS_RESULT %', (select json_build_object(
    'passed', count(*) filter (where ok), 'failed', count(*) filter (where not ok), 'total', count(*),
    'rows', json_agg(json_build_object('n', n, 'ok', ok, 'label', label, 'got', got, 'expected', expected) order by n))
    from t_results);
end $$;
SQL
    } | post || true)
    json=$(jq -r '.message // ""' <<<"$resp" | sed -n 's/.*EVA_RLS_RESULT //p' | head -1)
    if [ -z "$json" ]; then echo "RLS test did not run: $(jq -r '.message // .' <<<"$resp" | head -c 600)" >&2; exit 1; fi
    jq -r '.rows[] | "\(if .ok then "PASS" else "FAIL" end)  \(.n). \(.label)  [got \(.got), expected \(.expected)]"' <<<"$json"
    jq -r '"\(.passed) passed, \(.failed) failed, \(.total) total"' <<<"$json"
    [ "$(jq -r .failed <<<"$json")" = 0 ]
    ;;
  *) echo "usage: sql.sh -c SQL | -f FILE | --rls-test" >&2; exit 2 ;;
esac
