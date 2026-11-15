# 03-04 · Edge cases: cómo responde Eva

*Claude, domingo 15/11. Versión 2: rehecha con los comentarios del equipo y con el concepto actual (sin estación; brazalete, arnés y hub de la nave). Es una propuesta para Hassi y Naza: lo que el equipo acepte va al Registro.*

## Resumen

Quedan tres casos: paro cardíaco, despresurización de cabina y alerta solar, esta última pendiente de confirmar en el alcance. Salen glucosa (2A), hueso y calcio (2B), salud mental (4) y todo lo que dependía de la estación. Eva guía, registra y avisa; la descarga la decide el análisis de ritmo validado del arnés, nunca un modelo de chat. El inyectable del arnés todavía no tiene caso asignado porque la vía y el fármaco siguen sin decidir.

Reglas que valen para todos los casos:

- Monitoreo no invasivo: nada de agujas ni filamentos para medir. El tratamiento puede ser invasivo solo ante una emergencia mortal.
- Solo dos dispositivos personales, brazalete y arnés, más el hub de la nave, que no es un equipo médico. No hay estación ni bandejas iluminadas: una luz del arnés señala el elemento a usar.
- La descarga solo ocurre con un ritmo desfibrilable detectado por el análisis validado del subsistema del arnés. La IA conversacional no autoriza, no adelanta ni cancela una descarga.
- La primera instrucción de cada emergencia sale de un protocolo local, no de la IA.
- Los tiempos en segundos de este documento son parámetros de la simulación, no umbrales clínicos.

## Casos seleccionados

Qué puede medir WearMeds, qué hace Eva y qué no hace nunca. Los datos de la nave llegan al hub; no son sensores de WearMeds.

| Caso | Qué puede medir (brazalete, arnés, nave) | Qué hace Eva | Qué nunca hace |
|---|---|---|---|
| 1 · Paro cardíaco | Arnés: ECG por electrodos de contacto (lo usa el análisis de ritmo del subsistema de desfibrilación), contacto de los electrodos, respiración por impedancia, movimiento. Brazalete: pulso óptico, movimiento y respuesta al aviso (toque, voz o botón). | Comprueba si la persona responde, asigna un ayudante, le avisa antes de la descarga, guía la RCP paso a paso y registra todo en la bitácora. | Decidir, adelantar o cancelar una descarga desde la IA; descargar por un dato anormal genérico; diagnosticar un infarto con el ECG del arnés. |
| 3A · Despresurización de cabina | Nave (vía hub): presión de cabina y presión parcial de O₂. Brazalete: SpO₂ como tendencia (pierde calidad con frío y movimiento), pulso, respuesta al aviso. Arnés: frecuencia respiratoria. | Replica la alarma de presión de la nave en todos los brazaletes con una sola instrucción (máscara de oxígeno), después pide ayudar a quien no pudo y registra el perfil de presión. | Crear una alarma de despresurización con datos de los wearables; esperar al SpO₂ para avisar; decidir oxígeno o recompresión. |
| 3B · Alerta solar (pendiente de confirmar en el alcance) | Nave (vía hub): monitor de radiación de a bordo. Con enlace: NASA DONKI y NOAA GOES, con 3 a 22 min de demora. Dosímetro personal en el brazalete: opcional, a evaluar en la tarea 02. | Aviso en dos pasos (preparate, al refugio) según el criterio de refugio de la nave; registra el tiempo en refugio de cada pasajero y la dosis de a bordo. | Mandar al refugio solo por una erupción en rayos X; estimar dosis individuales o hablar de síndrome de radiación; interrumpir una reanimación por la alerta. |

## Casos que salen y por qué

