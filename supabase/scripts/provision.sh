#!/usr/bin/env bash
# Provision (or re-run against) Eva's Supabase project end to end. Idempotent: every step checks
# what already exists. Never prints a secret: values go only to the chmod-600 secrets file.
#
#   SUPABASE_ACCESS_TOKEN must be set (personal access token), e.g.  set -a; . ~/.eva-supabase.env; set +a
#   supabase/scripts/provision.sh [--no-site]
#
# Env (defaults in brackets):
#   EVA_SB_ORG [hgzeaymrcvldebqandwq]   EVA_SB_NAME [eva-wearmeds]   EVA_SB_REGION [sa-east-1]
#   EVA_SB_SECRETS [/home/user/.eva-supabase-project.env]   (DB password, keys, test users)
#   EVA_GROQ_ENV [/home/user/.eva-demo.env]                  (line GROQ_API_KEY=..., read, never printed)
#   EVA_SITE_DIR  (public site folder linked to the Vercel project eva-wearmeds)
#   EVA_VERCEL    (vercel CLI path)          EVA_BIN [app/target/release/eva] (for the chain check)
#
# Free plan only: it creates one project on the free tier and nothing billable. If Supabase refuses
# (e.g. the owner's free-project limit), it stops and says why; nothing is upgraded.
set -euo pipefail
umask 077

here=$(cd "$(dirname "$0")" && pwd)
sb=$(cd "$here/.." && pwd)
repo=$(cd "$sb/.." && pwd)
ORG=${EVA_SB_ORG:-hgzeaymrcvldebqandwq}
NAME=${EVA_SB_NAME:-eva-wearmeds}
REGION=${EVA_SB_REGION:-sa-east-1}
SECRETS=${EVA_SB_SECRETS:-/home/user/.eva-supabase-project.env}
GROQ_ENV=${EVA_GROQ_ENV:-/home/user/.eva-demo.env}
SITE_DIR=${EVA_SITE_DIR:-}
VERCEL=${EVA_VERCEL:-vercel}
EVA_BIN=${EVA_BIN:-$repo/app/target/release/eva}
API=https://api.supabase.com/v1
DO_SITE=1
[ "${1:-}" = "--no-site" ] && DO_SITE=0

: "${SUPABASE_ACCESS_TOKEN:?set SUPABASE_ACCESS_TOKEN (personal access token)}"
work=$(mktemp -d "${TMPDIR:-/tmp}/eva-sb.XXXXXX")
trap 'rm -rf "$work"' EXIT
say() { printf '\n== %s\n' "$*"; }

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

mgmt() { # mgmt METHOD PATH [curl args...]  → body on stdout, fails on HTTP >= 400 (body to stderr, secrets-free)
  local m=$1 p=$2; shift 2
  local code; code=$(curl -sS -X "$m" -H "Authorization: Bearer $SUPABASE_ACCESS_TOKEN" -H 'content-type: application/json' \
    -o "$work/resp" -w '%{http_code}' "$@" "$API$p")
  if [ "$code" -ge 400 ]; then
    echo "Management API $m $p → HTTP $code" >&2
    jq -r '.message // .error // .' "$work/resp" 2>/dev/null | head -c 600 >&2; echo >&2
    return 1
  fi
  cat "$work/resp"
}

