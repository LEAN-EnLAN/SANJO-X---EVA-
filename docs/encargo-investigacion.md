# HERMES — Encargo de investigación médica y arquitectura para una misión a Marte

## 0. Cómo usar este archivo

Este archivo es un encargo autocontenido. Podés recibirlo en un chat sin contexto previo. Leelo completo antes de investigar. Las decisiones actuales de este documento prevalecen sobre imágenes, ideas o conversaciones anteriores del proyecto.

Tu tarea es realizar una investigación con fuentes y entregar una propuesta concreta, trazable y técnicamente defendible. No te limites a repetir esta arquitectura ni a listar sensores: seleccioná qué emergencias conviene abordar, demostrá qué información permitiría reconocerlas y justificá componentes reales para una arquitectura candidata de vuelo.

No estamos buscando un prototipo barato para construir durante el hackathon. Buscamos qué tecnologías, subsistemas y componentes tendría sentido evaluar para que una organización como NASA desarrollara este sistema. No afirmes que NASA debería aprobar una pieza ni que una propuesta está lista para volar: justificá su selección y explicitá las brechas de validación.

Respondé en español claro. Las referencias y nombres comerciales pueden estar en inglés. Empezá por una síntesis corta y tablas; dejá el detalle en anexos. El equipo necesita comprender y comparar sin leer una pared de texto.

### 0.1 Prioridad central: elegir los mejores componentes para HERMES

**El resultado principal debe ser una selección óptima y justificada de componentes, no un catálogo de piezas que simplemente podrían servir.** Investigá con amplitud, compará alternativas reales y recomendá una combinación concreta para esta misión. No hay una marca, tecnología, pantalla, procesador ni sensor favorito que debas confirmar.

“Mejor” significa mejor adecuación al sistema completo: seguridad y desempeño clínico pertinentes, fiabilidad para la misión, monitoreo no invasivo, interfaz intuitiva, bajo consumo, ergonomía, integración, disponibilidad y trazabilidad. No significa el más barato, el más conocido, el de mayor resolución ni el de mayor número de sensores. El costo es informativo; no es el criterio dominante porque no hay un límite económico fijado.

Método obligatorio de selección:
1. Convertir el alcance en requisitos verificables. Separar obligatorios, preferencias y datos pendientes.
2. Aplicar primero un filtro de cumplimiento: una ventaja estética o un score alto no compensan incumplir un requisito crítico.
3. Para cada subsistema decisivo, comparar preferentemente tres o más candidatos reales de fabricantes distintos cuando existan. Si solo hay uno o dos viables, explicar la limitación; no completar la lista con opciones irrelevantes.
4. Usar datos comparables y condiciones explícitas. Si falta un dato crítico, mantener al componente como candidato condicionado en lugar de declararlo ganador.
5. Explicar por qué gana el elegido y por qué pierden las alternativas. Mostrar qué evidencia, ensayo o cambio de requisito podría invertir la decisión.
6. Evaluar la combinación completa: procesador, pantalla, interfaces, fuentes de alimentación, RF, almacenamiento, sensado y tratamiento deben funcionar juntos. El componente óptimo aislado puede empeorar el sistema.
7. Presentar una configuración principal y una alternativa de respaldo. No dejar la decisión al lector con una lista sin conclusión.

Optimizar energía, masa, volumen y complejidad sin reducir la calidad de las señales o la capacidad de respuesta exigidas por los casos clínicos seleccionados. Identificar opciones dominadas y, cuando haya compromisos reales, mostrar la frontera de alternativas en lugar de inventar un ganador absoluto.

**Preferencia expresa del equipo: pantalla flexible.** Buscar activamente la mejor opción flexible táctil y compararla con la mejor plana. Elegir la flexible cuando cumpla los requisitos y resulte defendible para HERMES; no descartarla por ser menos habitual o más difícil de prototipar. Si solo una plana permite cumplir los requisitos o la flexible introduce compromisos incompatibles con la misión, recomendar la plana con evidencia y explicar la brecha concreta.

## 1. Proyecto y definición exacta