- **2A · Glucosa:** medir glucosa exige un sensor con filamento bajo la piel (CGM) y el encargo prohíbe el monitoreo invasivo; no hay glucosa no invasiva validada. El glucómetro y el medidor de cetonas estaban en la estación. Un pasajero que no responde, por esta u otra causa, queda cubierto por el flujo general (E1.1 y T6).
- **2B · Hueso y calcio:** la pérdida ósea es crónica, no una emergencia de minutos; el denosumab tarda días en actuar y puede bajar el calcio de más. No usa ninguna de las cuatro funciones de forma aguda.
- **4 · Salud mental:** no es un evento agudo que el brazalete o el arnés reconozcan con seguridad, y la bitácora es un registro compartido, no un diario personal. Puede volver como línea futura si el equipo la quiere.
- **Lo que dependía de la estación:** bandejas iluminadas, DEA, manguito de presión, oxímetro de dedo, glucómetro, medidor de cetonas y la ruta a la estación más cercana.

## Caso 1 · Paro cardíaco

Formato de cada edge case: situación, qué hace Eva, qué nunca hace y criterio de la simulación. El identificador coincide con `sim/scenarios.json`.

### E1.1 · ECG plano: ¿asistolia o electrodo suelto?

- **Situación:** el ECG del arnés se ve plano.
- **Qué hace Eva:** cruza el contacto de los electrodos del arnés, el pulso del brazalete, el movimiento y la respuesta al aviso. Electrodo suelto con pulso presente: pide al pasajero que ajuste el arnés, sin alarma general. ECG plano, sin pulso y sin respuesta: inicia el flujo de persona que no responde (asigna ayudante y guía la RCP).
- **Qué nunca hace:** alarma general por un solo sensor; tratar la asistolia como ritmo para descargar (no es desfibrilable).
- **Simulación:** cero alarmas generales por electrodo suelto; ayudante asignado dentro del plazo cuando no hay pulso ni respuesta.

### E1.2 · Ejercicio que ensucia el ECG

- **Situación:** en un viaje largo hay ejercicio diario y el movimiento mete ruido en el ECG.
- **Qué hace Eva:** con aceleración alta y pulso regular en el brazalete, no alarma y pide confirmación al pasajero.
- **Qué nunca hace:** alarma general solo por un ECG con ruido; tocar el análisis de ritmo del subsistema, que maneja sus propios artefactos.
- **Simulación:** cero falsas alarmas en 30 minutos de ejercicio.

### E1.3 · Ritmo desfibrilable detectado por el arnés

- **Situación:** el análisis de ritmo validado del subsistema del arnés detecta un ritmo desfibrilable.
- **Qué hace Eva:** si la persona está consciente, el subsistema le da la opción de cancelar, como los chalecos desfibriladores actuales. Eva asigna un ayudante y le avisa "no la toques hasta que termine la descarga"; después guía la RCP según el entrenamiento para microgravedad y registra la hora de cada paso.
- **Qué nunca hace:** la IA no autoriza, no adelanta ni cancela la descarga; no hay descarga por un dato anormal genérico ni por una respuesta del modelo de lenguaje.
- **Simulación:** el ayudante recibe la advertencia antes de la descarga simulada; cero descargas originadas en Eva.

### E1.4 · El ayudante asignado no confirma

- **Situación:** el pasajero más cercano no confirma, o también es paciente.
- **Qué hace Eva:** desde el primer segundo, una persona recibe la tarea con nombre y el resto un aviso informativo. Si no confirma en el plazo, Eva reasigna al siguiente y abre la tarea a todos los cercanos (la toma el primero que acepta); el líder de la tripulación siempre queda avisado. Registra quién aceptó y cuándo.
- **Qué nunca hace:** dejar la emergencia sin un responsable con nombre; mandar a todos a la misma tarea sin coordinar.
- **Simulación:** reasignación dentro del plazo. Variante a comparar: asignar a una persona contra avisar a todos los capacitados (tiempo hasta la primera confirmación y cuántas personas llegan).

### E1.5 · Paro durante una alerta solar

- **Situación:** un pasajero entra en paro mientras rige la orden de ir al refugio.
- **Qué hace Eva:** la vida inmediata manda. La RCP sigue donde está la persona; Eva pide un segundo ayudante solo para relevar las compresiones (las guías recomiendan rotar cada 2 minutos aproximadamente). Si hay que trasladar a la persona al refugio, lo decide la tripulación. Registra la dosis de a bordo durante el evento.
- **Qué nunca hace:** interrumpir la RCP por la alerta; ordenar un traslado durante la reanimación.
- **Simulación:** la RCP no se interrumpe por la alerta; segundo ayudante pedido dentro del plazo.

