# 03-04 · Edge cases: cómo responde Eva

**Actualizado dom 15/11: sin arnés; desfibrilación con el DEA de la estación; inyectable fuera del alcance.**

*Claude, domingo 15/11. Versión 2: rehecha con los comentarios del equipo y con el concepto actual (brazalete, hub de la nave y DEA de la estación de emergencia). Es una propuesta para Hassi y Naza: lo que el equipo acepte va al Registro.*

## Resumen

Quedan tres casos: paro cardíaco, despresurización de cabina y alerta solar, esta última pendiente de confirmar en el alcance. Salen glucosa (2A), hueso y calcio (2B), salud mental (4) y todo lo que dependía de una estación médica propia. Eva guía, registra y avisa; la descarga la decide el DEA de la estación de emergencia de la nave, que lee el ritmo con sus propios parches, nunca Eva ni un modelo de chat. El inyectable queda fuera del alcance.

Reglas que valen para todos los casos:

- Monitoreo no invasivo: nada de agujas ni filamentos para medir. El tratamiento puede ser invasivo solo ante una emergencia mortal.
- Un solo dispositivo personal, el brazalete, más el hub de la nave, que no es un equipo médico. No hay arnés ni estación médica propia. El DEA está en la estación de emergencia de la nave: cuando hace falta, la estación se ilumina y el brazalete muestra el camino.
- La descarga la decide solo el DEA, que analiza el ritmo con sus parches en el pecho. Eva y la IA conversacional no autorizan, no adelantan ni cancelan una descarga.
- La primera instrucción de cada emergencia sale de un protocolo local, no de la IA.
- Los tiempos en segundos de este documento son parámetros de la simulación, no umbrales clínicos.

## Casos seleccionados

Qué puede medir WearMeds, qué hace Eva y qué no hace nunca. Los datos de la nave llegan al hub; no son sensores de WearMeds.

| Caso | Qué puede medir (brazalete, nave) | Qué hace Eva | Qué nunca hace |
|---|---|---|---|
| 1 · Paro cardíaco | Brazalete: pulso óptico (pérdida de pulso y sospecha de ritmo irregular), movimiento, respuesta al aviso (toque, voz o botón) y ECG de una derivación a pedido si la persona colabora. DEA de la estación de la nave: lee el ritmo con sus parches y decide la descarga. | Comprueba si la persona responde, asigna a quien empieza la RCP y a quien trae el DEA, ilumina la estación, guía la RCP en la pantalla del brazalete, repite las indicaciones del DEA y registra todo en la bitácora. | Decidir, adelantar o cancelar una descarga; pedir una descarga por un dato del brazalete; diagnosticar un infarto con el ECG de muñeca. |
| 3A · Despresurización de cabina | Nave (vía hub): presión de cabina y presión parcial de O₂. Brazalete: SpO₂ como tendencia (pierde calidad con frío y movimiento), pulso, respiración estimada en reposo y respuesta al aviso. | Replica la alarma de presión de la nave en todos los brazaletes con una sola instrucción (máscara de oxígeno), después pide ayudar a quien no pudo y registra el perfil de presión. | Crear una alarma de despresurización con datos de los wearables; esperar al SpO₂ para avisar; decidir oxígeno o recompresión. |
| 3B · Alerta solar (pendiente de confirmar en el alcance) | Nave (vía hub): monitor de radiación de a bordo. Con enlace: NASA DONKI y NOAA GOES, con 3 a 22 min de demora. Dosímetro personal en el brazalete: opcional, a evaluar en la tarea 02. | Aviso en dos pasos (preparate, al refugio) según el criterio de refugio de la nave; registra el tiempo en refugio de cada pasajero y la dosis de a bordo. | Mandar al refugio solo por una erupción en rayos X; estimar dosis individuales o hablar de síndrome de radiación; interrumpir una reanimación por la alerta. |

## Casos que salen y por qué