Nombre de trabajo: **HERMES**.

Expansión propuesta, sin necesidad de cambiarla ni investigarla ahora: **Human-centered Emergency Response and Medical Expertise System**.

Definición del proyecto, conservar literalmente:

> Un copiloto de salud totalmente autónomo, no invasivo, e intuitivo  que prevenga, trate y documente de manera inmediata los riesgos críticos de salud

Aclaración acordada sobre esa definición: **el monitoreo debe ser no invasivo; el tratamiento puede ser invasivo ante una emergencia potencialmente mortal**. No proponer implantes, agujas, microneedles ni filamentos subcutáneos para obtener datos. Un tratamiento inyectable no habilita por sí mismo a utilizar sensores invasivos.

Escenario: viaje comercial a Marte con aproximadamente 15 pasajeros que no tienen formación de astronauta profesional. Todos recibirían capacitación básica de emergencia. No asumir un médico disponible a bordo ni asistencia inmediata desde Tierra. El objetivo es reducir muerte o daños graves y facilitar una respuesta que una persona común pueda comprender y ejecutar.

## 2. Decisiones cerradas

### 2.1 Equipamiento personal: dos dispositivos

1. **Brazalete** sobre muñeca/antebrazo: interfaz principal, pantalla táctil grande, alertas, procesamiento local/firmware, comunicaciones y sensores que se justifiquen en esa ubicación.
2. **Arnés** corporal/torácico: sensores y subsistemas terapéuticos que se justifiquen por contacto, posición anatómica, energía, seguridad y ergonomía.

**La estación médica compartida fue eliminada. No la vuelvas a incluir.** Los sensores y elementos de tratamiento propuestos deben distribuirse entre brazalete y arnés, incluidos consumibles y almacenamiento necesario. Si esto resulta inviable para una función, explicalo y proponé la modificación mínima; no escondas una estación adicional en la solución.

El **hub informático de la nave sí permanece**, pero no es una estación médica: aloja procesamiento, IA y comunicaciones. Su acceso a datos de la nave puede evaluarse; no asumir instrumentos médicos compartidos adicionales.

### 2.2 Uso

- Uso inicial únicamente **dentro de la nave, en ambiente presurizado**.
- No pedir resistencia al vacío ni operación exterior como requisitos actuales.
- Considerar igualmente radiación en tránsito interplanetario, microgravedad, ambiente de cabina, lanzamiento, materiales, disipación, limpieza y fiabilidad.
- No trasladar automáticamente una calificación para órbita baja al entorno de un viaje a Marte.

### 2.3 Pantalla: requisitos obligatorios

- Táctil.
- **Lado largo del área activa de imagen de al menos 100 mm aproximadamente**. Informar su medida exacta; no cumplirlo contando el marco, el PCB o la diagonal.
- Mostrar gráficos grandes y comprensibles, instrucciones de una acción por paso, alertas y orientación.
- Bajo consumo, buena legibilidad en cabina y funcionamiento fiable durante una emergencia.
- **Preferentemente flexible**, para adaptarse al antebrazo, siempre que cumpla los requisitos de tamaño activo, táctil, legibilidad, consumo y fiabilidad. No es un requisito absoluto: una plana puede ganar si la comparación demuestra que es la mejor solución viable. Distinguir curvatura fija, flexión repetida y plegado; no son capacidades equivalentes.
- No hay un modelo elegido definitivamente. Las recomendaciones anteriores de pantallas comerciales fueron exploratorias y no son requisitos.

### 2.4 Energía y presupuesto

No hay un presupuesto económico fijado. La **reducción del consumo** es una prioridad fuerte, compatible con la respuesta oportuna y la seguridad. No sacrificar una función de rescate únicamente para mejorar el consumo promedio.

Proponé una autonomía objetivo y justificála. No inventes un requisito de duración como si el equipo ya lo hubiera decidido. Compará al menos dos perfiles de uso y una situación de emergencia prolongada.

## 3. Cuatro funciones de alcance

El equipo las llama informalmente “cuatro actuadores”. Distinguí correctamente las intervenciones físicas de las funciones de software:

