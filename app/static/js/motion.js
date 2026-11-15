// Eva motion engine: springs only (docs/app/MOTION.md).
// - Spring tokens (mass / stiffness / damping) are read from CSS custom properties,
//   so CSS and JS share one source. CSS `linear()` easings are regenerated from them.
// - Every animation is a spring with position + velocity: setting a new target mid-flight
//   keeps the current velocity (interruptible, retargetable, no jumps).
// - Gestures drive values directly while the finger is down, then hand their release
//   velocity to a spring (continuous with the gesture).
// - prefers-reduced-motion: values jump to their target, idle loops stop.
(function () {
  "use strict";
  var reduced = window.matchMedia("(prefers-reduced-motion: reduce)");
  var css = getComputedStyle(document.documentElement);
  var NAMES = ["snappy", "gentle", "bouncy", "slow"];

  function token(name) {
    function n(k, d) { var v = parseFloat(css.getPropertyValue("--spring-" + name + "-" + k)); return isFinite(v) ? v : d; }
    return { mass: n("mass", 1), stiffness: n("stiffness", 170), damping: n("damping", 26) };
  }

  // sample a unit spring into a CSS linear() easing + duration
  function linearFor(cfg) {
    var x = 0, v = 0, t = 0, dt = 1 / 1000, pts = [0];
    while (t < 3) {
      var a = (-cfg.stiffness * (x - 1) - cfg.damping * v) / cfg.mass;
      v += a * dt; x += v * dt; t += dt; pts.push(x);
      if (t > 0.05 && Math.abs(x - 1) < 0.001 && Math.abs(v) < 0.01) break;
    }
    var out = [], n = 40;
    for (var i = 0; i <= n; i++) out.push(pts[Math.min(pts.length - 1, Math.round((pts.length - 1) * i / n))].toFixed(4));
    return { ease: "linear(" + out.join(", ") + ")", ms: Math.round(t * 1000) };
  }
  NAMES.forEach(function (name) {
    var l = linearFor(token(name));
    document.documentElement.style.setProperty("--ease-spring-" + name, l.ease);
    document.documentElement.style.setProperty("--dur-spring-" + name, l.ms + "ms");
  });

  // ---- one rAF loop for all active springs
  var active = new Set(), raf = 0, last = 0;
  function loop(now) {
    var dt = Math.min(0.064, (now - (last || now)) / 1000); last = now;
    active.forEach(function (s) { if (s._step(dt)) active.delete(s); });
    raf = active.size ? requestAnimationFrame(loop) : 0;
    if (!raf) last = 0;
  }
  function wake(s) { active.add(s); if (!raf) raf = requestAnimationFrame(loop); }

  function Spring(value, cfg, onUpdate) {
    this.x = value; this.v = 0; this.target = value;
    this.cfg = typeof cfg === "string" ? token(cfg) : (cfg || token("gentle"));
    this.onUpdate = onUpdate || function () {};
  }
  Spring.prototype.set = function (target, cfg, velocity) {
    this.target = target;
    if (cfg) this.cfg = typeof cfg === "string" ? token(cfg) : cfg;
    if (velocity !== undefined) this.v = velocity;
    if (reduced.matches) { this.x = target; this.v = 0; this.onUpdate(this.x); return this; }
    wake(this); return this;
  };
  // follow a gesture directly (no animation), remembering velocity for the release
  Spring.prototype.track = function (value, dt) {
    if (dt > 0) this.v = (value - this.x) / dt;
    this.x = value; this.target = value; active.delete(this); this.onUpdate(this.x);
  };
  Spring.prototype._step = function (dt) {
    var c = this.cfg, steps = Math.max(1, Math.ceil(dt * 240)), h = dt / steps;
    for (var i = 0; i < steps; i++) {
      var a = (-c.stiffness * (this.x - this.target) - c.damping * this.v) / c.mass;
      this.v += a * h; this.x += this.v * h;
    }
    var done = Math.abs(this.x - this.target) < 0.0005 * Math.max(1, Math.abs(this.target)) && Math.abs(this.v) < 0.01;
    if (done) { this.x = this.target; this.v = 0; }
    this.onUpdate(this.x);
    return done;
  };

  // drive a CSS custom property on an element with a spring
  function varSpring(el, prop, initial, cfg) {
    return new Spring(initial, cfg, function (x) { el.style.setProperty(prop, x.toFixed(4)); });
  }

  // ---- press feedback on .press: scale follows the finger down, springs back up
  document.addEventListener("pointerdown", function (e) {
    var el = e.target.closest && e.target.closest(".press");
    if (!el || reduced.matches) return;
    var s = el._press || (el._press = varSpring(el, "--press", 1, "snappy"));
    s.set(0.95, "snappy");
    function up() { s.set(1, "bouncy"); el.removeEventListener("pointerup", up); el.removeEventListener("pointerleave", up); el.removeEventListener("pointercancel", up); }
    el.addEventListener("pointerup", up); el.addEventListener("pointerleave", up); el.addEventListener("pointercancel", up);
  }, { passive: true });

  // ---- Eva mascot: state-driven springs
  // targets per state: y (px), s (scale), r (deg), glow (opacity), gs (glow scale), and an
  // optional idle loop that alternates targets so the motion keeps breathing.
  // Every state has its own image (static/img/eva/eva-<state>.webp, Codex kit). A change of
  // state cross-fades the two images on the gentle spring and gives the body a small squash
  // that the bouncy spring releases. `data-eva-fixed` (the tab-bar face) keeps its image and
  // only moves.
  var STATES = {
    idle:          { y: 0,  s: 1,    r: 0,  glow: 0.55, gs: 1,    cfg: "slow",   loop: { every: 2600, y: -3, s: 1.025, glow: 0.7 } },
    hello:         { y: -6, s: 1.05, r: 3,  glow: 0.8,  gs: 1.12, cfg: "bouncy", settle: { after: 480, y: 0, s: 1.02, r: 0 } },
    listening:     { y: -4, s: 1.06, r: -4, glow: 0.9,  gs: 1.15, cfg: "gentle", loop: { every: 900, s: 1.08, glow: 1 } },
    thinking:      { y: -2, s: 1,    r: -6, glow: 0.6,  gs: 1.05, cfg: "gentle", loop: { every: 800, r: 6 } },
    speaking:      { y: -1, s: 1.02, r: 0,  glow: 0.8,  gs: 1.1,  cfg: "snappy", loop: { every: 260, s: 1.06, y: -3 } },
    happy:         { y: -10, s: 1.08, r: 3, glow: 0.85, gs: 1.2,  cfg: "bouncy", settle: { after: 420, y: 0, s: 1.03 } },
    wink:          { y: -4, s: 1.04, r: 5,  glow: 0.75, gs: 1.1,  cfg: "bouncy", settle: { after: 360, y: 0, r: 2 } },
    concerned:     { y: 2,  s: 0.97, r: -3, glow: 0.7,  gs: 1,    cfg: "slow",   loop: { every: 2000, y: 0 } },
    alert:         { y: -2, s: 1.04, r: 0,  glow: 1,    gs: 1.25, cfg: "snappy", loop: { every: 500, s: 1, gs: 1.1 } },
    calm:          { y: 0,  s: 1,    r: 0,  glow: 0.65, gs: 1.05, cfg: "slow",   loop: { every: 3200, y: -2, glow: 0.8, gs: 1.12 } },
    sleeping:      { y: 2,  s: 0.98, r: -2, glow: 0.3,  gs: 0.95, cfg: "slow",   loop: { every: 3600, y: 0, s: 1, glow: 0.4 } },
    celebrate:     { y: -14, s: 1.1, r: -3, glow: 1,    gs: 1.3,  cfg: "bouncy", loop: { every: 520, y: -4, r: 3, s: 1.05 } },
    care:          { y: 1,  s: 0.99, r: -2, glow: 0.7,  gs: 1.05, cfg: "slow",   loop: { every: 2400, y: -1, r: 0 } },
    reading:       { y: 0,  s: 1,    r: -2, glow: 0.6,  gs: 1,    cfg: "gentle", loop: { every: 1800, r: 1 } },
    writing:       { y: 0,  s: 1,    r: 2,  glow: 0.6,  gs: 1,    cfg: "gentle", loop: { every: 700, r: -1, y: -1 } },
    sending:       { y: -5, s: 1.03, r: 4,  glow: 0.85, gs: 1.15, cfg: "snappy", loop: { every: 600, y: -2, gs: 1.05 } },
    "low-battery": { y: 3,  s: 0.97, r: -4, glow: 0.35, gs: 0.95, cfg: "slow",   loop: { every: 2800, y: 1 } }
  };

  function Mascot(el) {
    this.el = el;
    this.sp = {
      y: varSpring(el, "--eva-y", 0), s: varSpring(el, "--eva-s", 1), r: varSpring(el, "--eva-r", 0),
      glow: varSpring(el, "--eva-glow", 0.55), gs: varSpring(el, "--eva-gs", 1),
      sx: varSpring(el, "--eva-sx", 1), sy: varSpring(el, "--eva-sy", 1)
    };
    this.timer = 0; this.flip = false;
    this.setState(el.getAttribute("data-state") || "idle", true);
  }
  Mascot.prototype.apply = function (t, cfg) {
    var sp = this.sp;
    Object.keys(sp).forEach(function (k) { if (t[k] !== undefined) sp[k].set(t[k], cfg); });
  };
  // the new image fades in over the old one; the old one leaves when the new is fully in
  Mascot.prototype.swapArt = function (src) {
    var img = this.el.querySelector("img.eva-body:not([data-leaving])");
    if (!img || img.getAttribute("src") === src) return;
    if (reduced.matches) { img.setAttribute("src", src); return; }
    var next = img.cloneNode(false);
    next.setAttribute("src", src);
    next.classList.add("eva-next");
    next.style.opacity = "0";
    img.setAttribute("data-leaving", "");
    img.after(next);
    var done = false;
    function finish() {
      if (done) return; done = true;
      img.remove(); next.classList.remove("eva-next"); next.style.opacity = "";
    }
    var fade = new Spring(0, "gentle", function (x) {
      next.style.opacity = x.toFixed(3); img.style.opacity = (1 - x).toFixed(3);
      if (x > 0.995) finish();
    });
    var go = function () { fade.set(1, "gentle"); setTimeout(finish, 900); };
    if (next.complete) go(); else { next.onload = go; next.onerror = finish; }
  };
  Mascot.prototype.setState = function (state, first) {
    if (!STATES[state]) state = "idle";
    var st = STATES[state], self = this, changed = state !== this.state;
    this.state = state;
    this.el.setAttribute("data-state", state);
    if (!this.el.hasAttribute("data-eva-fixed")) this.swapArt("/static/img/eva/eva-" + state + ".webp");
    clearInterval(this.timer); clearTimeout(this.settle);
    // a small squash on every change of state, released by the bouncy spring
    if (changed && !first && !reduced.matches) {
      this.sp.sx.track(1.07, 0); this.sp.sy.track(0.9, 0);
      this.sp.sx.set(1, "bouncy"); this.sp.sy.set(1, "bouncy");
    }
    this.apply(st, st.cfg);
    if (st.settle) this.settle = setTimeout(function () { self.apply(st.settle, "gentle"); }, st.settle.after);
    if (st.loop && !reduced.matches) {
      this.flip = false;
      this.timer = setInterval(function () {
        if (!document.body.contains(self.el)) { clearInterval(self.timer); return; }
        self.flip = !self.flip;
        self.apply(self.flip ? st.loop : st, st.cfg);
      }, st.loop.every);
    }
  };

  function mountMascots(root) {
    root = root || document;
    if (root.matches && root.matches("[data-eva]") && !root._eva) root._eva = new Mascot(root);
    root.querySelectorAll("[data-eva]").forEach(function (el) { if (!el._eva) el._eva = new Mascot(el); });
  }
  function setMood(state) {
    document.querySelectorAll("[data-eva]").forEach(function (el) { (el._eva || (el._eva = new Mascot(el))).setState(state); });
  }

  document.addEventListener("DOMContentLoaded", function () { mountMascots(document); });
  document.addEventListener("htmx:load", function (e) { mountMascots(e.target); });
  reduced.addEventListener && reduced.addEventListener("change", function () { setMood(document.querySelector("[data-eva]")?.getAttribute("data-state") || "idle"); });

  // first-load screen: the splash (Codex kit) holds a moment, then fades out on the gentle spring
  // with a slight push-in. It never takes taps (pointer-events: none) and shows once per session.
  function hideSplash() {
    var sp = document.getElementById("splash");
    if (!sp) return;
    if (reduced.matches) { sp.remove(); return; }
    var o = varSpring(sp, "--splash-o", 1, "gentle"), sc = varSpring(sp, "--splash-s", 1, "gentle");
    o.set(0, "gentle"); sc.set(1.06, "gentle");
    setTimeout(function () { sp.remove(); }, 900);
  }
  if (document.readyState === "complete") setTimeout(hideSplash, 450);
  else window.addEventListener("load", function () { setTimeout(hideSplash, 450); });

  // warm the cache with every state image once the page is idle, so a change of state never waits
  function preloadStates() {
    if (!document.querySelector("[data-eva]")) return;
    Object.keys(STATES).forEach(function (k) { var i = new Image(); i.decoding = "async"; i.src = "/static/img/eva/eva-" + k + ".webp"; });
  }
  window.addEventListener("load", function () {
    if (window.requestIdleCallback) window.requestIdleCallback(preloadStates, { timeout: 2500 });
    else setTimeout(preloadStates, 1200);
  });

  window.EvaMotion = { Spring: Spring, token: token, varSpring: varSpring, setMood: setMood, mountMascots: mountMascots, reduced: reduced, linearFor: linearFor, states: Object.keys(STATES) };

  // ================================================================ app navigation motion
  // Screens: push slides in from the right, back from the left, tabs cross-fade with a
  // tiny scale, like iOS. The values live on springs, so a new navigation mid-flight
  // retargets from where the screen is instead of jumping. Swipe from the left edge
  // drags the screen with the finger; let go and it either completes (back) or springs home.
  function scr() { return document.getElementById("screen"); }
  function vset(prop) { return function (x) { var el = scr(); if (el) el.style.setProperty(prop, x.toFixed(4)); }; }
  var S = { x: new Spring(0, "gentle", vset("--scr-x")), o: new Spring(1, "gentle", vset("--scr-o")), s: new Spring(1, "gentle", vset("--scr-s")) };
  var pendingNav = null;

  function enter(kind) {
    var w = Math.min(window.innerWidth, 640);
    if (kind === "refresh" || reduced.matches) { S.x.track(0, 0); S.o.track(1, 0); S.s.track(1, 0); return; }
    if (kind === "back") { S.x.track(-w * 0.25, 0); S.o.track(0.4, 0); S.x.set(0, "gentle"); S.o.set(1, "gentle"); }
    else if (kind === "tab") { S.x.track(0, 0); S.s.track(0.985, 0); S.o.track(0, 0); S.s.set(1, "snappy"); S.o.set(1, "snappy"); }
    else if (kind === "fade") { S.o.track(0, 0); S.o.set(1, "gentle"); }
    else { S.x.track(w * 0.3, 0); S.o.track(0.3, 0); S.x.set(0, "gentle"); S.o.set(1, "gentle"); }
  }
  function navOf(elt) {
    var n = elt && elt.closest && elt.closest("[data-nav]");
    return n ? n.getAttribute("data-nav") : "push";
  }
  document.addEventListener("htmx:beforeRequest", function (e) {
    var t = e.detail.target;
    if (t && t.id === "screen" && !pendingNav) pendingNav = navOf(e.detail.elt);
  });
  window.addEventListener("popstate", function () { pendingNav = "back"; });
  document.addEventListener("htmx:afterSwap", function (e) {
    if (!e.detail.target || e.detail.target.id !== "screen") return;
    var kind = pendingNav || "push"; pendingNav = null;
    if (kind !== "refresh" && kind !== "fade") window.scrollTo(0, 0);
    enter(kind);
    grow(e.detail.target, kind);
    syncTabs();
  });
  document.addEventListener("htmx:historyRestore", function () { enter("back"); syncTabs(); pendingNav = null; });
  // let other scripts say what the next swap is ("refresh" keeps the screen still)
  function expect(kind) { pendingNav = kind; }

  // edge swipe back (touch only): a thin strip on the left edge owns the gesture
  function edgeSwipe() {
    if (document.querySelector(".edge-swipe") || !scr()) return;
    var strip = document.createElement("div");
    strip.className = "edge-swipe fixed inset-y-0 left-0 z-30";
    strip.style.width = "16px";
    document.body.appendChild(strip);
    var x0 = 0, lastT = 0, lastX = 0, on = false;
    strip.addEventListener("pointerdown", function (e) {
      if (!document.querySelector("#screen [data-back]")) return;
      on = true; x0 = lastX = e.clientX; lastT = performance.now(); strip.setPointerCapture(e.pointerId);
    });
    strip.addEventListener("pointermove", function (e) {
      if (!on) return;
      var now = performance.now(), dx = Math.max(0, e.clientX - x0);
      S.x.track(dx, (now - lastT) / 1000); S.o.track(1 - Math.min(0.5, dx / window.innerWidth), 0);
      lastX = e.clientX; lastT = now;
    });
    function end() {
      if (!on) return; on = false;
      var w = window.innerWidth;
      if (S.x.x > w * 0.33 || S.x.v > 700) {
        S.x.set(w, "snappy", S.x.v);
        var back = document.querySelector("#screen [data-back]");
        if (back) { pendingNav = "back"; back.click(); }
      } else { S.x.set(0, "bouncy", S.x.v); S.o.set(1, "gentle"); }
    }
    strip.addEventListener("pointerup", end); strip.addEventListener("pointercancel", end);
  }

  // ---- tab bar: aria-current follows the URL, the pill springs to it
  var pill = null;
  function syncTabs() {
    var bar = document.querySelector("[data-tabbar]");
    if (!bar) return;
    var p = location.pathname.replace(/^\/app\/?/, "") || "home";
    bar.querySelectorAll("[data-tab]").forEach(function (a) {
      if (a.getAttribute("data-tab") === p) a.setAttribute("aria-current", "page"); else a.removeAttribute("aria-current");
    });
    var el = bar.querySelector("[data-tab-pill]"), cur = bar.querySelector(".tab[aria-current]");
    if (!el) return;
    if (!pill) {
      pill = { x: varSpring(el, "--pill-x", 0, "gentle"), o: varSpring(el, "--pill-o", 0, "snappy"), s: varSpring(el, "--pill-s", 1, "bouncy") };
      if (cur) { pill.x.track(cur.offsetLeft, 0); pill.o.track(1, 0); }
      bar.setAttribute("data-pill-ready", "");
      return;
    }
    if (!cur) { pill.o.set(0, "snappy"); return; }
    var from = pill.x.x, to = cur.offsetLeft;
    if (pill.o.x < 0.05) pill.x.track(to, 0); else pill.x.set(to, "gentle");
    pill.o.set(1, "snappy");
    // stretch a little while it travels, like a drop of liquid
    if (Math.abs(to - from) > 4 && !reduced.matches) { pill.s.track(1.25, 0); pill.s.set(1, "bouncy"); }
  }
  window.addEventListener("resize", function () { if (pill) { pill = null; syncTabs(); } });

  // ---- bottom sheet: open with the gentle spring, follow the finger, release with velocity
  var sheet = null;
  function sheetSpring() {
    var host = document.getElementById("sheet");
    if (!host) return null;
    if (!sheet) {
      sheet = varSpring(host, "--sheet-y", 100, "gentle");
      var base = sheet.onUpdate;
      sheet.onUpdate = function (y) { base(y); if (y >= 99.5 && sheet.target >= 100) { host.classList.add("hidden"); host.setAttribute("aria-hidden", "true"); } };
      dragSheet(host);
    }
    return sheet;
  }
  function openSheet() {
    var host = document.getElementById("sheet"), sp = sheetSpring();
    if (!sp) return;
    if (sp.x >= 99) sp.track(100, 0);
    sp.set(0, "gentle");
    host.classList.remove("hidden"); host.setAttribute("aria-hidden", "false");
    var f = host.querySelector("button, input, a"); if (f) setTimeout(function () { f.focus({ preventScroll: true }); }, 50);
  }
  function closeSheet() { var sp = sheetSpring(); if (sp && sp.target !== 100) sp.set(100, "snappy"); }
  function dragSheet(host) {
    var panel = host.querySelector(".sheet-panel"), body = host.querySelector("#sheet-body");
    var y0 = 0, lastT = 0, on = false, h = 1;
    panel.addEventListener("pointerdown", function (e) {
      var onGrab = e.target.closest("[data-sheet-grab]");
      if (!onGrab && (e.target.closest("button, input, select, a, label, textarea") || body.scrollTop > 0)) return;
      on = true; y0 = e.clientY; lastT = performance.now(); h = panel.offsetHeight || 1;
      panel.setPointerCapture(e.pointerId);
    });
    panel.addEventListener("pointermove", function (e) {
      if (!on) return;
      var now = performance.now(), dy = e.clientY - y0;
      // pulling up past open resists (rubber band), pulling down follows 1:1
      var pct = dy >= 0 ? dy / h * 100 : -Math.sqrt(-dy) / h * 100;
      sheet.track(pct, (now - lastT) / 1000); lastT = now;
    });
    function end() {
      if (!on) return; on = false;
      if (sheet.x > 30 || sheet.v > 160) closeSheet(); else sheet.set(0, "bouncy", sheet.v);
    }
    panel.addEventListener("pointerup", end); panel.addEventListener("pointercancel", end);
  }
  document.addEventListener("htmx:afterSwap", function (e) { if (e.detail.target && e.detail.target.id === "sheet-body") openSheet(); });
  document.addEventListener("click", function (e) { if (e.target.closest && e.target.closest("[data-sheet-close]")) closeSheet(); });
  document.addEventListener("keydown", function (e) { if (e.key === "Escape") closeSheet(); });

  // ---- rings and bars fill up once when their screen appears (--g springs 0 → 1)
  function grow(root, kind) {
    if (kind === "refresh") return;
    (root || document).querySelectorAll("[data-grow]").forEach(function (el) {
      var g = varSpring(el, "--g", 0, "gentle"), base = g.onUpdate;
      g.onUpdate = function (x) { base(x); if (x === 1) el.classList.remove("growing"); };
      el.classList.add("growing");
      el.style.setProperty("--g", "0");
      requestAnimationFrame(function () { g.set(1, "gentle"); });
    });
  }

  document.addEventListener("DOMContentLoaded", function () { syncTabs(); edgeSwipe(); grow(document, "push"); });
  window.EvaMotion.enter = enter; window.EvaMotion.expect = expect; window.EvaMotion.openSheet = openSheet; window.EvaMotion.closeSheet = closeSheet; window.EvaMotion.syncTabs = syncTabs;
})();
