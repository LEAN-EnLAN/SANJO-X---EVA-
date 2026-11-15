// Eva static replay glue. Runs inside every captured screen instead of htmx, SSE and the
// Eva Link socket: links open other captured screens, story buttons tell the player to move
// the scenario on, and everything that would write to the ship says so in a toast.
// One file for every place the replay runs: the hosted site, the Android app's bundled copy
// (https://appassets.androidplatform.net) and a local copy (file:// or http://localhost).
(function () {
  "use strict";
  var inFrame = window.parent && window.parent !== window;
  var stateId = document.body.getAttribute("data-replay-state") || "";
  var isWrist = document.body.getAttribute("data-replay-view") === "wrist";
  var seat = (location.pathname.match(/-(\d[A-Z])-/) || [])[1] || "4B";
  function post(msg) {
    msg.eva = "replay"; msg.state = stateId;
    if (inFrame) { try { window.parent.postMessage(msg, "*"); } catch (e) {} }
  }
  var timer = 0;
  function toast(text) {
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
    timer = setTimeout(function () { host.innerHTML = ""; }, 2600);
  }
  var LIVE = "Static replay · run eva demo to do this live";
  function go(href) { if (href) location.href = href; }
  function lang() { try { return (window.parent.document.documentElement.lang || "en").slice(0, 2); } catch (e) { return "en"; } }

  // ---------------------------------------------------------------- live Eva
  // Where Eva's chat lives, or null for the captured answers only:
  // - the hosted site: its own /api/eva/chat (same topic guard as the app);
  // - the Android app's bundled copy: the hosted API by absolute URL (it allows that origin by CORS);
  // - file:// and plain http (a local copy): no API, captured answers.
  var HOSTED_API = "https://eva-wearmeds.vercel.app/api/eva/chat";
  var ASSET_HOST = "appassets.androidplatform.net";
  function chatUrl(loc) {
    loc = loc || location;
    if (loc.protocol !== "https:") return null;
    return loc.hostname === ASSET_HOST ? HOSTED_API : "/api/eva/chat";
  }
  window.evaReplayChatUrl = chatUrl; // read by the checks; changes nothing
  var OFFLINE_NOTE = { en: "Eva is offline here · recorded answer", es: "Eva está sin conexión · respuesta grabada" };
  function offlineToast() { toast(OFFLINE_NOTE[lang() === "es" ? "es" : "en"]); }

  // ask Eva; resolves with her text, or null (no API, no network, error, or an empty answer)
  function ask(text) {
    var url = chatUrl();
    if (!url || navigator.onLine === false || !window.fetch) return Promise.resolve(null);
    var ctl = window.AbortController ? new AbortController() : null;
    var t = setTimeout(function () { if (ctl) ctl.abort(); }, 15000);
    return fetch(url, { method: "POST", mode: "cors", credentials: "omit", headers: { "content-type": "application/json" },
      body: JSON.stringify({ text: text, lang: lang(), seat: seat }), signal: ctl ? ctl.signal : undefined })
      .then(function (r) { return r.json(); })
      .then(function (j) { clearTimeout(t); return j && typeof j.text === "string" && j.text.trim() ? j.text : null; })
      .catch(function () { clearTimeout(t); return null; });
  }

  var chatBox = document.getElementById("chat");
  function bubble(text, mine) {
    var d = document.createElement("div");
    if (mine) {
      d.className = "flex max-w-64 flex-col gap-1 self-end px-3.5 py-2.5 text-sm leading-snug text-ink animate-eva-rise"; d.style.cssText = "border:1px solid #2a2a2a;border-radius:16px 16px 4px 16px;background:#000";
      d.textContent = text;
    } else {
      d.className = "flex max-w-full items-end gap-2 self-start animate-eva-rise";
      d.innerHTML = '<img src="../static/img/eva/eva-face.webp" alt="Eva" class="size-8 shrink-0 object-contain"><div class="flex min-w-0 flex-col gap-1 px-3.5 py-2.5 text-sm leading-snug text-ink" style="white-space:pre-line;border:1px solid #2a2a2a;border-radius:16px 16px 16px 4px;background:#0b0b0b"></div>';
      d.lastChild.textContent = text;
    }
    chatBox.appendChild(d);
    window.scrollTo(0, document.documentElement.scrollHeight);
    return d;
  }
  // phone: the question and Eva's answer in the chat; without her, the captured answer page
  function askPhone(text, fallbackHref) {
    if (!text || !chatUrl()) return go(fallbackHref);
    bubble(text, true);
    var wait = bubble("…", false);
    ask(text).then(function (answer) {
      if (answer) { wait.lastChild.textContent = answer; speak(answer); return; }
      wait.remove(); offlineToast(); setTimeout(function () { go(fallbackHref); }, 700);
    });
  }
  // band: Eva's answer in her bubble; without her, the captured line stays
  var wristBubble = document.querySelector("[data-wrist-bubble]");
  function askWrist(text) {
    if (!wristBubble || !text) return;
    if (!chatUrl()) { toast(LIVE); return; }
    var was = wristBubble.textContent;
    wristBubble.textContent = "…";
    ask(text).then(function (answer) {
      if (answer) { wristBubble.textContent = answer.split("\n").slice(0, 2).join("\n"); wristBubble.style.whiteSpace = "pre-line"; speak(answer); return; }
      wristBubble.textContent = was; offlineToast();
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
  document.addEventListener("click", function (e) {
    var c = e.target.closest && e.target.closest("button.chip[data-href]");
    if (c && chatBox) {
      e.preventDefault(); e.stopImmediatePropagation();
      askPhone(c.textContent.replace(/\s+/g, " ").trim(), c.getAttribute("data-href"));
      return;
    }
    var w = e.target.closest && e.target.closest("button.w-chip");
    if (w && wristBubble && !w.hasAttribute("data-tool")) {
      e.preventDefault(); e.stopImmediatePropagation();
      askWrist(w.textContent.replace(/\s+/g, " ").trim());
    }
  }, true);

  // ---------------------------------------------------------------- voice, photo and video (Groq, through the site)
  // Same place as the chat: the hosted /api/eva/*, or the hosted URL from the app's bundled copy.
  function api(name) { var u = chatUrl(); return u ? u.replace(/chat$/, name) : null; }
  function es() { return lang() === "es"; }
  var MSG = {
    tap: ["Tap to talk", "Tocá para hablar"], rec: ["Listening · tap to stop", "Escuchando · tocá para terminar"],
    send: ["Sending to Eva…", "Enviando a Eva…"], nomic: ["The microphone is not available here", "El micrófono no está disponible acá"],
    long: ["That recording is too long", "La grabación es muy larga"], heard: ["Eva didn't catch that · try again", "Eva no lo entendió · probá de nuevo"],
    look: ["What do you see? Is this a health concern?", "¿Qué ves? ¿Es un problema de salud?"], photo: ["Photo", "Foto"], video: ["Video", "Video"],
    sheet: ["(Four frames from a short video, in order: top left, top right, bottom left, bottom right.)", "(Cuatro cuadros de un video corto, en orden: arriba a la izquierda, arriba a la derecha, abajo a la izquierda, abajo a la derecha.)"]
  };
  function m(k) { return MSG[k][es() ? 1 : 0]; }
  function post_(name, body) {
    var url = api(name);
    if (!url || navigator.onLine === false || !window.fetch) return Promise.reject(new Error("offline"));
    return fetch(url, { method: "POST", mode: "cors", credentials: "omit", headers: { "content-type": "application/json" }, body: JSON.stringify(body) });
  }
  function toBase64(blob) {
    return new Promise(function (ok, bad) {
      var r = new FileReader();
      r.onload = function () { ok(String(r.result).split(",")[1] || ""); };
      r.onerror = bad; r.readAsDataURL(blob);
    });
  }

  // Eva speaks: Orpheus WAV for English; Spanish (204) or any failure falls back to the browser's voice
  var voiceBox = document.querySelector("[data-eva-voice]");
  var lastWasVoice = false;
  function voiceOn() { return voiceBox ? voiceBox.checked : isWrist && lastWasVoice; }
  // The browser's own voice (the normal path while the site's TTS answers 204): the most natural voice it has for
  // the language, at the normal rate. Voices load late on some engines, so ask early.
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
      u.rate = 1.0; u.pitch = 1.0;
      speechSynthesis.cancel(); speechSynthesis.speak(u);
    } catch (e) {}
  }
  var playing = null;
  function speak(text) {
    if (!text || !voiceOn()) return;
    var line = text.replace(/\s+/g, " ").trim().slice(0, 400);
    post_("tts", { text: line, lang: es() ? "es" : "en" })
      .then(function (r) {
        var ct = r.headers.get("content-type") || "";
        if (r.status !== 200 || ct.indexOf("audio") < 0) throw new Error("tts " + r.status);
        return r.blob();
      })
      .then(function (b) {
        if (playing) { try { playing.pause(); } catch (e) {} }
        playing = new Audio(URL.createObjectURL(b));
        return playing.play();
      })
      .catch(function () { browserSay(line); });
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
  function answer(promise, fallbackHref) {
    var wait = chatBox ? bubble("…", false) : null;
    if (!chatBox && wristBubble) wristBubble.textContent = "…";
    return promise.then(function (text) {
      if (!text) throw new Error("empty");
      if (wait) wait.lastChild.textContent = text;
      if (!chatBox && wristBubble) { wristBubble.textContent = text.split("\n").slice(0, 2).join("\n"); wristBubble.style.whiteSpace = "pre-line"; }
      speak(text);
    }).catch(function () {
      if (wait) wait.remove();
      offlineToast();
      if (fallbackHref) setTimeout(function () { go(fallbackHref); }, 700);
    });
  }
  function chatText(text) { return ask(text).then(function (t) { if (!t) throw new Error("no answer"); return t; }); }
  var form = document.querySelector("[data-eva-form]");
  var fallbackHref = form ? form.getAttribute("data-href") : null;

  // the mic: tap to start, tap to stop (auto-stop at 30 s), then speech to text, then Eva
  var micBtn = document.querySelector("[data-eva-mic]");
  var caption = document.querySelector("[data-eva-caption]");
  var canRecord = !!(navigator.mediaDevices && navigator.mediaDevices.getUserMedia && window.MediaRecorder);
  function say(t) { if (caption) caption.textContent = t; }
  if (micBtn && chatUrl()) {
    if (canRecord) {
      say(m("tap")); micBtn.setAttribute("aria-label", m("tap"));
      var w0 = document.querySelector("[data-wrist]"); if (w0) w0.classList.remove("no-voice");
    }
    var rec = null, chunks = [], stopT = 0, tickT = 0, stream = null;
    var ring = "0 0 0 2px #1FE5C4";
    function stopRec() { clearTimeout(stopT); clearInterval(tickT); if (rec && rec.state !== "inactive") rec.stop(); }
    function pickMime() {
      var c = ["audio/webm;codecs=opus", "audio/ogg;codecs=opus", "audio/mp4", "audio/webm"];
      for (var i = 0; i < c.length; i++) if (MediaRecorder.isTypeSupported && MediaRecorder.isTypeSupported(c[i])) return c[i];
      return "";
    }
    micBtn.addEventListener("click", function (e) {
      e.preventDefault(); e.stopImmediatePropagation();
      if (rec && rec.state === "recording") { stopRec(); return; }
      if (!canRecord) { toast(m("nomic")); return; }
      navigator.mediaDevices.getUserMedia({ audio: true }).then(function (s) {
        stream = s; chunks = [];
        var mt = pickMime();
        rec = mt ? new MediaRecorder(s, { mimeType: mt, audioBitsPerSecond: 32000 }) : new MediaRecorder(s);
        rec.ondataavailable = function (ev) { if (ev.data && ev.data.size) chunks.push(ev.data); };
        rec.onstop = function () {
          micBtn.style.boxShadow = ""; micBtn.classList.remove("is-recording"); micBtn.removeAttribute("data-recording");
          if (stream) stream.getTracks().forEach(function (t) { t.stop(); });
          var type = (rec.mimeType || mt || "audio/webm").split(";")[0];
          var blob = new Blob(chunks, { type: type });
          say(m("send"));
          if (blob.size > 2.5 * 1024 * 1024) { toast(m("long")); say(m("tap")); return; }
          lastWasVoice = true;
          toBase64(blob)
            .then(function (b64) { return post_("stt", { audio: b64, mime: type, lang: es() ? "es" : "en" }); })
            .then(function (r) { return r.json(); })
            .then(function (j) {
              say(m("tap"));
              var heard = j && typeof j.text === "string" ? j.text.trim() : "";
              if (!heard) { toast(m("heard")); return; }
              showMine(heard);
              return answer(chatText(heard), fallbackHref);
            })
            .catch(function () { say(m("tap")); offlineToast(); });
        };
        rec.start(250);
        micBtn.style.boxShadow = ring; micBtn.classList.add("is-recording"); micBtn.setAttribute("data-recording", "");
        var t0 = Date.now();
        say(m("rec") + " · 0:00");
        tickT = setInterval(function () { var s2 = Math.floor((Date.now() - t0) / 1000); say(m("rec") + " · 0:" + (s2 < 10 ? "0" : "") + s2); }, 500);
        stopT = setTimeout(stopRec, 30000);
      }).catch(function () { toast(m("nomic")); });
    }, true);
  }

  // pictures for Eva's vision: downscaled JPEGs drawn on a canvas, never the original file
  function drawJpeg(src, w, h, max, q) {
    var k = Math.min(1, max / Math.max(w, h));
    var c = document.createElement("canvas"); c.width = Math.round(w * k); c.height = Math.round(h * k);
    c.getContext("2d").drawImage(src, 0, 0, c.width, c.height);
    return c.toDataURL("image/jpeg", q);
  }
  function photoFrames(file) {
    return new Promise(function (ok, bad) {
      var url = URL.createObjectURL(file), im = new Image();
      im.onload = function () { try { ok([drawJpeg(im, im.naturalWidth, im.naturalHeight, 1024, 0.8)]); } catch (e) { bad(e); } URL.revokeObjectURL(url); };
      im.onerror = function () { URL.revokeObjectURL(url); bad(new Error("image")); };
      im.src = url;
    });
  }
  // A video becomes ONE picture: 4 evenly spaced frames (each at most 768 px) on a 2 × 2 sheet, at most 1024 px,
  // so the request stays small and works with a vision model that takes a single image.
  function videoFrames(file) {
    return new Promise(function (ok, bad) {
      var url = URL.createObjectURL(file), v = document.createElement("video"), frames = [], i = 0, times = [], fin = false;
      v.muted = true; v.playsInline = true; v.preload = "auto";
      var done = function (err) {
        if (fin) return; fin = true; URL.revokeObjectURL(url);
        if (!frames.length) return bad(err || new Error("video"));
        var cw = frames[0].width, ch = frames[0].height, k = Math.min(1, 1024 / Math.max(cw * 2, ch * 2));
        var c = document.createElement("canvas"); c.width = Math.round(cw * 2 * k); c.height = Math.round(ch * 2 * k);
        var g = c.getContext("2d"); g.fillStyle = "#000"; g.fillRect(0, 0, c.width, c.height);
        frames.forEach(function (f, n) { g.drawImage(f, (n % 2) * cw * k, Math.floor(n / 2) * ch * k, cw * k, ch * k); });
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
          var f = document.createElement("canvas"); f.width = Math.round(v.videoWidth * kk); f.height = Math.round(v.videoHeight * kk);
          f.getContext("2d").drawImage(v, 0, 0, f.width, f.height); frames.push(f);
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
    return answer(post_("vision", { text: q, lang: es() ? "es" : "en", images: frames.slice(0, 4) })
      .then(function (r) { return r.json(); })
      .then(function (j) { return j && typeof j.text === "string" && j.text.trim() ? j.text : null; }), null);
  }
  document.addEventListener("change", function (e) {
    var f = e.target;
    if (!f.matches || !f.matches("[data-eva-photo], [data-eva-video]")) return;
    var file = f.files && f.files[0];
    f.value = "";
    if (!file) return;
    if (!api("vision")) { toast(LIVE); return; }
    var video = f.matches("[data-eva-video]");
    (video ? videoFrames(file) : photoFrames(file))
      .then(function (frames) { return askVision(frames, m(video ? "video" : "photo"), video); })
      .catch(function () { offlineToast(); });
  }, true);

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
  if (track) {
    var start = parseInt((document.querySelector("[data-wrist]") || track).getAttribute("data-start"), 10) || 0;
    var startNow = function () { page(start, false); };
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
  document.addEventListener("click", function (e) {
    var t = e.target.closest && e.target.closest("[data-story], [data-href], [data-replay]");
    if (!t || t.tagName === "FORM") return;
    e.preventDefault();
    e.stopPropagation();
    if (t.hasAttribute("data-story")) {
      if (inFrame) post({ type: "story", action: t.getAttribute("data-story") });
      else toast("Open the replay's start page to play the story");
      return;
    }
    if (t.hasAttribute("data-href")) return go(t.getAttribute("data-href"));
    var kind = t.getAttribute("data-replay");
    if (kind === "missing") toast("That screen is not in this step of the replay");
    else if (kind === "live-only") toast("Exports come from the live app · run eva demo");
    else toast(LIVE);
  }, true);

  document.addEventListener("submit", function (e) {
    var f = e.target;
    e.preventDefault();
    if (f.hasAttribute("data-story")) return post({ type: "story", action: f.getAttribute("data-story") });
    if (f.hasAttribute("data-href")) return go(f.getAttribute("data-href"));
    toast(LIVE);
  }, true);

  document.addEventListener("change", function (e) {
    var s = e.target;
    if (!s.matches || !s.matches("[data-replay-seat]")) return;
    var picked = s.value, was = document.body.getAttribute("data-seat");
    if (["4B", "3B", "1B"].indexOf(picked) >= 0 && inFrame) post({ type: "seat", seat: picked });
    else toast("This replay follows seats 4B, 3B and 1B");
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

  post({ type: "view", file: location.pathname.split("/").pop(), wrist: isWrist });
})();