- **2A · Glucosa:** medir glucosa exige un sensor con filamento bajo la piel (CGM) y el encargo prohíbe el monitoreo invasivo; no hay glucosa no invasiva validada. El glucómetro y el medidor de cetonas estaban en la estación. Un pasajero que no responde, por esta u otra causa, queda cubierto por el flujo general (E1.1 y T6).
- **2B · Hueso y calcio:** la pérdida ósea es crónica, no una emergencia de minutos; el denosumab tarda días en actuar y puede bajar el calcio de más. No usa ninguna de las cuatro funciones de forma aguda.
- **4 · Salud mental:** no es un evento agudo que el brazalete reconozca con seguridad, y la bitácora es un registro compartido, no un diario personal. Puede volver como línea futura si el equipo la quiere.
- **Lo que dependía de una estación médica propia:** bandejas iluminadas, manguito de presión, oxímetro de dedo, glucómetro, medidor de cetonas y la ruta a esa estación. El DEA vuelve, pero como equipo de la estación de emergencia de la nave (caso 1 y T16).
- **Arnés de pecho:** sale por decisión del equipo del dom 15/11. Su ECG continuo, la respiración por impedancia y la desfibrilación vestible se reemplazan por el monitoreo del brazalete y el DEA de la estación.
- **Inyectable:** fuera del alcance.

## Caso 1 · Paro cardíaco

Formato de cada edge case: situación, qué hace Eva, qué nunca hace y criterio de la simulación. El identificador coincide con `sim/scenarios.json`.

### E1.1 · Posible paro: sin pulso, sin movimiento y sin respuesta

- **Situación:** el brazalete deja de ver pulso óptico, no detecta movimiento y la persona no responde al aviso.
- **Qué hace Eva:** con las tres señales juntas inicia el flujo de paro sin esperar más datos. Asigna al pasajero capacitado más cercano para empezar la RCP y a un segundo para traer el DEA, ilumina la estación más cercana y le muestra el camino, y guía la RCP paso a paso en la pantalla del brazalete. Con el acelerómetro del brazalete del ayudante le dice si las compresiones van al ritmo de la guía (a validar). Cada pantalla dice qué hacer ahora, por qué y qué sigue (Russell et al., 2023). Si hay movimiento o la persona responde, no es este flujo: ver T17.
- **Qué nunca hace:** alarma general por una sola señal; demorar la RCP para confirmar el ritmo; decidir si corresponde una descarga, que lo decide el DEA.
- **Simulación:** ayudante de RCP y ayudante del DEA asignados dentro del plazo, y estación iluminada; cero descargas originadas en Eva.

### E1.2 · Ejercicio que ensucia la señal óptica

- **Situación:** en un viaje largo hay ejercicio diario y el movimiento mete ruido en el pulso óptico del brazalete.
- **Qué hace Eva:** con aceleración alta y movimiento de ejercicio no alarma: si hay movimiento, no es un paro. Si el pulso se ve irregular, pide confirmación al pasajero y, ya en reposo, puede pedirle un ECG de muñeca (T15).
- **Qué nunca hace:** alarma general solo por una señal óptica con ruido; pedir el ECG de muñeca en pleno movimiento.
- **Simulación:** cero falsas alarmas en 30 minutos de ejercicio.

### E1.3 · El DEA analiza el ritmo y decide la descarga

- **Situación:** el ayudante llega con el DEA de la estación y le pone los parches en el pecho a la persona.
- **Qué hace Eva:** le dice que siga las indicaciones de voz del DEA y las repite en la pantalla del brazalete, un paso por vez. Cuando el DEA analiza o va a descargar, avisa a todos los que ayudan "no la toques"; después de la descarga, o si el DEA indica que no corresponde, vuelve a guiar la RCP según el entrenamiento para microgravedad. Muestra qué sigue, el próximo análisis del DEA y el relevo de compresiones, para que el ayudante sepa si lo que hizo funcionó (Russell et al., 2023). Registra la hora de llegada del DEA, de cada análisis y de cada descarga; si el DEA no puede informar sus eventos al hub, el ayudante los marca con un toque (a validar).
- **Qué nunca hace:** autorizar, adelantar, cancelar ni contradecir una descarga; interpretar el ritmo con el brazalete o con un modelo de lenguaje.
- **Simulación:** el ayudante recibe "no la toques" antes de la descarga simulada; RCP retomada dentro del plazo; cero descargas originadas en Eva.

