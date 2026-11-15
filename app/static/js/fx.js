// Eva fx: the "alive" layer on top of motion.js (APP-POLISH §3, §4).
// Haptics, rolling counters, heartbeat phase, stagger, ripple, screen tint, Eva status,
// emergency takeover, celebration burst, skeletons, lite mode, off-screen pausing.
// Transform and opacity only; per-frame work runs in motion.js's single rAF loop.
(function () {
  "use strict";
  var M = function () { return window.EvaMotion; };
  var doc = document, html = doc.documentElement;
  var reduced = window.matchMedia("(prefers-reduced-motion: reduce)");
  var wrist = function () { return html.getAttribute("data-mode") === "wristband"; };

  // ---------------------------------------------------------------- haptics (§4.4, §9.6)
  var HAPTIC = { tick: 4, tap: 8, success: [12, 30, 12], warn: [40, 60, 40], emergency: [120, 60, 120, 60, 400], beat: 30,
    reminder: [20, 60, 20], hold: 20, wake: [300, 150, 300, 150, 300] };
  function haptic(name) {
    if (!navigator.vibrate) return;
    try { navigator.vibrate(HAPTIC[name] !== undefined ? HAPTIC[name] : name); } catch (e) {}
  }

  // ---------------------------------------------------------------- lite mode (§4.6 rule 9)
  var SMOOTH = "eva-smooth";
  function smoothPref() { try { return localStorage.getItem(SMOOTH) || "auto"; } catch (e) { return "auto"; } }
  function applySmooth() {
    var p = smoothPref(), probe = null;
    try { probe = localStorage.getItem("eva-lite-probe"); } catch (e) {}
    var lite = p === "lite" || (p === "auto" && (probe === "1" || ((navigator.hardwareConcurrency || 8) <= 4 && (navigator.deviceMemory || 8) <= 2)));
    html.classList.toggle("lite", lite);
    doc.querySelectorAll("[data-smooth-state]").forEach(function (el) { el.textContent = p === "auto" ? "Auto (" + (lite ? "Lite" : "Full") + ")" : p === "lite" ? "Lite" : "Full"; });
    doc.querySelectorAll("[data-smooth]").forEach(function (b) { b.setAttribute("aria-pressed", String(b.getAttribute("data-smooth") === p)); });
  }
  applySmooth();
  function probeFrames() {
    try { if (localStorage.getItem("eva-lite-probe") !== null) return; } catch (e) { return; }
    var n = 0, slow = 0, last = 0;
    M() && M().onFrame(function (dt, now) {
      if (last) { n++; if (now - last > 24) slow++; }
      last = now;
      if (n < 60) return true;
      try { localStorage.setItem("eva-lite-probe", slow > 12 ? "1" : "0"); } catch (e) {}
      applySmooth();
      return false;
    });
  }
  doc.addEventListener("click", function (e) {
    var b = e.target.closest && e.target.closest("[data-smooth]");
    if (!b) return;
    try { localStorage.setItem(SMOOTH, b.getAttribute("data-smooth")); } catch (er) {}
    applySmooth(); haptic("tap");
  });

  // ---------------------------------------------------------------- heartbeat phase
  // SSE redraws the vitals every tick; a negative delay keeps every beat loop on one
  // global clock, so a fresh node continues the rhythm instead of restarting it.
  function phase(root) {
    (root || doc).querySelectorAll(".beating, .beat-wave, .ecg-ribbon > svg").forEach(function (el) {
      var host = el.closest("[style*='--beat']") || el;
      var beat = parseFloat(getComputedStyle(host).getPropertyValue("--beat")) || 900;
      var extra = el.classList.contains("beat-wave") && el.previousElementSibling && el.previousElementSibling.classList.contains("beat-wave") ? beat * 0.16 : 0;
      el.style.animationDelay = (-(performance.now() % beat) + extra).toFixed(0) + "ms";
    });
  }

  // ---------------------------------------------------------------- rolling counters (§3.2)
  var prev = {};
  function splitCount(el) {
    var name = el.getAttribute("data-count"), text = el.textContent.trim();
    if (!/\d/.test(text)) { prev[name] = null; return; }
    var old = prev[name];
    prev[name] = text;
    el.setAttribute("aria-label", text);
    var cols = [];
    var out = doc.createElement("span");
    out.className = "count"; out.setAttribute("aria-hidden", "true");
    for (var i = 0; i < text.length; i++) {
      var ch = text[i];
      if (/\d/.test(ch)) {
        var slot = doc.createElement("span"); slot.className = "slot";
        var col = doc.createElement("span"); col.className = "col";
        for (var d = 0; d <= 9; d++) { var s = doc.createElement("span"); s.textContent = d; col.appendChild(s); }
        slot.appendChild(col); out.appendChild(slot); cols.push({ col: col, d: +ch });
      } else { var sep = doc.createElement("span"); sep.className = "sep"; sep.textContent = ch; out.appendChild(sep); }
    }
    el.textContent = ""; el.appendChild(out);
    var from = old && old.length === text.length ? old.replace(/\D/g, "") : null;
    var animate = !reduced.matches && !!M();
    cols.forEach(function (c, k) {
      var start = from !== null ? +from[k] : 0;
      if (!animate || start === c.d) { c.col.style.setProperty("--d", c.d); return; }
      c.col.style.setProperty("--d", start);
      var sp = M().varSpring(c.col, "--d", start, from !== null ? "snappy" : "gentle");
      setTimeout(function () { sp.set(c.d); }, (cols.length - 1 - k) * 30);
    });
    if (from !== null && from !== text.replace(/\D/g, "")) {
      el.classList.add("flash-tint", "is-flash");
      setTimeout(function () { el.classList.remove("is-flash"); }, 400);
    }
  }
  function counters(root) { (root || doc).querySelectorAll("[data-count]").forEach(splitCount); }

  // ---------------------------------------------------------------- stagger (§4.3)
  function stagger(root) {
    if (!M()) return;
    var kind = M().lastKind;
    if (kind === "refresh" || kind === "fade" || wrist()) return;
    var vh = window.innerHeight, kids = Array.prototype.filter.call(root.children, function (c) { return c.offsetParent !== null && !c.classList.contains("hidden"); });
    kids.forEach(function (el, i) {
      if (el.getBoundingClientRect().top > vh || reduced.matches) return;
      el.style.opacity = "0"; el.style.transform = "translateY(16px) scale(.98)"; el.style.willChange = "transform";
      var t = new (M().Spring)(0, "swift", function (x) {
        el.style.opacity = Math.min(1, x * 1.4).toFixed(3);
        el.style.transform = x >= 1 ? "" : "translateY(" + ((1 - x) * 16).toFixed(2) + "px) scale(" + (0.98 + 0.02 * x).toFixed(4) + ")";
        if (x >= 1) { el.style.opacity = ""; el.style.willChange = ""; }
      });
      setTimeout(function () { t.set(1); }, Math.min(i, 7) * 45);
    });
  }

  // ---------------------------------------------------------------- ripple (§4.4)
  doc.addEventListener("pointerdown", function (e) {
    var host = e.target.closest && e.target.closest(".btn-primary, .btn-secondary, .row, .btn-white, .btn-emergency");
    if (!host || reduced.matches || !M()) return;
    var ink = host._ink;
    if (!ink) { ink = host._ink = doc.createElement("span"); ink.className = "ripple-ink"; host.appendChild(ink); if (getComputedStyle(host).overflow !== "hidden") host.style.overflow = "hidden"; }
    var r = host.getBoundingClientRect();
    ink.style.left = (e.clientX - r.left) + "px"; ink.style.top = (e.clientY - r.top) + "px";
    var sp = new (M().Spring)(0, "gentle", function (x) { ink.style.transform = "scale(" + (x * 2.2).toFixed(3) + ")"; ink.style.opacity = (0.6 * (1 - x)).toFixed(3); });
    sp.set(1);
  }, { passive: true });

  // ---------------------------------------------------------------- screen tint (§2.1)
  var TINT = { "": "eva", home: "eva", eva: "eva", plan: "mood", food: "food", records: "oxygen", journal: "mood", checkin: "mood",
    ecg: "heart", band: "brand", storm: "temp", emergency: "critical", about: "eva" };
  var VTINT = { heart: "heart", lungs: "oxygen", temperature: "temp", sleep: "sleep", water: "water" };
  function tintFor() {
    var p = location.pathname.replace(/^\/app\/?/, "");
    if (p.indexOf("wrist") === 0) return "eva";
    if (p === "vitals") return VTINT[new URLSearchParams(location.search).get("k") || "heart"] || "heart";
    return TINT[p] || "eva";
  }
  function setTint() {
    var b = doc.body, t = tintFor();
    if (!b || b.getAttribute("data-view") !== "app" || b.getAttribute("data-tint") === t) return;
    b.style.setProperty("--tint-o", "0");
    setTimeout(function () { b.setAttribute("data-tint", t); b.style.setProperty("--tint-o", "1"); }, 200);
  }

  // ---------------------------------------------------------------- pausing (§4.6 rule 6)
  var io = "IntersectionObserver" in window ? new IntersectionObserver(function (es) {
    es.forEach(function (en) { en.target.classList.toggle("is-paused", !en.isIntersecting); });
  }) : null;
  function watchLoops(root) {
    if (!io) return;
    (root || doc).querySelectorAll(".vring, .aurora, .float, .ecg-ribbon, [data-loop]").forEach(function (el) { if (!el._io) { el._io = 1; io.observe(el); } });
  }
  doc.addEventListener("visibilitychange", function () { html.classList.toggle("is-hidden", doc.hidden); if (!doc.hidden) refreshStatus(); });

  // ---------------------------------------------------------------- celebration burst (§3.8)
  var COLORS = { water: ["#7DE3FF", "#2E9BFF"], mood: ["#C08BFF", "#6C5CFF"], heart: ["#FF7A59", "#FF2D6F"], food: ["#C6F06A", "#3DDC97"], eva: ["#5EF0D6", "#38B6FF"] };
  function burst(el, n, tone) {
    if (reduced.matches || !M()) { haptic("success"); return; }
    n = n || 36; if (html.classList.contains("lite")) n = Math.round(n / 2);
    var r = el ? el.getBoundingClientRect() : { left: innerWidth / 2, top: innerHeight / 2, width: 0, height: 0 };
    var ox = r.left + r.width / 2, oy = r.top + r.height / 2;
    var cv = doc.createElement("canvas"), dpr = Math.min(2, window.devicePixelRatio || 1);
    cv.width = innerWidth * dpr; cv.height = innerHeight * dpr;
    cv.style.cssText = "position:fixed;inset:0;width:100vw;height:100vh;pointer-events:none;z-index:80";
    doc.body.appendChild(cv);
    var ctx = cv.getContext("2d"); ctx.scale(dpr, dpr);
    var cols = (COLORS[tone] || COLORS.eva).concat(["#FFD166"]), ps = [];
    for (var i = 0; i < n; i++) {
      var a = (-90 + (Math.random() - 0.5) * 120) * Math.PI / 180, v = 320 + Math.random() * 300;
      ps.push({ x: ox, y: oy, vx: Math.cos(a) * v, vy: Math.sin(a) * v, r: Math.random() * 6, vr: (Math.random() - 0.5) * 12, k: i % 3, c: cols[i % cols.length] });
    }
    var t = 0;
    M().onFrame(function (dt) {
      t += dt; ctx.clearRect(0, 0, innerWidth, innerHeight);
      var fade = t > 0.6 ? Math.max(0, 1 - (t - 0.6) / 0.3) : 1;
      ps.forEach(function (p) {
        p.vy += 1400 * dt; p.vx *= Math.pow(0.9, dt * 10); p.vy *= Math.pow(0.9, dt); p.x += p.vx * dt; p.y += p.vy * dt; p.r += p.vr * dt;
        ctx.save(); ctx.globalAlpha = fade; ctx.fillStyle = p.c; ctx.translate(p.x, p.y); ctx.rotate(p.r);
        if (p.k === 0) { ctx.beginPath(); ctx.arc(0, 0, 3, 0, 6.283); ctx.fill(); }
        else if (p.k === 1) { ctx.fillRect(-5, -2, 10, 4); }
        else { ctx.beginPath(); ctx.moveTo(0, -4); ctx.lineTo(1, -1); ctx.lineTo(4, 0); ctx.lineTo(1, 1); ctx.lineTo(0, 4); ctx.lineTo(-1, 1); ctx.lineTo(-4, 0); ctx.lineTo(-1, -1); ctx.fill(); }
        ctx.restore();
      });
      if (t >= 0.9) { cv.remove(); return false; }
      return true;
    });
    haptic("success");
  }
  doc.addEventListener("htmx:afterRequest", function (e) {
    var el = e.detail.elt;
    if (!e.detail.successful || !el || !el.closest) return;
    var b = el.closest("[data-burst]");
    if (b) burst(b, b.hasAttribute("data-burst-mini") ? 12 : 36, b.getAttribute("data-burst"));
  });

  // ---------------------------------------------------------------- skeletons (§3.8)
  var skelTimer = 0;
  doc.addEventListener("htmx:beforeRequest", function (e) {
    var t = e.detail.target;
    if (!t || t.id !== "screen" || wrist()) return;
    var k = M() && e.detail.elt && e.detail.elt.closest && e.detail.elt.closest("[data-nav]");
    if (k && k.getAttribute("data-nav") === "refresh") return;
    clearTimeout(skelTimer);
    skelTimer = setTimeout(function () {
      var tpl = doc.getElementById("skel-default");
      if (tpl && doc.getElementById("screen") === t && !t.querySelector("[data-focus]")) { t.innerHTML = ""; t.appendChild(tpl.content.cloneNode(true)); }
    }, 160);
  });
  doc.addEventListener("htmx:afterRequest", function () { clearTimeout(skelTimer); });

  // ---------------------------------------------------------------- shared-element morph (§3.5)
  var canVT = function () { return "startViewTransition" in doc && !html.classList.contains("lite") && !reduced.matches && !wrist(); };
  var morphing = null;
  doc.addEventListener("htmx:beforeRequest", function (e) {
    var el = e.detail.elt, t = e.detail.target;
    if (!t || t.id !== "screen" || !el || !el.closest) return;
    var src = el.closest("[data-vital]");
    if (!src || !canVT()) { morphing = null; return; }
    doc.querySelectorAll("[style*='view-transition-name']").forEach(function (x) { x.style.viewTransitionName = ""; });
    var h = src.querySelector("[data-morph='vital-hero']"), v = src.querySelector("[data-morph='vital-value']");
    if (h) h.style.viewTransitionName = "vital-hero";
    if (v) v.style.viewTransitionName = "vital-value";
    morphing = src;
  });
  doc.addEventListener("htmx:beforeSwap", function (e) {
    if (!morphing || !e.detail.target || e.detail.target.id !== "screen") return;
    e.detail.swapOverride = "innerHTML transition:true";
    M() && M().expect("morph");
    morphing = null;
  });

  // ---------------------------------------------------------------- Eva status (§10.3)
  var status = null;
  function applyStatus(s, live) {
    if (!s || !s.mode) return;
    var was = status; status = s;
    html.setAttribute("data-eva-mode", s.mode);
    doc.querySelectorAll("[data-eva-chip]").forEach(function (c) {
      c.setAttribute("data-tone", s.mode === "online" ? "ok" : "warn");
      var w = c.querySelector("[data-eva-chip-text]"); if (w) w.textContent = s.mode === "online" ? "Online" : "On-board rules";
    });
    doc.querySelectorAll("[data-eva-hint]").forEach(function (h) { h.textContent = s.hint || ""; });
    doc.querySelectorAll("[data-eva-reason]").forEach(function (h) { h.hidden = h.getAttribute("data-eva-reason") !== s.reason; });
    doc.querySelectorAll("[data-eva-caption]").forEach(function (c) { c.textContent = s.mode === "online" ? c.getAttribute("data-online") : c.getAttribute("data-rules"); });
    var dismissed = false; try { dismissed = sessionStorage.getItem("eva-status-dismissed") === "1"; } catch (e) {}
    doc.querySelectorAll("[data-eva-banner]").forEach(function (b) { b.hidden = s.mode === "online" || dismissed || (b.hasAttribute("data-once") && seenOnce()); });
    if (live && was && was.mode !== "online" && s.mode === "online") {
      if (window.Eva) window.Eva.toast("Eva is online");
      haptic("success");
      if (M()) { M().setMood("happy"); setTimeout(function () { M().setMood("idle"); }, 2500); }
    }
  }
  function seenOnce() { try { var v = sessionStorage.getItem("eva-status-home"); sessionStorage.setItem("eva-status-home", "1"); return v === "1" && !doc.querySelector("[data-eva-banner][data-once]:not([hidden])"); } catch (e) { return false; } }
  function refreshStatus() {
    if (!doc.body || doc.body.getAttribute("data-view") === "station") return;
    fetch("/api/eva/status", { cache: "no-store" }).then(function (r) { return r.json(); }).then(function (s) { applyStatus(s, true); })
      .catch(function () { doc.querySelectorAll("[data-eva-chip]").forEach(function (c) { c.setAttribute("data-tone", "off"); var w = c.querySelector("[data-eva-chip-text]"); if (w) w.textContent = "Hub unreachable"; }); });
  }
  setInterval(refreshStatus, 60000);
  doc.addEventListener("click", function (e) {
    var d = e.target.closest && e.target.closest("[data-eva-dismiss]");
    if (d) { try { sessionStorage.setItem("eva-status-dismissed", "1"); } catch (er) {} doc.querySelectorAll("[data-eva-banner]").forEach(function (b) { b.hidden = true; }); }
    var c = e.target.closest && e.target.closest("[data-eva-chip]");
    if (c) { try { sessionStorage.removeItem("eva-status-dismissed"); } catch (er) {} var b = doc.querySelector("[data-eva-banner]"); if (b && status && status.mode !== "online") { b.hidden = false; var det = b.querySelector("details"); if (det) det.open = true; } }
  });

  // ---------------------------------------------------------------- emergency takeover (§3.7)
  var theme = function (c) { var m = doc.querySelector('meta[name="theme-color"]'); if (m) m.setAttribute("content", c); };
  function takeover(kind) {
    var tk = doc.getElementById("takeover");
    haptic(kind === "storm" ? "warn" : "emergency");
    if (!tk) return;
    theme(kind === "storm" ? "#5A3A06" : "#B3122A");
    if (reduced.matches || !M()) return;
    tk.classList.toggle("is-storm", kind === "storm");
    tk.classList.add("is-on"); tk.classList.remove("is-rings");
    tk.style.opacity = "1";
    var short = wrist();
    var disc = tk.querySelector(".tk-disc");
    var sp = M().varSpring(disc, "--tk", 0, short ? { mass: 1, stiffness: 900, damping: 55 } : "swift");
    sp.set(1);
    setTimeout(function () { tk.classList.add("is-rings"); }, 120);
    setTimeout(function () {
      var o = new (M().Spring)(1, "swift", function (x) { tk.style.opacity = x.toFixed(3); if (x <= 0.01) { tk.classList.remove("is-on", "is-rings"); disc.style.setProperty("--tk", "0"); } });
      o.set(0);
    }, short ? 500 : 1100);
  }
  doc.addEventListener("htmx:sseBeforeMessage", function (e) {
    var d = e.detail;
    if (!d) return;
    if (d.type === "eva-status") { e.preventDefault(); try { applyStatus(JSON.parse(d.data), true); } catch (er) {} }
    else if (d.type === "go-emergency") takeover("emergency");
    else if (d.type === "go-storm") takeover("storm");
    else if (d.type === "go-home") theme("#070A14");
  });
  // a case is open but this person is elsewhere: a short tap every 8 s on home, at most 5 times
  var nudges = 0;
  setInterval(function () {
    if (doc.hidden || nudges >= 5) return;
    var b = doc.querySelector("[data-case-banner][data-role='helper']");
    if (b) { nudges++; haptic([80]); }
  }, 8000);

  // CPR: one vibration per compression, locked to the metronome's own animation
  doc.addEventListener("animationiteration", function (e) {
    var t = e.target;
    if (!t.matches || !t.matches("[data-cpr-beat]")) return;
    var tg = doc.querySelector("[data-cpr-haptics]");
    if (tg && !tg.checked) return;
    haptic("beat");
  }, true);

  // ---------------------------------------------------------------- mount
  function mount(root) {
    counters(root); phase(root); watchLoops(root);
    if (status) applyStatus(status, false);
  }
  doc.addEventListener("htmx:load", function (e) { mount(e.target); });
  doc.addEventListener("htmx:afterSwap", function (e) {
    var t = e.detail.target;
    if (!t) return;
    if (t.id === "screen") {
      setTint(); stagger(t);
      if (t.querySelector("[data-celebrate]")) setTimeout(function () { burst(t.querySelector("[data-celebrate]"), 36, t.querySelector("[data-celebrate]").getAttribute("data-celebrate")); }, 400);
      if (!t.querySelector("[data-focus]") && !t.querySelector("[data-storm-active]")) theme("#070A14");
      if (t.querySelector("[data-focus]") || t.querySelector("[data-storm-active]")) theme(t.querySelector("[data-storm-active]") ? "#5A3A06" : "#B3122A");
    }
  });
  doc.addEventListener("DOMContentLoaded", function () {
    mount(doc); refreshStatus();
    if (doc.querySelector("#screen [data-focus]")) theme("#B3122A");
  });
  window.addEventListener("load", function () { setTimeout(probeFrames, 600); });
  // seat picker: the avatar chip opens a sheet with one row per seat
  doc.addEventListener("click", function (e) {
    var o = e.target.closest && e.target.closest("[data-seat-open]");
    if (!o) return;
    var tpl = doc.getElementById("seat-sheet"), body = doc.getElementById("sheet-body");
    if (!tpl || !body) return;
    body.innerHTML = ""; body.appendChild(tpl.content.cloneNode(true));
    M() && M().openSheet();
  });

  window.EvaFx = { haptic: haptic, burst: burst, counters: counters, takeover: takeover, status: function () { return status; }, refreshStatus: refreshStatus };
})();
