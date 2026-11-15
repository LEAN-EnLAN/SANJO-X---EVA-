# Traspaso entre sesiones · Sanjo-X · NASA Space Apps 2026 · Rosario

Estado al sábado 14/11/2026, 21:10 (hora de Rosario). Escrito por Claude a pedido de Lean.
**La fuente de verdad es el Drive** (carpeta `1NCFN_22gxWjRGJdxKjX11KcOk5v7bic7`). Si algo de este archivo choca con el Registro de decisiones, manda el Registro.

## Evento
- Desafío: **Create Health Monitoring Software for Astronauts on Space Missions**.
- Entrega local: **domingo 15/11 a las 15:00** (GitHub + materiales). Pitch 16:00. Cierre global 23:59.
- Equipo: Lean, Tomi, Hassi, Naza (Joaquín es dueño del Drive).
- Jurado local (según Jacobo): factibilidad, innovación, formato y presentación, pitch de los integrantes, foco del problema y ajuste al desafío. Hay un jurado médico y un patrocinador de Rosario. El jurado nos evalúa como inversores.

## Producto
- **Eva**: nombre propio del asistente (no sigla; así no se confunde con EVA, "caminata espacial", de NASA). Nombres anteriores (Vela, Hermes) solo en el historial del Registro y la Bitácora.
- **WearMeds**: marca de los wearables. "Power Mats" queda por confirmar.
- **Concepto IoT** para un viaje comercial a Marte:
  - Banda torácica (ECG, respiración) y pulsera con pantalla grande (pulso, movimiento, instrucciones paso a paso).
  - Estación compartida por cabina (presión, SpO₂, temperatura) que **ilumina la bandeja** que hay que usar.
  - **Caso 01**: la persona no responde o no puede actuar → Eva avisa al pasajero más cercano y lo guía; los demás reciben un aviso informativo.
  - **Caso 02**: puede actuar y moverse → Eva lo lleva a la estación, ilumina el elemento y muestra cómo usarlo.
  - **Alerta solar** con NASA DONKI (`https://ccmc.gsfc.nasa.gov/DONKI-API/get/FLR`) y NOAA GOES (`services.swpc.noaa.gov/json/goes/primary/xray-flares-latest.json`, `integral-protons-1-day.json`). Los rayos X llegan con la luz; el margen de aviso es hasta que llegan las partículas.
  - Registro encadenado de cada paso que viaja a la Tierra con confirmación de recepción (demora de 3 a 22 min por tramo).
- **Enfoque de producto para inversores**: simulación de un producto real vendido a inversores reales. Hace falta negocio, producto y prototipo. Modelo propuesto: kit por pasajero, estación por cabina, licencia de Eva por misión; plataforma personalizable con varias líneas (órbita, bases lunares, trabajo remoto en la Tierra). Sin precios hasta que el documento de negocio los defina.
- **Lenguaje**: "Eva **guides, records and warns**". Nunca "diagnostica", "trata" ni "totalmente autónomo".
- Arquitectura de IA en 3 capas: protocolo (menos de 1 s) → razonamiento con MedGemma 1.5 4B + MedASR a bordo (menos de 10 s; términos HAI-DEF, no open source) → registro y enlace. En el prototipo, la capa 2 usa Claude como reemplazo hasta descargar MedGemma: declararlo.

## Cifras verificadas (usar solo estas)
- ~70% de los astronautas en misiones de 6 a 12 meses en la ISS con cambios de visión significativos; 81% con al menos un signo de SANS.
- 85% de 71 astronautas con al menos un problema nasal o de senos paranasales.
- Demora a Marte: 3 a 22 min por tramo.
- Inspiration4 (NASA OSDR OSD-575 panel metabólico, OSD-569 hemograma): con línea basal personal, R+1 muestra 16 de 72 cambios contra 7 con rangos de laboratorio.
- **No usar** "90% ojos" ni "70% respiratorio": no tienen fuente.