### E1.4 · El ayudante asignado no confirma

- **Situación:** el pasajero más cercano no confirma, o también es paciente.
- **Qué hace Eva:** desde el primer segundo, una persona recibe la tarea con nombre y el resto un aviso informativo. Si no confirma en el plazo, Eva reasigna al siguiente y abre la tarea a todos los cercanos (la toma el primero que acepta); el líder de la tripulación siempre queda avisado. Lo mismo vale para las dos tareas, empezar la RCP y traer el DEA. Registra quién aceptó y cuándo.
- **Qué nunca hace:** dejar la emergencia sin un responsable con nombre; mandar a todos a la misma tarea sin coordinar.
- **Simulación:** reasignación dentro del plazo. Variante a comparar: asignar a una persona contra avisar a todos los capacitados (tiempo hasta la primera confirmación y cuántas personas llegan).

### E1.5 · Paro durante una alerta solar

- **Situación:** un pasajero entra en paro mientras rige la orden de ir al refugio.
- **Qué hace Eva:** la vida inmediata manda. La RCP sigue donde está la persona. Eva asigna a alguien para traer el DEA de la estación más cercana aunque quede fuera del refugio, y avisa a la tripulación, que puede cambiar esa asignación (criterio a validar con la operadora). Pide otro ayudante para relevar las compresiones y avisa cada relevo con tiempo (las guías recomiendan rotar cada 2 minutos aproximadamente). Si hay que trasladar a la persona al refugio, lo decide la tripulación. Registra la dosis de a bordo durante el evento.
- **Qué nunca hace:** interrumpir la RCP por la alerta; ordenar un traslado durante la reanimación; dejar de buscar el DEA por la orden de refugio sin que lo decida la tripulación.
- **Simulación:** la RCP no se interrumpe por la alerta; ayudante del DEA y relevo de compresiones pedidos dentro del plazo.

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
- **Qué hace Eva:** con la cabina normal y la persona respondiendo, pide reacomodar el brazalete, repite la medición y usa la respiración estimada en reposo y la respuesta al aviso.
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

- **Situación:** la base de sensores o la interfaz del brazalete quedan con poca carga.
- **Qué hace Eva:** modo mínimo: baja el brillo de la pantalla y la actividad de la radio, y avisa al pasajero y a la tripulación para cargar. La comprobación de respuesta, el botón de ayuda y la detección de pérdida de pulso siguen activos.
- **Qué nunca hace:** apagar una función de rescate para ahorrar energía.
- **Simulación:** modo mínimo dentro del plazo sin desactivar la comprobación de respuesta ni la detección de pérdida de pulso.

### T4 · La IA falla o tarda

- **Situación:** el modelo del hub no responde o tarda.
- **Qué hace Eva:** protocolo primero: la primera instrucción sale del firmware y del motor de protocolos, y la IA solo puede elegir entre pasos del protocolo. El DEA funciona solo, sin el hub ni Eva.
- **Qué nunca hace:** esperar a la IA para dar la primera instrucción; dejar que el chat escriba instrucciones de tratamiento libres.
- **Simulación:** primera instrucción en menos de 1 s con la IA caída.

### T5 · Sin enlace con la Tierra

- **Situación:** días sin comunicación.
- **Qué hace Eva:** todo funciona a bordo; la bitácora y los reportes quedan en cola y se envían al volver el enlace, con la hora real de cada evento.
- **Qué nunca hace:** condicionar una respuesta a la Tierra; perder registros.
- **Simulación:** reporte en cola y enviado al restablecerse el enlace.

