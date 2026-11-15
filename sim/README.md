# Simulación de edge cases

`scenarios.json` describe cada edge case de `../docs/edge-cases.md` como una línea de tiempo de señales (`events`), lo que Eva tiene que hacer y antes de cuándo (`expect`) y lo que no puede hacer nunca (`forbidden`).

La app (`../app/`) los va a reproducir y medir:
- primera instrucción en menos de 1 s;
- ayudante asignado en menos de 10 s (el escenario puede pedir confirmación antes);
- cero acciones prohibidas;
- registro encadenado completo.

Regla de producto que la simulación verifica: Eva guía, registra y avisa. Nunca descarga, inyecta, dosifica ni diagnostica; los tratamientos son de módulos certificados o de personas.