1. **Shock eléctrico:** desfibrilación destinada a salvar vidas ante indicaciones apropiadas. No es estimulación eléctrica general ni descarga ante cualquier dato anormal.
2. **Procedimientos de emergencia:** guía visual extremadamente intuitiva para el afectado o el compañero que auxilia. Evaluar qué tareas pueden ejecutarse con capacitación básica y qué tareas exceden esa capacidad.
3. **Inyectable:** investigar alternativas viables de administración terapéutica en emergencias. **La vía y el medicamento no están decididos.** No asumir que hay que encontrar una vena ni que una misma vía sirve para todos los fármacos.
4. **Bitácora:** registro de eventos, mediciones, decisiones, incertidumbre, alertas y acciones. Si no aporta una intervención aguda, tratarla como soporte documental para análisis posterior en Tierra, sin atribuirle un efecto terapéutico.

No incorporar por defecto otros actuadores discutidos anteriormente (respirador desplegable, compresión mecánica, ultrasonido terapéutico, etc.). Solo mencionarlos como brechas o líneas futuras si resultan imprescindibles para explicar una limitación.

## 4. Arquitectura de comunicaciones y software prevista

Arquitectura deseada por el equipo:

- Cada brazalete tiene firmware y un módulo de comunicación **LoRa**.
- Los brazaletes se comunican con un **hub a bordo**.
- Se desea además una **malla entre dispositivos** que permita retransmitir y comunicar alertas/datos entre pasajeros.
- La **IA médica principal se ejecuta localmente en el servidor/hub de la nave**.
- La banda transmite la información necesaria; el hub la evalúa y devuelve indicaciones, alertas o solicitudes de información.
- Todo debe funcionar **sin Internet y sin depender de Tierra**.

**Esta intención debe analizarse, no darse por resuelta.** LoRa no implica por sí mismo una red mesh, localización precisa ni transporte adecuado para cualquier señal.

Debés evaluar:

- LoRa punto a punto, protocolo de malla, nodos de retransmisión y diferencia respecto de LoRaWAN. Identificar qué componente y qué software implementan cada función.
- Tamaño/frecuencia de mensajes, número de nodos, latencia, confirmaciones, pérdidas, reintentos, congestión, ahorro de energía y prioridad de emergencias.
- Presupuesto de enlace realista en una nave metálica compartimentada. No utilizar el alcance en campo abierto como prueba de cobertura interior.
- Qué se envía como datos crudos, qué se procesa localmente y qué se almacena. Calcular el tráfico de ECG u otras señales con número de canales, frecuencia de muestreo y resolución explícitos.
- No transmitir imágenes, videos o pantallas completas por LoRa sin demostrar viabilidad. Evaluar instrucciones/identificadores y contenido gráfico precargado en el brazalete.
- Cómo identificar al pasajero más cercano: comparar métodos de localización adecuados. No afirmar que el RSSI de LoRa equivale a distancia fiable.
- Consumo de una malla que escucha/retransmite frente a un enlace directo al hub; evaluar la arquitectura mínima que mantiene la función buscada.
- Qué ocurre ante pérdida de enlace, partición de red, caída del hub o datos atrasados.
- Qué funciones de emergencia requieren un módulo local independiente y validado para no depender de un viaje de ida/vuelta al hub. No delegar ciegamente la autorización de descargas a una IA generativa.
- Autenticación, integridad, identidad de paciente, antigüedad de datos/comandos, prevención de duplicados y registro de fallas.

Si LoRa no cumple algún requisito, demostrarlo con números y presentar una alternativa o arquitectura híbrida, conservando explícitamente qué parte del planteo original cambia y por qué. No descartar ni aceptar la malla sin analizarla.

## 5. Experiencia de uso que debe sostener la arquitectura

### Caso A: la persona no responde o no puede ayudarse