### T6 · Falsas alarmas

- **Situación:** una señal sola parece grave.
- **Qué hace Eva:** usa la línea basal personal, pide confirmación al propio pasajero y solo da alarma general con dos señales independientes que coinciden. El pulso y su variabilidad salen del mismo sensor y no cuentan como dos. Durante una emergencia, los avisos no críticos quedan en silencio para quienes ayudan y se muestran después (Russell et al., 2023).
- **Qué nunca hace:** alarma general con una sola señal; demorar una intervención validada esperando datos que no hacen falta (sin pulso, sin movimiento y sin respuesta ya son tres señales y alcanzan para empezar la RCP; la descarga la decide el DEA).
- **Simulación:** cero alarmas generales con una sola señal anormal.

### T7 · El DEA falta, tiene poca batería o ya se usó

- **Situación:** el DEA no está en su estación, tiene poca batería o sus parches ya se usaron o vencieron. Si está en uso o lejos, ver T16.
- **Qué hace Eva:** lee el estado de cada estación (presencia del DEA y autochequeo del equipo, si la estación lo informa al hub: a validar), avisa a la tripulación para reponerlo y lo registra. Durante una emergencia, si el DEA más cercano no está disponible, manda al ayudante a la otra estación y lo dice en la pantalla; la guía de RCP sigue sin esperar al DEA.
- **Qué nunca hace:** iluminar una estación vacía o con un DEA no disponible; frenar la RCP mientras se busca un DEA.
- **Simulación:** faltante avisado a la tripulación; cero ayudantes mandados a una estación sin DEA disponible.

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
- **Qué hace Eva:** el brazalete mantiene la lógica local de emergencia: hace sonar una alarma local para llamar a quien esté cerca, muestra instrucciones precargadas (RCP y dónde está cada estación con DEA) y guarda la bitácora para sincronizarla después. Sin hub no se puede iluminar la estación, así que el camino lo muestra la pantalla del brazalete del paciente. El DEA funciona solo. Cada mensaje de emergencia pide confirmación y se reintenta.
- **Qué nunca hace:** dar por entregado un mensaje sin confirmación; dar por resuelta la malla LoRa, que sigue en evaluación (tarea 09).
- **Simulación:** alarma local en el brazalete del paciente dentro del plazo con el hub caído; bitácora sincronizada al volver.

### T11 · Encontrar al pasajero más cercano

- **Situación:** Eva necesita saber quién está más cerca, y la intensidad de señal de LoRa (RSSI) no es una distancia confiable dentro de una nave metálica.
- **Qué hace Eva:** usa una ubicación por compartimento (método a definir en la tarea 09) y le pide al asignado que confirme "voy"; si responde que no está cerca, reasigna.
- **Qué nunca hace:** tratar el RSSI como distancia.
- **Simulación:** el asignado responde "no estoy cerca" y Eva reasigna dentro del plazo.

### T12 · El traje queda entre las dos partes del brazalete

- **Qué pasa:** el pasajero se pone el traje con el módulo de sensores en la piel y la interfaz sobre la manga. Los imanes sostienen, pero los pines no tocan.
- **Riesgo:** la base de sensores queda con su batería chica y sin pines hacia la interfaz.
- **Cómo responde Eva:** la interfaz detecta que no hay contacto en los pines y pasa a modo traje. Los sensores bajan la frecuencia de muestreo a lo mínimo que mantiene las alarmas y siguen mandando los datos por LoRa, como cuando están unidas (las dos partes tienen LoRa). La pantalla muestra la carga de la batería chica y el tiempo estimado que queda. Con poca carga, avisa con tiempo para volver a unir las partes. El ECG de muñeca sigue disponible si las manos están descubiertas, porque las dos placas están arriba de la interfaz (T15).
- **Para validar:** autonomía real de la batería chica con LoRa transmitiendo a potencia mínima.

