#!/bin/bash
# Al iniciar cada sesión: trae el contexto vigente del Drive a docs/CONTEXT.md.
set -euo pipefail
cd "${CLAUDE_PROJECT_DIR:-.}"
bash scripts/sync-context.sh
