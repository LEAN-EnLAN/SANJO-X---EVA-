# 03-04 · Edge cases: cómo responde Eva

*Claude, sábado 14/11. Respuesta al documento "Variables médicas por casos" (Hassi y Naza) para la simulación de desarrollo. Es una propuesta: lo que el equipo acepte va al Registro.*

## Tres decisiones que el documento obliga a tomar

1. **Eva no descarga ni inyecta.** El documento pide desfibrilación en el traje e inyección automática de insulina, glucagón, calcitonina y denosumab. Si Eva decide un tratamiento, deja de ser "guides, records and warns", pasa a ser un dispositivo médico de máximo riesgo regulatorio y el médico del jurado lo va a cuestionar. **Propuesta: módulos certificados de terceros.** Un desfibrilador portátil (tipo chaleco cardioversor) o el sistema de insulina de lazo cerrado del propio pasajero deciden su tratamiento con su algoritmo certificado y con receta. Eva coordina a las personas: avisa, guía al ayudante, lleva a la estación y registra todo. Para inversores es una ventaja: los módulos son una línea de producto nueva sobre la misma plataforma.
2. **Los pasajeros no usan traje.** En un vuelo comercial nadie sale a caminar al espacio. El caso 3A pasa a ser **despresurización de cabina**. El traje queda para la tripulación, fuera de WearMeds v1.
3. **No todo se mide con un wearable.** Cada variable va a uno de tres niveles: **continuo** (banda y pulsera), **a demanda** (estación) o **laboratorio** (kit de la tripulación o análisis en Tierra). Lo que no se puede medir no se promete.

## Qué mide WearMeds en cada caso

| Caso | Continuo (banda y pulsera) | A demanda (estación) | Laboratorio o no factible en v1 |
|---|---|---|---|
| 1 · Paro cardíaco | ECG y ritmo, frecuencia cardíaca, respiración e impedancia torácica (banda), pulso periférico y SpO₂ (pulsera), movimiento y postura, temperatura de piel, respuesta al aviso | Presión arterial (manguito), temperatura, SpO₂ de dedo, DEA | Presión arterial media continua, temperatura central continua (solo estimada), capnografía (solo con el kit de vía aérea de la tripulación) |
| 2A · Glucosa | Frecuencia cardíaca, respiración, temperatura de piel; glucosa y tendencia desde el sensor CGM del pasajero (integración) | Glucómetro de confirmación, medidor de cetonas en sangre capilar, glucagón de emergencia del pasajero | pH, lactato, potasio, sodio, osmolaridad, cortisol y catecolaminas continuos |
| 2B · Hueso y calcio | Actividad y ejercicio, frecuencia cardíaca, QT aproximado en el ECG de la banda | Presión arterial, tira de orina, registro de hidratación | Calcio iónico y total, fósforo, magnesio, PTH, vitamina D, CTX/NTX, osteocalcina |
| 3A · Despresurización de cabina | SpO₂, frecuencia cardíaca y respiratoria, respuesta al aviso; presión de cabina, ppO₂ y ppCO₂ desde los sensores de la nave | SpO₂ de dedo, temperatura | Microburbujas de nitrógeno (requiere ecografía Doppler), consumo de O₂ por persona |
| 3B · Radiación | Dosímetro personal activo (propuesta de hardware para la pulsera), frecuencia cardíaca, temperatura; NASA DONKI y NOAA GOES | Temperatura, registro de náuseas y vómitos | Hemograma, marcadores de daño en el ADN, amilasa, citoquinas |
| 4 · Salud mental | Frecuencia cardíaca en reposo, HRV, sueño, temperatura de piel, actividad electrodérmica (si el hardware la tiene); autorreporte y diario | — | Serotonina y dopamina (no se miden en la piel), melatonina continua; cortisol en sudor es experimental |

## Edge cases por caso

Formato: **situación** → qué hace Eva → qué no hace nunca → qué mide la simulación.

### Caso 1 · Paro cardíaco

