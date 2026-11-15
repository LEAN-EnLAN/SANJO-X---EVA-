// Wristband mode (APP-POLISH §9): switch between the phone UI and the wristband UI when the
// screen changes shape, rewrite /app/* navigations to their wrist twins, and drive the
// five-page track (scroll-snap paging, dots, arrows, keys, ambient dim, Help hold, haptics).
(function () {
  "use strict";
  var html = document.documentElement;
  if (!/^\/app(\/|$)/.test(location.pathname)) return;
  var W = "(orientation: landscape) and (max-height: 480px) and (min-aspect-ratio: 8/5), (max-width: 340px) and (max-height: 340px)";
  var mq = window.matchMedia(W);
  var TWINS = { eva: "eva", ecg: "ecg", emergency: "emergency", storm: "storm" };
  var reduced = window.matchMedia("(prefers-reduced-motion: reduce)");
  var fx = function (n) { if (window.EvaFx) window.EvaFx.haptic(n); else if (navigator.vibrate) try { navigator.vibrate(8); } catch (e) {} };
  var mode = function () { return html.getAttribute("data-mode"); };
  function forced() { try { return localStorage.getItem("eva-mode"); } catch (e) { return null; } }
  function seatQ(search) { var s = new URLSearchParams(search || location.search).get("seat") || (document.body && document.body.getAttribute("data-seat")); return s ? "?seat=" + encodeURIComponent(s) : ""; }
  function wristTwin(path) { var k = path.replace(/^\/app\/?/, ""); return "/app/wrist" + (TWINS[k] ? "/" + TWINS[k] : ""); }
  function phoneTwin(path) { var k = path.replace(/^\/app\/wrist\/?/, ""); return "/app" + (TWINS[k] ? "/" + TWINS[k] : ""); }

  // ---- flip on rotation / resize (debounced; never while typing: the keyboard changes the shape)
  var t = 0;
  function evaluate() {
    clearTimeout(t);
    t = setTimeout(function () {
      var a = document.activeElement;
      if (a && (a.tagName === "INPUT" || a.tagName === "TEXTAREA")) return;
      var f = forced();
      html.toggleAttribute("data-letterbox", (f || (mq.matches ? "wristband" : "phone")) === "wristband" && !mq.matches);
      if (f) return;
      var want = mq.matches ? "wristband" : "phone";
      if (want === mode()) return;
      html.setAttribute("data-mode", want);
      var path = location.pathname, wristNow = path.indexOf("/app/wrist") === 0;
      var twin = (want === "wristband" ? (wristNow ? path : wristTwin(path)) : (wristNow ? phoneTwin(path) : path)) + seatQ();
      if (window.EvaMotion) window.EvaMotion.expect("fade");
      // the SSE connection stays open: only #screen changes
      htmx.ajax("GET", twin, { target: "#screen" }).then(function () { history.replaceState({ htmx: true }, "", twin); if (window.EvaMotion) window.EvaMotion.syncTabs(); });
    }, 150);
  }
  if (mq.addEventListener) mq.addEventListener("change", evaluate);
  window.addEventListener("resize", evaluate);
  window.addEventListener("orientationchange", evaluate);

  // ---- one place that rewrites navigation: every /app/<x> load into #screen goes to its wrist twin
  document.addEventListener("htmx:configRequest", function (e) {
    if (mode() !== "wristband") return;
    var tg = e.detail.target, path = e.detail.path || "";
    if (!tg || tg.id !== "screen" || e.detail.verb !== "get") return;
    var u = new URL(path, location.origin);
    if (!/^\/app(\/|$)/.test(u.pathname) || u.pathname.indexOf("/app/wrist") === 0 || u.pathname === "/app/tools") return;
    e.detail.path = wristTwin(u.pathname) + seatQ(u.search);
  });
  document.addEventListener("htmx:afterSwap", function (e) {
    if (mode() !== "wristband" || !e.detail.target || e.detail.target.id !== "screen") return;
    var p = location.pathname;
    if (p.indexOf("/app/wrist") !== 0) history.replaceState({ htmx: true }, "", wristTwin(p) + seatQ());
    mountTrack(); arrive();
  });

  // ---- the track
  var track = null, pages = [], dots = [], cur = 0, idle = 0, dimT = 0, homeT = 0;
  function go(i, smooth) {
    if (!track) return;
    i = Math.max(0, Math.min(pages.length - 1, i));
    track.scrollTo({ left: i * track.clientWidth, behavior: smooth && !reduced.matches ? "smooth" : "auto" });
  }
  function setCur(i) {
    if (i === cur && dots[i] && dots[i].classList.contains("on")) return;
    var changed = i !== cur; cur = i;
    var tints = { eva: "#1FE5C4", heart: "#F28A1A", brand: "#F28A1A" };
    dots.forEach(function (d, k) { d.classList.toggle("on", k === i); });
    var w = document.querySelector("[data-wrist]");
    if (w) w.style.setProperty("--dot", tints[pages[i] && pages[i].getAttribute("data-tint")] || "#1FE5C4");
    var prev = document.querySelector("[data-wrist-step='-1']"), next = document.querySelector("[data-wrist-step='1']");
    if (prev) prev.hidden = i === 0; if (next) next.hidden = i === pages.length - 1;
    if (changed) fx("tick");
  }
  function mountTrack() {
    var w = document.querySelector("[data-wrist]");
    track = w && w.querySelector("[data-wrist-track]");
    if (!track || track._m) return;
    track._m = 1;
    pages = Array.prototype.slice.call(track.querySelectorAll(".w-page"));
    dots = Array.prototype.slice.call(w.querySelectorAll(".w-dots > i"));
    cur = -1;
    var start = parseInt(w.getAttribute("data-start"), 10) || 0;
    requestAnimationFrame(function () { go(start, false); setCur(start); });
    if ("IntersectionObserver" in window) {
      var io = new IntersectionObserver(function (es) {
        es.forEach(function (en) {
          en.target.classList.toggle("is-paused", !en.isIntersecting);
          if (en.isIntersecting && en.intersectionRatio >= 0.6) setCur(pages.indexOf(en.target));
        });
      }, { root: track, threshold: [0, 0.6] });
      pages.forEach(function (p) { io.observe(p); });
    }
    if (!window.isSecureContext || !navigator.mediaDevices) w.classList.add("no-voice");
    // the ECG result returns to the face after 8 s
    if (start === 3 && w.querySelector("[data-wrist-ecg-result]")) setTimeout(function () { if (cur === 3) go(0, true); }, 8000);
    ecgCountdown(w);
    rulesNote(w);
    wake();
  }
  document.addEventListener("click", function (e) {
    var s = e.target.closest && e.target.closest("[data-wrist-step]");
    if (s) { go(cur + parseInt(s.getAttribute("data-wrist-step"), 10), true); return; }
    var g = e.target.closest && e.target.closest("[data-wrist-go]");
    if (g) { go(parseInt(g.getAttribute("data-wrist-go"), 10), true); return; }
    var v = e.target.closest && e.target.closest(".w-vrow");
    if (v && cur === 0) { go(1, true); return; }
    var p = e.target.closest && e.target.closest("[data-panel]");
    if (p) { openPanel = p.classList.contains("is-open") ? null : p.getAttribute("data-panel"); applyPanel(); fx("tap"); return; }
    var hb = e.target.closest && e.target.closest("[data-haptic]");
    if (hb) fx(hb.getAttribute("data-haptic"));
  });
  // a tapped vitals panel stays open across the live refreshes
  var openPanel = null;
  function applyPanel() { document.querySelectorAll("[data-panel]").forEach(function (p) { p.classList.toggle("is-open", p.getAttribute("data-panel") === openPanel); }); }
  document.addEventListener("htmx:afterSwap", function (e) { if (e.detail.target && e.detail.target.classList && e.detail.target.classList.contains("w-vitals")) applyPanel(); });
  document.addEventListener("keydown", function (e) {
    if (mode() !== "wristband" || !track || document.querySelector("[data-focus]")) return;
    if (e.key === "ArrowRight") { e.preventDefault(); go(cur + 1, true); }
    else if (e.key === "ArrowLeft") { e.preventDefault(); go(cur - 1, true); }
    else if (e.key === "Escape") { openPanel = null; applyPanel(); go(0, true); }
    else if (e.key === "h" || e.key === "H") go(4, true);
    else if (e.key === "Enter") { var b = pages[cur] && pages[cur].querySelector(".w-btn, .w-mic, .w-pill"); if (b && document.activeElement === document.body) b.click(); }
  });

  // ---- ambient dim after 20 s, back to the face after 30 s; any touch or alert wakes it
  function wake() {
    var w = document.querySelector("[data-wrist]");
    if (w) w.classList.remove("wrist-ambient");
    clearTimeout(dimT); clearTimeout(homeT);
    if (!w) return;
    dimT = setTimeout(function () { if (!document.querySelector("[data-focus]")) { go(0, false); w.classList.add("wrist-ambient"); } }, 20000);
    homeT = setTimeout(function () { if (cur !== 0) go(0, true); }, 30000);
  }
  ["pointerdown", "keydown"].forEach(function (k) { document.addEventListener(k, function () { if (mode() === "wristband") wake(); }, { passive: true }); });
  document.addEventListener("htmx:sseBeforeMessage", function (e) { if (mode() === "wristband" && e.detail && /^go-|banner/.test(e.detail.type) && e.detail.type !== "banner") wake(); });
  window.addEventListener("devicemotion", function (e) { var a = e.acceleration; if (a && Math.abs(a.x || 0) + Math.abs(a.y || 0) + Math.abs(a.z || 0) > 12) wake(); });

  // ---- takeover arrivals: the pattern the screen asks for (patient: three long pulses, then one every 10 s)
  var wakeT = 0;
  function arrive() {
    clearInterval(wakeT);
    var a = document.querySelector("#screen [data-wrist-arrive]");
    if (!a) return;
    var k = a.getAttribute("data-wrist-arrive");
    fx(k);
    if (k === "wake") wakeT = setInterval(function () { if (document.querySelector("[data-wrist-arrive='wake']")) fx([80]); else clearInterval(wakeT); }, 10000);
    if (navigator.wakeLock && navigator.wakeLock.request) navigator.wakeLock.request("screen").catch(function () {});
  }
  document.addEventListener("htmx:sseBeforeMessage", function (e) { if (mode() === "wristband" && e.detail && e.detail.type === "go-home" && document.querySelector("#screen [data-focus], #screen [data-storm-active]")) fx("success"); });

  // ---- Help: hold 2 s to call (a bump must not call the crew)
  function holdable(sel, ms, onDone, hintSel) {
    var el = null, t0 = 0, ok = false, raf = 0;
    document.addEventListener("pointerdown", function (e) {
      el = e.target.closest && e.target.closest(sel);
      if (!el) return;
      e.preventDefault(); t0 = performance.now(); ok = false;
      var ring = el.querySelector("[data-help-ring]"), core = el.querySelector(".w-helpcore"), hint = hintSel && el.querySelector(hintSel), lastBuzz = 0;
      if (hint && reduced.matches) hint.textContent = "2…";
      (function step(now) {
        if (!el) return;
        var p = Math.min(1, (now - t0) / ms);
        if (ring && !reduced.matches) ring.style.strokeDashoffset = String(100 - p * 100);
        if (core) core.style.setProperty("--hold-s", String(1 - 0.04 * Math.min(1, p * 4)));
        if (hint && reduced.matches) hint.textContent = p < 0.5 ? "2…" : "1…";
        if (now - lastBuzz >= 400) { lastBuzz = now; fx("hold"); }
        if (p >= 1) { ok = true; var done = el; el = null; reset(done); onDone(done); return; }
        raf = requestAnimationFrame(step);
      })(t0);
    });
    function reset(target, early) {
      var ring = target.querySelector("[data-help-ring]"), core = target.querySelector(".w-helpcore"), hint = hintSel && target.querySelector(hintSel);
      if (ring) ring.style.strokeDashoffset = "100";
      if (core) core.style.setProperty("--hold-s", "1");
      if (hint) hint.textContent = early ? "Keep holding to call" : "Hold 2 seconds";
    }
    function up() { cancelAnimationFrame(raf); if (el && !ok) reset(el, true); el = null; }
    document.addEventListener("pointerup", up); document.addEventListener("pointercancel", up);
  }
  holdable("[data-wrist-help]", 2000, function (b) { fx("emergency"); htmx.trigger(b, "wrist-help"); }, "[data-help-hint]");
  holdable("[data-wrist-hold]", 2000, function (b) { fx("success"); htmx.trigger(b, "wrist-hold"); });

  // ---- Eva quick-ask: her newest reply in the bubble, 2-line chunks every 3.5 s
  var chunkT = 0;
  document.addEventListener("htmx:afterSwap", function (e) {
    if (!e.detail.target || e.detail.target.id !== "chat" || mode() !== "wristband") return;
    var msgs = e.detail.target.querySelectorAll("[data-eva-msg]"), last = msgs[msgs.length - 1], b = document.querySelector("[data-wrist-bubble]");
    if (!last || !b) return;
    var text = Array.prototype.map.call(last.querySelectorAll("[data-say]"), function (s) { return s.textContent.trim(); }).filter(Boolean);
    if (!text.length) text = [last.textContent.trim()];
    var i = 0; clearInterval(chunkT);
    b.textContent = text[0];
    if (text.length > 1) chunkT = setInterval(function () { i++; if (i >= text.length) { clearInterval(chunkT); return; } b.textContent = text[i]; }, 3500);
  });
  function rulesNote(w) {
    var b = w.querySelector("[data-wrist-bubble]");
    if (!b || html.getAttribute("data-eva-mode") !== "rules") return;
    try { if (sessionStorage.getItem("eva-wrist-rules")) return; sessionStorage.setItem("eva-wrist-rules", "1"); } catch (e) {}
    b.textContent = "On-board mode. I can still log and warn.";
  }

  // ---- ECG countdown ring and seconds (a tick every 10 s)
  function ecgCountdown(w) {
    var box = w.querySelector("[data-wrist-ecg]");
    if (!box) return;
    var left = parseInt(box.getAttribute("data-left"), 10) || 30000, end = performance.now() + left, last = -1;
    var secs = box.querySelector("[data-wrist-secs]"), ring = box.querySelector("[data-wrist-ecg-ring]");
    var iv = setInterval(function () {
      if (!document.body.contains(box)) return clearInterval(iv);
      var ms = Math.max(0, end - performance.now()), s = Math.ceil(ms / 1000);
      if (secs) secs.textContent = s;
      if (ring) ring.style.strokeDashoffset = String(100 - ms / 300);
      if (s !== last && s > 0 && s % 10 === 0 && last !== -1) fx("tick");
      if (s === 0 && last !== 0) fx("success");
      last = s;
    }, 250);
  }

  // ---- dev affordance: 5 taps on the time open the phone UI
  var taps = [];
  document.addEventListener("click", function (e) {
    if (!e.target.closest || !e.target.closest("[data-wrist-time]")) return;
    var now = Date.now(); taps = taps.filter(function (x) { return now - x < 2000; }); taps.push(now);
    if (taps.length >= 5) location.href = "/app" + seatQ() + (seatQ() ? "&" : "?") + "mode=phone";
  });

  document.addEventListener("DOMContentLoaded", function () { if (mode() === "wristband") { mountTrack(); arrive(); } });
  document.addEventListener("wrist:refresh", function () { /* hx-trigger listens on body */ });
})();
