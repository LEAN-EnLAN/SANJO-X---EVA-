# Eva explainer · story version (non-technical, EN + ES)

- `eva_story.py`: 12 Manim scenes with pictogram characters and big captions. `EVA_LANG=en|es`.
- `script.json`: narration and captions in both languages (`es_tts` holds a phonetic spelling for the voice only).
- `narr/<lang>/`: narration rendered with Piper TTS. Spanish voice: es_AR "daniela" (dataset CC BY-SA 4.0, credit required). English voice: en_US "ljspeech" (public domain).
- Sound effects come from `../manim/sfx` (earshot bank, MIT).
- `build.sh <lang>`: renders every scene at 1080p30 and joins them with loudness normalisation.