- **E1.1 ECG plano: ¿asistolia o electrodo suelto?** Eva cruza la impedancia del electrodo, el pulso de la pulsera, el movimiento y la respuesta al aviso. Electrodo suelto con pulso presente: le pide al pasajero que ajuste la banda, sin alarma general. ECG plano, sin pulso y sin respuesta en 15 s: dispara el caso 01. *Simulación: cero alarmas generales por electrodo suelto.*
- **E1.2 Ejercicio que parece arritmia.** En un viaje largo hay ejercicio diario y el movimiento ensucia el ECG. Con aceleración alta y pulso regular en la pulsera, Eva no alarma y pide confirmación al pasajero. *Simulación: cero falsas alarmas en 30 minutos de ejercicio.*
- **E1.3 Fibrilación ventricular con módulo desfibrilador.** El dispositivo certificado avisa y, si el paciente no lo cancela, descarga entre 25 y 60 s después de detectar el ritmo. Eva guía al ayudante: "no toques a la persona hasta que termine la descarga", después RCP adaptada a microgravedad (fijar al paciente a una superficie) y trae el DEA de la estación si no hay módulo. **Nunca:** decidir una descarga. *Simulación: el ayudante recibe la advertencia antes de la descarga.*
- **E1.4 El más cercano no puede ayudar.** Si es también paciente o no confirma en 20 s, Eva pasa al siguiente y después al líder de la tripulación. Registra quién aceptó y cuándo. *Simulación: reasignación en menos de 25 s.*
- **E1.5 Paro en plena tormenta solar.** La vida inmediata manda: Eva mantiene la RCP y propone trasladar al paciente al refugio con dos ayudantes. La dosis de un evento de partículas se acumula en horas, así que unos minutos fuera del refugio pesan poco. Registra la dosis de los tres. *Simulación: la RCP no se interrumpe por la alerta.*

### Caso 2A · Glucosa

- **E2.1 Hipoglucemia falsa de noche.** Dormir sobre el sensor CGM da lecturas bajas falsas. Eva despierta al pasajero; si responde con coherencia, pide confirmar con el glucómetro de la estación antes de cualquier acción. *Simulación: confirmación pedida en el 100% de las lecturas bajas con el pasajero lúcido.*
- **E2.2 Hipoglucemia real y el pasajero confundido.** Pasa a caso 01: el ayudante lleva azúcar oral si la persona puede tragar, o el glucagón de emergencia del pasajero, que se ilumina en la estación. **Nunca:** inyectar; Eva guía a una persona a usar el dispositivo del pasajero.
- **E2.3 Retraso del sensor.** La glucosa del líquido intersticial llega con 5 a 20 minutos de retraso respecto de la sangre, y más en cambios rápidos. Eva usa la tendencia, no solo el valor: una caída rápida adelanta el aviso. *Simulación: aviso antes del umbral cuando la tendencia es de caída rápida.*
- **E2.4 Falla el sensor del lazo cerrado.** Eva detecta datos ausentes o incoherentes, avisa al pasajero y a la tripulación que el sistema quedó en modo manual. **Nunca:** controlar la bomba.
- **E2.5 Sospecha de cetoacidosis** (glucosa alta sostenida, náuseas, respiración rápida). Medidor de cetonas de la estación; si da alto, avisa a la tripulación y a la Tierra y registra. **Nunca:** sugerir dosis de insulina.

### Caso 2B · Hueso y calcio (reformulado)

La pérdida ósea en el espacio es crónica, del orden de 1 a 1,5% por mes, no una emergencia de minutos. El riesgo agudo real es el **cálculo renal** por el calcio que sale del hueso. **Proponemos sacar la inyección automática:** el denosumab tarda unos 9 a 10 días en bajar el calcio y se aplica en semanas, no sirve en una emergencia y puede provocar el problema contrario (calcio bajo).

