# Kit Codex v2: assets de Eva en fondo negro y blanco

**Qué es:** un paquete para que Codex (OpenAI) genere todas las imágenes de la landing de **Eva by WearMeds** en dos versiones. Una va sobre **negro puro (#000000)** y la otra sobre **blanco puro (#FFFFFF)**, así se funden con la página sin que se note el recuadro. También genera capas sueltas para animaciones simples al estilo Apple: despiece, giro del producto, brillos y planetas.

> **La estación se eliminó (decisión del equipo); los assets de estación del kit v1 quedan obsoletos.**
> Ahora el equipo personal es solo la **pulsera** (ancha, con pantalla flexible curva sobre el antebrazo) y el **arnés** (sensores, parches de desfibrilación, compartimento del autoinyector y una luz ámbar que marca qué usar). También quedan reemplazados los v1 S2, F1–F3, L1–L6, L12 y L13.

## Cómo se usa
1. Abrir esta carpeta en Codex y pegar el texto de `PROMPT-FOR-CODEX.txt`.
2. Codex revisa la documentación actual de la API, genera **borradores** con `gpt-image-2.5-flare` y después las **finales** con `gpt-image-2.5-sunburst`.
3. Al final corre `python generate.py --post-only --upscale --qa --derive --sheet --zip` y entrega `eva-assets-v2.zip`.

Para hacerlo a mano: `pip install openai pillow`, exportar `OPENAI_API_KEY` y correr `python generate.py --help`.

## Qué hay adentro (62 assets)
- **Pulsera (14):** giro en 5 ángulos (OR), despiece de 5 piezas más la armada (WX), puesta en el antebrazo (WA1) y sobre pedestal en negro y en blanco (PL).
- **Arnés (12):** armado y despiece en 5 piezas (HX), brillos de "ítem a usar" para negro y blanco (GL) y figuras sin rostro, de pie y sentada (FIG).
- **Espacio (15):** Tierra, Marte y Sol con su halo aparte, superficie de Marte, fulguración, partículas opcionales, estrellas y nave con brillo de motor aparte.
- **Escenas (10):** S1–S5, cada una en negro y en blanco, con los bordes fundidos al color de la página y la marca "AI-generated".
- **Cabina para parallax (6):** fondo, plano medio y primer plano, en negro y en blanco.
- **Sombras (5):** PNG transparentes para usar en la versión blanca. Se hacen localmente, sin IA.

Además, `--derive` arma los "chips" cuadrados y los cuadros del inventario a partir de las capas.

## Qué NO se genera
- El arco o pulso de señal Tierra–Marte: lo dibujamos en SVG.
- Las partículas del motion spec: van en canvas.
- El hub de cómputo de la nave: si hace falta, va como nodo SVG abstracto.
- Los textos y la UI: van en HTML.

## Reglas que no se negocian
- Nada de logos, insignias de NASA, banderas ni marcas.
- Nada de texto en las imágenes.
- Las pantallas van en negro puro.
- Nada de caras reconocibles ni agujas.
- La marca "AI-generated" va solo en las escenas.
- La luz viene siempre de arriba a la izquierda.

**Nota:** el MOTION-SPEC escribe el wordmark como "EVA". El nombre es **Eva**.