## Caso 3A · Despresurización de cabina

### E3.1 · Cae la presión de cabina

- **Situación:** el sistema de la nave declara una caída de presión.
- **Qué hace Eva:** la hipoxia quita la capacidad de actuar: el tiempo de conciencia útil es de 3 a 5 minutos a una altitud equivalente de 25.000 pies, menos de un minuto por encima de 35.000, y puede reducirse a la mitad en una descompresión rápida. Eva replica la alarma de la nave en todos los brazaletes con una sola instrucción, "Put your oxygen mask on now", con vibración y sonido. Después: "ayudá a quien no pudo".
- **Qué nunca hace:** esperar al SpO₂, que mide la consecuencia y baja cuando la hipoxia ya empezó; usar un modelo predictivo de oxígeno como disparador.
- **Simulación:** instrucción en menos de 1 s desde la alarma de la nave.

### E3.2 · Un sensor marca una caída que la nave no confirma

- **Situación:** una lectura aislada de presión indica caída, pero la nave no declara alarma.
- **Qué hace Eva:** no difunde la instrucción a todos; avisa a la tripulación para que verifique y registra la lectura. Si la nave declara la alarma, pasa a E3.1.
- **Qué nunca hace:** generar su propia alarma de despresurización con un solo sensor o con datos de los wearables.
- **Simulación:** cero difusiones a todos ante una lectura aislada; aviso a la tripulación dentro del plazo.

### E3.3 · SpO₂ del brazalete poco confiable

- **Situación:** frío, vasoconstricción o movimiento bajan la calidad del SpO₂ del brazalete.
- **Qué hace Eva:** con la cabina normal y la persona respondiendo, pide reacomodar el brazalete, repite la medición y usa la frecuencia respiratoria del arnés y la respuesta al aviso.
- **Qué nunca hace:** alarma general por un SpO₂ de baja calidad.
- **Simulación:** cero alarmas generales con la cabina normal y la persona respondiendo.

### E3.4 · Síntomas después de la despresurización

- **Situación:** horas después aparecen dolor articular o síntomas en la piel.
- **Qué hace Eva:** registra el perfil de presión y tiempo de esa persona, avisa a la tripulación y deja el caso en la bitácora para la Tierra.
- **Qué nunca hace:** decidir oxígeno o recompresión, que son decisiones humanas; afirmar que hay burbujas, que no se miden.
- **Simulación:** aviso a la tripulación y registro dentro del plazo.

## Caso 3B · Alerta solar (pendiente de confirmar)

### E3.5 · Erupción en rayos X sin protones todavía

- **Situación:** DONKI informa una erupción, pero el monitor de radiación de a bordo todavía no sube.
- **Qué hace Eva:** aviso en dos pasos: primero "preparate", después "al refugio" cuando se cumple el criterio de refugio de la nave. La simulación usa como ejemplo el nivel S1 de NOAA (10 pfu con energía de 10 MeV o más); el criterio real lo fija la operadora.
- **Qué nunca hace:** mandar al refugio solo por la erupción en rayos X.
- **Simulación:** no ordena refugio antes del criterio; orden de refugio dentro del plazo cuando se cumple.

### E3.6 · Sin enlace con la Tierra

- **Situación:** conjunción solar (unas dos semanas) o falla de comunicaciones: no llegan DONKI ni GOES.
- **Qué hace Eva:** usa el monitor de radiación de a bordo como disparador. Aun con enlace, GOES mide cerca de la Tierra y la nave puede estar en otra posición respecto del Sol, así que el dato de a bordo manda.
- **Qué nunca hace:** esperar datos de la Tierra para avisar.
- **Simulación:** orden de refugio con el enlace caído; tiempo en refugio registrado por pasajero.

## Edge cases transversales

