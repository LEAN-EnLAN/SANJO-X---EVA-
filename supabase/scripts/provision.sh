#!/usr/bin/env bash
# Set up (or re-run) Eva inside an EXISTING Supabase project, in its own `eva` schema (ADR-014,
# "Shared project"). Idempotent. Never prints a secret, never reads the host app's data, and only
# ever APPENDS to project-wide settings (exposed schemas, auth redirect URLs).
#
#   set -a; . ~/.eva-supabase.env; set +a        # SUPABASE_ACCESS_TOKEN (personal access token)
#   EVA_SITE_DIR=… EVA_VERCEL=… supabase/scripts/provision.sh [--no-site]
#
# Env (defaults in brackets):
#   EVA_SB_REF [rcxmvcxyrwufmoylbrrz = changuito]
#   EVA_SB_SECRETS [/home/user/.eva-supabase-project.env]  (anon key, URL, eva-test passwords; chmod 600)
#   EVA_GROQ_ENV [/home/user/.eva-demo.env]                 (line GROQ_API_KEY=…, read, never printed)
#   EVA_SITE_DIR  public site folder linked to the Vercel project eva-wearmeds
#   EVA_VERCEL    vercel CLI path           EVA_BIN [app/target/release/eva] (cloud chain check)
#
# The host project's service-role key is fetched into memory for the admin user API only; it is
# never written to disk.
set -euo pipefail
umask 077

here=$(cd "$(dirname "$0")" && pwd)
sb=$(cd "$here/.." && pwd)
repo=$(cd "$sb/.." && pwd)
export EVA_SB_REF=${EVA_SB_REF:-rcxmvcxyrwufmoylbrrz}
REF=$EVA_SB_REF
SECRETS=${EVA_SB_SECRETS:-/home/user/.eva-supabase-project.env}
GROQ_ENV=${EVA_GROQ_ENV:-/home/user/.eva-demo.env}
SITE_DIR=${EVA_SITE_DIR:-}
VERCEL=${EVA_VERCEL:-vercel}
EVA_BIN=${EVA_BIN:-$repo/app/target/release/eva}
SITE_URL=https://eva-wearmeds.vercel.app
API=https://api.supabase.com/v1
DO_SITE=1
[ "${1:-}" = "--no-site" ] && DO_SITE=0

: "${SUPABASE_ACCESS_TOKEN:?set SUPABASE_ACCESS_TOKEN (personal access token)}"
work=$(mktemp -d "${TMPDIR:-/tmp}/eva-sb.XXXXXX")
trap 'rm -rf "$work"' EXIT
say() { printf '\n== %s\n' "$*"; }
sql() { "$here/sql.sh" -c "$1"; }
sqlv() { sql "$1" | jq -r '.[0] | to_entries[0].value // empty'; }   # first column of first row

# ---------------------------------------------------------------- secrets file helpers
touch "$SECRETS"; chmod 600 "$SECRETS"
put() { # put KEY  (value on stdin; never echoed)
  python3 -I -c '
import os, sys
path, key = sys.argv[1], sys.argv[2]
val = sys.stdin.read().rstrip("\n")
lines = [l for l in open(path).read().splitlines() if l and not l.startswith(key + "=")]
lines.append(key + "=" + val)
fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600)
os.write(fd, ("\n".join(lines) + "\n").encode())
os.close(fd)
os.chmod(path, 0o600)
' "$SECRETS" "$1"
}
load() { set -a; . "$SECRETS"; set +a; }
load

mgmt() { # mgmt METHOD PATH [curl args...] → body on stdout; fails on HTTP >= 400 (message only)
  local m=$1 p=$2; shift 2
  local code; code=$(curl -sS -X "$m" -H "Authorization: Bearer $SUPABASE_ACCESS_TOKEN" -H 'content-type: application/json' \
    -o "$work/resp" -w '%{http_code}' "$@" "$API$p")
  if [ "$code" -ge 400 ]; then
    echo "Management API $m $p → HTTP $code: $(jq -r '.message // .error // "?"' "$work/resp" 2>/dev/null | head -c 400)" >&2
    return 1
  fi
  cat "$work/resp"
}