- **E2B.1 Dolor de cólico renal** (dolor lumbar intenso, náuseas, pulso alto). Eva guía, registra y avisa; el analgésico del kit lo decide la tripulación.
- **E2B.2 Prevención.** Recordatorios de hidratación y ejercicio; aviso si la hidratación registrada baja varios días.
- **E2B.3 QT largo en el ECG** (puede venir de electrolitos o fármacos). Aviso discreto a la tripulación, no al pasajero.

### Caso 3A · Despresurización de cabina

- **E3.1 Cae la presión.** La hipoxia quita la capacidad de actuar: el tiempo de conciencia útil es de 3 a 5 minutos a una altitud equivalente de 25.000 pies, menos de un minuto por encima de 35.000 y la mitad en una descompresión rápida. Eva no espera al SpO₂, que llega tarde: dispara con el sensor de presión de la nave y manda una sola instrucción a todas las pulseras, "Put your oxygen mask on now", con vibración. Recién después: "ayudá a quien no pudo". *Simulación: instrucción en menos de 1 s desde la caída de presión.*
- **E3.2 SpO₂ poco confiable** (frío, vasoconstricción). Eva usa la frecuencia respiratoria, la respuesta al aviso y la presión de cabina; el SpO₂ de dedo de la estación confirma después.
- **E3.3 Síntomas de descompresión después** (dolor articular, piel). Eva registra el perfil de presión y tiempo y avisa a la tripulación; el oxígeno o la recompresión son decisión humana. Las microburbujas no se miden.

### Caso 3B · Tormenta solar y radiación

- **E3.4 DONKI avisa pero GOES todavía no muestra protones.** Aviso en dos pasos: primero "preparate", después "al refugio" cuando el flujo de protones cruza el nivel S1 de NOAA (10 pfu con energía de 10 MeV o más). *Simulación: no manda al refugio solo por la erupción en rayos X.*
- **E3.5 Sin enlace con la Tierra** (conjunción solar, unas dos semanas). Los datos externos no llegan; Eva usa el dosímetro de a bordo como disparador.
- **E3.6 Un pasajero vomita después de la exposición.** Eva registra la hora: vomitar en menos de 1 hora sugiere una dosis alta, más de 2 horas una menor, y no vomitar en 8 a 10 horas sugiere menos de 1 Gy. El vómito también viene del mareo espacial o de la ansiedad, así que Eva solo registra, lo cruza con el dosímetro y avisa a la tripulación y a la Tierra. **Nunca:** estimarle la dosis al pasajero ni hablar de síndrome de radiación.
- **E3.7 Emergencia médica dentro del refugio.** Los equipos de la estación tienen que ser portátiles; el caso 01 se resuelve adentro.

Los linfocitos bajan con la radiación y sirven para estimar dosis con dos hemogramas separados por horas. Ojo con Inspiration4: ahí bajaron por el estrés del vuelo, no por radiación. No mezclarlo en el pitch.

### Caso 4 · Salud mental y diario

- **E4.1 El pasajero no quiere que su diario vaya a Houston.** El diario queda en el dispositivo por defecto. A la Tierra va solo un índice de bienestar si el pasajero lo autoriza, y él elige qué compartir.
- **E4.2 Frases de riesgo (autolesión) en el diario.** Es la única excepción, acordada en el consentimiento: Eva avisa en forma discreta al líder de la tripulación y ofrece contacto con el psicólogo en Tierra (asíncrono, con 3 a 22 minutos de demora). **Nunca:** diagnosticar depresión. Necesita validación clínica antes de presentarlo como funcional.
- **E4.3 Varias noches malas y HRV baja.** Sugerencias de sueño y de luz, sin etiquetas.
- **E4.4 Rasgos de voz.** Son experimentales y cambian con el acento y el idioma: en v1 solo se usa la transcripción, no el tono.

## Edge cases transversales (los que más exigen al producto)