1. Reconocer una posible emergencia mediante señales y comprobaciones apropiadas; no equiparar inmovilidad a inconsciencia.
2. Asignar el auxilio al **pasajero más cercano**, considerando que todos tienen capacitación mínima.
3. Enviar al auxiliador ubicación, contexto e instrucciones visuales pertinentes.
4. Los demás pasajeros reciben un **aviso informativo**, distinto de la asignación de auxilio. No interpretar esto como una prohibición absoluta de ayudar.
5. Proponer confirmación de recepción y reasignación si no responde el auxiliador, sin dar por cerrado ese flujo.

### Caso B: responde y puede actuar

1. Evaluar también su capacidad para moverse y ejecutar instrucciones; estar consciente no garantiza poder ayudarse.
2. Mostrar la secuencia gráfica adecuada para usar los elementos del brazalete/arnés.
3. Si no puede ejecutar una acción, pasar a asistencia de un compañero.
4. La versión anterior guiaba a una estación e iluminaba compartimentos. **Ahora esa estación no existe**: investigar si se puede trasladar la identificación/iluminación del elemento a un módulo del arnés.

Priorizar imágenes comprensibles, una acción por paso, alto contraste y controles grandes. Evaluar voz, vibración y texto breve de respaldo por accesibilidad; no asumir que un dibujo sin texto se comprende universalmente. No diseñar nuevas láminas todavía: especificar requisitos de interacción y su relación con el hardware.

## 6. Investigación médica: elegir primero casos útiles y abordables

No partas de una lista arbitraria de enfermedades ni de un catálogo de sensores. Buscá la mejor relación entre:

- Gravedad y daño evitable.
- Probabilidad en la población y duración de misión consideradas.
- Posibilidad de intervenir con las cuatro funciones definidas.
- Capacidad de reconocer el evento con monitoreo no invasivo y síntomas observables.
- Tiempo disponible para actuar.
- Dificultad, masa, volumen, consumibles y consumo del hardware.
- Error diagnóstico, falsas alarmas, eventos no detectados y daño potencial de una intervención incorrecta.
- Posibilidad de que un pasajero con formación básica ejecute la respuesta.

Fuente inicial del equipo:
https://ntrs.nasa.gov/api/citations/20220003136/downloads/NASA-T_1.pdf

Leer el documento, su metodología, escenarios y apéndices. No confundir su priorización de capacidades/Exclusion Score con un ranking de mortalidad o incidencia. No trasladar probabilidades de cuatro astronautas a quince turistas ni modificar la duración de misión sin explicarlo. Si faltan datos de población o duración, declarar escenarios comparables, incertidumbre y análisis de sensibilidad.

Investigar inicialmente un conjunto amplio —orientativamente 8–12 síndromes/eventos agudos— y seleccionar un núcleo recomendado según los resultados, orientativamente 3–5. Esas cantidades son una guía de presentación, no un requisito que obligue a incluir casos débiles. Diferenciar evento reconocible, diagnóstico confirmado y causa subyacente.

Para cada candidato entregar:

| Campo | Contenido requerido |
|---|---|
| Evento/síndrome | Qué situación aguda se pretende reconocer y atender |
| Valor clínico | Por qué atenderlo puede evitar daño grave |
| Señales necesarias | Qué datos son realmente necesarios y cuándo |
| Corroboración | Señales independientes, síntomas y comprobaciones |
| Ubicación | Qué se mide en brazalete y qué en arnés |
| Medición real | Sensor, principio físico, límites y calidad de señal |
| Interpretación | Qué puede inferirse y qué no puede concluirse |
| Respuesta | Descarga, procedimiento, inyectable y/o registro |
| Automatización | Qué hace el sistema y qué necesita un humano |
| Brechas | Qué impide una atención completa con el alcance actual |
| Coste técnico | Energía, masa, volumen, consumibles y complejidad |

El equipo quiere **datos cruzados**, preferentemente varios. No imponer tres o cuatro números por caso si no tienen valor clínico. Distinguir señales independientes de métricas derivadas del mismo sensor: pulso y variabilidad de pulso no son dos confirmaciones independientes. Evitar tanto el diagnóstico con un único dato inespecífico como demorar una intervención validada esperando mediciones innecesarias.