### T1 · Dos emergencias a la vez

- **Situación:** un paro y otra emergencia en el mismo minuto.
- **Qué hace Eva:** una despresurización se difunde a todos al instante y en paralelo; las demás tareas van en cola por gravedad, con el paro primero. Cada ayudante recibe una sola tarea y el líder de la tripulación ve las dos.
- **Qué nunca hace:** darle dos tareas al mismo ayudante.
- **Simulación:** cero ayudantes con dos tareas; las dos emergencias con ayudante dentro del plazo.

### T2 · El ayudante entra en pánico o no sigue los pasos

- **Situación:** el ayudante confirmó pero no avanza.
- **Qué hace Eva:** instrucciones de a una, con pictograma, texto breve, voz y el botón Repetir; si no avanza en el plazo, suma o pasa a otra persona.
- **Qué nunca hace:** mostrar varios pasos juntos o texto largo durante la emergencia.
- **Simulación:** refuerzo o reasignación dentro del plazo.

### T3 · Batería baja

- **Situación:** el brazalete o el arnés quedan con poca carga.
- **Qué hace Eva:** modo mínimo: baja el brillo de la pantalla y la actividad de la radio, y avisa al pasajero y a la tripulación para cargar. La comprobación de respuesta, el botón de ayuda y el análisis de ritmo del subsistema siguen activos.
- **Qué nunca hace:** apagar una función de rescate para ahorrar energía.
- **Simulación:** modo mínimo dentro del plazo sin desactivar la comprobación de respuesta ni el análisis de ritmo.

### T4 · La IA falla o tarda

- **Situación:** el modelo del hub no responde o tarda.
- **Qué hace Eva:** protocolo primero: la primera instrucción sale del firmware y del motor de protocolos, y la IA solo puede elegir entre pasos del protocolo. El subsistema de desfibrilación funciona sin el hub.
- **Qué nunca hace:** esperar a la IA para dar la primera instrucción; dejar que el chat escriba instrucciones de tratamiento libres.
- **Simulación:** primera instrucción en menos de 1 s con la IA caída.

### T5 · Sin enlace con la Tierra

- **Situación:** días sin comunicación.
- **Qué hace Eva:** todo funciona a bordo; la bitácora y los reportes quedan en cola y se envían al volver el enlace, con la hora real de cada evento.
- **Qué nunca hace:** condicionar una respuesta a la Tierra; perder registros.
- **Simulación:** reporte en cola y enviado al restablecerse el enlace.

### T6 · Falsas alarmas

- **Situación:** una señal sola parece grave.
- **Qué hace Eva:** usa la línea basal personal, pide confirmación al propio pasajero y solo da alarma general con dos señales independientes que coinciden. El pulso y su variabilidad salen del mismo sensor y no cuentan como dos.
- **Qué nunca hace:** alarma general con una sola señal; demorar una intervención validada esperando datos que no hacen falta (la descarga depende del análisis de ritmo, no de esta regla).
- **Simulación:** cero alarmas generales con una sola señal anormal.

### T7 · Falta un elemento o ya se usó

- **Situación:** el inyectable del arnés ya se usó o venció, o un electrodo no hace contacto.
- **Qué hace Eva:** lo detecta en el autochequeo al ponerse el arnés y periódicamente (contacto de electrodos; presencia del inyectable si el hardware lo permite), avisa al pasajero y a la tripulación para reponerlo y lo registra. Si pasa durante una emergencia, dice qué falta y sigue con lo que funciona, por ejemplo la guía de RCP.
- **Qué nunca hace:** indicar el elemento de otro pasajero (riesgo de mezclar pacientes); iluminar un compartimento vacío.
- **Simulación:** faltante avisado en el autochequeo; cero indicaciones hacia elementos de otro pasajero.

### T8 · Pasajero mayor o que no habla inglés

