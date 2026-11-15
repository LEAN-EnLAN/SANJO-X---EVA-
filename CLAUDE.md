# Sanjo-X · Eva by WearMeds · NASA Space Apps 2026

Reglas para cualquier sesión (humana o de Claude) que trabaje en este repo.

## Fuente de verdad
El Drive del equipo manda: carpeta `1NCFN_22gxWjRGJdxKjX11KcOk5v7bic7`. Este repo guarda código y entregables; las decisiones viven en el Drive.

## Antes de empezar
1. Leé, en este orden, los cuatro documentos del Drive:
   - 00 · LEEME `1kx8kXyDjkLznPdqTOViRES5Dn0y1vCQFItNFjxV4fyM`
   - Registro de decisiones e IA `1FoxLKt0NPCAk9odVmSA9fDhpt1ehqYbUEJBkpeE9G_U`
   - Bitácora del proyecto `1B9wELOyk1TcahQLVJTe_lSVPQ551TNZHn2ebxuW_BUQ`
   - 06 · Concepto actual: Eva `1M2ll0OgTznUeJLpwHmY7HVFUhyIqtuhWTD9awS11oJk`
2. Leé `HANDOFF.md` y `docs/CONTEXT.md`. `docs/CONTEXT.md` se regenera desde el Drive al iniciar cada sesión (`scripts/sync-context.sh`). Si el script avisa que no pudo, leé los documentos con el conector de Drive.

## Registro de decisiones
- Toda decisión va al Registro con **hora (de Rosario) y responsable**: `Sáb 14/11 21:30 · DECISIÓN: … · Nombre.`
- **Agregá siempre abajo**, al final de la sección "Decisiones" (antes de "Propuestas pendientes de confirmar"). Nunca edites ni borres lo que escribió otra persona o sesión. Si algo queda superado, agregá una línea nueva que lo diga.
- Todo uso de IA va a la sección **"Uso de IA (requisito de NASA)"**: fecha, herramienta, tarea, archivos y quién lo revisó.

## Código
- Stack: **Rust + htmx + HTML**, un solo binario. Código **100% propio**.
- **Nada de copiar código open source.** De HealthLog, OpenNutriTracker, nasa/hermes, Human Atlas o hds-core se toman patrones e ingeniería inversa, nunca líneas de código. Ojo con las licencias: HealthLog es PolyForm Noncommercial; OpenNutriTracker, GPL-3.
- Sin logos de NASA (insignia, logotipo, banderas, parches) en nada nuestro ni en imágenes de IA. Las imágenes de IA se declaran en el README y en la descripción de la entrega (no hace falta la marca visible dentro de la app).

## Producto y lenguaje
- El asistente se llama **Eva** (nombre propio, no sigla). Los wearables son **WearMeds**.
- Eva **guides, records and warns**. Nunca escribir que diagnostica, trata o es totalmente autónoma.
- Usá solo las cifras verificadas de `HANDOFF.md`.
- Textos del producto y de la entrega en **inglés** (requisito de NASA).
- Diseño vigente: **twocreate** (ver `HANDOFF.md`). Los sistemas anteriores están descartados.

## Ramas y PRs
- Una rama por tarea de la Fase 2: `fase2/NN-nombre`. Commits chicos y en español o inglés claro.
- Al cerrar una tarea: agregá la decisión al Registro del Drive y abrí un **PR en borrador** contra `main`.

## Carpetas
- `landing/` landing pública (twocreate) y kit de Codex · `wire/` wireframe · `proto/` prototipo interactivo y sus herramientas · `app/` app Rust + htmx (por empezar) · `docs/` contexto generado · `scripts/` utilidades.