# ---------------------------------------------------------------- 0 · inspect, refuse collisions
say "0 · inspect project $REF (names only)"
mgmt GET "/projects/$REF" | jq -r '"  \(.name) · \(.region) · \(.status)"'
URL="https://$REF.supabase.co"
eva_exists=$(sqlv "select (to_regclass('eva.schema_migrations') is not null)::text")
schema_exists=$(sqlv "select exists(select 1 from pg_namespace where nspname = 'eva')::text")
if [ "$schema_exists" = true ] && [ "$eva_exists" != true ]; then
  echo "a schema named eva exists but is not Eva's (no eva.schema_migrations): stopping" >&2; exit 3
fi
fn_exists=$(mgmt GET "/projects/$REF/functions" | jq -r '[.[].slug] | index("eva-chat") != null')
if [ "$fn_exists" = true ] && [ "$eva_exists" != true ]; then echo "an eva-chat function exists that Eva did not create: stopping" >&2; exit 3; fi
buckets=$(sqlv "select coalesce(string_agg(id, ','), '') from storage.buckets where id in ('eva-team-assets', 'eva-public-media')")
if [ -n "$buckets" ] && [ "$eva_exists" != true ]; then echo "bucket(s) $buckets exist but Eva did not create them: stopping" >&2; exit 3; fi
pols=$(sqlv "select count(*)::text from pg_policies where schemaname = 'storage' and policyname like 'eva-%' and policyname not in ('eva-team-assets read','eva-team-assets upload','eva-team-assets update','eva-team-assets delete','eva-public-media list','eva-public-media upload','eva-public-media update','eva-public-media delete')")
[ "$pols" = 0 ] || { echo "unexpected eva-* storage policies exist: stopping" >&2; exit 3; }
secret_names=$(mgmt GET "/projects/$REF/secrets" | jq -r '[.[].name] | join(",")')
echo "  eva schema: ${eva_exists} · eva-chat: ${fn_exists} · eva buckets: ${buckets:-none} · function secret names: ${secret_names:-none}"

# ---------------------------------------------------------------- 1 · keys
say "1 · API keys (anon → secrets file; service role → memory only)"
mgmt GET "/projects/$REF/api-keys?reveal=true" > "$work/keys.json"
jq -r '([.[] | select(.name == "anon")][0].api_key) // ([.[] | select(.type == "publishable")][0].api_key)' "$work/keys.json" | put SUPABASE_ANON_KEY
SRK=$(jq -r '([.[] | select(.name == "service_role")][0].api_key) // ([.[] | select(.type == "secret")][0].api_key)' "$work/keys.json")
rm -f "$work/keys.json"
printf '%s' "$REF" | put SUPABASE_PROJECT_REF
printf '%s' "$URL" | put SUPABASE_URL
sed -i '/^SUPABASE_DB_PASSWORD=/d' "$SECRETS"   # left over from the dedicated-project attempt; unused
load
[ -n "$SUPABASE_ANON_KEY" ] && [ -n "$SRK" ] || { echo "keys missing" >&2; exit 5; }