### T13 · Las partes se separan o la interfaz queda lejos

- **Qué pasa:** la interfaz se suelta (golpe, maniobra) o el pasajero se la saca y el módulo de sensores sigue en la piel.
- **Cómo responde Eva:** el módulo de sensores sigue midiendo con su batería y guarda los datos. Como tiene LoRa propio, sigue mandando sus datos al hub aunque la interfaz no esté. Si la interfaz no responde por más de unos segundos, el hub avisa al pasajero y a la tripulación. Una alarma crítica detectada sin interfaz se manda por la malla desde el hub y, si hay, por la pulsera del compañero más cercano. Sin la interfaz no hay pantalla ni ECG a pedido.

### T14 · Mala señal óptica en la muñeca

- **Qué pasa:** el sensado multionda funciona mejor en el dedo (por transmisión). En la muñeca se mide por reflexión, con más ruido por movimiento, frío o poca perfusión.
- **Cómo responde Eva:** cada medición viaja con un índice de calidad de señal. Con calidad baja, Eva no dispara alarmas con ese dato solo: lo cruza con el movimiento y la respuesta al aviso, y pide quedarse quieto o ajustar la banda. Si la persona puede colaborar, pide un ECG de muñeca (T15).

### T15 · ECG de muñeca: pedirlo bien y saber cuándo no sirve

- **Qué pasa:** Eva quiere un registro de ECG de una derivación (por ejemplo, el pulso óptico se volvió irregular). La interfaz tiene dos placas metálicas arriba: la persona apoya un dedo de cada mano, quieta, unos 30 segundos. No usa la piel de la muñeca.
- **Cómo responde Eva:** lo pide solo si la persona está consciente y puede colaborar, con una instrucción por paso y una cuenta regresiva. Si el contacto es malo o hay movimiento, lo dice y repite. El registro va al hub con su índice de calidad y queda en la bitácora. Si la persona no responde, no pide el ECG: sigue el flujo de paro (E1.1) y el ritmo lo lee el DEA. Con el traje entre las dos partes, el ECG funciona igual si las manos están descubiertas; con guantes no está disponible y la pantalla lo dice.
- **Límite:** no es continuo, necesita que la persona colabore y nunca decide una descarga. Una derivación no reemplaza un ECG de 12 ni descarta un infarto.

### T16 · El DEA está lejos o lo están usando

- **Situación:** el DEA de la estación más cercana ya se está usando en otra emergencia, o esa estación queda lejos del paciente.
- **Qué hace Eva:** el escenario usa dos estaciones con DEA (cuántas hay y dónde, a confirmar con la operadora). Eva elige la estación con el DEA disponible y la ruta más corta por compartimentos, ilumina esa estación y le muestra el camino al ayudante en la pantalla (Russell et al., 2023). Si el ayudante no confirma o avisa que no llega, reasigna a quien esté más cerca de la otra estación. Mientras tanto la RCP sigue, y quien la hace ve que el DEA está en camino.
- **Qué nunca hace:** mandar a buscar un DEA que ya está en uso; sacar el DEA de una reanimación en curso; frenar la RCP mientras llega el DEA.
- **Simulación:** ayudante mandado a la estación con DEA disponible y reasignación dentro del plazo; cero ayudantes mandados a un DEA en uso.

### T17 · Pérdida de pulso por mal contacto del sensor

- **Situación:** el brazalete deja de ver pulso porque la base de sensores quedó floja o se corrió, no porque el corazón se paró. Es un riesgo de falsa alarma.
- **Qué hace Eva:** antes de escalar, cruza la pérdida de pulso con el movimiento y el índice de calidad de la señal, y pide respuesta por voz y vibración ("Are you OK? Tap to answer"). Si hay movimiento o la persona responde, le pide ajustar el brazalete, repite la medición y lo registra, sin alarma general. Si no hay movimiento y no responde, inicia el flujo de paro de E1.1 sin esperar a revisar el contacto.
- **Qué nunca hace:** alarma general por una pérdida de pulso sola; demorar la RCP para revisar el sensor cuando la persona no responde.
- **Simulación:** cero alarmas generales con mal contacto y persona que responde; ayudante asignado dentro del plazo cuando no hay respuesta.

