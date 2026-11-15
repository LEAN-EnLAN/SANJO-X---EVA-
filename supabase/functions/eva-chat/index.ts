// eva-chat entry point. The handler (auth, rate limit, topic guard, Groq) is in handler.ts.
import { handler } from "./handler.ts";

Deno.serve(handler);
