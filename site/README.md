# Public site (eva-wearmeds.vercel.app)

The curated copy that is deployed to Vercel: landing (`index.html`, `a/`), pitch (`pitch/`), app demo launcher and captured views (`app/`), video page (`video/`), research hub (`hub/`), APK download page (`download/`), wireframes, status page, and the serverless API in `api/`:

- `api/eva/chat.ts` (Groq, light model, topic guard)
- `api/eva/stt.ts` (Whisper large v3 turbo)
- `api/eva/tts.ts` (Orpheus English; Spanish falls back to the browser's voice)
- `api/eva/vision.ts` (Qwen vision)

Keys live only in Vercel's encrypted environment (`GROQ_API_KEY`); none are in this folder. The large binaries (video MP4s, the APK, pitch exports) are left out of git and served from the live site.
