# Atribuciones del prototipo
- Anatomía: datos derivados de **BodyParts3D** (© The Database Center for Life Science, CC BY 4.0), obtenidos a través de Human Atlas (MIT). El conversor `tools/build_anatomy.py` es código propio.
- Datos: NASA OSDR, Inspiration4 (OSD-575 panel metabólico, OSD-569 hemograma); `tools/build_i4.py` arma `data/inspiration4.json` desde `data-src/`.
- Fondos de entornos: imágenes generadas con IA (Codex).
- La capa 2 de razonamiento usa Claude en lugar de MedGemma mientras no esté descargado el modelo.
