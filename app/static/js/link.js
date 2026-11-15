// Eva Link in the browser: the same binary frames as src/protocol (docs/app/PROTOCOL.md).
// Each open app or dashboard is one mesh node: it says Hello, answers the hub's pings
// (the hub measures RTT per client), receives presence, and can ping any other node
// end to end through the hub ("triangulation" via the local server).
(function () {
  "use strict";
  var VERSION = 1, FLAG_DST = 1, FLAG_ACK = 2, FLAG_REL = 16, FLAG_CRC = 8;
  var T = { HELLO: 0, WELCOME: 1, PING: 2, PONG: 3, ACK: 4, NACK: 5, VITALS: 6, ALARM: 7, GUIDANCE: 8, RECORD: 9, TEXT: 10, PRESENCE: 11, BYE: 12 };
  var ROLE = { hub: 0, ship: 1, ground: 2, app: 3, dash: 4, band: 5 };
  var ROLE_NAME = ["hub", "ship", "ground", "app", "dash", "band"];
  var enc = new TextEncoder(), dec = new TextDecoder();

  // ---- wire helpers (LEB128 varints, length-prefixed strings)
  function W() { this.b = []; }
  W.prototype.u8 = function (v) { this.b.push(v & 255); return this; };
  W.prototype.uvar = function (v) { v = Math.floor(v); do { var byte = v % 128; v = Math.floor(v / 128); this.b.push(v > 0 ? byte | 128 : byte); } while (v > 0); return this; };
  W.prototype.bytes = function (u8) { this.uvar(u8.length); for (var i = 0; i < u8.length; i++) this.b.push(u8[i]); return this; };
  W.prototype.str = function (s) { return this.bytes(enc.encode(s)); };
  W.prototype.done = function () { return new Uint8Array(this.b); };
  function R(u8) { this.u = u8; this.p = 0; }
  R.prototype.left = function () { return this.u.length - this.p; };
  R.prototype.u8 = function () { if (this.p >= this.u.length) throw new Error("truncated"); return this.u[this.p++]; };
  R.prototype.uvar = function () { var out = 0, mul = 1, b; do { b = this.u8(); out += (b & 127) * mul; mul *= 128; } while (b & 128); return out; };
  R.prototype.take = function (n) { if (this.left() < n) throw new Error("truncated"); var s = this.u.subarray(this.p, this.p + n); this.p += n; return s; };
  R.prototype.bytes = function () { return this.take(this.uvar()); };
  R.prototype.str = function () { return dec.decode(this.bytes()); };

  function crc16(u8, from, to) { var c = 0xffff; for (var i = from; i < to; i++) { c ^= u8[i] << 8; for (var k = 0; k < 8; k++) c = c & 0x8000 ? ((c << 1) ^ 0x1021) & 0xffff : (c << 1) & 0xffff; } return c; }

  var seq = 1;
  function frame(kind, payload, dst, reliable) {
    var flags = (dst !== undefined && dst !== null ? FLAG_DST : 0) | (reliable ? FLAG_REL : 0);
    var prio = kind === T.ALARM ? 0 : kind === T.TEXT || kind === T.GUIDANCE ? 1 : kind === T.VITALS ? 2 : kind === T.RECORD ? 3 : 0;
    var w = new W().u8((VERSION << 5) | flags).u8((prio << 5) | kind).uvar(seq++).uvar(link.id || 0);
    if (flags & FLAG_DST) w.uvar(dst);
    w.bytes(payload || new Uint8Array(0));
    return w.done();
  }
  function decodeAll(u8) {
    var r = new R(u8), out = [];
    while (r.left() > 0) {
      var start = r.p, b0 = r.u8();
      if (b0 >> 5 !== VERSION) throw new Error("version");
      var flags = b0 & 31, b1 = r.u8();
      var f = { flags: flags, prio: b1 >> 5, kind: b1 & 31, seq: r.uvar(), src: r.uvar() };
      if (flags & FLAG_DST) f.dst = r.uvar();
      if (flags & FLAG_ACK) f.ack = r.uvar();
      f.payload = r.bytes();
      if (flags & FLAG_CRC) { var end = r.p, got = r.take(2); if (((got[0] << 8) | got[1]) !== crc16(u8, start, end)) throw new Error("crc"); }
      out.push(f);
    }
    return out;
  }

  var link = { id: 0, ws: null, nodes: [], hubRtt: null, pings: {}, results: {}, open: false };

  function send(bytes) { if (link.ws && link.ws.readyState === 1) link.ws.send(bytes); }
  function nowMs() { return Math.round(performance.now()); }

  function connect() {
    var view = document.body.getAttribute("data-view") || "app";
    var seatName = document.body.getAttribute("data-seat");
    var url = (location.protocol === "https:" ? "wss://" : "ws://") + location.host + "/link" + (seatName ? "?seat=" + encodeURIComponent(seatName) : "");
    var ws = new WebSocket(url);
    ws.binaryType = "arraybuffer";
    link.ws = ws;
    ws.onopen = function () {
      var name = view === "dash" ? "Dashboard" : "App · " + (seatName || "");
      var hello = new W().u8(view === "dash" ? ROLE.dash : ROLE.app).str(name).uvar(nowMs()).done();
      send(frame(T.HELLO, hello));
      link.open = true; status();
    };
    ws.onmessage = function (ev) {
      var frames; try { frames = decodeAll(new Uint8Array(ev.data)); } catch (e) { return; }
      frames.forEach(onFrame);
    };
    ws.onclose = function () { link.open = false; link.id = 0; status(); setTimeout(connect, 2000); };
  }

  function onFrame(f) {
    var r = new R(f.payload);
    switch (f.kind) {
      case T.WELCOME: link.id = r.uvar(); break;
      case T.PING: // answer the hub (or a peer) with the same payload
        send(frame(T.PONG, f.payload, f.src === 0 ? undefined : f.src));
        break;
      case T.PONG: {
        var t = r.uvar(), rtt = nowMs() - t;
        if (f.src === 0) link.hubRtt = rtt; else { link.results[f.src] = rtt; showResults(); }
        status(); break;
      }
      case T.PRESENCE: {
        var n = r.uvar(), nodes = [];
        for (var i = 0; i < n; i++) nodes.push({ id: r.uvar(), role: ROLE_NAME[r.u8()] || "?", name: r.str(), rtt: r.uvar() });
        link.nodes = nodes; status(); break;
      }
      case T.TEXT: {
        var from = r.str(), to = r.str(), text = r.str();
        if (window.Eva) window.Eva.toast(from + ": " + text);
        if (f.flags & FLAG_REL) send(frame(T.ACK, new W().uvar(1).uvar(f.seq).done(), f.src === 0 ? undefined : f.src));
        break;
      }
    }
  }

  function status() {
    var dot = document.querySelector("[data-link-dot]"), txt = document.querySelector("[data-link-rtt]");
    if (dot) { dot.classList.toggle("bg-ok", link.open); dot.classList.toggle("bg-faint", !link.open); }
    if (txt) txt.textContent = link.open ? ("Mesh " + link.nodes.length + (link.hubRtt !== null ? " · " + link.hubRtt + " ms" : "")) : "Offline";
    document.querySelectorAll("[data-mesh-count]").forEach(function (el) { el.textContent = link.nodes.length + " online"; });
  }

  // measure our own RTT to the hub every 5 s
  setInterval(function () { if (link.open) send(frame(T.PING, new W().uvar(nowMs()).done())); }, 5000);

  function pingAll() {
    link.results = {};
    link.nodes.forEach(function (n) { if (n.id !== link.id) send(frame(T.PING, new W().uvar(nowMs()).done(), n.id)); });
    showResults();
  }
  function showResults() {
    var box = document.querySelector("[data-mesh-results]");
    if (!box) return;
    box.innerHTML = "";
    link.nodes.forEach(function (n) {
      if (n.id === link.id) return;
      var p = document.createElement("p");
      p.textContent = "#" + n.id + " " + n.name + " · " + (link.results[n.id] !== undefined ? link.results[n.id] + " ms end to end" : "waiting…");
      box.appendChild(p);
    });
  }
  function sendText(text, to) {
    var payload = new W().str(document.body.getAttribute("data-seat") || "dash").str(to ? String(to) : "*").str(text).done();
    send(frame(T.TEXT, payload, to, true));
  }

  document.addEventListener("DOMContentLoaded", connect);
  window.EvaLink = { link: link, pingAll: pingAll, sendText: sendText, frame: frame, decodeAll: decodeAll, T: T };
})();
