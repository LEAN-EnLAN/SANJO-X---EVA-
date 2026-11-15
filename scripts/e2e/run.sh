#!/usr/bin/env bash
# Eva happy path in a headless browser.
#
#   scripts/e2e/run.sh                 # build (release), start `eva demo`, run the story, stop
#   EVA_BIN=path/to/eva scripts/e2e/run.sh   # use an existing binary
#   HEADED=1 scripts/e2e/run.sh        # watch it
#
# Screenshots, downloads (CSV, JSON, report PDF), results.json and the server log
# land in $OUT (default: scripts/e2e/out/<timestamp>). Needs Node with `playwright`
# resolvable (installed globally here) and a Chromium for it. Runs offline: Eva uses
# her on-board rules, so the run is deterministic and needs no API key.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../.." && pwd)"
PORT="${PORT:-18080}"
GPORT="${GPORT:-18081}"
OUT="${OUT:-$here/out/$(date +%Y%m%d-%H%M%S)}"
mkdir -p "$OUT"

if [ -z "${EVA_BIN:-}" ]; then
  echo "building eva (release)…"
  (cd "$root/app" && cargo build --release --quiet)
  EVA_BIN="${CARGO_TARGET_DIR:-$root/app/target}/release/eva"
fi

DATA="$(mktemp -d)"
cleanup() {
  [ -n "${PID:-}" ] && kill "$PID" 2>/dev/null || true
  wait "${PID:-}" 2>/dev/null || true
  rm -rf "$DATA"
}
trap cleanup EXIT

# never send a key to the network from a test run
env -u GROQ_API_KEY RUST_LOG=eva=info "$EVA_BIN" demo --host 127.0.0.1 --port "$PORT" --ground-port "$GPORT" --data "$DATA" --offline >"$OUT/server.log" 2>&1 &
PID=$!
for _ in $(seq 1 100); do
  curl -fsS "http://127.0.0.1:$PORT/healthz" >/dev/null 2>&1 && curl -fsS "http://127.0.0.1:$GPORT/healthz" >/dev/null 2>&1 && break
  sleep 0.1
done

set +e
BASE="http://127.0.0.1:$PORT" GROUND="http://127.0.0.1:$GPORT" OUT="$OUT" node "$here/happy-path.js"
status=$?
# stop the demo, then check the chain files it left behind (ship chain + ground mirror)
kill "$PID" 2>/dev/null; wait "$PID" 2>/dev/null; PID=""
echo "eva verify:"
"$EVA_BIN" verify --data "$DATA" | tee "$OUT/verify.txt" || status=1
echo "output: $OUT"
exit $status