Hacer una clasificación multicriterio con criterios y pesos explícitos, evidencia y sensibilidad a pesos alternativos. No fabricar un score médico validado ni presentar preferencias de ingeniería como recomendaciones clínicas establecidas.

## 7. Desfibrilación: alcance y límites

Investigar un **subsistema de desfibrilación externo/vestible** con análisis de ritmo e indicaciones validadas. No presentar una banda de ECG como desfibrilador.

- Inconsciencia o ausencia de respiración normal no son por sí solas autorización para descargar.
- Distinguir ritmos desfibrilables de no desfibrilables y desfibrilación de otras intervenciones.
- Analizar autonomía, advertencias, protección del auxiliador, contacto de electrodos, artefactos, cancelación por paciente consciente cuando corresponda y continuidad tras la intervención.
- Comparar equipos/subsistemas existentes y las adaptaciones para arnés, microgravedad, cabina, compatibilidad electromagnética y electrónica cercana.
- La confirmación humana no debe sustituir al análisis médico del dispositivo; evaluar su aporte y el retraso que introduce.
- Entregar arquitectura funcional y evidencia. No hace falta un circuito de alta tensión, instrucciones de fabricación ni parámetros clínicos para aplicar descargas.

## 8. Inyectables: comparar alternativas de verdad

No asumir “inyección intravenosa automática” como solución inicial. Comparar, según la indicación clínica y el fármaco, autoinyector, administración intramuscular, subcutánea u otras vías pertinentes. Tratar las opciones intravenosas y las tecnologías de acceso automático como desarrollos de complejidad específica. Podés comparar una alternativa no inyectable si cubre el mismo objetivo con evidencia, identificando ese cambio.

Responder para cada opción:

- Qué emergencia concreta y qué tratamiento justificarían esa vía.
- Qué evidencia y nivel de madurez existen.
- Quién lo administra: paciente, compañero o mecanismo automatizado.
- Si exige acceder a una vena o identificar un sitio anatómico y cómo limita eso su uso.
- Qué ocurre si la persona está inconsciente, se mueve, no puede colaborar o no se alcanza el lugar correcto.
- Qué problemas permanecen en microgravedad.
- Cómo se evita una selección errónea, repetición indebida o mezcla de pacientes.
- Disponibilidad, esterilidad, almacenamiento, conservación durante la misión, caducidad, consumibles y residuos punzantes.
- Qué sensores no invasivos pueden fundamentar la sospecha y cuándo son insuficientes para una administración autónoma.
- Qué parte podría comprarse y qué parte exige investigación/desarrollo y validación.

No plantear insulina como tratamiento genérico de un desmayo ni usar un supuesto valor de glucosa de un reloj no validado para justificarla. No se requieren dosis personalizadas, instrucciones de inyección ni un diseño listo para tratar personas.

## 9. Selección de sensores y cantidad óptima

Después de priorizar casos, construir una matriz **casos × señales × sensores**. Buscar reutilización de un sensor entre varios casos, sin convertirlo en un sensor universal ni duplicar métricas como evidencia independiente.

Proponer:

- Conjunto mínimo que cubre el núcleo seleccionado.
- Sensores adicionales con beneficio incremental claro.
- Componentes descartados y motivo.
- Número de sensores físicos, puntos de contacto y canales. No confundir un chip multicanal con varios sensores físicos.
- Ubicación y condiciones necesarias de contacto, inmovilidad, calibración, higiene y reposición.
- Qué es medición directa, estimación algorítmica o dato introducido por una persona.
- Limitaciones por movimiento, contacto, pigmentación/perfusión cuando aplique, microgravedad y fisiología espacial.
- Necesidad de manguito, emisor óptico, electrodos, transductores o accesorios que no estén incluidos en el chip anunciado.

No dar por resueltas presión arterial sin manguito, temperatura central mediante temperatura cutánea, glucosa no invasiva, diagnóstico de infarto con ECG limitado ni detección general de infecciones solo con signos vitales. Verificar cada afirmación según su uso pretendido.

## 10. Componentes de vuelo: evidencia que se exige

