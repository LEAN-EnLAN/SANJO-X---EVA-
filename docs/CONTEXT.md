# Contexto del proyecto (generado desde el Drive)

Generado: Sat 14/11/2026 21:40 (hora de Rosario) por `scripts/sync-context.sh`. No editar a mano: editá el Drive.

---

<!-- 00 · LEEME · 1kx8kXyDjkLznPdqTOViRES5Dn0y1vCQFItNFjxV4fyM -->

# **Sanjo-X · NASA Space Apps 2026 · Rosario**

**Desafío:** Create Health Monitoring Software for Astronauts on Space Missions (decidido sáb 14/11, 11:05).  
**Entrega local:** domingo 15/11 a las **15:00** (GitHub \+ materiales). Pitch 16:00. Cierre global 23:59.

## **Producto**

**Eva**: copiloto de salud para un viaje comercial a Marte. Guía a cualquier pasajero, aunque no sea médico, en el primer minuto de una emergencia; registra todo y lo envía a la Tierra. Nombre final elegido el sáb 14/11 a las 18:20 (nombre propio; el historial está en la Bitácora). Los wearables se venden con la marca WearMeds. Lo presentamos como un producto para inversores, no solo como un proyecto de hackathon.

## **Cómo está ordenada la carpeta**

> * **00 · LEEME**: este documento. Estado del día y reglas.  
> * **Registro de decisiones e IA**: lo que se decidió (con hora y quién) y cada uso de IA.  
> * **Fase 1 · Investigación y Dirección**: una carpeta por cada punto de nuestra Fase 1 (01 a 07).  
> * **Datos NASA**: los archivos de datos reales y su procedencia.

## **Tres reglas**

> 1. Cada uno trabaja dentro de la carpeta del punto que le toca. Los documentos de Claude terminan en **(Claude)**; los del equipo, con el nombre de quien lo escribe.  
> 2. No duplicar archivos: se edita el que existe. Para volver atrás, usar el historial de Drive.  
> 3. Una decisión existe solo si está en **Registro de decisiones e IA**. Todo uso de IA también se anota ahí (requisito de NASA).

## **Estado de la Fase 1**

> * **01 · Enfermedades y prioridad**: borrador de Claude listo, falta validar en equipo.  
> * **02 · Datos por enfermedad**: borrador de Claude listo.  
> * **03 · Dispositivos y datos NASA**: hecho (Claude).  
> * **04 · Proyectos similares**: en curso (Lean busca en GitHub). Evaluación de repos base lista (Claude).  
> * **05 · Patrones ganadores**: borrador de Claude listo.  
> * **06 · Alcance**: borrador de Claude listo, falta decidir en equipo.  
> * **07 · Entregables**: lista completa (Claude).

## **Fase 2 · Armado y detalles (en curso)**

* **01 Ganadores y qué definir en el informe**: Lean (análisis de Claude listo)  
* **02 Presupuesto energético y componentes**: Tomi  
* **03 Casos agudos, dato, interpretación y acción**: Hassi y Naza  
* **04 Variables médicas**: Hassi y Naza  
* **05 Pantallas: Tomi (pantallas generadas con imágenes: pendientes)**  
* **06 Mockup happy path**: Lean  
* **07 Escalabilidad**: Naza  
* **08 Landing page**: Lean (landing y wireframe hechos con el design system twocreate)  
* **09 Justificación de modelos de IA médicos**: Lean  
* **10 Maqueta del video de 30 s: Lean**  
* **11 Soporte de celular en muñeca: Tomi**

## **Documentos clave**