- **T1 · Dos emergencias a la vez.** Cola por gravedad: paro o vía aérea, después hipoxia, después metabólico, después el resto. Cada ayudante recibe una sola tarea.
- **T2 · El ayudante entra en pánico o no sigue los pasos.** Reasignación a los 20 s sin confirmación; instrucciones de a una, con pictograma, voz y botón Repetir.
- **T3 · Batería baja.** Modo mínimo: ECG en ráfagas; detección de caída y respuesta al aviso siempre activas. Conecta con la tarea 02 (Tomi).
- **T4 · La IA falla o tarda.** Protocolo primero: la primera instrucción nunca depende del modelo, y la IA solo puede elegir pasos del protocolo.
- **T5 · Días sin enlace con la Tierra.** Todo funciona a bordo; el registro espera en cola y se confirma después.
- **T6 · Falsas alarmas.** Línea basal personal, confirmación del propio pasajero y alarma general solo con dos señales que coinciden.
- **T7 · Falta un equipo en la estación.** Un sensor por bandeja; Eva indica la estación más cercana que lo tenga.
- **T8 · Pasajero mayor o que no habla inglés.** Pictogramas, voz en su idioma y Repetir siempre visible.
- **T9 · La microgravedad cambia la línea basal** (los fluidos se redistribuyen hacia la cabeza). La basal se recalibra en los primeros días de vuelo.

## La simulación de desarrollo

- Los escenarios están en `sim/scenarios.json`, en la rama `fase2/03-04-casos-variables` del repo. Cada uno trae las señales en el tiempo, lo que Eva tiene que hacer y antes de cuándo, y lo que no puede hacer.
- La app en Rust los reproduce y mide cuatro cosas: tiempo a la primera instrucción (menos de 1 s), ayudante asignado (menos de 10 s), acciones prohibidas (cero) y registro completo.
- Para el pitch: correr dos o tres escenarios en vivo (E1.4, E3.1 y T1) muestra el producto bajo presión mejor que cualquier diapositiva.

## Cambios sugeridos al documento de variables (Hassi y Naza)

- Marcar cada variable con su nivel: continuo, estación o laboratorio.
- En los casos 1 y 2A, describir la descarga y la inyección como módulos certificados de terceros, no como acciones de Eva.
- Sacar la inyección de calcitonina y denosumab del caso 2B y reformularlo como prevención y cólico renal.
- Reformular el caso 3A como despresurización de cabina.
- Sacar serotonina, dopamina y marcadores de ADN de lo que mide WearMeds.

## Fuentes

- Desfibrilador portátil y botones de respuesta: [ZOLL LifeVest, tarjeta para emergencias](https://lifevest.zoll.com/-/media/lifevest-zoll-com/medical-professionals/first-responder-resources/lifevest_ems_pocketcard.ashx) · [Revisión en PMC](https://pmc.ncbi.nlm.nih.gov/articles/PMC5089470)
- Tiempo de conciencia útil: [FAA, hipoxia](https://www.faa.gov/sites/faa.gov/files/about/office_org/headquarters_offices/avs/II.1.2_Hypoxia.doc) · [FAA AC 61-107B](https://www.faa.gov/documentLibrary/media/Advisory_Circular/AC_61-107B_CHG_1.pdf)
- Radiación, vómito y linfocitos: [AFRRI, manejo de víctimas por radiación](https://afrri.usuhs.edu/sites/default/files/2020-07/4edmmrchandbook.pdf) · [Revisión en PMC](https://pmc.ncbi.nlm.nih.gov/articles/PMC3863169)
- Retraso del CGM: [Accu-Chek, lag time](https://www.accu-chek.co.uk/training/cgm/lag-time) · [Revisión en PMC](https://pmc.ncbi.nlm.nih.gov/articles/PMC4667340)
- Denosumab en hipercalcemia: [Ensayo fase II](https://www.qxmd.com/r/24915117) · [Ficha de la FDA](https://www.accessdata.fda.gov/drugsatfda_docs/label/2026/125320s225lbl.pdf)
- Escala de tormentas de radiación S1: NOAA SWPC Space Weather Scales.