Queremos modelos concretos, comprables y con datos técnicos. **No inventes certificaciones espaciales, disponibilidad ni consumo para completar tablas.**

Clasificar cada componente como:

1. Calificación/evidencia espacial documentada, indicando exactamente su alcance.
2. Antecedente de vuelo verificable, indicando misión, configuración y condiciones; no equivale automáticamente a calificación para Marte.
3. Componente comercial candidato sin calificación espacial demostrada, con plan de evaluación y brechas.
4. Tecnología experimental/no adquirible o desarrollo a medida.

No existe una etiqueta universal “certificado para el espacio” suficiente para esta misión. Informar documento, norma/ensayo, entidad, fecha y límites. ISO 9001/13485, CE, FCC, RoHS, grado automotriz o rango térmico no equivalen a calificación espacial ni validación de la función médica.

Si no hay un modelo espacial comprable que cumpla una función, **declarar la ausencia** y comparar candidatos comerciales o alternativas. Un equipo médico validado en Tierra no queda validado en microgravedad ni en tránsito interplanetario.

Para cada pieza seleccionada entregar:

| Campo | Detalle |
|---|---|
| Función y ubicación | Qué hace, dónde va y qué casos habilita |
| Fabricante y MPN | Modelo exacto, versión y variante |
| Tipo de compra | Chip, sensor completo, módulo, equipo o desarrollo a medida |
| Estado de ciclo de vida | Activo/NRND/EOL/descontinuado, con fecha de comprobación |
| Adquisición | Fabricante/distribuidor autorizado; stock, consulta o plazo real |
| Fuentes | Página oficial, datasheet y documento de calificación/antecedente |
| Alimentación | Rieles de tensión y circuitos adicionales necesarios |
| Potencia | Modos activos, espera, sueño, máximos/picos disponibles y condiciones |
| Medidas físicas | Dimensiones, masa y accesorios necesarios |
| Interfaz | Bus, velocidad, compatibilidad con controlador y firmware |
| Desempeño | Rango, exactitud, muestreo, latencia, calibración y limitaciones |
| Aptitud de misión | Radiación, térmico, mecánico, materiales, EMC y brechas |
| Alternativa | Un sustituto razonable y sus compromisos |

Marcar “no publicado” o “por medir” si falta algo. No usar stock agregado o una tienda de repuestos como evidencia de pieza original y trazable. No recomendar como compra una referencia descontinuada sin explicarlo.

## 11. Investigar el brazalete y arnés completos

Además de sensores médicos y radio, contemplar todos los elementos necesarios:

- Pantalla y táctil, controlador/puente y memoria gráfica.
- Procesador local, memoria, watchdog, arranque y recuperación.
- Micrófono: modelo, alimentación, consumo, interfaz, inteligibilidad, ruido de cabina, distancia de uso y necesidad o no de arreglo de micrófonos.
- Altavoz, amplificador, vibrador y controladores correspondientes.
- Botón físico de ayuda y método sencillo de confirmación/repetición.
- Radio LoRa, antena, adaptación e integración; justificar cualquier módulo adicional de localización o enlace.
- Enlace brazalete–arnés: tecnología, confiabilidad, latencia, consumo y fallas.
- Almacenamiento local para bitácora e instrucciones gráficas offline.
- Batería, protección, medidor de carga, reguladores, carga/conectores y reserva de emergencia.
- Gestión de potencia y separación de alimentación de comunicaciones, sensores y tratamiento cuando corresponda.
- Electrodos/contactos, textiles, soportes, carcasa, correas, aislamiento y consumibles.
- Subsistema de desfibrilación y opciones de administración terapéutica.

Para la pantalla, buscar prioritariamente opciones flexibles táctiles y comparar las mejores candidatas con una o más opciones planas competitivas. La dificultad de fabricar un prototipo de hackathon no es motivo para descartar una opción: este encargo apunta al sistema de misión. La recomendación debe favorecer una flexible que cumpla lo requerido; si no hay evidencia suficiente, distinguir preferencia de diseño y selección pendiente de validación. Especificar área activa y tamaño total, táctil, resolución, brillo en cd/m²/nits, reflejos, consumo por contenido/brillo/modo, ciclo de vida, radio de curvatura y táctil bajo curvatura si aplica. No asumir que un cable FPC flexible significa que el panel se dobla. Verificar guantes si se proponen; no son un requisito espacial exterior actual.

