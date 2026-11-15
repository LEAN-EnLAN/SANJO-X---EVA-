#!/bin/bash
# Regenera docs/CONTEXT.md desde el Drive (LEEME + Registro + Concepto actual).
# Usa la exportación a Markdown de Google Docs; funciona mientras los docs se puedan leer por link.
# Si falla, deja el CONTEXT.md anterior y avisa (la sesión debe leer el Drive con el conector).
set -uo pipefail
cd "$(dirname "$0")/.."
out=docs/CONTEXT.md
tmp="$(mktemp)"
docs=(
  "00 · LEEME|1kx8kXyDjkLznPdqTOViRES5Dn0y1vCQFItNFjxV4fyM"
  "Registro de decisiones e IA|1FoxLKt0NPCAk9odVmSA9fDhpt1ehqYbUEJBkpeE9G_U"
  "06 · Concepto actual: Eva|1M2ll0OgTznUeJLpwHmY7HVFUhyIqtuhWTD9awS11oJk"
)
{
  echo "# Contexto del proyecto (generado desde el Drive)"
  echo
  echo "Generado: $(TZ=America/Argentina/Cordoba date '+%a %d/%m/%Y %H:%M') (hora de Rosario) por \`scripts/sync-context.sh\`. No editar a mano: editá el Drive."
} > "$tmp"
ok=1
for d in "${docs[@]}"; do
  name="${d%%|*}"; id="${d##*|}"
  body="$(curl -fsSL --max-time 20 "https://docs.google.com/document/d/${id}/export?format=md" 2>/dev/null)"
  if [ -z "$body" ] || printf '%s' "$body" | head -c 200 | grep -qi '<html'; then ok=0; break; fi
  { echo; echo "---"; echo; echo "<!-- ${name} · ${id} -->"; echo; printf '%s\n' "$body"; } >> "$tmp"
done
if [ "$ok" = 1 ]; then
  mkdir -p docs && mv "$tmp" "$out"
  echo "docs/CONTEXT.md actualizado desde el Drive."
else
  rm -f "$tmp"
  echo "AVISO: no se pudo regenerar docs/CONTEXT.md desde el Drive. Leé LEEME, Registro y Concepto actual con el conector de Drive antes de trabajar."
fi
exit 0
