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
  // what Eva is doing, in words, for each mascot state (motion.js has the motion and the art)
  var STATUS = {
    idle: "Watching quietly", hello: "Hi there", listening: "Listening…", thinking: "Thinking…",
    speaking: "Answering", happy: "Done", wink: "Here for you", concerned: "Keeping an eye on it",
    alert: "Guiding an emergency", calm: "Counting the pushes with you", sleeping: "Ship night · resting",
    celebrate: "Goal met", care: "Here with you", reading: "Reading your records", writing: "Writing with you",
    sending: "Sending to the ground", "low-battery": "Band battery low · re-join the parts"
  };
  function mood(state) {
    if (M()) M().setMood(state);
    document.querySelectorAll("[data-eva-status]").forEach(function (s) { if (STATUS[state]) s.textContent = STATUS[state]; });
  }
  function current() { var m = document.querySelector("[data-eva]:not([data-eva-fixed])") || document.querySelector("[data-eva]"); return m ? m.getAttribute("data-state") : "idle"; }
  // a short state (hello, wink, sending) that goes back to what Eva was doing, unless
  // something else changed it meanwhile; never over an emergency
  function blip(state, ms) {
    var was = current();
    if (was === "alert" || was === "calm" || was === state) return;
    mood(state);
    setTimeout(function () { if (current() === state) mood(was === "hello" || was === "wink" ? "idle" : was); }, ms);
  }
  // Eva says hello when her screen opens; she writes along while the journal box has focus
  function greet(root) {
    var h = root && root.querySelector && root.querySelector("[data-eva-hello]");
    if (h && /^(idle|sleeping)$/.test(current())) blip("hello", 1600);
  }
  document.addEventListener("DOMContentLoaded", function () { greet(document); });
  document.addEventListener("htmx:afterSwap", function (e) { if (e.target && e.target.id === "screen") greet(e.target); });
  document.addEventListener("focusin", function (e) { if (e.target.matches && e.target.matches("[data-eva-writes]") && current() !== "alert") mood("writing"); });
  document.addEventListener("focusout", function (e) { if (e.target.matches && e.target.matches("[data-eva-writes]") && current() === "writing" && !e.target.value.trim()) mood("idle"); });

  document.addEventListener("htmx:sseBeforeMessage", function (e) {
    var d = e.detail;
    if (d && d.type === "eva") {
      e.preventDefault();
      if (M()) M().setMood(d.data);
      var status = document.querySelectorAll("[data-eva-status]");
      var words = STATUS;
      status.forEach(function (s) { s.textContent = words[d.data] || s.textContent; });
    }
  });

  // ---------------------------------------------------------------- tool buttons and forms
  // Every UI tool posts to /api/eva/tool, the endpoint Eva's own calls use, so each tap is
  // validated, executed and written to the hash chain the same way.
  // Forms name their tool arguments `arg.<key>`; they are folded into one `args` JSON field.
  document.addEventListener("htmx:configRequest", function (e) {
    var form = e.detail.elt && e.detail.elt.closest && e.detail.elt.closest("[data-tool-form]");
    if (!form) return;
    var p = e.detail.parameters, args = {};
    var get = function (k) { return p instanceof FormData ? p.get(k) : p[k]; };
    var keys = p instanceof FormData ? Array.from(p.keys()) : Object.keys(p);
    keys.forEach(function (k) {
      if (k.indexOf("arg.") !== 0) return;
      var name = k.slice(4), v = get(k), field = form.querySelector('[name="' + k + '"]');
      args[name] = (field && field.hasAttribute("data-int")) || /^\d+$/.test(v) ? parseInt(v, 10) : v;
      if (p instanceof FormData) p.delete(k); else delete p[k];
    });
    if (p instanceof FormData) p.set("args", JSON.stringify(args)); else p.args = JSON.stringify(args);
  });
  // after a tool runs: open the screen it leads to (data-then), or redraw the current
  // screen in place so the numbers it changed update; a tool run from the sheet closes it.
  document.addEventListener("htmx:afterRequest", function (e) {
    var elt = e.detail.elt;
    if (!elt || !elt.closest || !e.detail.successful) return;
    var t = elt.closest("[data-tool], [data-tool-form]");
    if (!t) return;
    var inSheet = !!elt.closest("#sheet");
    if (inSheet && M()) M().closeSheet();
    var then = t.getAttribute("data-then");
    if (then) {
      if (M()) M().expect("push");
      htmx.ajax("GET", then, { target: "#screen" }).then(function () { history.pushState({ htmx: true }, "", then); M() && M().syncTabs(); });
      return;
    }
    var tgt = e.detail.target;
    // on the wristband the track stays where it is; only its live parts redraw
    if (document.documentElement.getAttribute("data-mode") === "wristband") { document.body.dispatchEvent(new CustomEvent("wrist:refresh", { bubbles: true })); return; }
    if (tgt && tgt.id === "toast" && document.getElementById("screen") && !document.getElementById("chat")) {
      if (M()) M().expect("refresh");
      htmx.ajax("GET", location.pathname + location.search, { target: "#screen" });
    }
  });

  // tap Eva: a wink, then back to what she was doing
  document.addEventListener("click", function (e) {
    var t = e.target.closest && e.target.closest("[data-eva-tap]");
    if (!t || !M()) return;
    blip("wink", 1400);
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
    // on her on-board rules Eva cannot transcribe: say so instead of failing silently
    if (rulesMode()) { toast("Voice needs Eva online · type or use the chips", "bad"); return; }
    if (window.EvaFx) window.EvaFx.haptic("tap");
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
          else { toast("Voice needs Eva online · type or use the chips", "bad"); M() && M().setMood("idle"); }
        }).catch(function () { toast("Could not reach Eva", "bad"); });
      };
      rec.start();
      btn.classList.add("is-recording");
      M() && M().setMood("listening");
    }).catch(function () { toast("Microphone blocked", "bad"); });
  });
  function rulesMode() { return document.documentElement.getAttribute("data-eva-mode") === "rules"; }
  // photo / video buttons explain themselves in rules mode instead of opening the camera
  document.addEventListener("click", function (e) {
    var l = e.target.closest && e.target.closest("label:has([data-eva-photo]), label:has([data-eva-video])");
    if (l && rulesMode()) { e.preventDefault(); toast("Photos and video need Eva online · type or use the chips", "bad"); }
  }, true);
  document.addEventListener("pointerup", function () {
    if (rec && rec.state === "recording") { rec.stop(); if (window.EvaFx) window.EvaFx.haptic([8, 40, 8]); }
    document.querySelectorAll("[data-eva-mic].is-recording").forEach(function (b) { b.classList.remove("is-recording"); });
  });

  // fallback: the browser's own speech recognition, if any
  function browserDictation() {
    var SR = window.SpeechRecognition || window.webkitSpeechRecognition;
    if (!SR) return toast("No microphone support here. Type instead.", "bad");
    var r = new SR(); r.lang = "en-US";
    r.onresult = function (ev) { chatPost(ev.results[0][0].transcript); };
    r.start();
  }

  // speak Eva's newest reply when "Voice replies" is on; keep the newest line in view above the dock
  document.addEventListener("htmx:afterSwap", function (e) {
    if (!e.target || e.target.id !== "chat") return;
    var lastMsg = e.target.lastElementChild;
    if (lastMsg) lastMsg.scrollIntoView({ block: "end", behavior: M() && M().reduced.matches ? "auto" : "smooth" });
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
      a.onplay = function () { mood("speaking"); };
      a.onended = function () { mood("idle"); };
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
      if (chat) { chat.innerHTML = html; htmx.process(chat); return; }
      toast(task === "meal" ? "Meal logged from your photo" : "Sent to Eva");
      if (M()) M().expect("refresh");
      htmx.ajax("GET", location.pathname + location.search, { target: "#screen" });
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
      var meal = t.hasAttribute("data-meal") || window.confirm("Is this a meal? OK = log it as a meal, Cancel = ask Eva about it.");
      sendImages([t.files[0]], meal ? "meal" : "symptom", false); t.value = "";
    }
    if (t.matches("[data-eva-video]") && t.files[0]) {
      sampleVideoFrames(t.files[0], 3).then(function (frames) { sendImages(frames, "symptom", true); }); t.value = "";
    }
  });


  // ---------------------------------------------------------------- ECG: countdown ring + live trace
  // The server owns the 30 s window (data-left = ms to go); the page draws a sweeping trace
  // like a bedside monitor and re-asks the server for the result when the window closes.
  var BEAT = [0, 0, 0.12, 0.18, 0.1, 0, 0, -0.12, 1, -0.3, 0, 0, 0.08, 0.22, 0.28, 0.18, 0.05, 0, 0, 0];
  function ecgSample(t, hr) {
    var period = 60 / (hr || 70), ph = (t % period) / period;   // phase inside one beat
    var i = ph * 34;                                              // beat shape uses the first 20 of 34 slots
    if (i >= BEAT.length - 1) return 0;
    var a = Math.floor(i), f = i - a;
    return BEAT[a] * (1 - f) + BEAT[a + 1] * f + (Math.random() - 0.5) * 0.02;
  }
  function mountEcg(root) {
    root = root || document;
    var boxes = Array.prototype.slice.call(root.querySelectorAll("[data-ecg]"));
    if (root.matches && root.matches("[data-ecg]")) boxes.push(root);
    boxes.forEach(function (box) {
      if (box._ecg) return; box._ecg = true;
      var left = parseInt(box.getAttribute("data-left"), 10) || 30000, t0 = performance.now();
      var secs = document.querySelector("[data-ecg-secs]"), ring = document.querySelector("[data-ecg-ring]");
      var cv = document.querySelector("[data-ecg-trace]"), hr = parseInt(box.getAttribute("data-hr"), 10) || 70;
      var ctx = cv && cv.getContext("2d"), dpr = window.devicePixelRatio || 1, W = 0, H = 0, x = 0, lastY = null, lastT = 0;
      var ink = getComputedStyle(document.documentElement).getPropertyValue("--color-ink").trim() || "#f4f3f0";
      function size() { if (!cv) return; W = cv.width = cv.clientWidth * dpr; H = cv.height = cv.clientHeight * dpr; x = 0; lastY = null; }
      size();
      var slow = M() && M().reduced.matches, lastSec = -1;
      function frame(now) {
        if (!document.body.contains(box)) return;
        var el = now - t0, rem = Math.max(0, left - el), sec = Math.ceil(rem / 1000);
        if (secs && sec !== lastSec) { secs.textContent = sec; lastSec = sec; }
        if (ring) ring.style.strokeDashoffset = (rem / 30000 * 100).toFixed(2);
        if (ctx && W) {
          // 25 mm/s feel: the sweep crosses the strip in ~3 s
          var dt = lastT ? (now - lastT) / 1000 : 0, step = W / 3 * dt;
          var n = Math.max(1, Math.round(step / dpr));
          for (var k = 0; k < n; k++) {
            var nx = x + step / n, t = (now / 1000) - (n - k) * dt / n;
            var y = H * 0.66 - ecgSample(t, hr) * H * 0.6;
            ctx.clearRect(nx, 0, 14 * dpr, H);                      // erase just ahead of the pen
            if (lastY !== null && nx > x) { ctx.strokeStyle = ink; ctx.lineWidth = 1.6 * dpr; ctx.beginPath(); ctx.moveTo(x, lastY); ctx.lineTo(nx, y); ctx.stroke(); }
            x = nx; lastY = y;
            if (x >= W) { x = 0; lastY = null; }
          }
        }
        lastT = now;
        if (rem > 0) slow ? setTimeout(function () { frame(performance.now()); }, 1000) : requestAnimationFrame(frame);
      }
      slow ? frame(performance.now()) : requestAnimationFrame(frame);
    });
  }
  document.addEventListener("DOMContentLoaded", function () { mountEcg(document); });
  document.addEventListener("htmx:load", function (e) { mountEcg(e.target); });

  // ---------------------------------------------------------------- service worker (offline shell + queued posts)
  if ("serviceWorker" in navigator) {
    window.addEventListener("load", function () {
      navigator.serviceWorker.register("/sw.js", { scope: "/" }).catch(function () {});
    });
    window.addEventListener("online", function () {
      navigator.serviceWorker.controller && navigator.serviceWorker.controller.postMessage("replay");
    });
    navigator.serviceWorker.addEventListener && navigator.serviceWorker.addEventListener("message", function (e) {
      if (e.data && e.data.type === "replayed" && e.data.count) { toast("Sent " + e.data.count + " saved item(s)"); blip("sending", 2400); }
    });
  }

  window.Eva = { toast: toast, speak: speak, openDev: openDev, closeDev: closeDev, sampleVideoFrames: sampleVideoFrames };
})();