## Diseño
- **Vigente: design system twocreate** (`twocreate.com-DESIGN.md`): lienzo #f9f7f4, tinta #0c0c0c, reglas #e3e1dd; tipografía grotesca peso 400 (PP Neue Montreal, sustituto Inter) en 111 / 66.6 / 44.4 / 33.3 / 22.2 px; sin botones rellenos, sin esquinas redondeadas, sin sombras; dípticos imagen ancha + detalle; wordmark final "Eva by WearMeds".
- **Descartados**: NASA Horizon modernizado, AuthKit "circuito luminoso", kit HELIO X y el kit visual de Tomás.
- Assets: 21 imágenes de Codex (S1–S5 escenas, L1–L13 capas, F1–F3 estación) + preview de motion. Todas llevan "AI-generated".
- **Sin logos de NASA** (insignia, logotipo, banderas, parches) en nada nuestro ni en imágenes de IA. Sacar el logo de NASA de las láminas del equipo.

## Código
- Stack: **Rust + htmx + HTML**, un solo binario. Nadie del equipo tiene experiencia en Rust: Claude escribe, el equipo revisa.
- **Código 100% propio**: de los proyectos open source (HealthLog, OpenNutriTracker, nasa/hermes, Human Atlas, hds-core) se toman patrones e ingeniería inversa, nunca código. HealthLog es PolyForm Noncommercial; OpenNutriTracker GPL-3.
- Patrón de NASA Hermes (JPL, Apache-2.0): "el diccionario es el contrato" (XTCE).
- Requisitos NASA: inglés, ≤7 slides o video ≤30 s, links sin login, datos NASA en el centro, uso de IA declarado, marca de agua en imágenes de IA.

## Artifacts (privados: compartir desde Share)
- Landing: https://claude.ai/artifact/Ja6v2YathtvXUeaS7EcdRo
- Wireframe: https://claude.ai/artifact/UUnPWQ5vwBjevhU5DdG7u8
- Prototipo: https://claude.ai/artifact/DjrUadizcmVevJFVpCqR4m (todavía con el estilo oscuro anterior)

## Fase 2 · tareas y responsables
| # | Tarea | Responsable |
|---|---|---|
| 01 | Ganadores y qué definir en el informe | Lean (análisis listo) |
| 02 | Presupuesto energético y componentes | Tomi |
| 03 | Casos agudos: dato, interpretación y acción | Hassi y Naza |
| 04 | Variables médicas | Hassi y Naza |
| 05 | Pantallas (las generadas con imágenes, pendientes) | Tomi |
| 06 | Mockup happy path | Lean |
| 07 | Escalabilidad | Naza |
| 08 | Landing page | Lean (landing y wireframe hechos) |
| 09 | Justificación de modelos de IA médicos | Lean |
| 10 | Maqueta del video de 30 s | Lean |
| 11 | Soporte de celular en muñeca | Tomi |

## Pendientes
- Repositorio: https://github.com/LEAN-EnLAN/sanjo-x-spaceapps-2026 (público). Una rama por tarea: `fase2/NN-nombre`.
- Laptop de la demo (SO, RAM, GPU).
- Descargar MedGemma 1.5 y MedASR (aceptar términos en Hugging Face).
- Decidir si la estación se muestra con hardware real (LEDs) o simulada.
- Verificar si DeltaProtocol (2018) fue ganador global o "Global Nominee".
- Documento de negocio; confirmar "Power Mats".
- Subir `eva-assets.zip` (21 MB) a la carpeta 08 a mano: supera el límite de subida de Claude.

