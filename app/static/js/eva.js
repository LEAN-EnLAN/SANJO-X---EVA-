// Eva app glue: Konami dev panel, Eva mood from SSE, voice (MediaRecorder → /api/eva/stt,
// /api/eva/tts with speechSynthesis fallback), photo / video (sampled frames) → /api/eva/vision,
// journal orb, toasts, tab state and the service worker. No secrets here: the browser
// only talks to this server.
(function () {
  "use strict";
  var M = function () { return window.EvaMotion; };
  var seat = function () { return document.body.getAttribute("data-seat") || "4B"; };

  // ---------------------------------------------------------------- toast
  function toast(text, tone) {
    var host = document.getElementById("toast");
    if (!host) return;
    var cls = tone === "bad" ? "border-critical/50 bg-critical/15 text-critical" : "border-ok/40 bg-ok/10 text-ok";
    host.innerHTML = '<p class="toast pointer-events-auto rounded-full border px-4 py-2 text-sm ' + cls + '" data-toast></p>';
    host.firstChild.textContent = text;
    settleToast(host.firstChild);
  }
  function settleToast(el) {
    if (!el || !M()) return;
    var y = M().varSpring(el, "--ty", -24, "bouncy");
    el.style.transform = "translateY(calc(var(--ty) * 1px))";
    y.set(0, "bouncy");
    setTimeout(function () { y.set(-80, "gentle"); setTimeout(function () { el.remove(); }, 600); }, 3200);
  }
  document.addEventListener("htmx:afterSwap", function (e) {
    if (e.target && e.target.id === "toast") settleToast(e.target.querySelector("[data-toast]"));
  });

  // ---------------------------------------------------------------- Eva mood via SSE
  document.addEventListener("htmx:sseBeforeMessage", function (e) {
    var d = e.detail;
    if (d && d.type === "eva") {
      e.preventDefault();
      if (M()) M().setMood(d.data);
      var status = document.querySelectorAll("[data-eva-status]");
      var words = { idle: "Watching quietly", listening: "Listening…", thinking: "Thinking…", speaking: "Answering", happy: "Done", concerned: "Keeping an eye on it", alert: "Guiding an emergency" };
      status.forEach(function (s) { s.textContent = words[d.data] || s.textContent; });
    }
  });

  // ---------------------------------------------------------------- tabs follow navigation
  document.addEventListener("htmx:afterSettle", function () {
    var p = location.pathname.replace(/^\/app\/?/, "") || "home";
    document.querySelectorAll("[data-tab]").forEach(function (a) {
      var on = a.getAttribute("data-tab") === p;
      if (a.getAttribute("data-tab") === "eva") {
        a.classList.toggle("ring-info", on); a.classList.toggle("ring-line", !on);
      } else if (a.getAttribute("data-tab") === "emergency") {
        a.classList.toggle("bg-critical", on); a.classList.toggle("text-ink", on); a.classList.toggle("text-muted", !on);
      } else {
        a.classList.toggle("bg-ink", on); a.classList.toggle("text-on-light", on); a.classList.toggle("text-muted", !on);
      }
    });
  });

  // ---------------------------------------------------------------- Konami → dev panel
  var KONAMI = ["ArrowUp", "ArrowUp", "ArrowDown", "ArrowDown", "ArrowLeft", "ArrowRight", "ArrowLeft", "ArrowRight", "b", "a"];
  var pos = 0;
  document.addEventListener("keydown", function (e) {
    if (e.key === "Escape") return closeDev();
    var k = e.key.length === 1 ? e.key.toLowerCase() : e.key;
    pos = k === KONAMI[pos] ? pos + 1 : (k === KONAMI[0] ? 1 : 0);
    if (pos === KONAMI.length) { pos = 0; openDev(); }
  });
  // touch alternative for phones: tap the clock / logo 5 times within 2 s
  var taps = [];
  document.addEventListener("click", function (e) {
    if (e.target.closest && e.target.closest("[data-konami-tap]")) {
      var now = Date.now(); taps = taps.filter(function (t) { return now - t < 2000; }); taps.push(now);
      if (taps.length >= 5) { taps = []; openDev(); }
    }
    if (e.target.closest && e.target.closest("[data-dev-close]")) closeDev();
    if (e.target.closest && e.target.closest("[data-mesh-ping-all]") && window.EvaLink) window.EvaLink.pingAll();
  });

  var panelX = null;
  function panelEl() { return document.querySelector("#dev-panel aside"); }
  function openDev() {
    var host = document.getElementById("dev-panel");
    if (!host) return;
    fetch("/dev/panel").then(function (r) { return r.text(); }).then(function (html) {
      host.innerHTML = html; host.classList.remove("hidden"); host.setAttribute("aria-hidden", "false");
      if (window.htmx) htmx.process(host);
      var p = panelEl();
      panelX = M().varSpring(p, "--panel-x-num", 100, "gentle");
      panelX.onUpdate = function (x) { p.style.setProperty("--panel-x", x.toFixed(2) + "%"); };
      panelX.set(0, "gentle");
      dragToDismiss(p);
    });
  }
  function closeDev() {
    var host = document.getElementById("dev-panel");
    if (!host || host.classList.contains("hidden")) return;
    if (panelX) { panelX.set(100, "snappy"); }
    setTimeout(function () { host.classList.add("hidden"); host.setAttribute("aria-hidden", "true"); host.innerHTML = ""; }, 380);
  }
  // keep the panel's own content in sync after an action re-renders it
  document.addEventListener("htmx:afterSwap", function (e) {
    if (e.target && e.target.id === "dev-panel") {
      var p = panelEl();
      if (p) { p.style.setProperty("--panel-x", "0%"); panelX = M().varSpring(p, "--panel-x-num", 0, "gentle"); panelX.onUpdate = function (x) { p.style.setProperty("--panel-x", x.toFixed(2) + "%"); }; dragToDismiss(p); }
    }
  });
  // the panel follows the finger, then springs open or closed with the release velocity
  function dragToDismiss(p) {
    var startX = 0, lastX = 0, lastT = 0, dragging = false, w = 1;
    p.addEventListener("pointerdown", function (e) {
      if (e.target.closest("button, select, input, a, table")) return;
      dragging = true; startX = lastX = e.clientX; lastT = performance.now(); w = p.offsetWidth || 1;
      p.setPointerCapture(e.pointerId);
    });
    p.addEventListener("pointermove", function (e) {
      if (!dragging) return;
      var now = performance.now(), dx = Math.max(0, e.clientX - startX);
      panelX.track(dx / w * 100, (now - lastT) / 1000); lastX = e.clientX; lastT = now;
    });
    p.addEventListener("pointerup", function () {
      if (!dragging) return; dragging = false;
      if (panelX.x > 35 || panelX.v > 120) closeDev(); else panelX.set(0, "gentle");
    });
  }

  // ---------------------------------------------------------------- journal orb follows the slider
  var WORDS = { 1: "Very unpleasant", 2: "Unpleasant", 3: "Neutral", 4: "Pleasant", 5: "Very pleasant" };
  document.addEventListener("input", function (e) {
    if (!e.target.matches("[data-valence]")) return;
    var v = e.target.value, orb = document.querySelector("[data-orb]"), word = document.querySelector("[data-mood-word]");
    if (orb) { for (var i = 1; i <= 5; i++) orb.classList.remove("orb-" + i); orb.classList.add("orb-" + v); }
    if (word) word.textContent = WORDS[v];
  });

  // ---------------------------------------------------------------- talk to Eva
  function chatPost(text) {
    if (!text) return;
    htmx.ajax("POST", "/api/eva/chat", { target: "#chat", values: { seat: seat(), text: text } });
  }

  var rec = null, chunks = [];
  document.addEventListener("pointerdown", function (e) {
    var btn = e.target.closest && e.target.closest("[data-eva-mic]");
    if (!btn) return;
    e.preventDefault();
    if (!navigator.mediaDevices || !window.MediaRecorder) return browserDictation();
    navigator.mediaDevices.getUserMedia({ audio: true }).then(function (stream) {
      chunks = [];
      rec = new MediaRecorder(stream);
      rec.ondataavailable = function (ev) { if (ev.data.size) chunks.push(ev.data); };
      rec.onstop = function () {
        stream.getTracks().forEach(function (t) { t.stop(); });
        var blob = new Blob(chunks, { type: rec.mimeType || "audio/webm" });
        if (blob.size < 2000) return;
        M() && M().setMood("thinking");
        var fd = new FormData();
        fd.append("seat", seat());
        fd.append("file", blob, "voice." + ((rec.mimeType || "").indexOf("ogg") >= 0 ? "ogg" : "webm"));
        fetch("/api/eva/stt", { method: "POST", body: fd }).then(function (r) { return r.json().then(function (j) { return { ok: r.ok, j: j }; }); }).then(function (res) {
          if (res.ok && res.j.text) chatPost(res.j.text);
          else { toast("Voice needs the server's Groq key. Type instead, or use dictation.", "bad"); M() && M().setMood("idle"); }
        }).catch(function () { toast("Could not reach Eva", "bad"); });
      };
      rec.start();
      M() && M().setMood("listening");
    }).catch(function () { toast("Microphone blocked", "bad"); });
  });
  document.addEventListener("pointerup", function () { if (rec && rec.state === "recording") rec.stop(); });

  // fallback: the browser's own speech recognition, if any
  function browserDictation() {
    var SR = window.SpeechRecognition || window.webkitSpeechRecognition;
    if (!SR) return toast("No microphone support here. Type instead.", "bad");
    var r = new SR(); r.lang = "en-US";
    r.onresult = function (ev) { chatPost(ev.results[0][0].transcript); };
    r.start();
  }

  // speak Eva's newest reply when "Voice replies" is on
  document.addEventListener("htmx:afterSwap", function (e) {
    if (!e.target || e.target.id !== "chat") return;
    var on = document.querySelector("[data-eva-voice]");
    if (!on || !on.checked) return;
    var msgs = e.target.querySelectorAll("[data-eva-msg]");
    var last = msgs[msgs.length - 1];
    if (!last) return;
    var text = Array.prototype.map.call(last.querySelectorAll("[data-say]"), function (s) { return s.textContent; }).join(". ");
    speak(text);
  });
  function speak(text) {
    var fd = new URLSearchParams({ text: text, seat: seat() });
    fetch("/api/eva/tts", { method: "POST", body: fd }).then(function (r) {
      if (!r.ok) throw new Error("fallback");
      return r.blob();
    }).then(function (b) {
      var a = new Audio(URL.createObjectURL(b));
      a.onended = function () { M() && M().setMood("idle"); };
      a.play();
    }).catch(function () {
      if (!window.speechSynthesis) return;
      var u = new SpeechSynthesisUtterance(text); u.lang = "en-US"; u.rate = 1.02;
      u.onstart = function () { M() && M().setMood("speaking"); };
      u.onend = function () { M() && M().setMood("idle"); };
      speechSynthesis.speak(u);
    });
  }

  // ---------------------------------------------------------------- photo and video
  function sendImages(blobs, task, isVideo) {
    var fd = new FormData();
    fd.append("seat", seat()); fd.append("task", task);
    if (isVideo) fd.append("video", "1");
    blobs.forEach(function (b, i) { fd.append("image", b, "frame" + i + ".jpg"); });
    M() && M().setMood("thinking");
    fetch("/api/eva/vision", { method: "POST", body: fd }).then(function (r) { return r.text(); }).then(function (html) {
      var chat = document.getElementById("chat");
      if (chat) { chat.innerHTML = html; htmx.process(chat); }
    });
  }
  // a video is never uploaded whole: sample 3 evenly spaced frames as JPEG
  function sampleVideoFrames(file, n) {
    return new Promise(function (resolve) {
      var v = document.createElement("video"), out = [], url = URL.createObjectURL(file);
      v.muted = true; v.playsInline = true; v.src = url;
      v.onloadedmetadata = function () {
        var d = isFinite(v.duration) ? v.duration : 1, i = 0;
        var c = document.createElement("canvas"), scale = Math.min(1, 768 / Math.max(v.videoWidth, v.videoHeight));
        c.width = v.videoWidth * scale; c.height = v.videoHeight * scale;
        function next() {
          if (i >= n) { URL.revokeObjectURL(url); return resolve(out); }
          v.currentTime = d * (i + 0.5) / n;
        }
        v.onseeked = function () {
          c.getContext("2d").drawImage(v, 0, 0, c.width, c.height);
          c.toBlob(function (b) { out.push(b); i++; next(); }, "image/jpeg", 0.8);
        };
        next();
      };
    });
  }
  document.addEventListener("change", function (e) {
    var t = e.target;
    if (t.matches("[data-eva-photo]") && t.files[0]) {
      var meal = window.confirm("Is this a meal? OK = log it as a meal, Cancel = ask Eva about it.");
      sendImages([t.files[0]], meal ? "meal" : "symptom", false); t.value = "";
    }
    if (t.matches("[data-eva-video]") && t.files[0]) {
      sampleVideoFrames(t.files[0], 3).then(function (frames) { sendImages(frames, "symptom", true); }); t.value = "";
    }
  });

  // ---------------------------------------------------------------- service worker (offline shell + queued posts)
  if ("serviceWorker" in navigator) {
    window.addEventListener("load", function () {
      navigator.serviceWorker.register("/sw.js", { scope: "/" }).catch(function () {});
    });
    window.addEventListener("online", function () {
      navigator.serviceWorker.controller && navigator.serviceWorker.controller.postMessage("replay");
    });
    navigator.serviceWorker.addEventListener && navigator.serviceWorker.addEventListener("message", function (e) {
      if (e.data && e.data.type === "replayed" && e.data.count) toast("Sent " + e.data.count + " saved item(s)");
    });
  }

  window.Eva = { toast: toast, speak: speak, openDev: openDev, closeDev: closeDev, sampleVideoFrames: sampleVideoFrames };
})();