- **Situación:** le cuesta leer, oír o responder rápido, o no entiende inglés.
- **Qué hace Eva:** usa el idioma elegido al embarcar, pictogramas con texto breve en ese idioma, voz y Repetir siempre visible; acepta respuesta por voz, toque o botón.
- **Qué nunca hace:** suponer que un dibujo sin texto se entiende siempre; dar instrucciones solo en inglés.
- **Simulación:** instrucciones en el idioma del perfil; respuesta por voz aceptada.

### T9 · La microgravedad cambia la línea basal

- **Situación:** en los primeros días los fluidos se redistribuyen hacia la cabeza y cambian los valores de reposo.
- **Qué hace Eva:** recalibra la línea basal en los primeros días de vuelo y lo marca en la bitácora; mientras tanto, los avisos no urgentes usan criterios más conservadores.
- **Qué nunca hace:** esperar la basal para responder a una señal de vida o muerte (sin pulso y sin respuesta).
- **Simulación:** cero alarmas por desvíos de basal en la adaptación; respuesta normal a un paro en ese período.

### T10 · Falla LoRa o el hub

- **Situación:** el brazalete pierde el enlace con el hub, o el hub cae.
- **Qué hace Eva:** el brazalete y el arnés mantienen la lógica local de emergencia: el subsistema de desfibrilación funciona solo, el brazalete hace sonar una alarma local para llamar a quien esté cerca, muestra instrucciones precargadas y guarda la bitácora para sincronizarla después. Cada mensaje de emergencia pide confirmación y se reintenta.
- **Qué nunca hace:** dar por entregado un mensaje sin confirmación; dar por resuelta la malla LoRa, que sigue en evaluación (tarea 09).
- **Simulación:** alarma local en el brazalete del paciente dentro del plazo con el hub caído; bitácora sincronizada al volver.

### T11 · Encontrar al pasajero más cercano

- **Situación:** Eva necesita saber quién está más cerca, y la intensidad de señal de LoRa (RSSI) no es una distancia confiable dentro de una nave metálica.
- **Qué hace Eva:** usa una ubicación por compartimento (método a definir en la tarea 09) y le pide al asignado que confirme "voy"; si responde que no está cerca, reasigna.
- **Qué nunca hace:** tratar el RSSI como distancia.
- **Simulación:** el asignado responde "no estoy cerca" y Eva reasigna dentro del plazo.

### T12 · El traje queda entre las dos partes del brazalete

- **Qué pasa:** el pasajero se pone el traje con el módulo de sensores en la piel y la interfaz sobre la manga. Los imanes sostienen, pero los pines no tocan.
- **Riesgo:** los sensores quedan con su batería chica y sin pines hacia la interfaz.
- **Cómo responde Eva:** la interfaz detecta que no hay contacto en los pines y pasa a modo traje. Los sensores bajan la frecuencia de muestreo a lo mínimo que mantiene las alarmas y siguen mandando los datos por LoRa, como cuando están unidas (las dos partes tienen LoRa). La pantalla muestra la carga de la batería chica y el tiempo estimado que queda. Con poca carga, avisa con tiempo para volver a unir las partes.
- **Para validar:** autonomía real de la batería chica con LoRa transmitiendo a potencia mínima.

### T13 · Las partes se separan o la interfaz queda lejos

- **Qué pasa:** la interfaz se suelta (golpe, maniobra) o el pasajero se la saca y el módulo de sensores sigue en la piel.
- **Cómo responde Eva:** el módulo de sensores sigue midiendo con su batería y guarda los datos. Como tiene LoRa propio, sigue mandando sus datos al hub aunque la interfaz no esté. Si la interfaz no responde por más de unos segundos, el hub avisa al pasajero y a la tripulación. Una alarma crítica detectada sin interfaz se manda por la malla desde el hub y, si hay, por la pulsera del compañero más cercano.

### T14 · Mala señal óptica en la muñeca

- **Qué pasa:** el sensado multionda funciona mejor en el dedo (por transmisión). En la muñeca se mide por reflexión, con más ruido por movimiento, frío o poca perfusión.
- **Cómo responde Eva:** cada medición viaja con un índice de calidad de señal. Con calidad baja, Eva no dispara alarmas con ese dato solo: lo cruza con el ECG del arnés y los datos de la nave, y pide quedarse quieto o ajustar la banda.