## IDs del Drive
| Qué | ID |
|---|---|
| Carpeta raíz | 1NCFN_22gxWjRGJdxKjX11KcOk5v7bic7 |
| 00 · LEEME | 1kx8kXyDjkLznPdqTOViRES5Dn0y1vCQFItNFjxV4fyM |
| Registro de decisiones e IA | 1FoxLKt0NPCAk9odVmSA9fDhpt1ehqYbUEJBkpeE9G_U |
| Bitácora del proyecto | 1B9wELOyk1TcahQLVJTe_lSVPQ551TNZHn2ebxuW_BUQ |
| 06 · Concepto actual: Eva | 1M2ll0OgTznUeJLpwHmY7HVFUhyIqtuhWTD9awS11oJk |
| Fase 1 (carpeta) | 1R6UCLLhGJT_V_gBMnNYJyRW34xnPJPpa |
| Fase 1 · 01 Enfermedades (carpeta / doc) | 14jMyw6EXsPj_SF80GR1dpxHhGo8vynxN / 1PftKznSB53vllz42P6Tl2jSi2bVCfnDD1AujxuQNTf4 |
| Fase 1 · 02 (carpeta / doc) | 1ndcl2DyTXi9eMdmY9FxJk52DXmYRt-0S / 10ByhVWohYAAz4Z-KdTwerTOtyG78z9O7uhvX_CyR7pI |
| Fase 1 · 03 (carpeta / doc) | 1U-QFrPPxWys9jv3wHZNSY28VwVJabRwd / 1HlE0Ztc34XIh5FrFa2bRHYWE6aamlWutZmmuQ3Y2ISE |
| Fase 1 · 04 (carpeta) | 1D-fs8WwbLPmia0ybHVX844MErsCQJvb- |
| 04 · Proyectos base (Lean) | 1kEXKCYfA6BRXqRiWiGqeD5aHvv_ixeQsry00Y3irzNs |
| 04 · Evaluación de repos | 1-CvVMwAMqPaPDFSN9pNwLIYzGYgalxeh0VpNTD8AIXc |
| 04 · NASA Hermes y el patrón | 1wNckWqya6cDkdRjHZEZLf49EUrGTUTIvzP0gZFHXj7I |
| Fase 1 · 05 (carpeta / doc) | 1rk7B5-3H7IVMoNKWmzb6cD1uLSmsF-JX / 1omJZ7HeSavV-wE_l1p_4qyvcCKIuQCf6K3-dIm-3dAM |
| Fase 1 · 06 (carpeta) | 1hbHHNPziEslCSA3kJzcxt_nQg4zb-1x3 |
| 06 · Alcance | 1VC1Y408MquRD8S5KuzMpEPZYWFIaqYH5fmJ6nHvW4SY |
| 06 · Ideas de innovación | 12FQtH7mqGxN9JvikN7feQ8OjmENLRQcWaAzeYfTUeRw |
| 06 · Copiloto: plan de trabajo | 1FSa70Jb7S_GyTVVifqfHN91LmSvgqFF84vnXXUognAk |
| Fase 1 · 07 (carpeta / doc) | 1hv4YrWZ7hDU6zi9mI-zRybMEM4Tyc1aH / 1ryFDiJvk-ORy0g143NN7U0ySvpan4zzNgkubQvJJWQE |
| Datos NASA (carpeta) | 173x9Ko2tuMrHpLWJY-luS5omgTWfZWo8 |
| Fase 2 (carpeta) | 1rk3nh0bfCLVkyvNHkr9skT4WF7x4mq4r |
| Fase 2 · 01 (carpeta / doc ganadores) | 1wERr_1eXdkMSfDBdRVYhh7EY34w5LYIu / 145Wv12WxHBFrVuG4UUiMWpvZBRPAkjtjT_Pv1B0IkzQ |
| Fase 2 · 02 Energía | 1yMwbXVdz764KftoFwcwicAmmUt7KSO7R |
| Fase 2 · 03 Casos agudos | 12oMddCXQCmA_eXltXNWJMc1ob7bgqsmf |
| Fase 2 · 04 Variables médicas | 1Lefw0dq7TrL8FqxED7lBuUI0ecqn7Gu5 |
| Fase 2 · 05 Pantallas | 19TXsUOX3KarDK1jMUH8QCytSdRJlSleL |
| Fase 2 · 06 Happy path | 1xXNF1vL-agbIIpevOh0PT0Z1Q4CU9qNY |
| Fase 2 · 07 Escalabilidad | 14U0L2KRG1e1NpOjmRZBjklCErNz05WGL |
| Fase 2 · 08 Landing | 126xeGXa_5W9kIS0awWwsuwEhLE_V9Jl5 |
| 08 · eva-codex-kit.zip | 1MMasR-6lW1AGYCEZkc4Fq5ZzHnuf0cVZ |
| Fase 2 · 09 Modelos de IA | 1zACnpV94JdXTstGIVQ1SHfM-_VEGsJIb |
| Fase 2 · 10 Video | 17imRGq67O3wMTGWmv88XE8FLOBb-RvnF |

## Reglas de trabajo
1. Leer primero LEEME, Registro, Bitácora y Concepto actual.
2. Toda decisión va al Registro con hora y responsable, **agregando abajo**, nunca editando lo de otros.
3. Todo uso de IA va a su sección del Registro.
4. Nada de copiar código open source.
5. Documentos de Claude terminan en "(Claude)"; los del equipo, con el nombre de quien escribe. No duplicar archivos.