Para justificar la elección de cualquier componente, distinguir:

- Uso documentado de **ese modelo exacto** en un dispositivo conocido.
- Experiencia y certificaciones del **fabricante**.
- Uso extendido de una **tecnología** similar.

No afirmar que una pantalla es “la del Samsung/iPhone” por compartir AMOLED, ni que una marca está avalada por NASA por vender productos compatibles con Raspberry Pi.

## 12. Presupuesto energético completo

Separar brazalete, arnés y hub. Entregar potencia por modo y energía por perfil de uso:

- Espera/monitoreo continuo.
- Comunicación normal y retransmisión mesh.
- Consulta e instrucciones gráficas.
- Emergencia, incluyendo comunicaciones, sonido y tratamiento.
- Pérdida de enlace y reintentos.
- Sueño y carga cuando corresponda.

Distinguir valor de datasheet, medición publicada y cálculo propio. Indicar tensión, corriente, contenido de pantalla, potencia RF, tasa de muestreo y demás condiciones. No comparar el consumo de un chip desnudo con una placa HDMI completa sin advertirlo.

Usar P = V × I y E = P × t cuando corresponda; sumar rieles y pérdidas explícitamente. Informar W/mW para potencia y Wh para energía. No convertir mAh a autonomía sin tensión, eficiencia y perfil temporal. No usar la potencia de la fuente recomendada como consumo real.

Para desfibrilación, distinguir almacenamiento de energía/potencia de pulso, carga del subsistema y energía consumida de la batería. No mezclar un pico terapéutico con consumo continuo. Evaluar reserva de emergencia, envejecimiento, temperatura y márgenes sin inventar requisitos numéricos no justificados.

Si el hub es fijo y tiene alimentación de la nave, aclarar esa condición; no trasladar su consumo al brazalete ni ignorarlo en el total del sistema.

## 13. IA médica y decisiones

La IA principal estará en el hub, offline. Proponer funciones y, si corresponde, modelos concretos con licencia, disponibilidad, requisitos de cómputo y evidencia pertinente. Diferenciar clasificación de señales, motor de protocolos, interfaz conversacional y modelo generativo.

No equiparar una buena nota en un examen médico con validación clínica. No inferir autorización terapéutica de una publicación académica. Explicar qué decisiones dependen de protocolos o subsistemas específicos y qué uso tendría un modelo de lenguaje.

Definir datos suficientes, incertidumbre, detección de información insuficiente, comprobación de calidad de señal, intervención humana, respaldo local ante fallas y trazabilidad. Evitar que el chat genere libremente instrucciones de tratamiento no verificadas.

## 14. Método y fuentes

Usar investigación web actualizada. Priorizar NASA/NTRS, ESA y otras agencias, publicaciones originales, reguladores, fabricantes, datasheets y distribuidores autorizados. Leer los documentos, no solo snippets de buscadores. Identificar fecha de consulta y versión de cada ficha.

Citar cada afirmación importante de capacidad médica, consumo, disponibilidad, calificación y antecedentes. Incluir enlace directo y sección/página. Señalar conflictos entre versiones. No presentar una cifra comercial sin condiciones como precisión demostrada ni una tecnología en animales como tratamiento humano listo.

Si no tenés navegación, acceso a una ficha o capacidad para verificar disponibilidad, decilo. Separar lo que podés concluir de lo que queda pendiente; no reemplazar evidencia faltante por memoria o por afirmaciones plausibles.

No detener todo por datos que el equipo todavía no definió. Declarar supuestos, analizar escenarios y destacar las pocas decisiones que realmente cambian la selección. Preguntar únicamente si una ambigüedad impide avanzar con sentido.

## 15. Entregables solicitados y orden de trabajo