### T15 · ECG de muñeca: pedirlo bien y saber cuándo no sirve

- **Qué pasa:** Eva quiere un registro de ECG de una derivación (por ejemplo, el pulso óptico se volvió irregular) y la persona tiene que tocar el electrodo exterior con un dedo de la otra mano, quieta, unos 30 segundos.
- **Cómo responde Eva:** lo pide solo si la persona está consciente y puede colaborar, con una instrucción por paso y una cuenta regresiva. Si el contacto es malo o hay movimiento, lo dice y repite. El registro va al hub con su índice de calidad. Si la persona no responde, usa los electrodos del arnés. Con el traje entre las dos partes, el ECG de muñeca no está disponible y la pantalla lo dice.
- **Límite:** una derivación no reemplaza un ECG de 12 ni descarta un infarto, y nunca es la única señal para el desfibrilador.

## La simulación de desarrollo

- Los escenarios están en `sim/scenarios.json`, en la rama `fase2/03-04-casos-variables`. Cada uno trae las señales en el tiempo, lo que Eva tiene que hacer y antes de cuándo, y lo que no puede hacer.
- La app en Rust mide cuatro cosas: primera instrucción (menos de 1 s), ayudante asignado (menos de 10 s, salvo que el escenario pida confirmación antes), acciones prohibidas (cero) y bitácora completa.
- Salieron los escenarios de 2A, 2B y 4 y las acciones que usaban la estación (iluminar bandejas, rutas a estaciones). Entraron los escenarios de E3.2, E3.4, T2, T5 a T11 y la variante de asignación de E1.4.
- Para el pitch: correr E1.3, E3.1 y T1 en vivo.

## Pendientes para Hassi y Naza

- Elegir el caso clínico del inyectable del arnés (vía y fármaco sin decidir).
- Confirmar si la alerta solar sigue en el alcance.
- Definir la técnica de RCP en microgravedad que entra en el entrenamiento y en la guía.
- Elegir entre las dos estrategias de asignación de E1.4 con los resultados de la simulación.

## Fuentes