* **Traspaso entre sesiones: resumen de todo lo decidido, para cualquier sesión nueva.**  
* **Encargo de investigación médica y arquitectura (anexo, Fase 2): https\://docs.google.com/document/d/1pSradgUdKCN5O2YeCmGXNlE20-SyGAsuZZhEqzMcr90/edit**  
* **Bitácora del proyecto: todas las discusiones en orden.**  
* **06 · Concepto actual**: Eva: la fuente de verdad del producto.  
* **Prototipo Eva**: [https\://claude.ai/artifact/DjrUadizcmVevJFVpCqR4m](https://claude.ai/artifact/DjrUadizcmVevJFVpCqR4m)  
* **Landing Eva by WearMeds: https\://claude.ai/artifact/Ja6v2YathtvXUeaS7EcdRo**  
* **Wireframe de la landing: https\://claude.ai/artifact/UUnPWQ5vwBjevhU5DdG7u8**  
* **Kit de imágenes para Codex**: carpeta 08 · Landing page.

---

<!-- Registro de decisiones e IA · 1FoxLKt0NPCAk9odVmSA9fDhpt1ehqYbUEJBkpeE9G_U -->

# **Registro de decisiones e IA**

## **Decisiones**

Una fila por decisión. Lo que no está acá no está decidido.

| Cuándo | Decisión | Quién | Por qué |
| :---- | :---- | :---- | :---- |
| Sáb 14/11 11:05 | Desafío: Create Health Monitoring Software for Astronauts | Equipo | Es lo investigado en el banco y hay datos humanos reales |
| Sáb 14/11 12:20 | Estructura de trabajo: la Fase 1 de 7 puntos del equipo | Lean | Simplicidad para trabajar en grupo |

Sáb 14/11 13:00 · Base de investigación: ecosistema NASA de telemetría y comandos (Hermes, F´, Open MCT) · Lean · Patrón "el diccionario es el contrato": conecta laboratorios, nutrición y wearables en un solo motor.  
Sáb 14/11 13:00 · Lenguaje: todo en Rust, sistema comprimido al máximo (un solo binario) · Lean.  
Sáb 14/11 13:45 · FOCO DEL PRODUCTO: copiloto de salud con IA (MedGemma 1.5 \+ MedASR \+ voz) que en 10 segundos ayuda a un astronauta en pánico, dice si está sano y qué hacer, registra todo y lo manda a Tierra. Enfoque práctico, ensayable, con foco en UI/UX. Plan en 06 · Copiloto de salud con IA · Equipo.  
**Sáb 14/11 13:20** · CÓDIGO 100% PROPIO: estudiamos y hacemos ingeniería inversa de proyectos open source (HealthLog, OpenNutriTracker, NASA Hermes, Human Atlas), pero no copiamos ni una línea. Stack: Rust \+ htmx \+ HTML. Reemplaza la línea de las 13:00 sobre el lenguaje · Lean.  
**Sáb 14/11 14:00** · Nombre provisorio del copiloto: Vela (también se discutieron Vigil, Tether, Ancla, Medcom y Kairos). Reemplazado a las 18:10 por Hermes y a las 18:20 por Eva · Equipo.  
**Sáb 14/11 14:25** · Prototipo visual sobre el kit HELIO X con la anatomía 3D de Human Atlas (MIT \+ BodyParts3D CC BY 4.0). Convertidor de anatomía escrito por nosotros · Lean.  
**Sáb 14/11 14:40** · Rediseño completo del prototipo con el design system AuthKit tras la crítica de "UI insulsa"; luego se arreglaron superposiciones y se simplificó la información · Lean.  
**Sáb 14/11 15:30** · GIRO DEL PRODUCTO A IoT SIMPLE Y BIEN HECHO: banda de pecho \+ pulsera con pantalla grande \+ estación compartida que ilumina el elemento que hace falta. Caso 01: el afectado no puede actuar, se guía al pasajero más cercano. Caso 02: puede actuar, se lo guía a la estación. Contexto: viaje comercial a Marte. Se suma alerta de tormenta solar con datos de NASA DONKI y NOAA GOES (los rayos X llegan con la luz; el margen es hasta que llegan las partículas). Motivo: DeltaProtocol (2018) fue visionario pero poco factible; los criterios de Jacobo premian factibilidad, foco y pitch · Equipo.  
**Sáb 14/11 15:30** · Cifras corregidas para el pitch: \~70% con cambios de visión significativos en misiones de 6 a 12 meses en la EEI; 85% de 71 astronautas con al menos un problema nasal o de senos; demora a Marte de 3 a 22 min por tramo. No usar "90% ojos" ni "70% respiratorio". Lenguaje del producto: "guía, registra y avisa" (no diagnostica) · Equipo.  
**Sáb 14/11 17:30** · Tareas de la Fase 2 y responsables: Tomi 02 y 05; Lean 01, 06, 08, 09 y 10; Hassi 03 y 04; Naza 03, 04 y 07\. Landing: mezcla de NASA hds-core y el design system de Superhuman. Imágenes con un kit para Codex (GPT-6.1 Terra escribe, gpt-image-2 dibuja porque Terra no genera imágenes). Nada de logos de NASA en contenido nuestro ni en imágenes de IA · Equipo.  
**Sáb 14/11 18:10** · Nombre del agente: Hermes (REEMPLAZADO a las 18:20 por Eva). Ojo: NASA JPL tiene un framework open source también llamado Hermes. Hay que aclarar siempre que nuestro Hermes es un agente propio, inspirado en ese patrón ("el diccionario es el contrato"), sin código de NASA · Equipo.  
Sáb 14/11 18:20 · NOMBRE FINAL DEL AGENTE: Eva. Reemplaza a Hermes en todo el proyecto (prototipo, landing, kit de imágenes, documentos y pitch). Ya no choca con el framework Hermes de NASA JPL. Desde las 20:55 se escribe Eva, como nombre propio, para no confundirlo con la sigla EVA (caminata espacial) de NASA · Equipo.  
Sáb 14/11 18:25 · PRODUCTO, NO SOLO PROYECTO: los wearables se venden con la marca WearMeds ("Power Mats" por confirmar) y Eva es el asistente. Se presenta como un producto real vendido a inversores reales; el jurado nos juzga como inversores. Hace falta negocio y producto, además del prototipo. Wearables personalizables y de calidad: una plataforma con varias líneas de desarrollo, no algo de una sola vez · Equipo.  
Sáb 14/11 18:30 · (REEMPLAZADO a las 19:30) Design system Eva v1: NASA Horizon (hds-core) modernizado con escenarios de color. Rojo \= ir a otro lado, azul \= acción en la página, ámbar \= lo que Eva ilumina, verde \= OK. Rojo de botones oscurecido a \#d42d24 para pasar contraste AA. Se exporta en un zip (carpeta Fase 2 · 05\) para generar pantallas con GPT Image 2.5 Flare (borradores) y Sunburst (versión final) y después rehacerlas en Rust \+ htmx \+ HTML · Lean.  
Sáb 14/11 19:30 · (DESCARTADO a las 20:55) Design system del kit de Tomás: manda el kit visual del equipo (las láminas de las fases), pasado de HERMES a Eva. Fondo blanco, texto \#050A2D, azul \#06356D, rojo \#EF0010 solo para diagonales, insignias y urgencias, amarillo \#F3CC43 solo para el elemento activo, Liberation Sans 400/700, pictogramas de línea planos. Pantalla del brazalete blanca de 800×480 con Repetir y Pedir ayuda siempre visibles. Las referencias que se le pasan a la IA van sin el emblema de NASA. Zip en Fase 2 · 05 · Lean.  
Sáb 14/11 20:55 · SE DESCARTA EL DESIGN SYSTEM DE TOMÁS: el equipo lo considera inútil. La landing y su wireframe se regeneran con un design system nuevo (pendiente de recibir). Las pantallas generadas con imágenes de Tomás quedan pendientes. El asistente se escribe Eva, nombre propio, en todo el material; los nombres anteriores (Vela, Hermes) quedan solo en este Registro y en la Bitácora · Lean.  
Sáb 14/11 21:05 · LANDING Y WIREFRAME CON EL DESIGN SYSTEM TWOCREATE: lienzo crema \#f9f7f4, tinta \#0c0c0c, tipografía grotesca 400 en escalas grandes, sin botones rellenos ni esquinas redondeadas, dípticos de imagen ancha \+ detalle y wordmark final "Eva by WearMeds". Usa los 21 assets de Codex y la capa del enlace azul del preview de motion. Motion: estación que enciende cada bandeja con la instrucción en la pantalla, enlace entre pulseras, partículas solares y señal Tierra–Marte. Landing: https\://claude.ai/artifact/Ja6v2YathtvXUeaS7EcdRo · Wireframe: https\://claude.ai/artifact/UUnPWQ5vwBjevhU5DdG7u8 · Lean.  
Sáb 14/11 21:20 · ENTORNO COMPARTIDO: tarea 11 (soporte de celular en muñeca) para Tomi. Repo público LEAN-EnLAN/sanjo-x-spaceapps-2026 con CLAUDE.md (reglas), HANDOFF.md y docs/CONTEXT.md regenerado desde el Drive al iniciar cada sesión. Una rama y una sesión en la nube por tarea de la Fase 2 (fase2/NN-nombre). Traspaso subido al Drive como "Traspaso entre sesiones". Las sesiones agregan abajo en este Registro, nunca editan lo de otros · Lean.  
Sáb 14/11 22:15 · ENCARGO DE INVESTIGACIÓN DEL EQUIPO (anexo en Fase 2): se elimina la estación compartida; quedan brazalete y arnés como únicos dispositivos médicos personales, más el hub informático de la nave (IA offline). Monitoreo no invasivo; tratamiento invasivo solo ante emergencia mortal. Cuatro funciones: descarga (desfibrilación vestible con análisis de ritmo validado), procedimientos guiados, inyectable (vía y fármaco sin decidir) y bitácora. LoRa \+ hub, malla a evaluar. Pantalla táctil ≥ \~100 mm de lado activo, preferentemente flexible. Unos 15 pasajeros. Concepto actual reescrito con estos cambios. Quedan para decidir: nombre (HERMES en el encargo vs Eva), lenguaje ("trata" vs "guides, records and warns"), alerta solar y rehacer la estación en la landing · Equipo (encargo), registrado por Lean.  

Dom 15/11 01:45 · BRAZALETE EN DOS PARTES: módulo de sensores interior (sobre la piel, batería chica propia) y módulo de interfaz exterior (pantalla y batería grande), unidos con imanes; pines con resorte alimentan los sensores al unirse; con el traje en el medio los pines no tocan y los sensores usan su batería. Referencia de sensado: Masimo rainbow. Hay que rehacer los assets del brazalete en la landing y el video · Equipo, registrado por Lean.  

Dom 15/11 02:30 · BRAZALETE: REFERENCIA VISUAL Y ECG DE MUÑECA: módulo rectangular robusto con pantalla plana y correa textil; la base mide con sensor óptico (pulso, regularidad, variabilidad, SpO₂ y respiración estimada en reposo), térmico, electrodos EDA, acelerómetro y giroscopio, tendencias de reposo y sueño, y control de señal. Propuesta de ECG de una derivación: electrodo interior en la muñeca y exterior para el dedo de la otra mano, 30 s quieto; no continuo, no sirve inconsciente ni con el traje en el medio, nunca única señal para el desfibrilador (el ECG permanente va en el arnés). La imagen de referencia todavía dice HERMES; el nombre vigente es Eva · Equipo, registrado por Lean.  
Sáb 14/11 22:25 · NOMBRE: el asistente se llama Eva en todo el proyecto, incluido el encargo de investigación (que decía HERMES). Choque cerrado · Lean.

## **Propuestas pendientes de confirmar**

> * **Datos principales:** Inspiration4, panel metabólico (OSD-575) y hemograma (OSD-569). Públicos y sin login.  
> * **Usuario:** astronauta en misión larga y oficial médico de a bordo, con la Tierra respondiendo con demora.  
> * **Métrica de impacto:** cambios detectados con línea basal personal vs rango estándar de laboratorio. Prueba en R+1 con el panel metabólico: 7 de 72 por rango, 16 de 72 por línea basal, 11 que solo ve la basal. Exploratorio: 4 personas, 3 puntos de basal.  
> * **Base de código:** HealthLog tiene licencia no comercial; ver la evaluación en 04 · Proyectos similares.  
> * Edge cases de los casos médicos (propuesta de Claude, sáb 21:40, para Hassi y Naza): Eva no descarga ni inyecta; desfibrilación e insulina como módulos certificados de terceros que Eva coordina. Caso 2B sin inyección automática (reformulado como prevención y cólico renal). Caso 3A como despresurización de cabina. Cada variable marcada como continua, de estación o de laboratorio. Ver "03-04 · Edge cases: cómo responde Eva" en Fase 2 · 03\.

## **Preguntas para mentor u organización**

> * ¿OSDR / Inspiration4 está en la lista oficial de Resources del desafío?  
> * ¿Se puede usar como base un repositorio open source de terceros, con atribución?

## **Uso de IA (requisito de NASA)**

| Cuándo | Herramienta | Tarea | Archivos | Revisó |
| :---- | :---- | :---- | :---- | :---- |
| Sáb 14/11 | Claude (Anthropic) | Resumen del banco previo, criterios, agenda y desafíos | 07 · Entregables | Pendiente |
| Sáb 14/11 | Claude (Anthropic) | Descarga de datos OSDR y análisis exploratorio de línea basal | Datos NASA | Pendiente |
| Sáb 14/11 | Claude (Anthropic) | Investigación de los puntos 01, 02, 03, 05, 06 y evaluación de repos base | Fase 1 | Pendiente |

**Uso de IA desde las 13:45 (pasar a la tabla al cerrar el día)**  
Sáb 14/11 · Claude (Anthropic) · Código del prototipo Eva (HTML/JS) y del convertidor de anatomía de Human Atlas · Prototipo · Hecho, revisado por Lean.  
Sáb 14/11 · Claude (Anthropic) · Wireframe de la landing (hds-core \+ Superhuman) · Fase 2 · 08 · Hecho, revisado por Lean.  
Sáb 14/11 · Claude (Anthropic) · Análisis de ganadores previos (DeltaProtocol, PúlsAR) · Fase 2 · 01 · Hecho.  
Sáb 14/11 · Claude (Anthropic) · Kit para Codex: prompts, estilo y script de generación de imágenes · Fase 2 · 08 · Hecho.  
Sáb 14/11 · Codex (OpenAI, GPT-6.1 Terra \+ gpt-image-2) · Imágenes de la landing y el video; llevan marca de agua "AI-generated" y nada de logos de NASA · Fase 2 · 08 · En curso.  
Sáb 14/11 · Claude (Anthropic) en el prototipo · La capa 2 de razonamiento usa Claude como reemplazo de MedGemma mientras no tengamos el modelo descargado · Prototipo · Declarar en la entrega.  
Sáb 14/11 · Claude (Anthropic) · (Descartado) Design system sobre el kit de Tomás (CSS de componentes, muestrario, referencias sin logo, prompts de 14 pantallas, guía de imagen a código) · Fase 2 · 05 · Hecho, falta revisión.  
Sáb 14/11 · OpenAI GPT Image 2.5 (Flare y Sunburst) · Pantallas de UI como referencia de diseño, con marca de agua "AI-generated" · Fase 2 · 05 · Pendiente.  
Sáb 14/11 · Codex (OpenAI, modelo de imágenes) · 21 assets (escenas, capas y cuadros de la estación) y preview de motion, con marca "AI-generated" · Fase 2 · 08 · Hecho.  
Sáb 14/11 · Claude (Anthropic) · Landing y wireframe con el design system twocreate, motion y textos en inglés · Fase 2 · 08 · Hecho, falta revisión.  
Sáb 14/11 · Claude (Anthropic) · Edge cases de los casos médicos con fuentes y 23 escenarios para la simulación (sim/scenarios.json) · Fase 2 · 03-04 · Hecho, falta revisión de Hassi y Naza.

---

<!-- 06 · Concepto actual: Eva · 1M2ll0OgTznUeJLpwHmY7HVFUhyIqtuhWTD9awS11oJk -->

# **06 · Concepto actual: Eva**

Fuente de verdad del producto al sábado 14/11, 22:15. Incorpora el encargo de investigación del equipo (anexo abajo). La versión anterior queda en el historial del Drive.

## **En una línea**

Eva es el asistente de salud de WearMeds para un viaje comercial a Marte con unos 15 pasajeros sin formación de astronauta, sin médico a bordo y sin ayuda inmediata desde la Tierra. Reconoce emergencias con monitoreo no invasivo, guía a quien puede ayudar, activa tratamientos de emergencia a través de subsistemas validados y documenta todo.

## **Qué cambió con el encargo**

* La estación compartida se eliminó. Todo va en dos dispositivos personales, brazalete y arnés, más el hub informático de la nave.  
* La banda torácica pasa a ser un arnés con sensores y subsistemas de tratamiento.  
* Monitoreo siempre no invasivo (sin implantes ni agujas para medir). El tratamiento puede ser invasivo ante una emergencia mortal.  
* Comunicación: LoRa en cada brazalete hacia un hub a bordo, con una malla entre dispositivos que hay que evaluar, no dar por buena. Todo funciona sin Internet ni Tierra.  
* Pantalla táctil con lado largo del área activa de al menos \~100 mm, preferentemente flexible.  
* Sin presupuesto económico fijado; el bajo consumo es prioridad fuerte.

## **Piezas**

* **Brazalete (muñeca y antebrazo), en dos partes que se unen con imanes:** un módulo de sensores interior, sobre la piel, que hace todas las mediciones (sensado óptico de varias longitudes de onda tipo pulsioximetría CO, referencia Masimo rainbow) y tiene una batería chica propia; y un módulo de interfaz exterior con pantalla táctil grande, alertas, firmware local, radio LoRa, botón de ayuda, voz, vibración y la batería grande. Al unirse, unos pines con resorte alimentan los sensores desde la batería grande. Si la tela del traje queda en el medio, los imanes sostienen igual pero los pines no tocan: los sensores usan su batería propia. Las dos partes tienen LoRa: la base manda sus datos por LoRa (unida o con el traje en el medio) y puede hablar con el hub si la interfaz se suelta. Abierto: autonomía de la batería chica y precisión de medir en la muñeca por reflexión.  
* **Arnés (torso):** sensores por contacto y subsistemas de tratamiento. Desfibrilación vestible con análisis de ritmo validado y un compartimento para el inyectable. Una luz identifica el elemento a usar (reemplaza la bandeja iluminada de la estación).  
* **Hub de la nave:** procesamiento, IA médica principal sin conexión y comunicaciones. No es un equipo médico.

## **Las cuatro funciones**

* **Descarga:** desfibrilación solo con ritmo desfibrilable detectado por el análisis validado del subsistema. Nunca por un dato anormal genérico ni por un modelo de lenguaje.  
* **Procedimientos:** guía visual de una acción por paso para el afectado o para el compañero que ayuda.  
* **Inyectable:** vía y fármaco sin decidir. Comparar autoinyector, intramuscular, subcutáneo y otras vías; no suponer acceso a una vena.  
* **Bitácora:** eventos, mediciones, decisiones, incertidumbre y acciones, para el análisis en Tierra.

## **Casos de uso**

* **Caso A, no responde o no puede ayudarse:** Eva reconoce la posible emergencia con señales cruzadas, asigna al pasajero más cercano, le manda ubicación e instrucciones y avisa a los demás de forma informativa. Si el auxiliador no confirma, reasigna.  
* **Caso B, responde y puede actuar:** Eva evalúa si puede moverse y seguir pasos y le muestra la secuencia para usar los elementos del brazalete y del arnés. Si no puede, pasa a un compañero.  
* **Alerta solar (NASA DONKI y NOAA GOES):** sigue en el concepto, pero el encargo no la menciona. Confirmar.

## **Choques para decidir en equipo**

* **Nombre: RESUELTO (sáb 22:25). Se llama Eva en todo, también en el encargo.**  
* **Lenguaje:** la definición literal del encargo dice "totalmente autónomo… que prevenga, trate"; el Registro decidió "guides, records and warns". Con descarga e inyectable en el alcance, propuesta para el pitch: "Eva guides, records and warns; validated subsystems treat".  
* **Estación:** la landing (Caso 02), los cuadros F1–F3, las capas de la estación y el documento de edge cases la incluyen. Hay que rehacer esa parte con el arnés.  
* **Alerta solar:** ¿sigue en el alcance?  
* **Marca:** "Power Mats" sigue sin confirmar.

## **Cómo hablar del producto**

* Cifras verificadas: \~70% con cambios de visión en misiones de 6 a 12 meses en la ISS; 85% de 71 astronautas con problemas nasales o de senos; demora de 3 a 22 minutos por tramo a Marte; con línea basal personal, Inspiration4 muestra 16 de 72 cambios contra 7 con rangos de laboratorio.  
* Sin logos ni marcas de NASA. Textos de la entrega en inglés.  
* Jurado (según Jacobo): factibilidad, innovación, formato, pitch de los integrantes y ajuste al desafío. Nos evalúa como inversores.

## **Pendientes**

* Ejecutar el encargo de investigación: casos clínicos, sensores, componentes, energía y comunicaciones. Propuesta de reparto: 03-04 casos y sensores (Hassi y Naza), 02 componentes y energía (Tomi), 09 IA y comunicaciones (Lean).  
* Rehacer el Caso 02 de la landing y los assets con el arnés en lugar de la estación.  
* Repositorio de GitHub, laptop de la demo, MedGemma y MedASR.

## **Anexos**

* Encargo de investigación médica y arquitectura (equipo): [https\://docs.google.com/document/d/1pSradgUdKCN5O2YeCmGXNlE20-SyGAsuZZhEqzMcr90/edit](https://docs.google.com/document/d/1pSradgUdKCN5O2YeCmGXNlE20-SyGAsuZZhEqzMcr90/edit)  
* Edge cases, cómo responde Eva (parcialmente obsoleto por la estación): [https\://docs.google.com/document/d/1CNWBJEB4Js406\_LJ3AJqSkPxbjGQzrc1j6EciFRMesU/edit](https://docs.google.com/document/d/1CNWBJEB4Js406_LJ3AJqSkPxbjGQzrc1j6EciFRMesU/edit)  
> * 

Dom 15/11 05:00 · EVA, EL COPILOTO, ES EL CENTRO: agente de IA multimodal (texto, voz, imágenes, video) que lee el brazalete en tiempo real, hace registros diarios y diario emocional, da apoyo al diagnóstico (con qué hacer, por qué, cómo decidió y qué sigue) y manda el registro a control de misión. El brazalete es su factor de forma. Se recupera la arquitectura en tres capas (reflejo < 1 s, razonamiento < 10 s con MedGemma, registro encadenado y enlace). Lean: interfaz, calidad de uso y chatbot. Voz de las demos por Groq (clave solo en variables de entorno). Diario emocional: ver "04 · Salud mental y diario emocional (Claude)" · Lean.  
