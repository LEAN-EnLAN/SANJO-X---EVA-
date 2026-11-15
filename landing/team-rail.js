/*
 * Team Rail: ID badges hanging on lanyards from a metal rail.
 * Original vanilla JS, no dependencies.
 *
 *   var rail = TeamRail.mount(el, members, opts);
 *   rail.flip(i, true|false) · rail.focus(i) · rail.destroy()
 *
 * member: { name, role, bio, tasks:[], tags:[], links:[{label, href}],
 *           accent?, photo?, initials?, team?, cord? }
 * opts:   { title, meta, teamLine, org, bandLabel, accents:[], minColumn,
 *           labels:{...}, onFlip(index, flipped) }
 */
(function (global) {
  'use strict';

  var ACCENTS = ['#6f8068', '#5f7190', '#9a6a55', '#8f7c48', '#77698a', '#5b817e'];
  var CORDS = [132, 190, 152, 208, 170];
  var STEP = 1 / 240;            // fixed physics sub-step, seconds
  var MAX_ANGLE = 26;            // degrees
  var STRETCH_K = 190;           // cord spring stiffness
  var STRETCH_C = 12;            // cord spring damping
  var DEG = 180 / Math.PI;
  var count = 0;

  var LABELS = {
    profile: 'Profile',
    tasks: 'Tasks',
    links: 'Links',
    idle: 'Hover or focus a badge',
    hintRail: 'Enter to flip · ← → to move',
    hintCarousel: 'Tap to flip · swipe for more',
    backRail: 'Click again or Esc to flip back',
    backCarousel: 'Tap again to flip back',
    dots: 'Choose a team member',
    show: 'Show',
    member: 'member',
    members: 'members'
  };

  /* ---------- small helpers ---------- */

  function hash(str) {
    var h = 2166136261;
    for (var i = 0; i < str.length; i++) {
      h ^= str.charCodeAt(i);
      h = Math.imul(h, 16777619);
    }
    return h >>> 0;
  }

  // Small deterministic generator (xorshift32) so every badge keeps its look.
  function rng(seed) {
    var x = seed || 1;
    return function () {
      x ^= x << 13; x >>>= 0;
      x ^= x >>> 17;
      x ^= x << 5; x >>>= 0;
      return x / 4294967296;
    };
  }

  function clamp(v, lo, hi) { return v < lo ? lo : v > hi ? hi : v; }
  function pad2(n) { return (n < 10 ? '0' : '') + n; }

  function make(tag, cls, attrs, text) {
    var n = document.createElement(tag);
    if (cls) n.className = cls;
    if (attrs) {
      for (var k in attrs) {
        if (attrs[k] != null) n.setAttribute(k, attrs[k]);
      }
    }
    if (text != null) n.textContent = text;
    return n;
  }

  function initialsOf(name) {
    var parts = String(name || '').trim().split(/\s+/).filter(Boolean);
    if (!parts.length) return '?';
    if (parts.length === 1) return parts[0].charAt(0).toUpperCase();
    return (parts[0].charAt(0) + parts[parts.length - 1].charAt(0)).toUpperCase();
  }

  function barcodeImage(seed) {
    var r = rng(seed), x = 0, dark = true, stops = [];
    while (x < 100) {
      var w = dark ? 0.7 + r() * 2.6 : 0.9 + r() * 2.2;
      var nx = Math.min(100, x + w);
      stops.push((dark ? '#161616' : 'transparent') + ' ' + x.toFixed(2) + '% ' + nx.toFixed(2) + '%');
      x = nx;
      dark = !dark;
    }
    return 'linear-gradient(90deg,' + stops.join(',') + ')';
  }

  function portraitImage(accent, seed) {
    var r = rng(seed ^ 0x9e3779b9);
    var angle = Math.round(r() * 360);
    var light = 18 + Math.round(r() * 14);
    var deep = 48 + Math.round(r() * 20);
    var hx = 20 + Math.round(r() * 45), hy = 12 + Math.round(r() * 30);
    return 'radial-gradient(circle at ' + hx + '% ' + hy + '%, rgba(255,255,255,.6), rgba(255,255,255,0) 58%),' +
      'linear-gradient(' + angle + 'deg, color-mix(in srgb, ' + accent + ' ' + light + '%, #fff),' +
      ' color-mix(in srgb, ' + accent + ' ' + deep + '%, #fff))';
  }

  function matchesFocusVisible(node) {
    try { return node.matches(':focus-visible'); } catch (e) { return true; }
  }

  /* ---------- DOM for one member ---------- */

  function buildItem(m, i, n, ctx) {
    var accent = m.accent || ctx.accents[i % ctx.accents.length];
    var seed = hash(String(m.name) + '|' + i);
    var cord = m.cord || CORDS[i % CORDS.length];
    var backId = ctx.uid + '-back-' + i;

    var li = make('li', 'tr-item', { 'data-index': i });
    li.style.setProperty('--accent', accent);
    li.style.setProperty('--cord', cord);

    var reel = make('span', 'tr-reel', { 'aria-hidden': 'true' });
    var hanger = make('div', 'tr-hanger');
    var strap = make('span', 'tr-strap', { 'aria-hidden': 'true' });
    var tag = make('div', 'tr-tag');
    var clip = make('span', 'tr-clip', { 'aria-hidden': 'true' });
    var badge = make('div', 'tr-badge');
    var btn = make('button', 'tr-flip', {
      type: 'button',
      'aria-pressed': 'false',
      'aria-controls': backId,
      'aria-label': m.name + (m.role ? ', ' + m.role : ''),
      tabindex: i === 0 ? '0' : '-1'
    });
    var card = make('div', 'tr-card');

    // Front face
    var front = make('div', 'tr-face tr-front', { 'aria-hidden': 'true' });
    var band = make('div', 'tr-band');
    band.appendChild(make('span', 'tr-band-l', null, ctx.bandLabel));
    band.appendChild(make('span', 'tr-band-r', null, pad2(i + 1) + '/' + pad2(n)));
    front.appendChild(band);
    front.appendChild(make('span', 'tr-slot'));
    var portrait = make('div', 'tr-portrait');
    portrait.style.backgroundColor = accent;
    portrait.style.backgroundImage = portraitImage(accent, seed);
    if (m.photo) {
      portrait.appendChild(make('img', null, { src: m.photo, alt: '', loading: 'lazy', decoding: 'async' }));
    } else {
      portrait.appendChild(make('span', 'tr-initials', null, m.initials || initialsOf(m.name)));
    }
    front.appendChild(portrait);
    front.appendChild(make('p', 'tr-name', null, m.name));
    if (m.role) front.appendChild(make('p', 'tr-role', null, m.role));
    var teamLine = m.team || ctx.teamLine;
    if (teamLine) front.appendChild(make('p', 'tr-team', null, teamLine));
    var foot = make('div', 'tr-foot');
    var code = make('span', 'tr-barcode');
    code.style.backgroundImage = barcodeImage(seed);
    foot.appendChild(code);
    var row = make('div', 'tr-foot-row');
    row.appendChild(make('span', null, null, ctx.serialPrefix + pad2(i + 1)));
    if (ctx.org) row.appendChild(make('span', null, null, ctx.org));
    foot.appendChild(row);
    front.appendChild(foot);

    // Back face
    var back = make('div', 'tr-face tr-back', { id: backId, 'aria-hidden': 'true' });
    back.appendChild(make('div', 'tr-band tr-band--thin'));
    back.appendChild(make('span', 'tr-slot'));
    var body = make('div', 'tr-back-body');
    body.appendChild(make('p', 'tr-kicker', null, ctx.labels.profile + ' · ' + pad2(i + 1)));
    body.appendChild(make('p', 'tr-name', null, m.name));
    if (m.role) body.appendChild(make('p', 'tr-role', null, m.role));
    if (m.bio) body.appendChild(make('p', 'tr-bio', null, m.bio));
    var tasks = (m.tasks || []).filter(Boolean);
    if (tasks.length) {
      body.appendChild(make('p', 'tr-label', null, ctx.labels.tasks));
      var tl = make('ul', 'tr-tasks', { role: 'list' });
      tasks.forEach(function (t) { tl.appendChild(make('li', null, null, t)); });
      body.appendChild(tl);
    }
    var tags = (m.tags || []).filter(Boolean).slice(0, 3);
    if (tags.length) {
      var tg = make('ul', 'tr-tags', { role: 'list' });
      tags.forEach(function (t) { tg.appendChild(make('li', null, null, t)); });
      body.appendChild(tg);
    }
    var links = (m.links || []).filter(function (l) { return l && l.href; });
    var anchors = [];
    if (links.length) {
      var ll = make('ul', 'tr-links', { role: 'list', 'aria-label': ctx.labels.links });
      links.forEach(function (l) {
        var item = make('li');
        var external = /^https?:/i.test(l.href);
        var a = make('a', null, {
          href: l.href,
          tabindex: '-1',
          target: external ? '_blank' : null,
          rel: external ? 'noopener noreferrer' : null
        }, l.label || l.href);
        anchors.push(a);
        item.appendChild(a);
        ll.appendChild(item);
      });
      body.appendChild(ll);
    }
    back.appendChild(body);
    var backHint = make('p', 'tr-back-hint', { 'aria-hidden': 'true' }, ctx.labels.backRail);
    back.appendChild(backHint);

    card.appendChild(front);
    card.appendChild(back);
    badge.appendChild(card);
    badge.appendChild(btn);
    tag.appendChild(clip);
    tag.appendChild(badge);
    hanger.appendChild(strap);
    hanger.appendChild(tag);
    li.appendChild(reel);
    li.appendChild(hanger);

    return {
      li: li, hanger: hanger, strap: strap, tag: tag, badge: badge,
      btn: btn, back: back, backHint: backHint, anchors: anchors, cord: cord
    };
  }

  /* ---------- mount ---------- */

  function mount(target, members, opts) {
    var host = typeof target === 'string' ? document.querySelector(target) : target;
    if (!host) throw new Error('TeamRail.mount: target element not found');
    if (host.__teamRail) host.__teamRail.destroy();

    members = (members || []).filter(function (m) { return m && m.name; });
    opts = opts || {};
    var labels = {};
    for (var key in LABELS) labels[key] = LABELS[key];
    if (opts.labels) for (var lk in opts.labels) labels[lk] = opts.labels[lk];

    var uid = 'tr' + (++count);
    var ctx = {
      uid: uid,
      labels: labels,
      accents: opts.accents && opts.accents.length ? opts.accents : ACCENTS,
      teamLine: opts.teamLine || '',
      org: opts.org == null ? '' : opts.org,
      bandLabel: opts.bandLabel || 'Crew',
      serialPrefix: opts.serialPrefix || 'No. '
    };
    var n = members.length;
    var minColumn = opts.minColumn || 250;

    var reduceMQ = global.matchMedia ? global.matchMedia('(prefers-reduced-motion: reduce)') : null;
    var reduced = !!(reduceMQ && reduceMQ.matches);
    var carousel = false;
    var scale = 1;
    var alive = true;
    var ac = typeof AbortController === 'function' ? new AbortController() : null;
    var cleanups = [];

    function on(node, type, fn, extra) {
      var o = { passive: !!(extra && extra.passive) };
      if (ac) o.signal = ac.signal;
      node.addEventListener(type, fn, o);
      if (!ac) cleanups.push(function () { node.removeEventListener(type, fn, o); });
    }

    // Skeleton
    var section = make('section', 'tr', { 'aria-labelledby': uid + '-title' });
    if (reduced) section.classList.add('tr--reduced');
    var head = make('header', 'tr-head');
    head.appendChild(make('h2', 'tr-title', { id: uid + '-title' }, opts.title || 'Team'));
    var metaText = opts.meta != null ? opts.meta : pad2(n) + ' ' + (n === 1 ? labels.member : labels.members);
    if (metaText) head.appendChild(make('p', 'tr-meta', null, metaText));
    section.appendChild(head);

    var stage = make('div', 'tr-stage');
    stage.appendChild(make('div', 'tr-rail', { 'aria-hidden': 'true' }));
    var track = make('ul', 'tr-track', { role: 'list' });
    track.style.setProperty('--tr-n', Math.max(1, n));
    stage.appendChild(track);
    section.appendChild(stage);

    var dots = make('div', 'tr-dots', { role: 'group', 'aria-label': labels.dots });
    section.appendChild(dots);

    var caption = make('p', 'tr-caption', { 'aria-live': 'polite', 'aria-atomic': 'true' });
    var capMain = make('span', 'tr-cap-main');
    var capHint = make('span', 'tr-cap-hint');
    caption.appendChild(capMain);
    caption.appendChild(capHint);
    section.appendChild(caption);

    var items = members.map(function (m, i) {
      var it = buildItem(m, i, n, ctx);
      track.appendChild(it.li);
      var dot = make('button', 'tr-dot', {
        type: 'button',
        'aria-label': labels.show + ' ' + m.name,
        'aria-current': i === 0 ? 'true' : 'false'
      });
      dots.appendChild(dot);
      it.dot = dot;
      return it;
    });

    host.appendChild(section);

    /* ---------- physics state ---------- */

    var S = items.map(function (it) {
      var L = it.cord + 190;               // pivot to badge centre, px
      return {
        a: 0, v: 0,                        // swing angle (deg) and velocity (deg/s)
        s: 0, sv: 0,                       // cord stretch (px) and velocity
        k: 9200 / L, c: 1.1,               // pendulum stiffness and damping
        hover: false, focus: false, flipped: false,
        drag: null, suppress: false, free: 0, timer: 0
      };
    });

    var raf = 0, last = 0, acc = 0;

    function render(i) {
      var st = S[i], it = items[i];
      var a = Math.abs(st.a) < 0.002 ? 0 : st.a;
      var s = Math.abs(st.s) < 0.01 ? 0 : st.s;
      it.hanger.style.transform = a ? 'rotate(' + a.toFixed(3) + 'deg)' : '';
      if (s) {
        var cordPx = it.cord * scale;
        it.strap.style.transform = 'scaleY(' + ((cordPx + s) / cordPx).toFixed(4) + ')';
        it.tag.style.transform = 'translateY(' + s.toFixed(2) + 'px)';
      } else {
        it.strap.style.transform = '';
        it.tag.style.transform = '';
      }
    }

    function renderAll() { for (var i = 0; i < items.length; i++) render(i); }

    function step(st, h, now) {
      if (st.drag) return;
      var settle = st.flipped || ((st.hover || st.focus) && now > st.free);
      var k = settle ? 140 : st.k;
      var c = settle ? 2 * Math.sqrt(k) : st.c;
      st.v += (-k * st.a - c * st.v) * h;
      st.a += st.v * h;
      if (st.a > MAX_ANGLE) { st.a = MAX_ANGLE; if (st.v > 0) st.v *= -0.3; }
      if (st.a < -MAX_ANGLE) { st.a = -MAX_ANGLE; if (st.v < 0) st.v *= -0.3; }
      st.sv += (-STRETCH_K * st.s - STRETCH_C * st.sv) * h;
      st.s += st.sv * h;
    }

    function atRest(st) {
      return !st.drag && Math.abs(st.a) < 0.02 && Math.abs(st.v) < 0.05 &&
        Math.abs(st.s) < 0.05 && Math.abs(st.sv) < 0.1;
    }

    function frame(now) {
      raf = 0;
      if (!alive || reduced) return;
      acc += Math.min(0.05, Math.max(0, (now - last) / 1000));
      last = now;
      while (acc >= STEP) {
        for (var i = 0; i < S.length; i++) step(S[i], STEP, now);
        acc -= STEP;
      }
      var moving = false;
      for (var j = 0; j < S.length; j++) {
        if (atRest(S[j]) && !S[j].drag) { S[j].a = S[j].v = S[j].s = S[j].sv = 0; }
        else moving = true;
        render(j);
      }
      if (moving) raf = global.requestAnimationFrame(frame);
      else acc = 0;
    }

    function kick() {
      if (raf || reduced || !alive) return;
      last = global.performance.now();
      raf = global.requestAnimationFrame(frame);
    }

    function impulse(i, dv) {
      if (reduced || i < 0 || i >= S.length) return;
      S[i].v = clamp(S[i].v + dv, -220, 220);
    }

    /* ---------- state changes ---------- */

    var active = 0;   // roving tab stop / caption subject
    var captionFor = null;

    function setRoving(i) {
      active = i;
      items.forEach(function (it, j) {
        it.btn.setAttribute('tabindex', j === i ? '0' : '-1');
        it.dot.setAttribute('aria-current', j === i ? 'true' : 'false');
      });
    }

    function refreshFront(i) {
      var st = S[i];
      items[i].li.classList.toggle('is-front', st.hover || st.focus || st.flipped);
    }

    function setCaption(i) {
      captionFor = i;
      var main, hint;
      if (i == null) {
        main = pad2(n) + ' ' + (n === 1 ? labels.member : labels.members) + (ctx.teamLine ? ' — ' + ctx.teamLine : '');
        hint = carousel ? labels.hintCarousel : labels.idle;
      } else {
        var m = members[i];
        main = m.name + (m.role ? ' — ' + m.role : '');
        if (S[i].flipped) hint = carousel ? labels.backCarousel : labels.backRail;
        else hint = carousel ? labels.hintCarousel : labels.hintRail;
      }
      if (capMain.textContent !== main) capMain.textContent = main;
      if (capHint.textContent !== hint) capHint.textContent = hint;
    }

    function setFlip(i, flipped) {
      var st = S[i], it = items[i];
      if (!st || st.flipped === flipped) return;
      if (flipped) {
        S.forEach(function (o, j) { if (j !== i && o.flipped) setFlip(j, false); });
      }
      st.flipped = flipped;
      it.li.classList.toggle('is-flipped', flipped);
      it.btn.setAttribute('aria-pressed', flipped ? 'true' : 'false');
      it.back.setAttribute('aria-hidden', flipped ? 'false' : 'true');
      it.anchors.forEach(function (a) { a.setAttribute('tabindex', flipped ? '0' : '-1'); });
      refreshFront(i);
      if (flipped) {
        // Neighbours get pushed away from the activated badge.
        impulse(i - 1, 58); impulse(i + 1, -58);
        impulse(i - 2, 26); impulse(i + 2, -26);
        kick();
      }
      setCaption(i);
      if (typeof opts.onFlip === 'function') opts.onFlip(i, flipped);
    }

    function scrollToIndex(i, smooth) {
      var li = items[i] && items[i].li;
      if (!li) return;
      var left = li.offsetLeft + li.offsetWidth / 2 - track.clientWidth / 2;
      if (typeof track.scrollTo === 'function') {
        track.scrollTo({ left: left, behavior: smooth && !reduced ? 'smooth' : 'auto' });
      } else {
        track.scrollLeft = left;
      }
    }

    function focusItem(i) {
      i = clamp(i, 0, n - 1);
      setRoving(i);
      if (carousel) {
        scrollToIndex(i, true);
        items[i].btn.focus({ preventScroll: true });
      } else {
        items[i].btn.focus();
      }
    }

    /* ---------- layout: rail vs carousel ---------- */

    function setMode(next) {
      if (next === carousel && section.classList.contains('tr--ready')) return;
      carousel = next;
      section.classList.toggle('tr--carousel', carousel);
      section.classList.add('tr--ready');
      items.forEach(function (it) {
        it.backHint.textContent = carousel ? labels.backCarousel : labels.backRail;
      });
      if (carousel) {
        S.forEach(function (st) { st.drag = null; st.s = st.sv = 0; });
        global.requestAnimationFrame(function () { if (alive) scrollToIndex(active, false); });
      } else {
        track.scrollLeft = 0;
      }
      setCaption(captionFor);
    }

    function layout() {
      var cs = global.getComputedStyle(section);
      var w = section.clientWidth - parseFloat(cs.paddingLeft) - parseFloat(cs.paddingRight);
      if (!(w > 0)) return;
      var wantCarousel = w < 600 || w < n * minColumn;
      var k;
      if (wantCarousel) {
        k = clamp((Math.min(300, w + 2 * parseFloat(cs.paddingLeft) - 56)) / 280, 0.8, 1);
      } else {
        var col = (w - 20 - (n - 1) * 20) / Math.max(1, n);
        k = clamp(col / 280, 0.6, 1);
      }
      if (Math.abs(k - scale) > 0.001) {
        scale = k;
        track.style.setProperty('--tr-k', k.toFixed(4));
      }
      setMode(wantCarousel);
      renderAll();
    }

    var ro = null;
    if (typeof ResizeObserver === 'function') {
      ro = new ResizeObserver(function () { if (alive) layout(); });
      ro.observe(section);
    } else {
      on(global, 'resize', layout);
    }
    layout();

    /* ---------- pointer: brush, hover intent, drag ---------- */

    var pX = 0, pT = 0, pVX = 0;  // pointer velocity, px per ms

    on(stage, 'pointermove', function (e) {
      if (e.pointerType !== 'mouse') return;
      var now = e.timeStamp || global.performance.now();
      var dt = now - pT;
      if (pT && dt > 0 && dt < 120) pVX = pVX * 0.55 + ((e.clientX - pX) / dt) * 0.45;
      else pVX = 0;
      pX = e.clientX; pT = now;
    }, { passive: true });

    items.forEach(function (it, i) {
      var st = S[i];

      on(it.hanger, 'pointerenter', function (e) {
        if (e.pointerType !== 'mouse') return;
        // Brushing past nudges the badge in the pointer's direction.
        impulse(i, clamp(-pVX * 26, -60, 60));
        kick();
        clearTimeout(st.timer);
        st.timer = setTimeout(function () {
          if (!alive) return;
          st.hover = true;
          refreshFront(i);
          setCaption(i);
          kick();
        }, 140);
      });

      on(it.hanger, 'pointerleave', function (e) {
        if (e.pointerType !== 'mouse') return;
        clearTimeout(st.timer);
        if (!st.hover) impulse(i, clamp(-pVX * 14, -36, 36));
        st.hover = false;
        refreshFront(i);
        kick();
      });

      function rubber(d) { return (d * 160) / (d + 160); }

      on(it.badge, 'pointerdown', function (e) {
        if (reduced || carousel || e.button !== 0 || e.pointerType === 'touch') return;
        if (e.target.closest && e.target.closest('a')) return;
        st.drag = { id: e.pointerId, x: e.clientX, y: e.clientY, moved: false, la: st.a, lt: e.timeStamp, va: 0 };
        try { it.badge.setPointerCapture(e.pointerId); } catch (err) { /* ignore */ }
      });

      on(it.badge, 'pointermove', function (e) {
        var d = st.drag;
        if (!d || d.id !== e.pointerId) return;
        var dx = e.clientX - d.x, dy = e.clientY - d.y;
        if (!d.moved && Math.sqrt(dx * dx + dy * dy) < 6) return;
        if (!d.moved) { d.moved = true; it.li.classList.add('is-dragging'); }
        st.s = rubber(Math.max(0, dy));
        var L = (it.cord + 190) * scale + st.s;
        var a = clamp(-Math.atan2(dx, L) * DEG, -MAX_ANGLE, MAX_ANGLE);
        var dt = Math.max(1, e.timeStamp - d.lt);
        d.va = d.va * 0.5 + ((a - d.la) / dt) * 1000 * 0.5;
        d.la = a; d.lt = e.timeStamp;
        st.a = a; st.v = 0; st.sv = 0;
        render(i);
      });

      function endDrag(e) {
        var d = st.drag;
        if (!d || d.id !== e.pointerId) return;
        st.drag = null;
        it.li.classList.remove('is-dragging');
        if (d.moved) {
          st.v = clamp(d.va, -240, 240);
          st.free = global.performance.now() + 900;
          st.suppress = true;
          setTimeout(function () { st.suppress = false; }, 350);
        }
        kick();
      }
      on(it.badge, 'pointerup', endDrag);
      on(it.badge, 'pointercancel', endDrag);

      on(it.dot, 'click', function () {
        setRoving(i);
        scrollToIndex(i, true);
        setCaption(i);
      });
    });

    /* ---------- click, focus, keyboard ---------- */

    on(track, 'click', function (e) {
      if (e.target.closest('a')) return;
      var badge = e.target.closest('.tr-badge');
      if (!badge) return;
      var li = badge.closest('.tr-item');
      var i = +li.getAttribute('data-index');
      if (S[i].suppress) { S[i].suppress = false; return; }
      setRoving(i);
      if (carousel && i !== nearestIndex()) scrollToIndex(i, true);
      setFlip(i, !S[i].flipped);
    });

    on(track, 'focusin', function (e) {
      var li = e.target.closest('.tr-item');
      if (!li) return;
      var i = +li.getAttribute('data-index');
      if (e.target.classList.contains('tr-flip')) setRoving(i);
      S[i].focus = matchesFocusVisible(e.target);
      refreshFront(i);
      setCaption(i);
      kick();
    });

    on(track, 'focusout', function (e) {
      var li = e.target.closest('.tr-item');
      if (!li) return;
      var i = +li.getAttribute('data-index');
      if (e.relatedTarget && li.contains(e.relatedTarget)) return;
      S[i].focus = false;
      refreshFront(i);
      kick();
    });

    on(track, 'keydown', function (e) {
      var li = e.target.closest('.tr-item');
      if (!li) return;
      var i = +li.getAttribute('data-index');
      var next = null;
      switch (e.key) {
        case 'ArrowRight': next = Math.min(n - 1, i + 1); break;
        case 'ArrowLeft': next = Math.max(0, i - 1); break;
        case 'Home': next = 0; break;
        case 'End': next = n - 1; break;
        case 'Escape':
          var open = S[i].flipped ? i : S.findIndex(function (st) { return st.flipped; });
          if (open > -1) {
            e.preventDefault();
            setFlip(open, false);
            items[open].btn.focus({ preventScroll: true });
          }
          return;
        default: return;
      }
      e.preventDefault();
      if (next !== i) focusItem(next);
    });

    /* ---------- carousel scroll ---------- */

    function nearestIndex() {
      var c = track.scrollLeft + track.clientWidth / 2, best = 0, bestD = Infinity;
      items.forEach(function (it, i) {
        var d = Math.abs(it.li.offsetLeft + it.li.offsetWidth / 2 - c);
        if (d < bestD) { bestD = d; best = i; }
      });
      return best;
    }

    var lastScroll = 0, scrollQueued = false;
    on(track, 'scroll', function () {
      if (!carousel || scrollQueued) return;
      scrollQueued = true;
      global.requestAnimationFrame(function () {
        scrollQueued = false;
        if (!alive) return;
        var delta = track.scrollLeft - lastScroll;
        lastScroll = track.scrollLeft;
        if (!reduced && Math.abs(delta) > 0.5) {
          // Badges lag behind the swipe, like weights on a moving rail.
          for (var i = 0; i < S.length; i++) impulse(i, clamp(-delta * 0.35, -14, 14));
          kick();
        }
        var idx = nearestIndex();
        if (idx !== active) {
          var hadFocus = track.contains(document.activeElement);
          setRoving(idx);
          if (!hadFocus) setCaption(idx);
        }
      });
    }, { passive: true });

    on(global, 'keydown', function (e) {
      // Esc closes an open badge even if focus is elsewhere on the page.
      if (e.key !== 'Escape' || track.contains(document.activeElement)) return;
      var open = S.findIndex(function (st) { return st.flipped; });
      if (open > -1) setFlip(open, false);
    });

    /* ---------- entrance & motion preference ---------- */

    function entrance() {
      if (reduced) return;
      S.forEach(function (st, i) {
        var r = rng(hash('in' + i + members[i].name));
        setTimeout(function () {
          if (!alive || reduced) return;
          st.v += (i % 2 ? 1 : -1) * (30 + r() * 18);
          st.sv += 110 + r() * 40;
          kick();
        }, 120 + i * 110);
      });
    }

    var io = null;
    if (typeof IntersectionObserver === 'function') {
      io = new IntersectionObserver(function (entries) {
        if (entries.some(function (en) { return en.isIntersecting; })) {
          io.disconnect();
          io = null;
          entrance();
        }
      }, { threshold: 0.2 });
      io.observe(stage);
    } else {
      entrance();
    }

    function onReduceChange() {
      reduced = !!(reduceMQ && reduceMQ.matches);
      section.classList.toggle('tr--reduced', reduced);
      if (reduced) {
        if (raf) global.cancelAnimationFrame(raf);
        raf = 0;
        S.forEach(function (st) { st.a = st.v = st.s = st.sv = 0; st.drag = null; });
        renderAll();
      }
    }
    if (reduceMQ) {
      if (reduceMQ.addEventListener) on(reduceMQ, 'change', onReduceChange);
      else if (reduceMQ.addListener) {
        reduceMQ.addListener(onReduceChange);
        cleanups.push(function () { reduceMQ.removeListener(onReduceChange); });
      }
    }

    setCaption(null);

    /* ---------- public API ---------- */

    var api = {
      root: section,
      flip: function (i, state) { setFlip(i, state == null ? !S[i].flipped : !!state); },
      focus: function (i) { focusItem(i); },
      nudge: function (i, deg) { impulse(i, deg == null ? 40 : deg); kick(); },
      destroy: function () {
        if (!alive) return;
        alive = false;
        if (raf) global.cancelAnimationFrame(raf);
        if (ac) ac.abort();
        cleanups.forEach(function (f) { f(); });
        if (ro) ro.disconnect();
        if (io) io.disconnect();
        S.forEach(function (st) { clearTimeout(st.timer); });
        if (section.parentNode) section.parentNode.removeChild(section);
        if (host.__teamRail === api) host.__teamRail = null;
      }
    };
    host.__teamRail = api;
    return api;
  }

  global.TeamRail = { mount: mount, version: '1.0.0' };
})(window);
