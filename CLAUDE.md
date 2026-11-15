# Sanjo-X · Eva by WearMeds · NASA Space Apps 2026

Reglas para cualquier sesión (humana o de Claude) que trabaje en este repo.

## Fuente de verdad
El Drive del equipo manda: carpeta `1NCFN_22gxWjRGJdxKjX11KcOk5v7bic7`. Este repo guarda código y entregables; las decisiones viven en el Drive.

## Antes de empezar
1. Leé, en este orden, los cuatro documentos del Drive:
   - 00 · LEEME `1kx8kXyDjkLznPdqTOViRES5Dn0y1vCQFItNFjxV4fyM`
   - Registro de decisiones `1FoxLKt0NPCAk9odVmSA9fDhpt1ehqYbUEJBkpeE9G_U`
   - Bitácora del proyecto `1B9wELOyk1TcahQLVJTe_lSVPQ551TNZHn2ebxuW_BUQ`
   - 06 · Concepto actual: Eva `1M2ll0OgTznUeJLpwHmY7HVFUhyIqtuhWTD9awS11oJk`
2. Leé `HANDOFF.md` y `docs/CONTEXT.md`. `docs/CONTEXT.md` se regenera desde el Drive al iniciar cada sesión (`scripts/sync-context.sh`). Si el script avisa que no pudo, leé los documentos con el conector de Drive.

## Registro de decisiones
- Toda decisión va al Registro con **hora (de Rosario) y responsable**: `Sáb 14/11 21:30 · DECISIÓN: … · Nombre.`
- **Agregá siempre abajo**, al final de la sección "Decisiones" (antes de "Propuestas pendientes de confirmar"). Nunca edites ni borres lo que escribió otra persona o sesión. Si algo queda superado, agregá una línea nueva que lo diga.
- Todo uso de herramientas generativas va a su sección del Registro (requisito de NASA; en el Drive se llama "Uso de IA"): fecha, herramienta, tarea, archivos y quién lo revisó.

## Código
- **Claves de API** (por ejemplo, Groq para voz en las demos): solo en variables de entorno locales (`GROQ_API_KEY`). Nunca en el repo, el Drive, un artefacto ni el código del navegador.
- Stack: **Rust + htmx + HTML**, un solo binario. Código **100% propio**.
- **Nada de copiar código open source.** De HealthLog, OpenNutriTracker, nasa/hermes, Human Atlas o hds-core se toman patrones e ingeniería inversa, nunca líneas de código. Ojo con las licencias: HealthLog es PolyForm Noncommercial; OpenNutriTracker, GPL-3.
- Sin logos de NASA (insignia, logotipo, banderas, parches) en nada nuestro ni en imágenes generadas. Las imágenes generadas con Super Inteligencia se declaran en el README y en la descripción de la entrega, con un enlace que funcione al aviso (no hace falta la marca visible dentro de la app).

## Producto y lenguaje
- El asistente se llama **Eva** (nombre propio, no sigla). Los wearables son **WearMeds**.
- Eva es el **agente copiloto de salud** (multimodal: texto, voz, imágenes y video): lee el brazalete en tiempo real, registra todo, lleva el diario emocional, da **apoyo al diagnóstico** (*diagnostic support*) y avisa. Nunca escribir que diagnostica por sí sola, trata, decide una descarga o es totalmente autónoma. La descarga la decide el desfibrilador (DEA) de la estación de la nave.
- Eva es la innovación; el brazalete WearMeds es su factor de forma. No hay arnés ni inyectable.
- Usá solo las cifras verificadas de `HANDOFF.md`.
- Textos del producto y de la entrega en **inglés** (requisito de NASA).
- Diseño vigente: **twocreate** (ver `HANDOFF.md`). Los sistemas anteriores están descartados.

## Reglas de texto v3 (dom 15/11, decisión del equipo)
Mandan sobre cualquier texto anterior en landing, hub de investigación, app, docs, decks, video, informe y documentos del Drive y del repo.
1. **Nunca "IA" / "AI".** Decir **"Super Inteligencia"** (ES) y **"Super Intelligence"** (EN): "Super Intelligence health copilot", "copiloto de salud con Super Inteligencia". Vale también para el aviso de imágenes ("generated with Super Intelligence"), que debe ser un enlace que funcione. Identificadores de código y nombres de terceros (OpenAI, ids de modelos) quedan como están, pero nunca en textos para el usuario.
2. **Sin Marte.** El viaje es **"in space" / "en el espacio"** ("Day 96 · in space"). Las demoras, genéricas: "minutes from Earth" / "a minutos de la Tierra".
3. **Sin cantidad de compartimentos.** Nunca "12 compartments". Decir "numbered compartments" / "compartimentos numerados", "configurable for each ship". La grilla visual puede quedar, sin decir un número.
4. **Sin "trained" / "capacitado" / "entrenado".** Es para todos: "the nearest passenger" / "el pasajero más cercano".
5. **ECG:** **dos dedos de la misma mano** (índice y meñique) sobre las **dos placas metálicas**, unos 30 s. No es un dedo de cada mano. Ninguna imagen con dos manos; hasta tener la nueva, usar una imagen sin manos o ninguna.
6. **Cómo se conecta:** el hub de la nave **"hosts the Super Intelligence · stores everything"** ("aloja la Super Inteligencia · guarda todo") y es el que almacena y reenvía. La Tierra **"receives and acknowledges the signed records"** ("recibe y confirma los registros firmados"), nunca "store and forward".
7. **"Why it matters" / "Por qué importa"** cita fuentes oficiales o verificables con enlace: Russell et al., npj Microgravity 9:46 (2023), doi:10.1038/s41526-023-00284-1; NASA Human Health and Performance data sharing (https://www.nasa.gov/directorates/esdmd/hhp/human-health-and-performance-data-sharing/); NASA OSDR (OSD-575, OSD-569). Si una cifra no tiene fuente verificable, se saca (la de supervivencia por minuto: Szabó et al., Front Pharmacol 10:1640 (2020), doi:10.3389/fphar.2019.01640, "sin desfibrilación").
8. **Equipo:** el rol de Naza es **"Escalabilidad" / "Scalability"**.
9. **Sección del hub de la nave:** solo lo que hace, **ejecuta · aloja · envía** (runs · hosts · sends). Sin imágenes de naves ni componentes de hardware.
10. **Footer:** cada referencia es un enlace real y el aviso de imágenes enlaza al texto del aviso.
11. **"Hermes" fue el nombre viejo del producto.** Todo "Hermes" que se refiera a nuestro producto se escribe **"Eva"**. No es un sistema de telecomunicaciones de NASA: no investigarlo ni describirlo como tal.

## Ramas y PRs
- Una rama por tarea de la Fase 2: `fase2/NN-nombre`. Commits chicos y en español o inglés claro.
- Al cerrar una tarea: agregá la decisión al Registro del Drive y abrí un **PR en borrador** contra `main`.

## Carpetas
- `landing/` landing pública (twocreate) y kit de Codex · `wire/` wireframe · `proto/` prototipo interactivo y sus herramientas · `app/` app Rust + htmx (por empezar) · `docs/` contexto generado · `scripts/` utilidades.
