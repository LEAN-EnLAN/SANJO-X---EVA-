# Contexto del proyecto (generado desde el Drive)

Generado: Sat 14/11/2026 21:16 (hora de Rosario) por `scripts/sync-context.sh`. No editar a mano: editá el Drive.

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
* **10 Maqueta del video de 30 s**: Lean

## **Documentos clave**

* **Bitácora del proyecto**: todas las discusiones en orden.  
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

## **Propuestas pendientes de confirmar**

> * **Datos principales:** Inspiration4, panel metabólico (OSD-575) y hemograma (OSD-569). Públicos y sin login.  
> * **Usuario:** astronauta en misión larga y oficial médico de a bordo, con la Tierra respondiendo con demora.  
> * **Métrica de impacto:** cambios detectados con línea basal personal vs rango estándar de laboratorio. Prueba en R+1 con el panel metabólico: 7 de 72 por rango, 16 de 72 por línea basal, 11 que solo ve la basal. Exploratorio: 4 personas, 3 puntos de basal.  
> * **Base de código:** HealthLog tiene licencia no comercial; ver la evaluación en 04 · Proyectos similares.

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

---

<!-- 06 · Concepto actual: Eva · 1M2ll0OgTznUeJLpwHmY7HVFUhyIqtuhWTD9awS11oJk -->

# **06 · Concepto actual: Eva**

*Fuente de verdad del producto al sábado 14/11, 20:55. Reemplaza a los borradores anteriores de esta carpeta, que quedan como historial.*

## **En una línea**

**Eva** es un copiloto de salud para un viaje comercial a Marte. Guía a cualquier pasajero, aunque no sea médico, en el primer minuto de una emergencia; registra todo y lo envía a la Tierra, que está a entre 3 y 22 minutos.

Los wearables se venden con la marca WearMeds y Eva es su asistente. Desde las 18:25 lo presentamos como un producto real para inversores, no solo como un proyecto de hackathon: hace falta negocio, producto y prototipo. Al ser wearables personalizables y de calidad, es una plataforma con varias líneas de desarrollo.

## **Pregunta que responde**

¿Cómo ayuda Eva a una persona sin formación médica a actuar en el primer minuto, cuando no hay médico a bordo y la Tierra tarda en responder?

## **Piezas**

> * **Banda torácica:** ECG y respiración.  
> * **Pulsera con pantalla grande:** pulso y movimiento; muestra instrucciones gráficas paso a paso.  
> * **Estación compartida:** presión arterial, SpO₂ y temperatura. Ilumina el elemento que hay que usar.  
> * **Núcleo Eva:** protocolo primero (instrucción en menos de 1 s) y IA a bordo después (MedGemma 1.5, en menos de 10 s).  
> * **Enlace:** registro encadenado que viaja a la Tierra con confirmación de recepción.

## **Casos**

> * **01 · No responde o no puede actuar:** Eva avisa al pasajero más cercano y lo guía para ayudar. Los demás reciben un aviso informativo.  
> * **02 · Puede actuar y moverse:** Eva lo guía a la estación, ilumina el elemento y muestra cómo usarlo.  
> * **Alerta solar:** con datos de NASA DONKI y NOAA GOES, avisa a cada pasajero cuándo ir al refugio. La ventana de aviso es la llegada de partículas, que tarda de decenas de minutos a horas; los rayos X llegan junto con la luz y no se pueden anticipar.  
> * En todos los casos: registra lo ocurrido y la asistencia.

## **Diferencia con DeltaMesh (Rosario 2018\)**

> * Ellos detectaban y avisaban; Eva resuelve qué pasa después del aviso: quién ayuda y cómo.  
> * Ellos no contemplaban médico a bordo ni demora con la Tierra; Eva está diseñado para eso.  
> * Ellos entregaron un informe sin prototipo; nosotros mostramos una demo que funciona con datos reales.

## **Cómo hablar del producto**

> * Decir: "guía, registra y avisa". No decir: "totalmente autónomo", "diagnostica" ni "trata".  
> * Cifras con fuente: \~70% con cambios de visión en misiones de 6 a 12 meses en la ISS; 81% con al menos un signo de SANS; 85% con problemas nasales o de senos paranasales; demora de 3 a 22 minutos por tramo a Marte.  
> * Sin logo ni marcas de NASA en ningún material.  
> * Nombre: Eva, como nombre propio (no sigla), para no confundirlo con la sigla EVA (caminata espacial) de NASA. El historial del nombre está en la Bitácora.

## **Criterios del jurado (según Jacobo)**

Factibilidad · innovación · formato y presentación · pitch de los integrantes · foco del problema y ajuste al desafío. Hay un jurado médico y un patrocinador de Rosario.

## **Pendientes**

> * Repositorio de GitHub y laptop de la demo.  
> * Descargar MedGemma 1.5 y MedASR (aceptar términos en Hugging Face).  
> * Decidir si la estación se demuestra con hardware real (LEDs) o simulada.  
> * Verificar si DeltaProtocol fue ganador global o nominado.  
> * Kit de Codex corrido: 21 assets y preview de motion recibidos y usados en la landing.  
> * Armar el negocio: cliente, precio, mercado, salida al mercado y línea de productos WearMeds.  
> * Confirmar el nombre "Power Mats".  
> * Landing y wireframe regenerados con el design system twocreate (sáb 21:05): revisarlos en equipo.  
> * Pantallas de UI generadas con imágenes (Tomás): pendientes.