# ---------------------------------------------------------------- 1 · project
say "1 · project $NAME ($REGION, free)"
REF=$(mgmt GET /projects | jq -r --arg n "$NAME" --arg o "$ORG" '.[] | select(.name == $n and .organization_id == $o) | .id' | head -1)
if [ -z "$REF" ]; then
  if [ -z "${SUPABASE_DB_PASSWORD:-}" ]; then
    python3 -I -c "import secrets,string;a=string.ascii_letters+string.digits;print(''.join(secrets.choice(a) for _ in range(32)))" | put SUPABASE_DB_PASSWORD
    load
  fi
  jq -n --arg n "$NAME" --arg o "$ORG" --arg r "$REGION" --arg pw "$SUPABASE_DB_PASSWORD" \
     '{name:$n, organization_id:$o, region:$r, db_pass:$pw}' > "$work/create.json"
  if ! mgmt POST /projects --data @"$work/create.json" > "$work/created.json"; then
    rm -f "$work/create.json"
    echo "Project not created. If the message above is free_project_limit: the org owner already has two active free projects; pause or delete one (not 'changuito' unless the team agrees) and re-run. No paid plan is used." >&2
    exit 3
  fi
  rm -f "$work/create.json"
  REF=$(jq -r .id "$work/created.json")
  echo "created $REF"
else
  echo "exists: $REF"
fi
printf '%s' "$REF" | put SUPABASE_PROJECT_REF
URL="https://$REF.supabase.co"
printf '%s' "$URL" | put SUPABASE_URL

for i in $(seq 1 90); do
  st=$(mgmt GET "/projects/$REF" | jq -r .status)
  [ "$st" = ACTIVE_HEALTHY ] && break
  printf '  status %s (%d)\n' "$st" "$i"; sleep 10
done
[ "$st" = ACTIVE_HEALTHY ] || { echo "project not healthy yet ($st); re-run later" >&2; exit 4; }
echo "ACTIVE_HEALTHY · $URL"

# ---------------------------------------------------------------- 2 · keys
say "2 · API keys (anon → public; service role → secrets file only)"
mgmt GET "/projects/$REF/api-keys?reveal=true" > "$work/keys.json"
jq -r '([.[] | select(.name == "anon")][0].api_key) // ([.[] | select(.type == "publishable")][0].api_key)' "$work/keys.json" | put SUPABASE_ANON_KEY
jq -r '([.[] | select(.name == "service_role")][0].api_key) // ([.[] | select(.type == "secret")][0].api_key)' "$work/keys.json" | put SUPABASE_SERVICE_ROLE_KEY
rm -f "$work/keys.json"; load
[ -n "$SUPABASE_ANON_KEY" ] && [ -n "$SUPABASE_SERVICE_ROLE_KEY" ] || { echo "keys missing" >&2; exit 5; }
echo "keys stored in $SECRETS"