## La simulación de desarrollo

- Los escenarios están en `sim/scenarios.json`, en la rama `fase2/03-04-casos-variables`. Cada uno trae las señales en el tiempo, lo que Eva tiene que hacer y antes de cuándo, y lo que no puede hacer.
- La app en Rust mide cuatro cosas: primera instrucción (menos de 1 s), ayudante asignado (menos de 10 s, salvo que el escenario pida confirmación antes), acciones prohibidas (cero) y bitácora completa.
- Salieron los escenarios de 2A, 2B y 4 y las acciones de la estación médica propia (iluminar bandejas). Entraron los escenarios de E3.2, E3.4, T2, T5 a T11 y la variante de asignación de E1.4. Con la decisión del dom 15/11 salieron las señales y acciones del arnés y del inyectable, y entraron las de la estación con DEA (iluminar la estación, mostrar la ruta, estado del DEA) y los escenarios T16 y T17.
- Para el pitch: correr E1.3, E3.1 y T1 en vivo.

## Pendientes para Hassi y Naza

- Confirmar con la operadora cuántas estaciones con DEA tiene la nave, dónde están y si informan al hub el estado del DEA y sus eventos (análisis y descarga).
- Confirmar si la alerta solar sigue en el alcance.
- Definir la técnica de RCP en microgravedad que entra en el entrenamiento y en la guía.
- Elegir entre las dos estrategias de asignación de E1.4 con los resultados de la simulación.

## Fuentes

