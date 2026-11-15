# Contexto del proyecto (generado desde el Drive)

Generado: Sun 15/11/2026 09:23 (hora de Rosario) por `scripts/sync-context.sh`. No editar a mano: editá el Drive.

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
* **Visor de investigación (carpetas con todos los docs, fuentes y progreso): https\://claude.ai/artifact/CXQN7Fn7UWVp6gWXPRfgAK**  
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
Sáb 14/11 22:25 · NOMBRE: el asistente se llama Eva en todo el proyecto, incluido el encargo de investigación (que decía HERMES). Choque cerrado · Lean.  
Dom 15/11 00:05 · EDGE CASES FILTRADOS (03-04 v2, con los comentarios del equipo): quedan paro cardíaco, despresurización de cabina y alerta solar (esta última pendiente de confirmar en el alcance). Salen 2A glucosa (el CGM es un filamento subcutáneo y el monitoreo tiene que ser no invasivo), 2B hueso y calcio (es crónico y no usa ninguna función de forma aguda), 4 salud mental (no es un evento agudo; la bitácora es compartida, no un diario personal) y todo lo que dependía de la estación eliminada. La descarga la decide el análisis de ritmo validado del arnés, nunca la IA. Escenarios en la rama fase2/03-04-casos-variables · Hassi y Naza (propuesta), registrado por Lean.  
Dom 15/11 01:45 · DECISIÓN: brazalete en dos partes unidas por imanes. Base sobre la piel con los sensores y una batería chica; interfaz con la pantalla y la batería grande; pines con resorte alimentan la base. Con el traje en el medio los pines no tocan y la base usa su batería · Equipo (registró Claude).  
Dom 15/11 02:20 · DECISIÓN: forma del brazalete según la referencia visual del equipo (módulo rectangular robusto, pantalla plana, correa textil). Sensores de la base: óptico, térmico, EDA, acelerómetro y giroscopio, tendencias de reposo y sueño, y control de señal · Equipo (registró Claude).  
Dom 15/11 02:25 · DECISIÓN: fuente principal del trabajo, Russell et al. 2023 (npj Microgravity 9:46), citada en el informe, la landing, el video, la app y la presentación · Lean.  
Dom 15/11 02:40 · DECISIÓN: las dos partes del brazalete tienen batería y LoRa propios; la base manda sus datos por LoRa, unida o con el traje en el medio · Lean.  
Dom 15/11 03:00 · DECISIÓN: SE ELIMINA EL ARNÉS. El brazalete es el único dispositivo en el cuerpo. Desfibrilación con el desfibrilador automático (DEA) de una estación de emergencia de la nave; el DEA analiza el ritmo y decide. Inyectable fuera del alcance. ECG a pedido con dos placas en la interfaz (un dedo de cada mano, 30 s); se descarta la versión muñeca \+ un dedo · Lean.  
Dom 15/11 05:00 · DECISIÓN: Eva, el agente copiloto de salud, es el centro de la propuesta y de la ingeniería; el brazalete es su factor de forma (lo que se vende y se evalúa objetivamente). Eva es multimodal (texto, voz, imágenes, video), lee el brazalete en tiempo real, hace registros diarios y diario emocional, da apoyo al diagnóstico y manda el registro a control de misión. Se recupera la arquitectura en tres capas del sáb 14/11. Lean se enfoca en la interfaz, la calidad de uso y el chatbot. Para las demos, voz con la API de Groq (la clave va solo en variables de entorno locales; recomendado rotarla porque se compartió en un chat). En el ensayo se presenta el plan de negocios con subsidio y escalabilidad · Lean.  
Dom 15/11 07:10 · DECISIÓN: se construye la app real en Rust \+ htmx (un binario), todo local y confiable primero. Alcance: app para astronautas (PWA instalable, responsive, panel de desarrollo oculto con el código Konami); dashboard para la Tierra o la ISS; registros y reportes con exportación rápida; ping y comunicación en tiempo real entre instancias tipo malla; el binario en modo servidor local enruta y mide a cada cliente; protocolo de comunicación muy liviano. Diccionarios de NASA y Hermes quedan para después. Cada versión completa se sube al Drive (carpeta "12 · App Eva: versiones") y cada paso se argumenta en el repo (docs/app, ADR) · Lean.  
Dom 15/11 07:10 · DECISIÓN: las comunicaciones se diseñan para espacio cercano, no para espacio profundo ("nunca vamos a llegar al espacio profundo"): latencia de segundos, enlaces intermitentes y con poco ancho de banda, no demoras de 20 minutos. Hay que revisar el relato de Marte en la landing, el video y el deck para que no contradiga esto · Lean.  
Dom 15/11 07:15 · DECISIÓN: Eva es el punto de venta y de innovación; la UI/UX se construye alrededor de ella. Eva súper simplificada, tierna y amigable; íconos simples (los fotorrealistas solo en pantallas de detalle grandes). Movimiento natural tipo resorte, interrumpible y continuo con el gesto, para que se use sin aprender. Pipelines separados de voz a texto, texto a voz, chat y visión, y tool calls propias que Eva y el usuario pueden disparar dentro de la app (ninguna puede dar una descarga ni un tratamiento). Modelos: Fable para planificar, Opus para ejecutar, Sonnet para lo menor · Lean.  
Dom 15/11 08:56 · DECISIÓN: el aviso de imágenes generadas con IA va en el README y en la descripción de la entrega, no dentro de la app. Se sacan las etiquetas "AI-generated" de la interfaz y las marcas pegadas en las fotos, el splash y las escenas del brazalete, en la app y en los wireframes. Reemplaza la regla anterior de marca visible en cada imagen (CLAUDE.md actualizado). · Lean.  
Dom 15/11 09:22 · DECISIÓN: brazalete v2, "más contenido y robusto": módulo rectangular gris grafito más ancho que alto, paragolpes de metal plateado con tornillos a la vista y marco engomado; dos módulos apilados unidos por imanes (base de sensores con dos contactos dorados con resorte y cuatro asientos de imán; módulo de pantalla táctil negra de borde a borde); botones naranjas (ayuda arriba en la pantalla y uno frontal en la base) y botón lateral; las dos placas del ECG de 30 s en el marco de la pantalla, a cada lado; dos correas textiles de la base, con barra metálica y refuerzo cosido en X, para que la pantalla se saque. Modo traje igual que antes. Colores: grafito \#3A3D42, plateado, naranja \#F28A1A solo en botones, dorado y vidrio negro. Actualizado en 06 · Concepto actual y en el repo (docs/CONTEXT.md, HANDOFF.md) · Lean.  
Dom 15/11 09:22 · DECISIÓN: diseño de la estación de emergencia de la nave: gabinete de pared tipo ISS con 12 compartimentos numerados en grilla de 3×4 (01 DEA, 02 respiración, 03 diagnóstico, 04 ecografía, 05 heridas, 06 sujeción, 07 medicación oral, 08 inyectables del botiquín de la tripulación, 09 acceso IV, 10 ojos y dental, 11 vía aérea, 12 higiene), variante con puertas laterales para 07 a 12; arriba, pantalla de Eva ("EVA · Take the AED"), botón rojo de emergencia y guía impresa de respaldo; el compartimento pedido se ilumina en verde azulado (\#1FE5C4); sensores de puerta y de elemento reportan "DEA retirado" al registro. El DEA analiza el ritmo y decide la descarga; Eva da apoyo al diagnóstico y guía. Ver "Estación de emergencia" en 06 · Concepto actual y docs/station.md · Lean.

## **Propuestas pendientes de confirmar**

> * **Datos principales:** Inspiration4, panel metabólico (OSD-575) y hemograma (OSD-569). Públicos y sin login.  
> * **Usuario:** astronauta en misión larga y oficial médico de a bordo, con la Tierra respondiendo con demora.  
> * **Métrica de impacto:** cambios detectados con línea basal personal vs rango estándar de laboratorio. Prueba en R+1 con el panel metabólico: 7 de 72 por rango, 16 de 72 por línea basal, 11 que solo ve la basal. Exploratorio: 4 personas, 3 puntos de basal.  
> * **Base de código:** HealthLog tiene licencia no comercial; ver la evaluación en 04 · Proyectos similares.  
> * Edge cases de los casos médicos (propuesta de Claude, sáb 21:40, para Hassi y Naza): Eva no descarga ni inyecta; desfibrilación e insulina como módulos certificados de terceros que Eva coordina. Caso 2B sin inyección automática (reformulado como prevención y cólico renal). Caso 3A como despresurización de cabina. Cada variable marcada como continua, de estación o de laboratorio. Ver "03-04 · Edge cases: cómo responde Eva" en Fase 2 · 03\.  
> * Dom 15/11 09:22 · PROPUESTA: arquitectura de enlace con un solo host, el hub de la nave (corre Eva, el registro firmado, el tablero y el enlace con la Tierra, y sirve la app). Brazaletes, teléfonos y la estación son clientes: los brazaletes hablan LoRa con el hub y se retransmiten en malla; los teléfonos van por Wi-Fi; la estación se suscribe a los eventos del caso y reporta puertas y elementos. La Tierra recibe los registros firmados por el enlace de espacio cercano (eva serve \--role ground). En la demo, la laptop es el hub (eva demo). Recomendación de Claude a la pregunta de Lean; falta confirmación del equipo. Ver "Cómo se conecta todo" en 06 · Concepto actual y docs/station.md.  
> * Dom 15/11 09:22 · PROPUESTA: respaldo por LoRa si se cae el hub: el brazalete transmite su alarma de pérdida de pulso, la estación la escucha directamente, ilumina el compartimento del DEA y suena; el botón rojo de la estación transmite una alarma a todos los brazaletes. Al volver el hub, estación y brazaletes suben lo ocurrido y la cadena de hashes lo ordena. Falta confirmación del equipo.

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
Sáb 14/11 · Claude (Anthropic, con subagentes) · Landing v2 con motion por scroll y arnés, visor de investigación, riel del equipo, análisis de motion y kit de Codex v2 (fondos negro y blanco) · Fase 2 · 08 · Hecho, falta revisión de Lean.  
Sáb 14/11 · Codex (OpenAI, modelo de imágenes) \+ Claude (Anthropic) · Assets del kit v2 (negro, blanco, equipos y espacio) y landing v3 que los usa · Fase 2 · 08 · Hecho, falta revisión de Lean.  
Dom 15/11 · Claude (Anthropic) · Edge cases v2: lectura de los comentarios del equipo, filtro de casos, documento rehecho con respuestas a cada comentario, y escenarios de simulación y docs/edge-cases.md actualizados · Fase 2 · 03-04 · Hecho, falta revisión de Hassi y Naza.  
Dom 15/11 · Claude (Anthropic) · Landing v10: alerta solar como simulación desde la cámara de la nave, hero con estrellas que siguen al mouse, tema negro puro fijo · Fase 2 · 08 · Hecho, falta revisión de Lean.  
Dom 15/11 · Claude (Anthropic) \+ earshot (MIT) · Primer corte del video explicativo en Manim (7 escenas vectoriales, 1:43, efectos de sonido sintetizados) · Fase 2 · 10 · Borrador; se rehace para público no técnico, en inglés y español.  
Dom 15/11 · Claude (Anthropic) · Cambio de producto: brazalete en dos partes (sensores adentro con batería chica propia; interfaz afuera con pantalla y batería grande; imanes y pines con resorte; con el traje en el medio los pines no tocan). Concepto actualizado; landing y video en curso · Fase 2 · 02 y 08 · En curso.  
Dom 15/11 · Claude (Anthropic, con subagentes) · Sin arnés en todo: Concepto reescrito, edge cases (Drive, docs/edge-cases.md y sim/scenarios.json, casos nuevos T16 y T17), wireframes de la app republicados y video narrado en inglés y español (Piper TTS) actualizado; fuente principal Russell et al. 2023 subida al Drive y citada en landing, video y app · Fase 2 · 03-04, 05, 08, 10 · Hecho, falta revisión del equipo.  
Dom 15/11 · Codex (OpenAI, GPT Image 2.5) con kits de Claude · 32 íconos fotorrealistas (eva-icons-kit) y 12 imágenes del brazalete real (eva-wristband-kit), con marca "AI-generated" · Fase 2 · 05 y 08 · Hecho; integrándose en los wireframes y la landing.  
Dom 15/11 · Claude (Anthropic, con subagentes) · Pantallas de Eva en los wireframes (chat multimodal, apoyo al diagnóstico, registro diario, diario emocional, brazalete "Eva live"); mascota recortada de los stickers del equipo (imagen de IA, con etiqueta visible); doc "04 · Salud mental y diario emocional" · Fase 2 · 04 y 05 · Hecho, falta revisión de Lean.  
Dom 15/11 · Claude (Anthropic, con subagentes) y Codex (OpenAI) · App Eva v0.2–v0.3.1 en Rust \+ htmx (pantallas del pasajero, tablero de tripulación y de Tierra, firmas Ed25519, benchmarks, pruebas de rotura, guía de demo), integración del kit de arte de Codex (44 íconos, 18 estados de Eva, ícono, splash, 10 fotos) y deck de 15 diapositivas con los clips. Las imágenes de IA se declaran en los README (app/README.md, design/app-wireframes/README.md y el README del paquete), sin marcas en pantalla · Fase 2 · 05, 10 y 12 · Revisó: Lean.  
Dom 15/11 · Claude (Anthropic, con subagentes) · Documentación del brazalete v2 y de la estación de emergencia, y propuesta de cómo se conecta todo (hub como único host, clientes, LoRa en malla y respaldo), a partir de las imágenes de referencia del equipo (IA) · 06 · Concepto actual: Eva, este Registro, docs/station.md, docs/CONTEXT.md y HANDOFF.md (rama claude/entorno-compartido), docs/app/ADR-011-station.md (rama fase2/12-app) · Revisó: Lean.

---

<!-- 06 · Concepto actual: Eva · 1M2ll0OgTznUeJLpwHmY7HVFUhyIqtuhWTD9awS11oJk -->

# **06 · Concepto actual: Eva**

> * Fuente de verdad del producto al domingo 15/11, 03:00. Cambio grande del día: no hay arnés. Todo va en el brazalete, más el hub y una estación de emergencia de la nave con desfibrilador. La versión anterior queda en el historial del Drive.

> * ## **En una línea**

> * Eva es el agente copiloto de tu salud, en asistencia de tus exploraciones, misiones y vida diaria. Es un asistente de IA multimodal (texto, voz, imágenes y video) que lee tu brazalete WearMeds en tiempo real, registra todo cada día, te acompaña con un diario emocional, da apoyo al diagnóstico, guía emergencias y manda el registro a control de misión en la Tierra. Primer caso: un viaje comercial a Marte con unos 15 pasajeros sin formación médica, sin médico a bordo y con la Tierra a minutos de distancia. Eva es la innovación; el brazalete es su factor de forma, lo que se vende y se evalúa objetivamente. La descarga la decide el desfibrilador de la estación, nunca Eva.

> * ## **Qué cambió hoy (dom 15/11)**

> * No hay arnés. El brazalete es el único dispositivo en el cuerpo.  
> * La desfibrilación pasa a una estación de emergencia de la nave con un desfibrilador automático (DEA). El DEA lee el ritmo con sus propios parches y decide si hace falta la descarga.  
> * El inyectable sale del alcance.  
> * ECG a pedido con dos placas metálicas en la interfaz: un dedo de cada mano, 30 segundos.  
> * Las dos partes del brazalete tienen batería y radio LoRa propias.  
> * Fuente principal del trabajo: Russell et al. 2023 (ver "Cómo hablar del producto").  
> * Corrección de Lean (dom 15/11): Eva, el agente, es el centro de la propuesta y de la ingeniería. Se recupera lo que había quedado afuera: registro en tiempo real, registros diarios, diario emocional, apoyo al diagnóstico y envío a control de misión.

> * ## **Eva, el copiloto: el centro de la propuesta**

> * Multimodal: entiende texto, voz, imágenes y video, y responde con texto y voz. En las demos, la voz (speech to text y text to speech) sale de la API de Groq; en la nave, de modelos locales sin Internet.  
> * Lee el brazalete en tiempo real y lo compara con tu línea basal personal. En la muñeca, la pantalla lo dice: "Eva te está leyendo en vivo".  
> * Registro diario automático (sueño, actividad, agua, comida, ánimo, signos frente a tu basal), siempre documentado y firmado a bordo.  
> * Diario emocional (journaling): registro de estado de ánimo con color y escritura expresiva. Base científica y variables en "04 · Salud mental y diario emocional (Claude)".  
> * Apoyo al diagnóstico: diagnóstico diferencial con probabilidades, siempre con qué hacer ahora, por qué, cómo decidió Eva y qué sigue (Russell et al. 2023). Lo confirma la tripulación o la Tierra.  
> * Guía de emergencias paso a paso, y envío del registro a control de misión con confirmación de recepción.  
> * Arquitectura en tres capas (plan del sáb 14/11, sigue vigente): reflejo en menos de 1 s con protocolos curados; razonamiento en menos de 10 s con un modelo médico multimodal (MedGemma 1.5 4B) más documentación de NASA y la línea basal; registro encadenado y enlace con la Tierra.  
> * Dónde va la ingeniería: Lean en la interfaz, la calidad de uso y el chatbot multimodal. La primera demo (el agente razonando) se actualiza apenas se aprueben los wireframes.  
> * Cuidado con la palabra: decir "apoyo al diagnóstico" (diagnostic support), no "diagnóstico" a secas. MedGemma requiere validación y el criterio de validez castiga exagerar.  
> * Negocio: es un producto que se vende (brazalete más suscripción a Eva, precios \[a definir\]), con un modelo subsidiado \[a definir\] y escalable a análogos en la Tierra (bases antárticas, barcos, trabajo remoto). Nos destaca la factibilidad: podemos hacerlo nosotros.

> * ## **Piezas**

> * Brazalete (muñeca), versión 2 (dom 15/11, más contenido y robusto), en dos módulos apilados que se unen con imanes:  
> * • Forma: módulo rectangular robusto, gris grafito, más ancho que alto (cerca de 16:9, como un teléfono chico), que va arriba de la muñeca. Esquinas con paragolpes de metal plateado mecanizado sujetos con tornillos a la vista, marco engomado de tacto suave y "EVA" impreso chico en el borde frontal.  
> * • Base (sobre la piel): sensores, batería chica y LoRa propios. Óptico (pulso, regularidad y variabilidad; SpO₂ y respiración estimada en reposo; referencia de sensado multionda: Masimo rainbow), térmico (temperatura de la piel), electrodos EDA (actividad electrodérmica), acelerómetro y giroscopio (movimiento, inmovilidad y temblor), tendencias de reposo y sueño, y control de señal (contacto y calidad). Es una placa delgada: en la cara de arriba tiene dos contactos dorados con resorte (pogo pins), centrados, que pasan la energía entre los dos módulos, y cuatro almohadillas grises en las esquinas (asientos de los imanes). En el borde frontal: orificio de micrófono, rejilla de parlante y un segundo botón naranja.  
> * • Módulo de pantalla (interfaz, afuera): pantalla táctil negra grande, plana, de borde a borde y con esquinas redondeadas; batería principal y LoRa propios; rejilla de parlante en el borde frontal; botón de ayuda naranja en el borde superior y un botón lateral (basculante o redondo); voz y vibración. Las dos placas metálicas del ECG van en el marco, una a cada lado de la pantalla.  
> * • Correas: dos correas textiles gris oscuro, una en el borde superior y otra en el inferior. Cada una pasa por una barra metálica plateada de la base y tiene un refuerzo cosido en X. Las correas son de la base, así que el módulo de pantalla se saca sin quitarse el brazalete.  
> * • Colores: cuerpo grafito \#3A3D42, paragolpes plateados, naranja \#F28A1A solo en los botones, contactos dorados y vidrio negro.  
> * • Unidas: pines con resorte (los dos contactos dorados) llevan energía de la batería principal a la base. Con la tela del traje en el medio, los imanes sostienen pero los pines no tocan: la base usa su batería y sigue mandando sus datos por LoRa.  
> * • ECG de una derivación a pedido: un dedo de cada mano sobre las dos placas, quieto, unos 30 segundos. No necesita la piel de la muñeca, así que funciona con el traje puesto si las manos están descubiertas. Ayuda a identificar arritmias como la fibrilación auricular, algo que el sensor óptico solo puede sospechar. No es continuo, necesita que la persona colabore, no reemplaza un ECG de 12 derivaciones y nunca decide una descarga.  
> * Estación de emergencia de la nave: desfibrilador automático (DEA) y botiquín básico. Eva indica dónde está y la estación se ilumina. Detalle en "Estación de emergencia".  
> * Hub de la nave: procesamiento, IA médica sin conexión, registro y comunicaciones. No es un equipo médico.

> * ## **Estación de emergencia**

> * Qué es: un gabinete médico montado en la pared de un módulo habitable, al estilo de la ISS (paneles beige, pasamanos y correas azules alrededor). Es un cliente del hub, no un host (ver "Cómo se conecta todo"). Referencias: imágenes conceptuales del equipo hechas con IA (dom 15/11).  
> * 12 compartimentos numerados en una grilla de 3×4, cada uno detrás de una ventana transparente con etiqueta y tira de luz. Las etiquetas de las imágenes están en español; en el producto van en inglés:  
> * • 01 DEA (AED): el desfibrilador y parches de repuesto.  
> * • 02 Respiración (Breathing): botella de oxígeno, máscara y bolsa-válvula-máscara.  
> * • 03 Diagnóstico (Diagnostics): monitor, termómetro y glucómetro.  
> * • 04 Ecografía (Ultrasound) · 05 Heridas (Wounds) · 06 Sujeción (Restraint).  
> * • 07 Medicación oral (Oral meds) · 08 Inyectables (Injectables): parte del botiquín de la estación que usa la tripulación, no un inyectable que se lleva puesto · 09 Acceso IV (IV access).  
> * • 10 Ojos y dental (Eyes and dental) · 11 Vía aérea (Airway) · 12 Higiene (Hygiene).  
> * Variante: gabinete central con puertas laterales que se abren y muestran los compartimentos 07 a 12\.  
> * Pantalla de Eva, arriba del gabinete: dice qué retirar y qué hacer, por ejemplo "EVA · Take the AED". Al lado va una guía impresa de respaldo ("AED retrieval guide").  
> * Botón rojo de emergencia ("PRESS IN EMERGENCY"), arriba del gabinete: abre un caso en el hub y además transmite una alarma por LoRa a todos los brazaletes, así funciona aunque el hub no responda.  
> * Luces: cuando Eva pide un elemento, su compartimento se ilumina en verde azulado (\#1FE5C4) y la pantalla dice qué retirar.  
> * Sensores: un sensor de puerta por compartimento y un sensor de elemento (peso o reed switch) reportan, por ejemplo, "puerta 01 abierta 00:41" y "DEA retirado 00:47". Esos reportes entran al registro firmado y a la línea de tiempo del caso.  
> * Por dentro: una computadora chica, radio LoRa y batería de respaldo.  
> * El DEA analiza el ritmo con sus propios parches y decide la descarga. Eva nunca la decide: da apoyo al diagnóstico, guía y registra.

> * ## **Cómo se conecta todo**

> * Recomendación del dom 15/11, a confirmar en equipo (ver "Para decidir en equipo"). Responde la pregunta de Lean: ¿el backend es un servidor en una computadora local o cada cliente aloja algo? ¿La estación es host o cliente?  
> * Un solo host, el hub de la nave: una computadora local (el servidor) que corre Eva, el registro firmado, el tablero y el enlace con la Tierra, y sirve la app (PWA). Es la única fuente de verdad del registro y del estado de Eva.  
> * Todo lo demás es cliente:  
> * • Brazaletes: cada parte habla por LoRa con el hub. Los brazaletes se retransmiten entre sí (malla) cuando uno queda fuera de alcance.  
> * • Teléfono o tableta con la app: pantalla grande opcional de la misma persona, conectada al hub por el Wi-Fi de la nave.  
> * • Estación: tiene su propia computadora chica, pantalla de Eva, luces, sensores, LoRa y batería de respaldo, pero no aloja nada. Se suscribe a los eventos del caso que manda el hub (iluminar el compartimento 01, mostrar "Take the AED") y le devuelve lo que pasa (puerta abierta, DEA retirado).  
> * Si se cae el hub (el camino crítico nunca depende de un solo equipo): el brazalete transmite su alarma de pérdida de pulso por LoRa; la estación la escucha directamente, ilumina el compartimento del DEA y suena una alerta; el botón rojo de la estación también transmite una alarma a todos los brazaletes. Cuando vuelve el hub, la estación y los brazaletes suben lo que pasó y la cadena de hashes lo mantiene en orden.  
> * Tierra: una instancia aparte, en tierra, recibe los registros firmados por el enlace de espacio cercano, con almacenamiento y reenvío y confirmación de recepción (ya construido: eva serve \--role ground).  
> * En la demo, la laptop de la mesa es el hub (eva demo): los teléfonos hacen de brazaletes y de pantallas, y una tableta o una segunda ventana del navegador hace de estación.  
> * Por qué así: una sola fuente de verdad para el registro y el estado de Eva; clientes baratos y simples; funciona sin conexión en la nave; la estación sigue en el camino de la alarma aunque falle el hub; y para la demo alcanza con una laptop.

> * ## **Funciones**

> * Monitoreo continuo con el brazalete: pulso, SpO₂, respiración estimada, temperatura, EDA, movimiento y sueño. Sospecha de ritmo irregular y detección de pérdida de pulso (sin pulso, sin movimiento y sin respuesta).  
> * ECG a pedido con las dos placas.  
> * Guía de emergencias: una acción por paso, diciendo qué hacer ahora, por qué, cómo decidió Eva y qué sigue (Russell et al. 2023).  
> * Desfibrilación: con el DEA de la estación, que analiza el ritmo y decide.  
> * Bitácora: eventos, mediciones, decisiones e incertidumbre, firmada a bordo y enviada a la Tierra cuando hay enlace.  
> * Alerta solar (NASA DONKI y NOAA GOES): sigue en el concepto; confirmar alcance.

> * ## **Casos de uso**

> * Caso A, no responde (posible paro): el brazalete detecta pérdida de pulso. Eva asigna al pasajero entrenado más cercano para empezar la RCP y a un segundo ayudante para traer el DEA de la estación. Guía la RCP en la pantalla del brazalete; el DEA analiza y, si corresponde, indica la descarga. Avisa a los demás de forma informativa y reasigna si nadie confirma.  
> * Caso B, responde y puede actuar: Eva evalúa, le pide un ECG con las dos placas si hace falta y lo guía. Si empeora, pasa a un compañero.

> * ## **Para decidir en equipo**

> * Alerta solar: ¿sigue en el alcance?  
> * Marca: "Power Mats" sigue sin confirmar.  
> * Cantidad y ubicación de las estaciones con DEA (propuesta: una por módulo habitable, a menos de un minuto de cualquier asiento).  
> * Autonomía de la batería chica con LoRa transmitiendo a potencia mínima.  
> * Sensado en muñeca: se mide por reflexión, con menos precisión que en el dedo. Evaluar cada parámetro antes de prometerlo.  
> * Datos por los pines cuando las partes están unidas, para ahorrar radio.  
> * Confirmar la arquitectura de enlace: el hub como único host; brazaletes, teléfonos y estación como clientes; LoRa en malla y respaldo directo brazalete-estación si cae el hub (propuesta del dom 15/11, ver "Cómo se conecta todo").

> * ## **Cómo hablar del producto**

> * Cifras verificadas: \~70% con cambios de visión en misiones de 6 a 12 meses en la ISS; 85% de 71 astronautas con problemas nasales o de senos; demora de 3 a 22 minutos por tramo a Marte; con línea basal personal, Inspiration4 muestra 16 de 72 cambios contra 7 con rangos de laboratorio.  
> * Fuente principal: Russell BK et al. The value of a spaceflight clinical decision support system for earth-independent medical operations. npj Microgravity 9, 46 (2023). doi:10.1038/s41526-023-00284-1. Lejos de la Tierra la tripulación tiene que detectar, diagnosticar y tratar sola (demoras de ida de más de 20 min); el oficial médico de la ISS suele tener unas 40 h de formación; un sistema de apoyo a la decisión que diga qué hacer ahora, por qué, cómo se decidió y qué sigue reduce el riesgo (análisis preliminar: al menos 34% menos muertes en una misión simulada de 26 meses). Eva lleva esa idea a pasajeros sin formación. Resumen en "00 · Fuente principal: Russell et al. 2023".  
> * Frase del pitch: "Eva, the health copilot for your missions, explorations and daily life. It reads, records, supports diagnosis and warns. The AED decides the shock."  
> * Sin logos ni marcas de NASA. Textos de la entrega en inglés.  
> * Jurado (según Jacobo): factibilidad, innovación, formato, pitch de los integrantes y ajuste al desafío. Nos evalúa como inversores.

> * ## **Anexos**

> * Fuente principal: https\://docs.google.com/document/d/1jGR0k\_S6ZOTzayrn1tBXRXNW3m9ZZAsKZaSn9XrpUL8/edit  
> * Encargo de investigación médica y arquitectura (equipo; menciona el arnés, ya descartado): https\://docs.google.com/document/d/1pSradgUdKCN5O2YeCmGXNlE20-SyGAsuZZhEqzMcr90/edit  
> * Edge cases, cómo responde Eva: https\://docs.google.com/document/d/1CNWBJEB4Js406\_LJ3AJqSkPxbjGQzrc1j6EciFRMesU/edit  
> * 