# ---------------------------------------------------------------- 3 · database (migrations, seed) over the session pooler
say "3 · migrations + seed"
pool=$(mgmt GET "/projects/$REF/config/database/pooler" | jq -c '[.[] | select(.database_type == "PRIMARY")][0] // .[0]')
PGHOST=$(jq -r .db_host <<<"$pool"); PGUSER=$(jq -r .db_user <<<"$pool")
export PGHOST PGUSER PGPORT=5432 PGDATABASE=postgres PGSSLMODE=require PGPASSWORD="$SUPABASE_DB_PASSWORD" PGOPTIONS="-c client_min_messages=warning"
psql -qAt -v ON_ERROR_STOP=1 -c "create schema if not exists supabase_migrations; create table if not exists supabase_migrations.schema_migrations (version text primary key, statements text[], name text);"
for f in "$sb"/migrations/*.sql; do
  b=$(basename "$f" .sql); v=${b%%_*}; n=${b#*_}
  if [ "$(psql -qAt -c "select 1 from supabase_migrations.schema_migrations where version = '$v'")" = 1 ]; then
    echo "  = $b"; continue
  fi
  psql -q -1 -v ON_ERROR_STOP=1 -f "$f" >/dev/null
  psql -qAt -v ON_ERROR_STOP=1 -c "insert into supabase_migrations.schema_migrations (version, name) values ('$v', '$n')"
  echo "  + $b"
done
psql -q -v ON_ERROR_STOP=1 -f "$sb/seed.sql" >/dev/null && echo "  seed applied (idempotent)"

# ---------------------------------------------------------------- 4 · auth
say "4 · auth: email/password (confirmed email), site URL, redirects; Google ready to enable"
jq -n '{
  site_url: "https://eva-wearmeds.vercel.app",
  uri_allow_list: "http://localhost:8080,http://localhost:8080/**,https://eva-wearmeds.vercel.app/**",
  disable_signup: false,
  external_email_enabled: true,
  mailer_autoconfirm: false,
  password_min_length: 10,
  external_google_enabled: false
}' > "$work/auth.json"
mgmt PATCH "/projects/$REF/config/auth" --data @"$work/auth.json" | jq -c '{site_url, uri_allow_list, disable_signup, external_email_enabled, mailer_autoconfirm, external_google_enabled}'

# ---------------------------------------------------------------- 5 · test users, one per role
say "5 · test users (passwords only in $SECRETS)"
srk_curl() { curl -sS -H "apikey: $SUPABASE_SERVICE_ROLE_KEY" -H "Authorization: Bearer $SUPABASE_SERVICE_ROLE_KEY" -H 'content-type: application/json' "$@"; }
for role in crew_lead mission_control team viewer; do
  R=$(tr '[:lower:]' '[:upper:]' <<<"$role")
  email_var="EVA_TEST_${R}_EMAIL"; pw_var="EVA_TEST_${R}_PASSWORD"
  email="eva-test-${role//_/-}@example.com"
  printf '%s' "$email" | put "$email_var"
  if [ -z "${!pw_var:-}" ]; then
    python3 -I -c "import secrets;print(secrets.token_urlsafe(18))" | put "$pw_var"; load
  fi
  jq -n --arg e "$email" --arg p "${!pw_var}" --arg r "$role" '{email:$e, password:$p, email_confirm:true, user_metadata:{name:("Eva test " + $r)}}' > "$work/u.json"
  code=$(srk_curl -o "$work/u.out" -w '%{http_code}' -X POST "$URL/auth/v1/admin/users" --data @"$work/u.json")
  if [ "$code" -ge 400 ]; then  # already exists → reset its password to the stored one
    uid=$(srk_curl "$URL/auth/v1/admin/users?per_page=1000" | jq -r --arg e "$email" '.users[] | select(.email == $e) | .id')
    [ -n "$uid" ] || { echo "cannot create $email (HTTP $code)" >&2; exit 6; }
    jq '{password, email_confirm}' "$work/u.json" > "$work/u2.json"
    srk_curl -o /dev/null -X PUT "$URL/auth/v1/admin/users/$uid" --data @"$work/u2.json"
  else
    uid=$(jq -r .id "$work/u.out")
  fi
  rm -f "$work/u.json" "$work/u2.json" "$work/u.out"
  srk_curl -o /dev/null -X POST "$URL/rest/v1/rpc/admin_set_role" --data "$(jq -n --arg t "$uid" --arg r "$role" '{target:$t, new_role:$r}')"
  echo "  $role → $email"
done
load

# ---------------------------------------------------------------- 6 · RLS tests
say "6 · RLS tests"
psql -v ON_ERROR_STOP=1 -f "$sb/tests/rls.test.sql" | tail -n 5

# ---------------------------------------------------------------- 7 · Edge Function eva-chat
say "7 · eva-chat: secret + deploy"
python3 -I - "$GROQ_ENV" <<'PY' > "$work/secrets.json"
import json, sys
key = next((l.split("=", 1)[1].strip().strip('"\'') for l in open(sys.argv[1]) if l.startswith("GROQ_API_KEY=")), "")
if not key: sys.exit("GROQ_API_KEY not found")
print(json.dumps([{"name": "GROQ_API_KEY", "value": key}]))
PY
mgmt POST "/projects/$REF/secrets" --data @"$work/secrets.json" >/dev/null && echo "  GROQ_API_KEY set"
rm -f "$work/secrets.json"
(cd "$repo" && npx -y supabase@latest functions deploy eva-chat --project-ref "$REF" --no-verify-jwt --use-api 2>&1 | tail -n 3)

token_for() { # token_for ROLE → access token on stdout
  local R; R=$(tr '[:lower:]' '[:upper:]' <<<"$1"); local e="EVA_TEST_${R}_EMAIL" p="EVA_TEST_${R}_PASSWORD"
  jq -n --arg e "${!e}" --arg p "${!p}" '{email:$e, password:$p}' |
    curl -sS -H "apikey: $SUPABASE_ANON_KEY" -H 'content-type: application/json' --data @- "$URL/auth/v1/token?grant_type=password" | jq -r .access_token
}
ask() { # ask TOKEN JSON
  curl -sS -H "apikey: $SUPABASE_ANON_KEY" -H "Authorization: Bearer $1" -H 'content-type: application/json' \
    -H 'origin: https://eva-wearmeds.vercel.app' --data "$2" "$URL/functions/v1/eva-chat" |
    jq -c '{source, reason, model, text: ((.text // .error) | .[0:160])}'
}
sleep 5
VT=$(token_for viewer)
echo "  in-scope : $(ask "$VT" '{"text":"How do I stay hydrated in microgravity?"}')"
echo "  off-topic: $(ask "$VT" '{"text":"Who won the football match last night?"}')"
echo "  spanish  : $(ask "$VT" '{"text":"Me duele la cabeza en la nave, ¿qué hago?","lang":"es"}')"
echo "  no JWT   : $(curl -sS -o /dev/null -w '%{http_code}' -X POST -H "apikey: $SUPABASE_ANON_KEY" --data '{"text":"hi"}' "$URL/functions/v1/eva-chat")"

# ---------------------------------------------------------------- 8 · cloud chain passes the app's verifier
say "8 · re-verify the mirrored chain with eva verify"
if [ -x "$EVA_BIN" ]; then
  CT=$(token_for crew_lead)
  SUPABASE_URL=$URL SUPABASE_ANON_KEY=$SUPABASE_ANON_KEY EVA_ACCESS_TOKEN=$CT node "$here/export-chain.mjs" --ship demo-ship --out "$work/cloud" >/dev/null
  "$EVA_BIN" verify --data "$work/cloud"
fi
curl -sS -H "apikey: $SUPABASE_ANON_KEY" "$URL/rest/v1/public_stats?select=ship,acknowledged_records,open_cases,passengers,station_mode" | jq -c .

# ---------------------------------------------------------------- 9 · public site
[ "$DO_SITE" = 1 ] && [ -n "$SITE_DIR" ] || { say "site step skipped (--no-site or EVA_SITE_DIR unset)"; exit 0; }
say "9 · Vercel env + /status/ page + deploy"
cd "$SITE_DIR"
for kv in SUPABASE_URL SUPABASE_ANON_KEY; do
  "$VERCEL" env rm "$kv" production -y >/dev/null 2>&1 || true
  printf '%s' "${!kv}" | "$VERCEL" env add "$kv" production >/dev/null 2>&1 && echo "  env $kv (production)"
done
mkdir -p status api
cp "$sb/site/status/index.html" status/index.html
cp "$sb/site/api/status.ts" api/status.ts
"$VERCEL" deploy --prod --yes 2>&1 | grep -E 'https://|Error' | tail -n 3
say "secret scan of the deployed folder (counts only)"
scan() { grep -rlF --exclude-dir=.vercel --exclude='.env*' --exclude-dir=node_modules -e "$1" . 2>/dev/null | wc -l; }
echo "  service_role: $(scan service_role) · sbp_: $(scan sbp_) · gsk_: $(scan gsk_) · db password: $(scan "$SUPABASE_DB_PASSWORD") · service key: $(scan "$SUPABASE_SERVICE_ROLE_KEY")"
sleep 3
echo "  /api/status: $(curl -sS https://eva-wearmeds.vercel.app/api/status | jq -c '{ok, ships: [.ships[]? | {ship, acknowledged_records}]}')"
