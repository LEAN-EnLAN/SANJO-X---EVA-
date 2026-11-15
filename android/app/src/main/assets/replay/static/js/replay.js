// Eva static replay glue. Runs inside every captured screen instead of htmx, SSE and the
// Eva Link socket: links open other captured screens, story buttons tell the player to move
// the scenario on, and everything that would write to the ship says so in a toast.
(function () {
  "use strict";
  var inFrame = window.parent && window.parent !== window;
  var stateId = document.body.getAttribute("data-replay-state") || "";
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

  // On the hosted site, Eva's chat is live: /api/eva/chat (same topic guard as the app).
  // Falls back to the captured answer if the API is missing or Eva is offline.
  // Never live from a local copy: file:// or the Android app's bundled replay (WebViewAssetLoader serves it on
  // https://appassets.androidplatform.net, which has no /api); those use the captured answers only.
  var LIVE_CHAT = location.protocol === "https:" && location.hostname !== "appassets.androidplatform.net";
  var chatBox = document.getElementById("chat");
  function lang() { try { return (window.parent.document.documentElement.lang || "en").slice(0, 2); } catch (e) { return "en"; } }
  function bubble(text, mine) {
    var d = document.createElement("div");
    if (mine) {
      d.className = "flex max-w-64 flex-col gap-1 self-end px-3.5 py-2.5 text-sm leading-snug text-ink animate-eva-rise"; d.style.cssText = "border:1px solid #2a2a2a;border-radius:16px 16px 4px 16px;background:#000";
      d.textContent = text;
    } else {
      d.className = "flex max-w-full items-end gap-2 self-start animate-eva-rise";
      d.innerHTML = '<img src="../static/img/eva/eva-face.webp" alt="Eva" class="size-8 shrink-0 object-contain"><div class="flex min-w-0 flex-col gap-1 px-3.5 py-2.5 text-sm leading-snug text-ink" style="white-space:pre-line;border:1px solid #2a2a2a;border-radius:16px 16px 16px 4px;background:#000"></div>';
      d.lastChild.textContent = text;
    }
    chatBox.appendChild(d);
    window.scrollTo(0, document.documentElement.scrollHeight);
    return d;
  }
  function askLive(text, fallbackHref) {
    if (!chatBox || !text) return go(fallbackHref);
    bubble(text, true);
    var wait = bubble("…", false);
    fetch("/api/eva/chat", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ text: text, lang: lang(), seat: "4B" }) })
      .then(function (r) { return r.json().then(function (j) { return { ok: r.ok, j: j }; }); })
      .then(function (x) {
        if (x.j && x.j.text) { wait.lastChild.textContent = x.j.text; return; }
        wait.remove(); toast(lang() === "es" ? "Eva está offline en esta demo" : "Eva is offline in this demo"); go(fallbackHref);
      })
      .catch(function () { wait.remove(); go(fallbackHref); });
  }
  if (chatBox && LIVE_CHAT) {
    document.addEventListener("submit", function (e) {
      var f = e.target;
      if (!f.matches || !f.matches("[data-eva-form]")) return;
      e.preventDefault(); e.stopImmediatePropagation();
      var inp = f.querySelector("[name=text]"); var t = inp ? inp.value.trim() : "";
      if (inp) inp.value = "";
      askLive(t, f.getAttribute("data-href"));
    }, true);
    document.addEventListener("click", function (e) {
      var c = e.target.closest && e.target.closest("button.chip[data-href]");
      if (!c) return;
      e.preventDefault(); e.stopImmediatePropagation();
      askLive(c.textContent.replace(/\s+/g, " ").trim(), c.getAttribute("data-href"));
    }, true);
  }

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
    var seat = s.value, was = document.body.getAttribute("data-seat");
    if (["4B", "3B", "1B"].indexOf(seat) >= 0 && inFrame) post({ type: "seat", seat: seat });
    else toast("This replay follows seats 4B, 3B and 1B");
    if (was) s.value = was;
  });

  // the player keeps its own shortcuts (arrow keys, space) while focus is in a screen
  document.addEventListener("keydown", function (e) {
    if (!inFrame || e.target.matches && e.target.matches("input, textarea, select")) return;
    if (e.key === "ArrowLeft" || e.key === "ArrowRight") post({ type: "key", key: e.key });
  });

  // like the app after Eva answers: keep the newest line in view above the dock
  if (/eva-answer\.html$/.test(location.pathname)) {
    window.addEventListener("load", function () {
      // scroll this document only (scrollIntoView would also move the player page)
      window.scrollTo(0, document.documentElement.scrollHeight);
    });
  }

  post({ type: "view", file: location.pathname.split("/").pop() });
})();