- Desfibrilador vestible, avisos y cancelación por el paciente: [ZOLL LifeVest, tarjeta para emergencias](https://lifevest.zoll.com/-/media/lifevest-zoll-com/medical-professionals/first-responder-resources/lifevest_ems_pocketcard.ashx) · [Revisión en PMC](https://pmc.ncbi.nlm.nih.gov/articles/PMC5089470)
- RCP, ritmos desfibrilables y relevo de compresiones: [AHA, guías de RCP y atención cardiovascular de emergencia](https://cpr.heart.org/en/resuscitation-science/cpr-and-ecc-guidelines)
- Tiempo de conciencia útil: [FAA, hipoxia](https://www.faa.gov/sites/faa.gov/files/about/office_org/headquarters_offices/avs/II.1.2_Hypoxia.doc) · [FAA AC 61-107B](https://www.faa.gov/documentLibrary/media/Advisory_Circular/AC_61-107B_CHG_1.pdf)
- Escala de tormentas de radiación S1: [NOAA SWPC, Space Weather Scales](https://www.swpc.noaa.gov/noaa-scales-explanation)
- Asignar la tarea con nombre (difusión de responsabilidad): Darley y Latané (1968), Journal of Personality and Social Psychology 8(4), 377-383.
- Alcance (sin estación, monitoreo no invasivo, sin filamentos subcutáneos): [Encargo de investigación médica y arquitectura de Eva](https://docs.google.com/document/d/1pSradgUdKCN5O2YeCmGXNlE20-SyGAsuZZhEqzMcr90/edit)

## Respuestas a los comentarios

Una entrada por comentario del documento anterior, en el orden en que aparecían. Los hilos siguen abiertos en Drive y se resuelven a mano.

- **Reacción sin texto, sobre "Eva coordina a las personas… lleva a la estación":** ya estaba resuelta. Respuesta: el párrafo se reescribió sin la estación. **Sin acción pendiente.**
- **Leandro, sobre "a demanda (estación)":** "La estación está en discusión, cambiarla a ser más específica para los módulos de desfibrilación únicamente". Respuesta: el encargo eliminó la estación y la desfibrilación quedó en el arnés como subsistema validado. El nivel "a demanda" desaparece; la tabla muestra solo brazalete, arnés y datos de la nave. **Aplicado.**
- **Tomás, sobre "Nunca: decidir una descarga":** "No tiene sentido". Respuesta: es cierto que el producto descarga, porque el arnés tiene desfibrilación. Lo que importa es quién decide: el análisis de ritmo validado del subsistema, no la IA ni el chat. Se reescribió así en E1.3 y en la tabla. **Aplicado en parte.**
- **Tomás, sobre "Eva pasa al siguiente y después al líder de la tripulación":** evaluar avisar a todos los capacitados en vez de a uno. Respuesta: todos reciben un aviso informativo desde el primer segundo, pero la tarea va a una persona con nombre para que haya un responsable claro (cuando la tarea es de todos, cada uno tiende a esperar a otro). Si no confirma, se abre a todos los cercanos. Las dos estrategias quedan como variante a comparar en la simulación (E1.4). **Aplicado en parte.**
- **Tomás, sobre "con dos ayudantes":** "¿Por qué dos?". Respuesta: eran para trasladar a la persona al refugio sin cortar la RCP, pero trasladar durante la reanimación no tiene sentido si la exposición de un evento solar se acumula en horas. Ahora la RCP sigue donde está, el segundo ayudante solo releva las compresiones y el traslado lo decide la tripulación (E1.5). **Aplicado.**
- **Tomás, sobre E2.1 (hipoglucemia falsa de noche):** "Al pedo integrarlo". Respuesta: de acuerdo, y además el CGM usa un filamento bajo la piel, que el encargo prohíbe para medir. Sale el caso 2A completo. **Aplicado.**
- **Tomás, sobre E2.2 (hipoglucemia real con confusión):** "Al pedo". Respuesta: sale con el caso 2A; el glucagón iluminado en la estación ya no existe. Un pasajero que no responde, por cualquier causa, queda cubierto por E1.1 y T6. **Aplicado.**
- **Leandro, sobre la introducción del caso 2B:** "Es totalmente innecesario y genera un problema futuro". Respuesta: sale el caso 2B completo, con sus tres edge cases y su escenario. **Aplicado.**
- **Leandro, sobre "Eva no espera al SpO₂, que llega tarde":** "¿Se tendría que mejorar el modelo predictivo de oxígeno para aumentar la velocidad del dato?". Respuesta: el dato más rápido es el sensor de presión de la nave, que mide la causa; el SpO₂ mide la consecuencia, cuando la hipoxia ya empezó. Un modelo predictivo no le gana a la medición directa y suma falsas alarmas. **No aplicado, porque el disparador correcto ya es la presión de la nave.**
- **Leandro, sobre "Put your oxygen mask on now":** "Puede provocar falsos positivos". Respuesta: Eva ya no dispara con datos propios: replica la alarma de la nave, y una lectura aislada solo avisa a la tripulación. Se agregó E3.2 con su escenario. **Aplicado.**
- **Joaquín, sobre los antiguos E3.6 y E3.7 (vómito tras la exposición, emergencia en el refugio):** "¿De más?". Respuesta: sí. El antiguo E3.6 era estimar dosis por la hora del vómito, un triage clínico que Eva no debe hacer y que se confunde con el mareo espacial. E3.7 dependía de los equipos de la estación; ahora cada pasajero lleva su brazalete y su arnés al refugio. Salen los dos. **Aplicado.**
- **Joaquín, sobre E4.1 (diario privado):** "El diario no es tan diario, es más bien bitácora; no tiene por qué ser privada ni personal". Respuesta: de acuerdo. La bitácora es la cuarta función y se comparte con la tripulación y la Tierra. Sale el caso del diario privado y, con él, el caso 4 queda fuera del núcleo. **Aplicado.**
