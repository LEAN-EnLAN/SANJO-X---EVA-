// Eva static replay glue. Runs inside every captured screen instead of htmx, SSE and the
// Eva Link socket: links open other captured screens, story buttons tell the player to move
// the scenario on, and everything that would write to the ship says so in a toast.
// One file for every place the replay runs: the hosted site, the Android app's bundled copy
// (https://appassets.androidplatform.net) and a local copy (file:// or http://localhost).
//
// Live Eva (Groq, through the site's /api/eva/*) everywhere a visitor expects her: the Eva chat (typing, the
// question chips, the big microphone, photo and video), the home "How am I today?" box and its microphone,
// "Ask Eva about it" on the vitals screens, the journal (typed or spoken note), the check-in and the wristband's
// Eva page. Every call shows "Eva is thinking…", retries once when Eva is busy (429/503), and falls back to the
// recorded answer when she can't be reached. Replies ask for the simple style: short, warm, everyday words.
(function () {
  "use strict";
  var inFrame = window.parent && window.parent !== window;
  var stateId = document.body.getAttribute("data-replay-state") || "";
  var viewKind = document.body.getAttribute("data-replay-view") || "";
  var isWrist = viewKind === "wrist";
  var isPhone = viewKind === "phone";
  var file = location.pathname.split("/").pop() || "";
  var seat = (file.match(/-(\d[A-Z])-/) || [])[1] || "4B";
  var scene = (file.match(/^(s\d)-/) || [])[1] || stateId || "s1";
  function post(msg) {
    msg.eva = "replay"; msg.state = stateId;
    if (inFrame) { try { window.parent.postMessage(msg, "*"); } catch (e) {} }
  }
  function getItem(kind, k) { try { var s = window[kind]; return s ? s.getItem(k) : null; } catch (e) { return null; } }
  function setItem(kind, k, v) { try { var s = window[kind]; if (s) s.setItem(k, v); } catch (e) {} }

  // the language: the player's (parent page), else the saved choice, else English
  function lang() {
    if (inFrame) { try { var l = window.parent.document.documentElement.lang; if (l) return l.slice(0, 2) === "es" ? "es" : "en"; } catch (e) {} }
    return getItem("localStorage", "eva-replay-lang") === "es" ? "es" : "en";
  }
  function es() { return lang() === "es"; }
  function L(en, sp) { return es() ? sp : en; }

  // ---------------------------------------------------------------- bigger, calmer screens (older visitors)
  // Body text at least 18 px, touch targets at least 48 px, a text-size choice (A / A+ / A++) shared with the
  // player, and Eva's status in plain words. Landing palette only: black, off-white, grey, teal, hairlines.
  var css = [
    "html[data-mode=phone] .text-body{font-size:1.125rem;line-height:1.6rem}",
    "html[data-mode=phone] .text-label{font-size:1rem;line-height:1.45rem}",
    "html[data-mode=phone] .text-caption,html[data-mode=phone] .text-xs{font-size:.9375rem;line-height:1.35rem}",
    "html[data-mode=phone] .text-sm{font-size:1.0625rem;line-height:1.5rem}",
    "html[data-mode=phone] .bubble{font-size:1.0625rem;line-height:1.5rem}",
    "html[data-mode=phone] .chip{height:3rem;font-size:1rem;padding:0 1.1rem}",
    "html[data-mode=phone] .tab-label{font-size:.8125rem;line-height:1rem}",
    "html[data-mode=phone] .badge{height:auto;min-height:1.75rem;font-size:.875rem;line-height:1.2rem;padding:.2rem .7rem}",
    "html[data-mode=phone] .link-pill{height:2.25rem;font-size:.875rem}",
    "html[data-mode=phone] button.size-10,html[data-mode=phone] label.size-10{width:3rem;height:3rem}",
    "html[data-mode=phone] #eva-text{height:3.5rem;padding-left:3.75rem;padding-right:3.75rem;font-size:1.125rem}",
    "html[data-mode=phone] [data-eva-caption]{font-size:1.0625rem;line-height:1.5rem;color:#f4f3f0}",
    "html[data-mode=phone] [data-eva-mic]{width:5rem;height:5rem}",
    "body:not([data-replay-view=phone]):not([data-replay-view=wrist]) .text-xs{font-size:.875rem;line-height:1.25rem}",
    // Eva's status line (replaces the old 'on-board rules' card): no tinted panel, a hairline only
    ".eva-status{display:flex;align-items:center;gap:.85rem;padding:.85rem 1rem;border:1px solid #2a2a2a;border-radius:1.25rem;background:transparent}",
    ".eva-status-face{width:2.75rem;height:2.75rem;flex:none;object-fit:contain}",
    ".eva-status-title{font-size:1.125rem;line-height:1.5rem;font-weight:600;color:#f4f3f0;display:flex;align-items:center;gap:.5rem}",
    ".eva-status-title:before{content:'';width:.6rem;height:.6rem;border-radius:99px;background:#1fe5c4;flex:none}",
    ".eva-status[data-state=offline] .eva-status-title:before{background:#9b9996}",
    ".eva-status-sub{font-size:1rem;line-height:1.45rem;color:#9b9996;margin-top:.15rem}",
    // Eva is thinking: three soft dots
    ".eva-think{display:inline-flex;align-items:center;gap:.5rem;color:#9b9996}",
    ".eva-think i{display:inline-block;width:.45rem;height:.45rem;border-radius:99px;background:#1fe5c4;opacity:.3;animation:evaDot 1.2s infinite ease-in-out}",
    ".eva-think i:nth-child(3){animation-delay:.2s}.eva-think i:nth-child(4){animation-delay:.4s}",
    "@keyframes evaDot{0%,80%,100%{opacity:.25;transform:translateY(0)}40%{opacity:1;transform:translateY(-3px)}}",
    "@media (prefers-reduced-motion:reduce){.eva-think i{animation:none;opacity:.8}}",
    // Eva's reply card (journal, check-in)
    ".eva-reply{display:flex;gap:.75rem;align-items:flex-start;padding:1rem;border:1px solid #2a2a2a;border-radius:1.25rem;background:#000;color:#f4f3f0;font-size:1.125rem;line-height:1.6rem;white-space:pre-line}",
    ".eva-reply img{width:2.5rem;height:2.5rem;flex:none;object-fit:contain}",
    ".eva-reply small{display:block;margin-top:.4rem;color:#9b9996;font-size:.9375rem;line-height:1.35rem}",
    ".eva-talk{display:inline-flex;align-items:center;justify-content:center;gap:.6rem;min-height:3.25rem;padding:0 1.25rem;border-radius:999px;border:1.5px solid #1fe5c4;color:#f4f3f0;background:#000;font-weight:600;font-size:1.0625rem;cursor:pointer;touch-action:none}",
    ".eva-talk[data-recording]{box-shadow:0 0 0 3px rgba(31,229,196,.35)}",
    ".eva-talk svg{width:1.4rem;height:1.4rem;stroke:#1fe5c4;fill:none;stroke-width:2}",
    "#toast [data-replay-toast]{font-size:1.0625rem;line-height:1.45rem;padding:.75rem 1.1rem;max-width:26rem;text-align:center;border-radius:1.25rem}",
    // the band: room for Eva's live answer
    "html[data-mode=wristband] .w-bubble.is-live{-webkit-line-clamp:4;line-clamp:4;max-height:calc(5.2em + 1rem);font-size:max(.95rem,12px);white-space:normal}",
    "html[data-mode=wristband] .w-bubble .eva-think{color:#f4f3f0}"
  ].join("\n");
  var st = document.createElement("style"); st.id = "eva-elder"; st.textContent = css; document.head.appendChild(st);

  // text size: 0 = A, 1 = A+, 2 = A++ (saved by the player; same origin, so the same storage)
  var TS_PHONE = ["16px", "18px", "20px"], TS_WIDE = ["16px", "17.5px", "19px"];
  function applyTs(v) {
    var n = parseInt(v == null ? getItem("localStorage", "eva-ts") : v, 10);
    if (!(n >= 0 && n <= 2)) n = 0;
    if (isWrist) return; // the band sizes itself from its own screen height
    document.documentElement.style.fontSize = (isPhone ? TS_PHONE : TS_WIDE)[n];
  }
  applyTs();

  var timer = 0;
  function toast(text, ms) {
    var host = document.getElementById("toast");
    if (!host) {
      host = document.createElement("div");
      host.id = "toast";
      host.setAttribute("role", "status");
      host.style.cssText = "position:fixed;left:0;right:0;top:12px;z-index:60;display:flex;justify-content:center;padding:0 16px;pointer-events:none";
      document.body.appendChild(host);
    }
    host.innerHTML = '<p class="pointer-events-auto rounded-full border border-line bg-raised px-4 py-2 text-sm text-ink" data-replay-toast></p>';
    host.firstChild.textContent = text;
    clearTimeout(timer);
    timer = setTimeout(function () { host.innerHTML = ""; }, ms || 4200);
  }
  function demoToast() { toast(L("This is a demo: the button shows how it works, but nothing is saved.", "Esto es una demo: el botón muestra cómo funciona, pero no se guarda nada.")); }
  function go(href) { if (href) location.href = href; }

  // ---------------------------------------------------------------- where Eva lives
  // - the hosted site: its own /api/eva/* (same topic guard as the app);
  // - the Android app's bundled copy: the hosted API by absolute URL (it allows that origin by CORS);
  // - a local dev server on localhost / 127.0.0.1 (vercel dev, or the checks' proxy): its own /api/eva/*;
  // - file:// and any other plain http: no API, recorded answers.
  var HOSTED_API = "https://eva-wearmeds.vercel.app/api/eva/chat";
  var ASSET_HOST = "appassets.androidplatform.net";
  function chatUrl(loc) {
    loc = loc || location;
    if (loc.protocol === "http:" && /^(localhost|127\.0\.0\.1)$/.test(loc.hostname)) return "/api/eva/chat";
    if (loc.protocol !== "https:") return null;
    return loc.hostname === ASSET_HOST ? HOSTED_API : "/api/eva/chat";
  }
  window.evaReplayChatUrl = chatUrl; // read by the checks; changes nothing
  function api(name) { var u = chatUrl(); return u ? u.replace(/chat$/, name) : null; }

  // ---------------------------------------------------------------- is Eva online? (said in plain words)
  var STATUS_KEY = "eva-live-status";
  var online = null; // null = not known yet
  function readCached() {
    try { var c = JSON.parse(getItem("sessionStorage", STATUS_KEY) || "null"); if (c && Date.now() - c.t < 180000) return !!c.on; } catch (e) {}
    return null;
  }
  var STATUS_TEXT = {
    on: [["Eva is online", "Ask anything about health in space. Type, tap a question, or tap the microphone to talk."],
         ["Eva está en línea", "Preguntale lo que quieras sobre la salud en el espacio. Escribí, tocá una pregunta o tocá el micrófono para hablar."]],
    off: [["Offline · Eva answers from her saved guide", "You can still tap the questions. Her answers come from her saved guide."],
          ["Sin conexión · Eva responde con su guía guardada", "Igual podés tocar las preguntas. Las respuestas vienen de su guía guardada."]]
  };
  var CAPTION = { on: ["Tap to talk to Eva", "Tocá para hablar con Eva"], off: ["Talking needs the internet · tap a question instead", "Hablar necesita internet · tocá una pregunta"] };
  var canRecord = !!(navigator.mediaDevices && navigator.mediaDevices.getUserMedia && window.MediaRecorder);
  function idleCaptionText() { return CAPTION[online !== false && canRecord && chatUrl() ? "on" : "off"][es() ? 1 : 0]; }
  function paintStatus() {
    var on = online !== false && !!chatUrl(); // unknown counts as online until the check answers
    var t = STATUS_TEXT[on ? "on" : "off"][es() ? 1 : 0];
    document.querySelectorAll("[data-eva-banner]").forEach(function (b) {
      b.setAttribute("data-state", on ? "online" : "offline");
      var a = b.querySelector("[data-eva-status-title]"), s = b.querySelector("[data-eva-status-sub]");
      if (a) a.textContent = t[0];
      if (s) s.textContent = t[1];
    });
    document.querySelectorAll("[data-eva-chip]").forEach(function (c) {
      var x = c.querySelector("[data-eva-chip-text]");
      if (x) x.textContent = on ? (c.getAttribute("data-online-text") || L("Online", "En línea")) : L("Saved guide", "Guía guardada");
      c.classList.toggle("text-eva", on); c.classList.toggle("text-muted", !on); c.classList.remove("text-warn");
      c.title = on ? L("Eva is online", "Eva está en línea") : L("Eva answers from her saved guide", "Eva responde con su guía guardada");
    });
    document.documentElement.setAttribute("data-eva-mode", on ? "online" : "rules");
    document.querySelectorAll("[data-eva-caption]").forEach(function (c) { if (!c.hasAttribute("data-busy")) c.textContent = idleCaptionText(); });
    var w0 = document.querySelector("[data-wrist]"); if (w0) w0.classList.toggle("no-voice", !(on && canRecord));
  }
  function setStatus(on) {
    var was = online; online = !!on;
    setItem("sessionStorage", STATUS_KEY, JSON.stringify({ on: online, t: Date.now() }));
    paintStatus();
    if (was !== online) post({ type: "status", online: online });
  }
  // A status check that costs no model call: POST { ping: true } answers { ok, live } (an older deploy answers it
  // with a plain guard reply, which still means Eva is there). Cached for 3 minutes in this tab.
  function probe() {
    var url = chatUrl();
    if (!url || navigator.onLine === false || !window.fetch) { setStatus(false); return; }
    var cached = readCached();
    if (cached !== null) { online = cached; paintStatus(); return; }
    var ctl = window.AbortController ? new AbortController() : null;
    var t = setTimeout(function () { if (ctl) ctl.abort(); }, 6000);
    fetch(url, { method: "POST", mode: "cors", credentials: "omit", cache: "no-store", headers: { "content-type": "application/json" }, body: JSON.stringify({ ping: true, text: "" }), signal: ctl ? ctl.signal : undefined })
      .then(function (r) {
        clearTimeout(t);
        if (r.status === 429) return setStatus(true); // busy, but there
        if (!r.ok) return setStatus(false);
        return r.json().then(function (j) { setStatus(!j || j.live !== false); }, function () { setStatus(true); });
      })
      .catch(function () { clearTimeout(t); setStatus(false); });
  }
  paintStatus();
  probe();
  window.addEventListener("online", function () { setItem("sessionStorage", STATUS_KEY, ""); probe(); });
  window.addEventListener("offline", function () { setStatus(false); });

  // ---------------------------------------------------------------- talking to the API: thinking, one retry
  function delay(ms) { return new Promise(function (ok) { setTimeout(ok, ms); }); }
  // resolves { ok: true, r } or { ok: false, kind: "busy" | "offline", r? }; never rejects
  function callApi(name, body, onRetry, timeoutMs) {
    var url = api(name);
    if (!url || navigator.onLine === false || !window.fetch) return Promise.resolve({ ok: false, kind: "offline" });
    var tried = false;
    function once() {
      var ctl = window.AbortController ? new AbortController() : null;
      var t = setTimeout(function () { if (ctl) ctl.abort(); }, timeoutMs || 20000);
      return fetch(url, { method: "POST", mode: "cors", credentials: "omit", headers: { "content-type": "application/json" },
        body: JSON.stringify(body), signal: ctl ? ctl.signal : undefined })
        .then(function (r) { clearTimeout(t); return r; }, function (e) { clearTimeout(t); throw e; });
    }
    function retry(kind, r) {
      if (tried) return { ok: false, kind: kind, r: r };
      tried = true;
      var ra = r ? parseFloat(r.headers.get("retry-after")) : NaN;
      var wait = isFinite(ra) && ra > 0 ? Math.min(ra, 3) : 1.5;
      if (onRetry) onRetry(kind);
      return delay(wait * 1000).then(once).then(handle, function () { return { ok: false, kind: "offline" }; });
    }
    function handle(r) {
      if (r.status === 429) return retry("busy", r);
      if (r.status === 502 || r.status === 503 || r.status === 504) return retry("offline", r);
      return { ok: true, r: r };
    }
    return once().then(handle, function () { return retry("offline"); });
  }
  window.evaReplayCallApi = callApi; // for the checks
  function failText(kind, noFallback) {
    var busy = kind === "busy";
    if (noFallback) return busy ? L("Lots of people are talking to Eva right now. Please try again in a minute.", "Mucha gente está hablando con Eva ahora. Probá de nuevo en un minuto.")
      : L("Eva can't be reached right now. Please try again later.", "No se puede hablar con Eva ahora. Probá más tarde.");
    return busy
      ? L("Lots of people are talking to Eva right now. Here is her saved answer.", "Mucha gente está hablando con Eva ahora. Esta es su respuesta guardada.")
      : L("Eva can't be reached right now. Here is her saved answer.", "No se puede hablar con Eva ahora. Esta es su respuesta guardada.");
  }

  // The demo story's simulated wristband readings (normal-day scenes only), so "How am I today?" gets a real answer.
  var CONTEXT = {
    "4B": { name: "Ana", pulse: 65, pulseLow: 58, pulseHigh: 78, spo2: 98, skin: 34.1, water: 1.2, waterGoal: 2, sleep: 7.2 },
    "3B": { name: "Leo", pulse: 65, pulseLow: 56, pulseHigh: 76, spo2: 98, skin: 33.7, water: 1.4, waterGoal: 2.2 },
    "1B": { name: "Tomás", pulse: 68, pulseLow: 60, pulseHigh: 82, spo2: 98, skin: 33.9, water: 1.1, waterGoal: 2 }
  };
  // Safety net for the simple style (the API does this too once deployed): no "Now: / Why: / How I decided:" labels,
  // no markdown, at most 5 short lines.
  var LABEL = /^\s*(?:[-*\u2022]\s*)?(?:\*\*)?(now|why|how i decided|next|ahora|por qu\u00e9|por que|c\u00f3mo decid\u00ed|como decidi|despu\u00e9s|despues|siguiente)(?:\*\*)?\s*:\s*(?:\*\*)?/i;
  function tidy(text) {
    return text.replace(/\*\*|__|`/g, "").split(/\n+/).map(function (l) { return l.replace(LABEL, "").replace(/^\s*(?:[-*\u2022]|\d+[.)])\s+/, "").replace(/\s+$/, "").trim(); })
      .filter(Boolean).map(function (l) { return l.charAt(0).toUpperCase() + l.slice(1); }).slice(0, 5).join("\n");
  }
  var turns = [];
  // ask Eva; resolves { text } or { error: "busy" | "offline" }
  // The model sometimes answers a question about the visitor's own health with the off-topic line (the guard itself
  // had already said it is a health question): ask once more, saying so plainly.
  var OFF_TOPIC_RE = /I only talk about space travel and health in space|Solo hablo de viajes espaciales/i;
  function ask(text, onRetry, again) {
    var body = { text: text, lang: lang(), seat: seat, style: "simple", history: again ? [] : turns.slice(-4) };
    if ((scene === "s1" || scene === "s5") && CONTEXT[seat]) body.context = CONTEXT[seat];
    return callApi("chat", body, onRetry).then(function (res) {
      if (!res.ok) { if (res.kind === "offline") setStatus(false); return { error: res.kind }; }
      return res.r.json().then(function (j) {
        var t = j && typeof j.text === "string" ? tidy(j.text.trim()) : "";
        if (!t) return { error: "offline" };
        if (!again && j.source === "model" && OFF_TOPIC_RE.test(t)) return ask(L("This is a question about my health: ", "Es una pregunta sobre mi salud: ") + text, onRetry, true);
        setStatus(true);
        turns.push({ role: "user", content: text }, { role: "assistant", content: t });
        return { text: t };
      }, function () { return { error: "offline" }; });
    });
  }
  window.evaReplayAsk = ask; // for the checks

  // Chip labels sent with a little more context, so the topic guard always knows they are about health.
  var ASK_AS = {
    "how am i?": ["How am I today? How is my health?", "¿Cómo estoy hoy? ¿Cómo está mi salud?"],
    "i feel unwell": ["I feel unwell. What should I do about my health?", "Me siento mal. ¿Qué hago por mi salud?"],
    "log my day": ["How is my health today? Tell me briefly about my sleep, water and pulse.", "¿Cómo está mi salud hoy? Contame breve sobre mi sueño, agua y pulso."],
    "summary of my day": ["How is my health today? Tell me briefly about my sleep, water and pulse.", "¿Cómo está mi salud hoy? Contame breve sobre mi sueño, agua y pulso."],
    "show my week": ["How was my health this week: my sleep, water and pulse?", "¿Cómo estuvo mi salud esta semana: sueño, agua y pulso?"]
  };
  function asked(label) {
    var k = ASK_AS[label.toLowerCase()];
    return k ? k[es() ? 1 : 0] : label;
  }
  function thinkingHtml(again) {
    return '<span class="eva-think" role="status"><span>' + (again ? L("Eva is still thinking", "Eva sigue pensando") : L("Eva is thinking", "Eva está pensando")) + "</span><i></i><i></i><i></i></span>";
  }

  // ---------------------------------------------------------------- the chat (phone)
  var chatBox = isWrist ? null : document.getElementById("chat");
  function bubble(text, mine) {
    var d = document.createElement("div");
    if (mine) {
      d.className = "flex max-w-72 flex-col gap-1 self-end px-4 py-3 text-body leading-snug text-ink animate-eva-rise"; d.style.cssText = "border:1px solid #2a2a2a;border-radius:18px 18px 4px 18px;background:#000";
      d.setAttribute("data-mine", "");
      d.textContent = text;
    } else {
      d.className = "flex max-w-full items-end gap-2 self-start animate-eva-rise";
      d.setAttribute("data-eva-live", "");
      d.innerHTML = '<img src="../static/img/eva/eva-face.webp" alt="Eva" class="size-9 shrink-0 object-contain"><div class="flex min-w-0 flex-col gap-1 px-4 py-3 text-body leading-snug text-ink" style="white-space:pre-line;border:1px solid #2a2a2a;border-radius:18px 18px 18px 4px;background:#0b0b0b"></div>';
      d.lastChild.textContent = text;
    }
    chatBox.appendChild(d);
    window.scrollTo(0, document.documentElement.scrollHeight);
    return d;
  }
  function thinkingBubble() {
    var w = bubble("", false);
    w.setAttribute("data-eva-thinking", "");
    w.lastChild.innerHTML = thinkingHtml(false);
    return w;
  }
  function settle(wait, text) {
    wait.removeAttribute("data-eva-thinking");
    wait.lastChild.textContent = text;
    window.scrollTo(0, document.documentElement.scrollHeight);
  }
  // phone: the question and Eva's answer in the chat; without her, the recorded answer page
  function askPhone(label, fallback, sendText) {
    if (!label) return;
    if (!chatUrl() || !chatBox) return go(fallback);
    bubble(label, true);
    var wait = thinkingBubble();
    ask(sendText || asked(label), function () { wait.lastChild.innerHTML = thinkingHtml(true); }).then(function (res) {
      if (res.text) { settle(wait, res.text); speak(res.text); return; }
      wait.remove(); toast(failText(res.error, !fallback), 5000);
      if (fallback) setTimeout(function () { go(fallback); }, 1800);
    });
  }
  // band: Eva's answer in her bubble; without her, the recorded line stays
  var wristBubble = document.querySelector("[data-wrist-bubble]");
  function showOnWrist(text) {
    if (!wristBubble) return;
    wristBubble.classList.add("is-live");
    // a short screen: the first sentences that fit, the rest is spoken
    var parts = text.replace(/\s+/g, " ").match(/[^.!?]+[.!?]*/g) || [text], out = "";
    for (var i = 0; i < parts.length; i++) { if (out && (out + parts[i]).length > 150) break; out += parts[i]; }
    wristBubble.textContent = out.trim();
  }
  function askWrist(label, sendText) {
    if (!wristBubble || !label) return;
    if (!chatUrl()) { toast(L("Eva needs the internet for that. Here is her saved line.", "Eva necesita internet para eso. Esta es su frase guardada.")); return; }
    var was = wristBubble.textContent;
    wristBubble.classList.add("is-live");
    wristBubble.innerHTML = thinkingHtml(false);
    ask(sendText || asked(label), function () { wristBubble.innerHTML = thinkingHtml(true); }).then(function (res) {
      if (res.text) { showOnWrist(res.text); speak(res.text); return; }
      wristBubble.textContent = was; toast(failText(res.error, true), 5000);
    });
  }
  document.addEventListener("submit", function (e) {
    var f = e.target;
    if (!chatBox || !f.matches || !f.matches("[data-eva-form]")) return;
    e.preventDefault(); e.stopImmediatePropagation();
    var inp = f.querySelector("[name=text]"); var t = inp ? inp.value.trim() : "";
    if (inp) inp.value = "";
    askPhone(t, f.getAttribute("data-href"));
  }, true);

  // ---------------------------------------------------------------- Eva speaks
  // Orpheus WAV for English; Spanish (204) or any failure falls back to the browser's voice. Spoken answers are on
  // when the visitor ticked "Voice replies", when they talked to Eva by voice, and always during the guided tour
  // (there the player speaks, so captions and answers never talk over each other).
  var voiceBox = document.querySelector("[data-eva-voice]");
  var lastWasVoice = false;
  function tourOn() { return getItem("sessionStorage", "eva-tour") === "1"; }
  if (voiceBox) {
    voiceBox.checked = tourOn() || getItem("localStorage", "eva-voice") === "1";
    voiceBox.addEventListener("change", function () { setItem("localStorage", "eva-voice", voiceBox.checked ? "1" : "0"); if (!voiceBox.checked) hush(); });
  }
  function voiceOn() { return tourOn() || lastWasVoice || (voiceBox ? voiceBox.checked : false); }
  var voices = [];
  function loadVoices() { try { voices = speechSynthesis.getVoices() || []; } catch (e) {} }
  if (window.speechSynthesis) { loadVoices(); try { speechSynthesis.addEventListener("voiceschanged", loadVoices); } catch (e) {} }
  function bestVoice(want) {
    if (!voices.length) loadVoices();
    var natural = /natural|neural|premium|enhanced|google|siri|samantha|paulina|monica|sabina/i;
    var best = null, bestScore = -1;
    voices.forEach(function (v) {
      var l = (v.lang || "").replace("_", "-");
      var rank = -1;
      for (var i = 0; i < want.length; i++) if (l.toLowerCase().indexOf(want[i].toLowerCase()) === 0) { rank = want.length - i; break; }
      if (rank < 0) return;
      var score = rank * 10 + (natural.test(v.name) ? 5 : 0) + (v.localService === false ? 2 : 0) + (v.default ? 1 : 0);
      if (score > bestScore) { best = v; bestScore = score; }
    });
    return best;
  }
  function browserSay(text) {
    try {
      if (!window.speechSynthesis) return;
      var u = new SpeechSynthesisUtterance(text);
      var want = es() ? ["es-AR", "es-419", "es-US", "es-MX", "es"] : ["en-US", "en-GB", "en"];
      u.voice = bestVoice(want);
      u.lang = u.voice ? u.voice.lang : want[0];
      u.rate = 0.95; u.pitch = 1.0; // a touch slower: easier to follow
      speechSynthesis.cancel(); speechSynthesis.speak(u);
    } catch (e) {}
  }
  var playing = null;
  function hush() {
    if (playing) { try { playing.pause(); } catch (e) {} playing = null; }
    try { if (window.speechSynthesis) speechSynthesis.cancel(); } catch (e) {}
  }
  function speak(text) {
    if (!text || !voiceOn()) return;
    var line = text.replace(/\s+/g, " ").trim().slice(0, 400);
    if (inFrame && tourOn()) { post({ type: "speak", text: line, lang: lang() }); return; }
    callApi("tts", { text: line, lang: es() ? "es" : "en" }, null, 25000)
      .then(function (res) {
        if (!res.ok) throw new Error("tts");
        var ct = res.r.headers.get("content-type") || "";
        if (res.r.status !== 200 || ct.indexOf("audio") < 0) throw new Error("tts " + res.r.status);
        return res.r.blob();
      })
      .then(function (b) {
        hush();
        playing = new Audio(URL.createObjectURL(b));
        post({ type: "spoke", via: "tts" });
        return playing.play();
      })
      .catch(function () { post({ type: "spoke", via: "browser" }); browserSay(line); });
  }

  // one exchange in the chat: the person's bubble (text and/or pictures), then Eva's answer
  function showMine(text, pics) {
    if (!chatBox) return null;
    var d = bubble(text || "", true);
    (pics || []).forEach(function (src) {
      var im = document.createElement("img"); im.src = src; im.alt = "";
      im.style.cssText = "display:block;max-width:100%;max-height:140px;border-radius:10px;margin-top:6px;border:1px solid #2a2a2a";
      d.appendChild(im);
    });
    return d;
  }
  // promise resolves { text } or { error }; thinking first, then the answer (chat or band), or the fallback
  function answer(promise, fallback) {
    var wait = chatBox ? thinkingBubble() : null;
    var was = wristBubble ? wristBubble.textContent : "";
    if (!chatBox && wristBubble) { wristBubble.classList.add("is-live"); wristBubble.innerHTML = thinkingHtml(false); }
    return promise.then(function (res) {
      if (res && res.text) {
        if (wait) settle(wait, res.text);
        if (!chatBox) showOnWrist(res.text);
        speak(res.text);
        return;
      }
      if (wait) wait.remove();
      if (!chatBox && wristBubble) wristBubble.textContent = was;
      toast(failText(res && res.error, !fallback), 5000);
      if (fallback) setTimeout(function () { go(fallback); }, 1800);
    });
  }
  var form = document.querySelector("[data-eva-form]");
  var fallbackHref = form ? form.getAttribute("data-href") : null;

  // ---------------------------------------------------------------- the microphone: tap OR hold to talk
  // Tap: starts listening, tap again to stop. Hold: listens while held, stops on release. Auto-stop at 30 s.
  // Then speech to text (Groq), then Eva. Used by the Eva chat, the band and the journal.
  function pickMime() {
    var c = ["audio/webm;codecs=opus", "audio/ogg;codecs=opus", "audio/mp4", "audio/webm"];
    for (var i = 0; i < c.length; i++) if (MediaRecorder.isTypeSupported && MediaRecorder.isTypeSupported(c[i])) return c[i];
    return "";
  }
  function toBase64(blob) {
    return new Promise(function (ok, bad) {
      var r = new FileReader();
      r.onload = function () { ok(String(r.result).split(",")[1] || ""); };
      r.onerror = bad; r.readAsDataURL(blob);
    });
  }
  var MSG = {
    rec: ["Listening… tap again when you finish", "Escuchando… tocá otra vez al terminar"], recHold: ["Listening… let go when you finish", "Escuchando… soltá al terminar"],
    send: ["Eva is listening to your words…", "Eva está escuchando tus palabras…"], nomic: ["The microphone is not available here. You can type instead.", "El micrófono no está disponible acá. Podés escribir."],
    long: ["That recording is too long. Try a shorter one.", "La grabación es muy larga. Probá una más corta."], heard: ["Eva didn't catch that. Please try again, a little closer.", "Eva no lo entendió. Probá de nuevo, un poco más cerca."],
    look: ["What do you see? Is this a health concern?", "¿Qué ves? ¿Es un problema de salud?"], photo: ["Photo", "Foto"], video: ["Video", "Video"],
    sheet: ["(Four frames from a short video, in order: top left, top right, bottom left, bottom right.)", "(Cuatro cuadros de un video corto, en orden: arriba a la izquierda, arriba a la derecha, abajo a la izquierda, abajo a la derecha.)"]
  };
  function m(k) { return MSG[k][es() ? 1 : 0]; }
  var activeMic = null; // one recording at a time
  function attachMic(btn, opts) {
    var caption = opts.caption || null;
    var rec = null, chunks = [], stopT = 0, tickT = 0, stream = null, startedAt = 0, pointerDown = false, skipClick = false, holdMode = false, opening = false;
    function say(t, busy) {
      if (!caption) return;
      caption.textContent = t;
      if (busy) caption.setAttribute("data-busy", ""); else caption.removeAttribute("data-busy");
    }
    function idle() { say(idleCaptionText(), false); }
    function stopRec() { clearTimeout(stopT); clearInterval(tickT); if (rec && rec.state !== "inactive") rec.stop(); }
    function recording() { return !!rec && rec.state === "recording"; }
    function start() {
      if (opening || recording()) return;
      if (!canRecord) { toast(m("nomic")); return; }
      if (!chatUrl()) { toast(L("Talking to Eva needs the internet. Tap a question instead.", "Hablar con Eva necesita internet. Tocá una pregunta.")); return; }
      if (activeMic && activeMic !== btn) return;
      activeMic = btn; opening = true;
      hush(); post({ type: "hush" });
      navigator.mediaDevices.getUserMedia({ audio: true }).then(function (s) {
        opening = false;
        stream = s; chunks = [];
        var mt = pickMime();
        rec = mt ? new MediaRecorder(s, { mimeType: mt, audioBitsPerSecond: 32000 }) : new MediaRecorder(s);
        rec.ondataavailable = function (ev) { if (ev.data && ev.data.size) chunks.push(ev.data); };
        rec.onstop = function () {
          activeMic = null;
          btn.style.boxShadow = ""; btn.classList.remove("is-recording"); btn.removeAttribute("data-recording");
          if (stream) stream.getTracks().forEach(function (t) { t.stop(); });
          var type = (rec.mimeType || mt || "audio/webm").split(";")[0];
          var blob = new Blob(chunks, { type: type });
          if (blob.size < 200) { idle(); toast(m("heard")); return; }
          if (blob.size > 2.5 * 1024 * 1024) { toast(m("long")); idle(); return; }
          say(m("send"), true);
          lastWasVoice = true;
          toBase64(blob)
            .then(function (b64) { return callApi("stt", { audio: b64, mime: type, lang: es() ? "es" : "en" }, function () { say(m("send"), true); }, 25000); })
            .then(function (res) {
              if (!res.ok) { idle(); if (res.kind === "offline") setStatus(false); toast(failText(res.kind, true), 5000); return; }
              return res.r.json().then(function (j) {
                idle();
                var heard = j && typeof j.text === "string" ? j.text.trim() : "";
                if (!heard) { toast(m("heard")); return; }
                opts.onText(heard);
              });
            })
            .catch(function () { idle(); toast(failText("offline", true), 5000); });
        };
        rec.start(250);
        startedAt = Date.now();
        // released before the microphone opened (a quick tap, or the permission prompt): tap mode
        holdMode = pointerDown;
        btn.style.boxShadow = "0 0 0 3px #1FE5C4"; btn.classList.add("is-recording"); btn.setAttribute("data-recording", "");
        var tick = function () { var s2 = Math.floor((Date.now() - startedAt) / 1000); say(m(holdMode ? "recHold" : "rec") + " 0:" + (s2 < 10 ? "0" : "") + s2, true); };
        tick(); tickT = setInterval(tick, 500);
        stopT = setTimeout(stopRec, 30000);
      }).catch(function () { opening = false; activeMic = null; toast(m("nomic")); });
    }
    function toggle() { if (recording()) stopRec(); else start(); }
    btn.addEventListener("pointerdown", function (e) {
      if (e.button !== undefined && e.button !== 0) return;
      pointerDown = true; skipClick = true;
      if (recording()) { stopRec(); return; }
      start();
    });
    function release() {
      if (!pointerDown) return;
      pointerDown = false;
      // a real hold (the microphone was open and the finger stayed down): stop on release
      if (recording() && holdMode && Date.now() - startedAt > 700) stopRec();
      else holdMode = false;
    }
    btn.addEventListener("pointerup", release);
    btn.addEventListener("pointercancel", release);
    btn.addEventListener("click", function (e) {
      e.preventDefault(); e.stopImmediatePropagation();
      if (skipClick) { skipClick = false; return; } // already handled by the pointer
      toggle(); // keyboard (Enter / Space) and assistive tech
    }, true);
    btn.addEventListener("contextmenu", function (e) { e.preventDefault(); });
    return { start: start, toggle: toggle, stop: stopRec, recording: recording };
  }

  var micBtn = document.querySelector("[data-eva-mic]");
  var mainMic = null;
  if (micBtn) {
    micBtn.setAttribute("aria-label", L("Tap or hold to talk to Eva", "Tocá o mantené para hablar con Eva"));
    mainMic = attachMic(micBtn, {
      caption: document.querySelector("[data-eva-caption]"),
      onText: function (heard) {
        if (chatBox) { showMine(heard); answer(ask(asked(heard)), fallbackHref); }
        else if (wristBubble) answer(ask(asked(heard)), null);
      }
    });
    window.evaReplayMic = mainMic; // for the checks
  }

  // ---------------------------------------------------------------- pictures for Eva's vision
  // downscaled JPEGs drawn on a canvas, never the original file
  function drawJpeg(src, w, h, max, q) {
    var k = Math.min(1, max / Math.max(w, h));
    var c = document.createElement("canvas"); c.width = Math.round(w * k); c.height = Math.round(h * k);
    c.getContext("2d").drawImage(src, 0, 0, c.width, c.height);
    return c.toDataURL("image/jpeg", q);
  }
  function photoFrames(f) {
    return new Promise(function (ok, bad) {
      var url = URL.createObjectURL(f), im = new Image();
      im.onload = function () { try { ok([drawJpeg(im, im.naturalWidth, im.naturalHeight, 1024, 0.8)]); } catch (e) { bad(e); } URL.revokeObjectURL(url); };
      im.onerror = function () { URL.revokeObjectURL(url); bad(new Error("image")); };
      im.src = url;
    });
  }
  // A video becomes ONE picture: 4 evenly spaced frames (each at most 768 px) on a 2 × 2 sheet, at most 1024 px,
  // so the request stays small and works with a vision model that takes a single image.
  function videoFrames(f) {
    return new Promise(function (ok, bad) {
      var url = URL.createObjectURL(f), v = document.createElement("video"), frames = [], i = 0, times = [], fin = false;
      v.muted = true; v.playsInline = true; v.preload = "auto";
      var done = function (err) {
        if (fin) return; fin = true; URL.revokeObjectURL(url);
        if (!frames.length) return bad(err || new Error("video"));
        var cw = frames[0].width, ch = frames[0].height, k = Math.min(1, 1024 / Math.max(cw * 2, ch * 2));
        var c = document.createElement("canvas"); c.width = Math.round(cw * 2 * k); c.height = Math.round(ch * 2 * k);
        var g = c.getContext("2d"); g.fillStyle = "#000"; g.fillRect(0, 0, c.width, c.height);
        frames.forEach(function (fr, n) { g.drawImage(fr, (n % 2) * cw * k, Math.floor(n / 2) * ch * k, cw * k, ch * k); });
        ok([c.toDataURL("image/jpeg", 0.8)]);
      };
      v.onerror = function () { done(new Error("video")); };
      v.onloadedmetadata = function () {
        var d = isFinite(v.duration) && v.duration > 0 ? v.duration : 1;
        for (var n = 0; n < 4; n++) times.push(Math.min(d - 0.05, d * (n + 0.5) / 4));
        v.currentTime = Math.max(0, times[0]);
      };
      v.onseeked = function () {
        try {
          var kk = Math.min(1, 768 / Math.max(v.videoWidth, v.videoHeight));
          var fr = document.createElement("canvas"); fr.width = Math.round(v.videoWidth * kk); fr.height = Math.round(v.videoHeight * kk);
          fr.getContext("2d").drawImage(v, 0, 0, fr.width, fr.height); frames.push(fr);
        } catch (e) { return done(e); }
        i++;
        if (i < times.length) v.currentTime = Math.max(0, times[i]); else done();
      };
      setTimeout(function () { done(new Error("timeout")); }, 20000);
      v.src = url;
    });
  }
  function askVision(frames, label, isVideo) {
    var inp = document.getElementById("eva-text");
    var typed = inp && inp.value.trim();
    if (inp) inp.value = "";
    var q = (typed || m("look")) + (isVideo ? " " + m("sheet") : "");
    showMine((typed ? typed + " · " : "") + label, frames.slice(0, 1));
    return answer(callApi("vision", { text: q, lang: es() ? "es" : "en", style: "simple", images: frames.slice(0, 4) }, null, 35000)
      .then(function (res) {
        if (!res.ok) return { error: res.kind };
        return res.r.json().then(function (j) { return j && typeof j.text === "string" && j.text.trim() ? { text: tidy(j.text.trim()) } : { error: "offline" }; }, function () { return { error: "offline" }; });
      }), null);
  }
  document.addEventListener("change", function (e) {
    var f = e.target;
    if (!f.matches || !f.matches("[data-eva-photo], [data-eva-video]")) return;
    var picked = f.files && f.files[0];
    f.value = "";
    if (!picked) return;
    if (!api("vision")) { toast(L("Photos need Eva online.", "Las fotos necesitan a Eva en línea.")); return; }
    var video = f.matches("[data-eva-video]");
    (video ? videoFrames(picked) : photoFrames(picked))
      .then(function (frames) { return askVision(frames, m(video ? "video" : "photo"), video); })
      .catch(function () { toast(L("Eva couldn't open that file. Try another photo.", "Eva no pudo abrir ese archivo. Probá con otra foto.")); });
  }, true);

  // ---------------------------------------------------------------- reply cards (journal, check-in)
  function replyCard(after, id) {
    var old = document.getElementById(id); if (old) old.remove();
    var c = document.createElement("div");
    c.id = id; c.className = "eva-reply animate-eva-rise"; c.setAttribute("aria-live", "polite");
    c.innerHTML = '<img src="../static/img/eva/eva-face.webp" alt="Eva"><div><p></p></div>';
    c.querySelector("p").innerHTML = thinkingHtml(false);
    after.parentNode.insertBefore(c, after.nextSibling);
    try { c.scrollIntoView({ block: "nearest", behavior: "smooth" }); } catch (e) {}
    return c;
  }
  function addSmall(card, text) { var s = document.createElement("small"); s.textContent = text; card.querySelector("div").appendChild(s); }
  function fillCard(card, promise, offlineLine, note) {
    return promise.then(function (res) {
      var p = card.querySelector("p");
      if (res && res.text) { p.textContent = res.text; card.setAttribute("data-live", ""); speak(res.text); }
      else {
        p.textContent = offlineLine;
        if (res && res.error === "busy") addSmall(card, L("Eva is busy right now. Try again in a minute.", "Eva está ocupada. Probá de nuevo en un minuto."));
      }
      if (note) addSmall(card, note);
    });
  }
  var SAVED_DEMO = ["Saved in your journal (demo).", "Guardado en tu diario (demo)."];
  var SAVED_LATER = ["Saved in your journal (demo). Eva will answer when she is back online.", "Guardado en tu diario (demo). Eva te va a responder cuando vuelva a estar en línea."];

  // the journal: Save sends the note (typed or spoken) to Eva, and her reply appears under it
  var journal = document.querySelector("form[data-journal]");
  var jtext = journal ? journal.querySelector("textarea") : null;
  function sendJournal() {
    var word = ((document.querySelector("[data-mood-word]") || {}).textContent || "").trim();
    var pick = function (n) { return Array.prototype.slice.call(journal.querySelectorAll("input[name=" + n + "]:checked")).map(function (x) { return x.value; }); };
    var feels = pick("labels"), affects = pick("affects"), note = jtext ? jtext.value.trim().slice(0, 400) : "";
    var text = es()
      ? "Nota de mi diario. Me siento " + word.toLowerCase() + (feels.length ? " (" + feels.join(", ") + ")" : "") + "." + (affects.length ? " Me afecta: " + affects.join(", ") + "." : "") + (note ? " " + note : "")
      : "My journal note. I feel " + word.toLowerCase() + (feels.length ? " (" + feels.join(", ") + ")" : "") + "." + (affects.length ? " What affects me: " + affects.join(", ") + "." : "") + (note ? " " + note : "");
    var save = journal.querySelector("button[type=submit]");
    var card = replyCard(save || journal.lastElementChild, "eva-journal-reply");
    if (!chatUrl()) { card.querySelector("p").textContent = L(SAVED_DEMO[0], SAVED_DEMO[1]); return; }
    fillCard(card, ask(text), L(SAVED_LATER[0], SAVED_LATER[1]));
  }
  if (journal && jtext && chatUrl()) {
    var talkRow = document.createElement("div");
    talkRow.style.cssText = "display:flex;flex-direction:column;align-items:flex-start;gap:.4rem";
    talkRow.innerHTML = '<button type="button" class="eva-talk" data-journal-mic><svg viewBox="0 0 24 24" aria-hidden="true"><rect x="9" y="3" width="6" height="11" rx="3"></rect><path d="M5 11a7 7 0 0 0 14 0M12 18v3"></path></svg><span></span></button><p class="text-caption text-muted" data-journal-caption></p>';
    talkRow.querySelector("span").textContent = L("Say it instead", "Decilo en voz alta");
    jtext.parentNode.insertBefore(talkRow, jtext.nextSibling);
    var jcap = talkRow.querySelector("[data-journal-caption]");
    var jcapIdle = L("Tap, talk, then tap again. Eva writes it down and answers.", "Tocá, hablá y tocá otra vez. Eva lo escribe y te responde.");
    jcap.textContent = jcapIdle;
    var jmic = attachMic(talkRow.querySelector("[data-journal-mic]"), {
      caption: null,
      onText: function (heard) { jtext.value = (jtext.value ? jtext.value.trim() + " " : "") + heard; sendJournal(); }
    });
    window.evaReplayJournalMic = jmic; // for the checks
  }
  // registered before the generic submit handler below, so the demo toast does not show for the journal
  document.addEventListener("submit", function (e) {
    if (!journal || e.target !== journal) return;
    e.preventDefault(); e.stopImmediatePropagation();
    sendJournal();
  }, true);

  // the check-in: Good / The same / Worse goes to Eva, and her reply appears under the buttons
  var CHECKIN = {
    good: ["Check-in about my health: I feel good today.", "Control de mi salud: hoy me siento bien."],
    "the same": ["Check-in about my health: I feel the same as before, not better.", "Control de mi salud: me siento igual que antes, no mejor."],
    worse: ["Check-in about my health: I feel worse than before.", "Control de mi salud: me siento peor que antes."]
  };
  function checkIn(b) {
    var key = ((b.querySelector(".text-lg") || b).textContent || "").trim().toLowerCase();
    var q = (CHECKIN[key] || CHECKIN.good)[es() ? 1 : 0];
    var card = replyCard(b.parentNode, "eva-checkin-reply");
    var note = key === "worse" ? L("In the real app, the crew lead is told now.", "En la app real, se avisa ahora al jefe de tripulación.") : "";
    if (!chatUrl()) { card.querySelector("p").textContent = L(SAVED_DEMO[0], SAVED_DEMO[1]); if (note) addSmall(card, note); return; }
    fillCard(card, ask(q), L(SAVED_LATER[0], SAVED_LATER[1]), note);
  }

  // ---------------------------------------------------------------- chips and buttons that ask Eva
  var VITAL_ASK = {
    heart: ["Is my pulse okay?", "¿Mi pulso está bien?"], lungs: ["Is my blood oxygen okay?", "¿Mi oxígeno en sangre está bien?"],
    temperature: ["Is my skin temperature okay?", "¿Mi temperatura de la piel está bien?"], sleep: ["How did I sleep last night?", "¿Cómo dormí anoche?"],
    water: ["Am I drinking enough water?", "¿Estoy tomando suficiente agua?"]
  };
  var evaPage = "s1-" + seat + "-eva.html";
  function toEva(hash) { go(evaPage + hash); }
  // the home box: say plainly what it does, and make its microphone a real target
  document.querySelectorAll("a.card-hero[aria-label='Ask Eva']").forEach(function (a) {
    a.setAttribute("data-eva-hero", "");
    a.setAttribute("aria-label", L("Ask Eva: how am I today?", "Preguntale a Eva: ¿cómo estoy hoy?"));
    var pill = a.querySelector("span.h-12");
    if (pill) {
      pill.style.height = "3.5rem";
      var mic = pill.querySelector("span.size-10");
      if (mic) { mic.setAttribute("data-hero-mic", ""); mic.style.width = mic.style.height = "3rem"; mic.removeAttribute("aria-hidden"); mic.setAttribute("role", "button"); mic.setAttribute("aria-label", L("Talk to Eva", "Hablar con Eva")); }
    }
  });
  document.addEventListener("click", function (e) {
    var t = e.target;
    if (!t.closest) return;
    // Eva chat chips: questions go to Eva live
    var c = t.closest("button.chip[data-href]");
    if (c && chatBox) {
      e.preventDefault(); e.stopImmediatePropagation();
      askPhone(c.textContent.replace(/\s+/g, " ").trim(), c.getAttribute("data-href"));
      return;
    }
    var tool = t.closest("button.chip[data-tool]");
    if (tool && chatBox) {
      var name = tool.getAttribute("data-tool"), label = tool.textContent.replace(/\s+/g, " ").trim();
      if (name === "daily_summary" || name === "show_screen") { e.preventDefault(); e.stopImmediatePropagation(); askPhone(label, null); return; }
      if (name === "start_ecg_check" && seat === "4B") { e.preventDefault(); e.stopImmediatePropagation(); go("s1-4B-ecg.html"); return; }
    }
    var ci = t.closest("button[data-tool=checkin]");
    if (ci) { e.preventDefault(); e.stopImmediatePropagation(); checkIn(ci); return; }
    // the band's chips
    var w = t.closest("button.w-chip");
    if (w && wristBubble && !w.hasAttribute("data-tool")) {
      e.preventDefault(); e.stopImmediatePropagation();
      askWrist(w.textContent.replace(/\s+/g, " ").trim());
      return;
    }
    // home: the "How am I today?" box asks Eva; its microphone opens Eva listening
    var hero = t.closest("a[data-eva-hero]");
    if (hero) {
      e.preventDefault(); e.stopImmediatePropagation();
      if (!chatUrl()) { var hh = hero.getAttribute("href"); go(hh && hh !== "#" ? hh : evaPage); return; }
      toEva(t.closest("[data-hero-mic]") ? "#talk" : "#ask=" + encodeURIComponent(L("How am I today?", "¿Cómo estoy hoy?")));
      return;
    }
    // vitals: "Ask Eva about it"
    var about = t.closest("a.btn-secondary[href$='-eva.html']");
    if (about && chatUrl()) {
      var kind = (file.match(/vitals-([a-z]+)\.html$/) || [])[1];
      if (kind && VITAL_ASK[kind]) { e.preventDefault(); e.stopImmediatePropagation(); toEva("#ask=" + encodeURIComponent(VITAL_ASK[kind][es() ? 1 : 0])); }
    }
  }, true);

  // arriving on the Eva chat with a question (#ask=...) or ready to listen (#talk)
  if (chatBox && /^#(ask=|talk)/.test(location.hash)) {
    var h = location.hash;
    try { history.replaceState(null, "", location.pathname + location.search); } catch (e) {}
    var arrive = function () {
      if (h.indexOf("#ask=") === 0) { var q = decodeURIComponent(h.slice(5)).slice(0, 300); if (q) askPhone(q, fallbackHref); }
      else if (mainMic) { lastWasVoice = true; mainMic.start(); }
    };
    if (document.readyState === "complete") setTimeout(arrive, 150); else window.addEventListener("load", function () { setTimeout(arrive, 150); });
  }

  // ---------------------------------------------------------------- the band's pages
  // The app's wrist.js is not in the replay: page dots, arrows, "talk to Eva" and the start page here.
  var track = document.querySelector("[data-wrist-track]");
  var pages = track ? Array.prototype.slice.call(track.querySelectorAll(".w-page")) : [];
  var cur = 0;
  function setCur(i) {
    cur = i;
    var tints = { eva: "#1FE5C4", heart: "#F28A1A", brand: "#F28A1A" };
    var w = document.querySelector("[data-wrist]");
    document.querySelectorAll(".w-dots > i").forEach(function (d, k) { d.classList.toggle("on", k === i); });
    if (w && pages[i]) w.style.setProperty("--dot", tints[pages[i].getAttribute("data-tint")] || "#1FE5C4");
    var prev = document.querySelector("[data-wrist-step='-1']"), next = document.querySelector("[data-wrist-step='1']");
    if (prev) prev.hidden = i === 0;
    if (next) next.hidden = i === pages.length - 1;
  }
  function page(i, smooth) {
    if (!track) return;
    i = Math.max(0, Math.min(pages.length - 1, i));
    track.scrollTo({ left: i * track.clientWidth, behavior: smooth ? "smooth" : "auto" });
    setCur(i);
  }
  function evaPageIndex() { for (var i = 0; i < pages.length; i++) if (pages[i].querySelector("[data-eva-mic]")) return i; return -1; }
  if (track) {
    var start = parseInt((document.querySelector("[data-wrist]") || track).getAttribute("data-start"), 10) || 0;
    var talkNow = /^#talk/.test(location.hash) && evaPageIndex() >= 0;
    if (talkNow) { start = evaPageIndex(); try { history.replaceState(null, "", location.pathname + location.search); } catch (e) {} }
    var startNow = function () { page(start, false); if (talkNow && mainMic) { lastWasVoice = true; mainMic.start(); } };
    if (document.readyState === "complete") startNow(); else window.addEventListener("load", startNow);
    track.addEventListener("scroll", function () {
      var i = Math.round(track.scrollLeft / Math.max(1, track.clientWidth));
      if (i !== cur) setCur(i);
    }, { passive: true });
    document.addEventListener("click", function (e) {
      var s = e.target.closest && e.target.closest("[data-wrist-step]");
      if (s) { e.preventDefault(); e.stopImmediatePropagation(); page(cur + parseInt(s.getAttribute("data-wrist-step"), 10), true); return; }
      var g = e.target.closest && e.target.closest("[data-wrist-go]");
      if (g) { e.preventDefault(); e.stopImmediatePropagation(); page(parseInt(g.getAttribute("data-wrist-go"), 10), true); return; }
      var v = e.target.closest && e.target.closest(".w-vrow");
      if (v && cur === 0) { e.preventDefault(); e.stopImmediatePropagation(); page(1, true); return; }
      var p = e.target.closest && e.target.closest("[data-panel]");
      if (p) {
        e.preventDefault(); e.stopImmediatePropagation();
        var open = !p.classList.contains("is-open");
        document.querySelectorAll("[data-panel]").forEach(function (x) { x.classList.toggle("is-open", open && x === p); });
      }
    }, true);
  }

  // ---------------------------------------------------------------- navigation and story
  var TOOL_SAID = {
    log_water: ["Done: one more glass of water. (Demo: nothing is saved.)", "Listo: un vaso de agua más. (Demo: no se guarda nada.)"],
    request_help: ["On a real wristband, holding Help calls the nearest passenger and the crew. See it in the scene 'Ana's heart stops'.", "En una pulsera real, mantener Ayuda llama al pasajero más cercano y a la tripulación. Miralo en la escena 'Se detiene el corazón de Ana'."]
  };
  document.addEventListener("click", function (e) {
    var t = e.target.closest && e.target.closest("[data-story], [data-href], [data-replay]");
    if (!t || t.tagName === "FORM") return;
    e.preventDefault();
    e.stopPropagation();
    if (t.hasAttribute("data-story")) {
      if (inFrame) post({ type: "story", action: t.getAttribute("data-story") });
      else toast(L("Open the demo's start page to follow the story.", "Abrí la página de inicio de la demo para seguir la historia."));
      return;
    }
    if (t.hasAttribute("data-href")) return go(t.getAttribute("data-href"));
    var kind = t.getAttribute("data-replay");
    var said = TOOL_SAID[t.getAttribute("data-tool")];
    if (said) toast(said[es() ? 1 : 0], 5500);
    else if (kind === "missing") toast(L("That screen is not part of this demo step.", "Esa pantalla no es parte de este paso de la demo."));
    else if (kind === "live-only") toast(L("Downloads come from the real app. This demo only shows them.", "Las descargas vienen de la app real. Esta demo solo las muestra."));
    else demoToast();
  }, true);

  document.addEventListener("submit", function (e) {
    var f = e.target;
    e.preventDefault();
    if (f.hasAttribute("data-story")) return post({ type: "story", action: f.getAttribute("data-story") });
    if (f.hasAttribute("data-href")) return go(f.getAttribute("data-href"));
    demoToast();
  }, true);

  document.addEventListener("change", function (e) {
    var s = e.target;
    if (!s.matches || !s.matches("[data-replay-seat]")) return;
    var picked = s.value, was = document.body.getAttribute("data-seat");
    if (["4B", "3B", "1B"].indexOf(picked) >= 0 && inFrame) post({ type: "seat", seat: picked });
    else toast(L("This demo follows Ana (4B), Leo (3B) and Tomás (1B).", "Esta demo sigue a Ana (4B), Leo (3B) y Tomás (1B)."));
    if (was) s.value = was;
  });

  // arrows: the band's pages on the band; the player's scenes everywhere else
  document.addEventListener("keydown", function (e) {
    if (e.target.matches && e.target.matches("input, textarea, select")) return;
    if (e.key !== "ArrowLeft" && e.key !== "ArrowRight") return;
    if (track) { e.preventDefault(); page(cur + (e.key === "ArrowRight" ? 1 : -1), true); return; }
    if (inFrame) post({ type: "key", key: e.key });
  });

  // like the app after Eva answers: keep the newest line in view above the dock
  if (/eva-answer\.html$/.test(location.pathname)) {
    window.addEventListener("load", function () {
      // scroll this document only (scrollIntoView would also move the player page)
      window.scrollTo(0, document.documentElement.scrollHeight);
    });
  }

  // ---------------------------------------------------------------- messages from the player
  window.addEventListener("message", function (e) {
    var d = e.data;
    if (!d || d.eva !== "launcher") return;
    if (inFrame && e.source !== window.parent) return;
    if (d.type === "ts") applyTs(d.ts);
    else if (d.type === "lang") paintStatus();
    else if (d.type === "hush") hush();
    else if (d.type === "ask" && typeof d.text === "string") {
      var q = d.text.slice(0, 300);
      if (chatBox) askPhone(q, fallbackHref);
      else if (wristBubble) { var k = evaPageIndex(); if (k >= 0) page(k, true); askWrist(q); }
    } else if (d.type === "talk") {
      if (mainMic) {
        var k2 = evaPageIndex(); if (track && k2 >= 0) page(k2, true);
        lastWasVoice = true; mainMic.start();
      } else if (isWrist) go(scene + "-" + seat + "-wrist.html#talk");
      else toEva("#talk");
    }
  });

  post({ type: "view", file: file, wrist: isWrist, mic: !!micBtn });
})();