Trabajar en este orden: requisitos → casos clínicos → datos/señales → sensores → respuestas → arquitectura → componentes → energía → brechas. Se puede iterar cuando aparezcan restricciones reales.

### A. Resumen ejecutivo

Máximo una página: núcleo de casos recomendado, sensores mínimos, arquitectura propuesta, principales compromisos y decisiones pendientes. Diferenciar seleccionado, candidato y no viable actualmente.

### B. Matriz de casos y señales

Candidatos evaluados, priorización explicada, casos seleccionados/descartados, mediciones cruzadas y función de cada dato. Incluir límites de diagnóstico y qué atención no puede proporcionar el sistema.

### C. Arquitectura funcional

Distribución entre brazalete, arnés y hub. Comunicación, IA central, lógica local de emergencia, fallas y dos flujos de usuario. No añadir estación médica compartida.

### D. Lista de componentes

Entregar primero la **lista principal recomendada**, con un modelo concreto por función, y después las comparativas que justifican cada elección. Tabla completa con MPN, compra/ciclo de vida, fichas, potencia por modo, dimensiones, interfaces, evidencia espacial/médica y alternativas. Diferenciar pieza adquirible de subsistema que requiere desarrollo.

Para cada componente decisivo, indicar: requisitos que cumple, resultado frente a competidores, razón de selección, compromisos aceptados, nivel de confianza y dato/ensayo pendiente. Cerrar con una configuración integrada del brazalete, arnés y hub y una alternativa de respaldo. Para la pantalla, declarar expresamente si la flexible ganó o perdió y por qué.

### E. Comparación terapéutica

Desfibrilación y alternativas de inyectable, función humana/autónoma, madurez, integración, consumibles, límites y validaciones pendientes. No hace falta un manual para intervenir sobre personas.

### F. Energía y comunicación

Cálculos auditables con supuestos, tráfico y latencias; batería/reserva propuestas. No dar por hecho que una mesh LoRa puede transportar todas las señales crudas o satisfacer los tiempos de una intervención crítica.

### G. Evidencia y decisiones

Registro de fuentes con enlaces y páginas; separar hechos, cálculos y propuestas. Cerrar con las decisiones que el equipo debe tomar y qué resultado cambiaría cada una.

Si podés crear archivos, entregar informe Markdown y tablas CSV o XLSX, con enlaces a datasheets oficiales. Si no, usar tablas copiables. No dedicar la investigación a hacer imágenes bonitas, código de una PWA o un prototipo de bajo costo: esta tarea es establecer el fundamento médico y de ingeniería.

## 16. Verificación final antes de responder

- ¿Se respetó el monitoreo no invasivo y se separó del tratamiento invasivo de emergencia?
- ¿Hay únicamente brazalete y arnés como dispositivos médicos personales, más hub informático?
- ¿La pantalla es táctil y cumple el lado largo del área activa requerido?
- ¿Se eligieron casos por utilidad y viabilidad, antes de inflar el catálogo de sensores?
- ¿Se distinguieron señales independientes y métricas derivadas?
- ¿Se justificaron LoRa, mesh, localización, tráfico y respaldo local ante fallas?
- ¿Ninguna descarga se autoriza por un dato anormal genérico ni por una respuesta libre de un LLM?
- ¿Las opciones inyectables no presuponen una vena accesible o un diagnóstico no demostrado?
- ¿Se seleccionó la mejor combinación justificada, en vez de entregar solo componentes compatibles o una lista de compras?
- ¿Las comparativas muestran por qué gana cada modelo y qué podría cambiar esa decisión?
- ¿Se investigaron activamente pantallas flexibles táctiles y se respetó su preferencia condicionada al cumplimiento?
- ¿Los componentes exactos tienen fuentes y estado de adquisición verificable?
- ¿Se distinguió certificación, antecedente de vuelo, uso terrestre y candidato sin validar?
- ¿Los consumos incluyen unidades, condiciones, alcance y límites?
- ¿Las conclusiones son claras, resumidas y respaldadas sin prometer viabilidad total donde falta evidencia?