- Soporte de decisiones en emergencias de vuelo (qué hacer ahora, por qué y qué sigue; si la acción funcionó; reevaluación programada; dónde está cada elemento; filtrar avisos no críticos): [Russell BK et al., npj Microgravity 9:46 (2023)](https://doi.org/10.1038/s41526-023-00284-1), doi:10.1038/s41526-023-00284-1
- RCP, DEA, ritmos desfibrilables y relevo de compresiones: [AHA, guías de RCP y atención cardiovascular de emergencia](https://cpr.heart.org/en/resuscitation-science/cpr-and-ecc-guidelines)
- Tiempo de conciencia útil: [FAA, hipoxia](https://www.faa.gov/sites/faa.gov/files/about/office_org/headquarters_offices/avs/II.1.2_Hypoxia.doc) · [FAA AC 61-107B](https://www.faa.gov/documentLibrary/media/Advisory_Circular/AC_61-107B_CHG_1.pdf)
- Escala de tormentas de radiación S1: [NOAA SWPC, Space Weather Scales](https://www.swpc.noaa.gov/noaa-scales-explanation)
- Asignar la tarea con nombre (difusión de responsabilidad): Darley y Latané (1968), Journal of Personality and Social Psychology 8(4), 377-383.
- Alcance (sin estación médica propia, monitoreo no invasivo, sin filamentos subcutáneos): [Encargo de investigación médica y arquitectura de Eva](https://docs.google.com/document/d/1pSradgUdKCN5O2YeCmGXNlE20-SyGAsuZZhEqzMcr90/edit)

## Respuestas a los comentarios

Una entrada por comentario del documento anterior, en el orden en que aparecían. Los hilos siguen abiertos en Drive y se resuelven a mano.

- **Reacción sin texto, sobre "Eva coordina a las personas… lleva a la estación":** ya estaba resuelta. Respuesta: el párrafo se reescribió sin la estación. **Sin acción pendiente.**
- **Leandro, sobre "a demanda (estación)":** "La estación está en discusión, cambiarla a ser más específica para los módulos de desfibrilación únicamente". Respuesta: el encargo eliminó la estación médica propia. Desde el dom 15/11 tampoco hay arnés: la desfibrilación queda en el DEA de la estación de emergencia de la nave, como pedía el comentario. El nivel "a demanda" desaparece; la tabla muestra solo brazalete y datos de la nave. **Aplicado.**
- **Tomás, sobre "Nunca: decidir una descarga":** "No tiene sentido". Respuesta: lo que importa es quién decide. Desde el dom 15/11 descarga el DEA de la estación, que lee el ritmo con sus parches y decide; Eva y el chat nunca. Se reescribió así en E1.3 y en la tabla. **Aplicado en parte.**
- **Tomás, sobre "Eva pasa al siguiente y después al líder de la tripulación":** evaluar avisar a todos los capacitados en vez de a uno. Respuesta: todos reciben un aviso informativo desde el primer segundo, pero la tarea va a una persona con nombre para que haya un responsable claro (cuando la tarea es de todos, cada uno tiende a esperar a otro). Si no confirma, se abre a todos los cercanos. Las dos estrategias quedan como variante a comparar en la simulación (E1.4). **Aplicado en parte.**
- **Tomás, sobre "con dos ayudantes":** "¿Por qué dos?". Respuesta: eran para trasladar a la persona al refugio sin cortar la RCP, pero trasladar durante la reanimación no tiene sentido si la exposición de un evento solar se acumula en horas. Ahora la RCP sigue donde está, un ayudante trae el DEA, otro releva las compresiones y el traslado lo decide la tripulación (E1.5). **Aplicado.**
- **Tomás, sobre E2.1 (hipoglucemia falsa de noche):** "Al pedo integrarlo". Respuesta: de acuerdo, y además el CGM usa un filamento bajo la piel, que el encargo prohíbe para medir. Sale el caso 2A completo. **Aplicado.**
- **Tomás, sobre E2.2 (hipoglucemia real con confusión):** "Al pedo". Respuesta: sale con el caso 2A; el glucagón iluminado en la estación ya no existe. Un pasajero que no responde, por cualquier causa, queda cubierto por E1.1 y T6. **Aplicado.**
- **Leandro, sobre la introducción del caso 2B:** "Es totalmente innecesario y genera un problema futuro". Respuesta: sale el caso 2B completo, con sus tres edge cases y su escenario. **Aplicado.**
- **Leandro, sobre "Eva no espera al SpO₂, que llega tarde":** "¿Se tendría que mejorar el modelo predictivo de oxígeno para aumentar la velocidad del dato?". Respuesta: el dato más rápido es el sensor de presión de la nave, que mide la causa; el SpO₂ mide la consecuencia, cuando la hipoxia ya empezó. Un modelo predictivo no le gana a la medición directa y suma falsas alarmas. **No aplicado, porque el disparador correcto ya es la presión de la nave.**
- **Leandro, sobre "Put your oxygen mask on now":** "Puede provocar falsos positivos". Respuesta: Eva ya no dispara con datos propios: replica la alarma de la nave, y una lectura aislada solo avisa a la tripulación. Se agregó E3.2 con su escenario. **Aplicado.**
- **Joaquín, sobre los antiguos E3.6 y E3.7 (vómito tras la exposición, emergencia en el refugio):** "¿De más?". Respuesta: sí. El antiguo E3.6 era estimar dosis por la hora del vómito, un triage clínico que Eva no debe hacer y que se confunde con el mareo espacial. E3.7 dependía de los equipos de la estación; ahora cada pasajero lleva su brazalete al refugio. Salen los dos. **Aplicado.**
- **Joaquín, sobre E4.1 (diario privado):** "El diario no es tan diario, es más bien bitácora; no tiene por qué ser privada ni personal". Respuesta: de acuerdo. La bitácora es la cuarta función y se comparte con la tripulación y la Tierra. Sale el caso del diario privado y, con él, el caso 4 queda fuera del núcleo. **Aplicado.**