# ---------------------------------------------------------------- 2 · migrations (eva.schema_migrations)
say "2 · migrations"
for f in "$sb"/migrations/*.sql; do
  b=$(basename "$f" .sql); v=${b%%_*}; n=${b#*_}
  if [ "$(sqlv "select (to_regclass('eva.schema_migrations') is not null and exists(select 1 from eva.schema_migrations where version = '$v'))::text")" = true ]; then
    echo "  = $b"; continue
  fi
  { echo "begin;"; cat "$f"; echo; echo "insert into eva.schema_migrations (version, name) values ('$v', '$n');"; echo "commit;"; } > "$work/m.sql"
  "$here/sql.sh" -f "$work/m.sql" >/dev/null
  echo "  + $b"
done
sql "notify pgrst, 'reload schema'" >/dev/null

# ---------------------------------------------------------------- 3 · expose eva to the Data API (append only)
say "3 · PostgREST exposed schemas (append eva)"
cur=$(mgmt GET "/projects/$REF/postgrest" | jq -r '.db_schema')    # select one field: the response also holds a secret
if tr ',' '\n' <<<"$cur" | sed 's/ //g' | grep -qx eva; then
  echo "  already: $cur"
else
  jq -n --arg s "$cur,eva" '{db_schema: $s}' > "$work/pg.json"
  mgmt PATCH "/projects/$REF/postgrest" --data @"$work/pg.json" | jq -r '"  now: \(.db_schema)"'
fi

# ---------------------------------------------------------------- 4 · seed
say "4 · seed (fictional demo data)"
"$here/sql.sh" -f "$sb/seed.sql" >/dev/null && echo "  applied (idempotent)"

# ---------------------------------------------------------------- 5 · auth: append redirect URLs only
say "5 · auth redirect URLs (append; site URL, providers, sign-ups and SMTP untouched)"
allow=$(mgmt GET "/projects/$REF/config/auth" | jq -r '.uri_allow_list // ""')
new=$allow
for u in "$SITE_URL" "$SITE_URL/**" "http://localhost:8080" "http://localhost:8080/**"; do
  tr ',' '\n' <<<"$new" | grep -qxF "$u" || new="${new:+$new,}$u"
done
if [ "$new" != "$allow" ]; then
  jq -n --arg l "$new" '{uri_allow_list: $l}' > "$work/auth.json"
  mgmt PATCH "/projects/$REF/config/auth" --data @"$work/auth.json" | jq -r '"  uri_allow_list: \(.uri_allow_list)"'
else
  echo "  already: $allow"
fi

# ---------------------------------------------------------------- 6 · eva-test users
say "6 · eva-test users (passwords only in $SECRETS)"
srk_curl() { curl -sS -H "apikey: $SRK" -H "Authorization: Bearer $SRK" -H 'content-type: application/json' "$@"; }
for spec in crew-lead:crew_lead mission-control:mission_control team:team viewer:viewer team-2:team new:; do
  slug=${spec%%:*}; role=${spec#*:}
  V=$(tr '[:lower:]-' '[:upper:]_' <<<"$slug")
  email="eva-test-$slug@example.com"; pw_var="EVA_TEST_${V}_PASSWORD"
  printf '%s' "$email" | put "EVA_TEST_${V}_EMAIL"
  [ -n "${!pw_var:-}" ] || { python3 -I -c "import secrets;print(secrets.token_urlsafe(18))" | put "$pw_var"; load; }
  uid=$(sqlv "select id::text from auth.users where email = '$email'")     # only our own test users
  jq -n --arg e "$email" --arg p "${!pw_var}" --arg s "$slug" '{email:$e, password:$p, email_confirm:true, user_metadata:{name:("Eva test " + $s)}}' > "$work/u.json"
  if [ -z "$uid" ]; then
    uid=$(srk_curl -X POST "$URL/auth/v1/admin/users" --data @"$work/u.json" | jq -r '.id // empty')
    [ -n "$uid" ] || { echo "cannot create $email" >&2; exit 6; }
  else
    jq '{password, email_confirm}' "$work/u.json" | srk_curl -o /dev/null -X PUT "$URL/auth/v1/admin/users/$uid" --data @-
  fi
  rm -f "$work/u.json"
  if [ -n "$role" ]; then
    sql "begin; set local role service_role; set local request.jwt.claims = '{\"role\":\"service_role\"}';
         insert into eva.profiles (id, email, role) values ('$uid', '$email', '$role') on conflict (id) do update set role = excluded.role; commit;" >/dev/null
  fi
  echo "  ${role:-no Eva profile (first-use test)} → $email"
done
load

# ---------------------------------------------------------------- 7 · RLS tests (rolled back)
say "7 · RLS tests on the hosted project"
"$here/sql.sh" --rls-test > "$work/rls.txt" || { grep -v '^PASS' "$work/rls.txt"; exit 7; }
grep -v '^PASS' "$work/rls.txt"

# ---------------------------------------------------------------- 8 · eva-chat
say "8 · eva-chat: EVA_GROQ_API_KEY secret + deploy"
python3 -I - "$GROQ_ENV" <<'PY' > "$work/secrets.json"
import json, sys
key = next((l.split("=", 1)[1].strip().strip('"\'') for l in open(sys.argv[1]) if l.startswith("GROQ_API_KEY=")), "")
if not key: sys.exit("GROQ_API_KEY not found")
print(json.dumps([{"name": "EVA_GROQ_API_KEY", "value": key}]))
PY
mgmt POST "/projects/$REF/secrets" --data @"$work/secrets.json" >/dev/null && echo "  EVA_GROQ_API_KEY set"
rm -f "$work/secrets.json"
(cd "$repo" && npx -y supabase@latest functions deploy eva-chat --project-ref "$REF" --no-verify-jwt --use-api 2>&1 | tail -n 2)

token_for() { # token_for SLUG → access token
  local V; V=$(tr '[:lower:]-' '[:upper:]_' <<<"$1"); local e="EVA_TEST_${V}_EMAIL" p="EVA_TEST_${V}_PASSWORD"
  jq -n --arg e "${!e}" --arg p "${!p}" '{email:$e, password:$p}' |
    curl -sS -H "apikey: $SUPABASE_ANON_KEY" -H 'content-type: application/json' --data @- "$URL/auth/v1/token?grant_type=password" | jq -r .access_token
}
ask() { # ask TOKEN JSON
  curl -sS -H "apikey: $SUPABASE_ANON_KEY" -H "Authorization: Bearer $1" -H 'content-type: application/json' \
    -H "origin: $SITE_URL" --data "$2" "$URL/functions/v1/eva-chat" |
    jq -c '{source, reason, model, text: ((.text // .error) | .[0:140])}'
}
sleep 8
VT=$(token_for viewer)
echo "  in-scope : $(ask "$VT" '{"text":"How do I stay hydrated in microgravity?"}')"
echo "  off-topic: $(ask "$VT" '{"text":"Who won the football match last night?"}')"
echo "  spanish  : $(ask "$VT" '{"text":"Me duele la cabeza en la nave, ¿qué hago?","lang":"es"}')"
echo "  no JWT   : HTTP $(curl -sS -o /dev/null -w '%{http_code}' -X POST -H "apikey: $SUPABASE_ANON_KEY" --data '{"text":"hi"}' "$URL/functions/v1/eva-chat")"

# ---------------------------------------------------------------- 9 · cloud chain passes the app's verifier
say "9 · eva verify on the mirrored chain"
if [ -x "$EVA_BIN" ]; then
  SUPABASE_URL=$URL SUPABASE_ANON_KEY=$SUPABASE_ANON_KEY EVA_ACCESS_TOKEN=$(token_for crew-lead) \
    node "$here/export-chain.mjs" --ship demo-ship --out "$work/cloud" >/dev/null
  "$EVA_BIN" verify --data "$work/cloud"
fi
echo "  public_stats (anon): $(curl -sS -H "apikey: $SUPABASE_ANON_KEY" -H 'accept-profile: eva' "$URL/rest/v1/public_stats?select=ship,acknowledged_records,open_cases,passengers,station_mode" | jq -c .)"

# ---------------------------------------------------------------- 10 · public site
[ "$DO_SITE" = 1 ] && [ -n "$SITE_DIR" ] || { say "site step skipped (--no-site or EVA_SITE_DIR unset)"; exit 0; }
say "10 · Vercel env + /status/ page + deploy"
cd "$SITE_DIR"
existing=$("$VERCEL" env ls production 2>/dev/null | awk '{print $1}')
for kv in SUPABASE_URL SUPABASE_ANON_KEY; do
  if grep -qx "$kv" <<<"$existing"; then echo "  env $kv already set"; continue; fi
  printf '%s' "${!kv}" | "$VERCEL" env add "$kv" production >/dev/null 2>&1 && echo "  env $kv added (production)"
done
mkdir -p status
cp "$sb/site/status/index.html" status/index.html
cp "$sb/site/api/status.ts" api/status.ts
"$VERCEL" deploy --prod --yes 2>&1 | grep -E '^https://|Error' | tail -n 2
say "secret scan of the deployed folder (counts of files)"
scan() { grep -rlF --exclude-dir=.vercel --exclude='.env*' --exclude-dir=node_modules -e "$1" . 2>/dev/null | wc -l; }
echo "  service key: $(scan "$SRK") · groq key: $(scan "$(sed -n 's/^GROQ_API_KEY=//p' "$GROQ_ENV" | tr -d '"'"'")") · eva-test passwords: $(for v in $(compgen -v | grep '^EVA_TEST_.*_PASSWORD$'); do scan "${!v}"; done | awk "{s+=\$1} END {print s+0}") · sbp_: $(scan sbp_) · gsk_: $(scan gsk_)"
sleep 4
echo "  $SITE_URL/api/status → $(curl -sS "$SITE_URL/api/status" | jq -c '{ok, ships: [.ships[]? | {ship, acknowledged_records}]}')"
echo "  $SITE_URL/status/ → HTTP $(curl -sS -o /dev/null -w '%{http_code}' "$SITE_URL/status/")"
