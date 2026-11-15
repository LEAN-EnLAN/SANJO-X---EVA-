#!/usr/bin/env bash
# Build app/static/css/app.css from app/styles/app.css with the Tailwind v4 CLI.
# Scans app/templates and app/src for class names. The output is committed, so
# `cargo build` works without Node; rerun this after changing templates or styles.
set -euo pipefail
cd "$(dirname "$0")/../app"
[ -d node_modules/@tailwindcss/cli ] || npm install --no-audit --no-fund --loglevel=error
npm run --silent css "$@"
