# Eva for Android (APK)

Phase 3 deliverable **03 · Final · Mobile APK, tested**.

The APK is a small native wrapper (Java, no Capacitor/Cordova; its only library is `androidx.webkit` for the asset loader). Since 0.3.4 it works **without a ship hub**: the first screen offers the **online demo** (recommended), the **offline demo** bundled in the APK, or a **ship hub**, the laptop on the local network that runs `eva demo`. It doesn't contain the server. It opens `http(s)://<hub>:8080/app?seat=…` in a full-screen WebView and adds what a browser tab can't: a hub and seat picker, trust-on-first-use certificate pinning, mic and camera grants for the hub only, a native "hub unreachable" screen, and keep-screen-on.

| | |
|---|---|
| Package | `tech.sanjox.eva` (debug: `tech.sanjox.eva.debug`, can be installed alongside) |
| Version | 0.3.6 (code 306) |
| SDK | minSdk 26 (Android 8.0), targetSdk / compileSdk 36 (Android 16) |
| Size | release 2.3 MB (2,395,408 bytes), debug 3.5 MB (the offline demo is 4.9 MB of assets, compressed) |
| Signing | release: own keystore (RSA 3072, v2 scheme), cert SHA-256 `55:F1:78:EE:7A:AE:7E:AD:80:83:5D:AA:59:EE:47:0A:5E:D7:B9:29:00:FD:19:4A:B7:7F:42:50:58:88:44:D7` |
| Source | `android/` (Gradle wrapper 8.14.7, AGP 8.13.2) |

## Install from a QR (0.3.5)

`https://eva-wearmeds.vercel.app/download/` ("Install Eva on Android", EN/ES): a Download APK button (`/download/eva.apk`, served as `application/vnd.android.package-archive`), the version and size, three steps (allow the install, open Eva, choose "Try the demo") and a QR code to that page, an inline SVG generated locally (`pitch4/assets/qr.js`, ECC M, 4-module quiet zone, checked with jsQR).

## What 0.3.6 changes

- The offline bundle is the new everyone-friendly demo already live on the site (a copy of `public-sites/app/` without the old board): the first-visit intro, the guided tour with speech, "Talk to Eva", tap **or hold** to talk, the journal's "Say it instead", plainer captions ("Tap to talk to Eva"). The launcher still hides the site and board links inside the app and starts with a UTF-8 document.
- No native change beyond the version: the microphone is still granted to both demo origins (Vercel online, the asset host offline), and media plays without a tap, so Eva's spoken answers (a WAV from a `blob:` URL, or the browser's voice) and the tour's speech play in the WebView.

## What 0.3.5 adds

- **The demo is the new launcher**, online (`/app/`) and bundled (a copy of `public-sites/app/` without the old board): "Pick a view", scenes s1–s6, the three phones, the crew dashboard, the station, the ground and **three wristband views** (`#view=w4B`, `w3B`, `w1B`; deep links like `#view=w3B&scene=s2`).
- **Wristband in both demos**: the band screens are captured from the real app at 640 × 360 in wristband mode (home track, emergency s2–s4 for all three seats, storm s6, Eva and ECG pages). The launcher uses the app's own rule (landscape, height ≤ 480 px, aspect ≥ 8:5, or a tiny square): on a band-shaped screen, or a phone turned to landscape, it opens the chosen seat's band full screen with no menu; `?mode=wristband` / `?mode=phone` override. The demos never lock the orientation, so the launcher always sees the real screen.
- **Live Eva everywhere**: one `replay.js` for the site (relative `/api/eva/*`) and the bundle (absolute `https://eva-wearmeds.vercel.app/api/eva/*`, allowed by CORS for `https://appassets.androidplatform.net`). Chat, **voice** (tap the mic, up to 30 s, Whisper; a thin teal ring while recording), **Eva speaks** (the speaker toggle; Orpheus WAV in English, the browser's best voice for Spanish or on any failure), **photo** (one JPEG ≤ 1024 px) and **video** (4 evenly spaced frames on one 2 × 2 sheet ≤ 1024 px). No network or an error: a small toast and the captured answer.
- **Microphone in the demo modes**: `RECORD_AUDIO` is asked at runtime and the WebView's audio capture is granted only to the demo's own origin (Vercel online, the asset host offline); camera capture is never granted to a demo (photos and video go through the file chooser, with the camera). The hub's policy is unchanged.

## Modes (0.3.4)

| Mode | Opens | Network and trust |
|---|---|---|
| **Try the demo (online)**, recommended, first choice | `https://eva-wearmeds.vercel.app/app/`: the guided replay of the real app; Eva's chat is live through `/api/eva/chat` | normal system TLS (public CA); **no** trust-on-first-use, an SSL error just cancels. The WebView keeps only `/app/…` and `/api/…` on that exact host; anything else opens in the browser (intents and other schemes are dropped). Since 0.3.5 the microphone is granted to the demo's own origin (voice to Eva). |
| **Offline demo** | the same demo bundled in `assets/replay/`, served by `WebViewAssetLoader` at `https://appassets.androidplatform.net/assets/replay/index.html` (not `file://`) | works with no network (captured answers). Since 0.3.5, when the phone is online, Eva is live from the bundle too: `replay.js` calls the hosted API by absolute URL from the asset host. |
| **Connect to a ship hub** | the hub form as before (address, http/https, seat, keep-screen-on, pinned certificate) | unchanged: TOFU pinning for the hub only, LAN-only cleartext. |

- The choice is remembered (installs from 0.3.3 that saved a hub stay on it). The online demo opens the offline one, with a toast, when the phone has no network at launch or when the site's main page fails to load.
- Back to settings: back at the root (Leave / **Settings**), a long-press on the page background (Reload / **Settings**), the error screen, or the launcher shortcut **Eva settings**.
- Refresh the bundle after a new replay: copy `scratchpad/webdemo/out/` into `android/app/src/main/assets/replay/` (drop `_local.html`) and wrap `index.html` in a UTF-8 document skeleton (`<!doctype html>`, `<meta charset="utf-8">`, the viewport meta): the asset loader sends no charset, and the hosted page relies on Vercel's header.
- (0.3.4) Live chat from the offline bundle was off because `/api/eva/chat` had no CORS; since 0.3.5 the site allows the asset-host origin and the bundle uses it.

## Install

1. Scan the QR on `https://eva-wearmeds.vercel.app/download/`, or copy `eva-v0.3.6.apk` to the phone (USB, chat, or `adb install -r eva-v0.3.6.apk`). It is signed with the same key as 0.3.1–0.3.5, so it installs over them. Open it and tap **Try the demo (online)**, or **Offline demo** with no network: no hub needed.
   **Hub mode: update the hub too**: there the APK is only a WebView around the hub; the copy rules v3 wording ships in the `eva` 0.3.3 binary. If the phone still shows the old look after the hub update, Android Settings → Apps → Eva → Storage → Clear cache (the service worker cache `eva-shell-v9` normally evicts the old one).
2. Open it from Files. Android asks to allow **Install unknown apps** for the app that opens it (Files, Chrome, the chat app): allow it, go back, tap **Install**. Play Protect may warn about an unknown developer: choose **Install anyway**.
3. To switch between the debug and release builds, or to a build signed with another key, uninstall first. Debug and release have different package names, so both can be installed at the same time.

## Connect to the hub

On the laptop (see [RUNNING.md](RUNNING.md) and [DEMO.md](DEMO.md)):

```bash
cd app && ./target/release/eva demo --tls     # prints "on the LAN: https://<ip>:8080/app"
```

On the phone (same Wi-Fi or the laptop's hotspot), the first run shows **Connect to the ship hub**:

- **Hub address**: `192.168.0.10:8080` (IP or name, plus port; 8080 if left out). You can also paste a full URL such as `https://192.168.0.10:8080/app?seat=3B`; its scheme and seat take priority over the form.
- **Connection**: *https* if the hub runs with `--tls` (needed for voice), *http* otherwise. Plain http is only accepted for local network addresses.
- **Open as**: Ana · 4B, Leo · 3B, Tomás · 1B, **Crew dashboard** (`/dash`), **Station** (`/station`) or **Wristband (landscape)** (`/app/wrist?seat=4B&mode=wristband`, locked to landscape: a phone strapped to a forearm stands in for the band).
- **Keep the screen on while Eva is open**: on by default, so an emergency alert is never hidden behind a locked screen. The screen can still turn off when Eva is in the background.

The choice is remembered. To change it later:

- **long-press the Eva icon** on the launcher, then tap **Hub settings**;
- or press **Back** on the first page, then tap **Change hub**;
- or tap **Change hub** on the error screen.

Back goes through the WebView history first.

If the hub can't be reached, a native screen shows the address, what to check (same Wi-Fi, `eva demo` running, http or https), the WebView error code, **Retry** and **Change hub**. With no network at all, it says so.

## Voice, photo and video need https (`--tls`)

The microphone and camera (`getUserMedia`, hold-to-talk with `MediaRecorder`) only work in a **secure context**. A page loaded over `http://192.168.x.x` isn't one, so Android's WebView blocks mic and camera no matter what the app grants. Over plain http everything else works: live vitals, alerts, the journal, typing to Eva, the dashboard and the station. Photo upload by file picker also works over http. For **voice**, start the hub with `--tls` and pick *https*.

Permissions are requested at runtime the first time the page asks for them (microphone for voice, camera for photo and video). They're granted to the WebView **only for the configured origin** (same scheme, host and port), never to another page or port.

## Certificate pinning (trust on first use)

`eva --tls` uses a self-signed certificate (rcgen, `data/certs/eva-cert.pem`). Instead of asking the team to install it as a CA on every phone, the app pins it:

1. On the first https connection the app shows **Trust this hub?** with the certificate's **SHA-256 fingerprint**. Compare it with the one on the laptop:
   ```bash
   openssl x509 -noout -fingerprint -sha256 -in app/data/certs/eva-cert.pem
   ```
2. **Trust** saves the fingerprint for that host and the page loads. **Don't trust** cancels and shows "Certificate not trusted".
3. After that, the app proceeds only when the hub presents **exactly** that certificate. If it changes, the app cancels and shows "Hub certificate changed" with both fingerprints. It never asks again on its own.
4. A host other than the configured hub is never prompted for and never accepted. Expired, not-yet-valid or malformed certificates are always refused.
5. If the hub recreated its certificate (deleted `data/certs/`, new IP with `--tls-host`), open **Hub settings** and tap **Forget** next to the trusted certificate, then connect again.

The certificate's names don't matter once it's pinned. A hub reached by an IP that isn't in its certificate (a hotspot, or the emulator's `10.0.2.2`) works, because the pin is what's checked.

**mkcert** (ADR-004): if the mkcert root CA is installed on the phone as a user CA, the app trusts it through its network security config and shows no prompt.

**Offline shell:** Chromium doesn't register service workers on an origin whose certificate it was told to accept. With a TOFU-pinned hub, the APK runs the PWA live but without the service-worker cache and outbox replay. The hub is on the same LAN, so the demo doesn't depend on them. With mkcert, or over `http://localhost`, they work as in the browser.

## Cleartext policy

A network security config can't express IP ranges, so it permits cleartext and the app enforces the ranges in code (`HubConfig.isCleartextAllowed`, applied to every WebView request and to the setup form). `http://` and `ws://` are allowed only to 10/8, 172.16/12, 192.168/16, 127/8, 169.254/16, 100.64/10, IPv6 loopback, link-local and ULA, `localhost`, `*.local`, `*.lan` and `*.home.arpa`. Anything else over http gets a 403 inside the app. https is never downgraded. Mixed content is blocked.

## Rotation, wristband mode and smoothness (0.3.2)

- **Rotation**: `MainActivity` runs `screenOrientation="fullUser"` and already handles orientation changes without a reload, so turning the phone (auto-rotate on) flips the PWA into wristband mode and back. *Wristband (landscape)* locks `sensorLandscape`.
- **Eva status toast**: after the hub answers once, the app asks `/api/eva/status` inside the WebView (same origin, same certificate pin) and shows for 3 s "Eva online" or "Eva: on-board rules (the hub has no key)", so the teammate knows before the demo.
- **WebView smoothness**: `OVER_SCROLL_NEVER` (no grey stretch; the page owns the gesture), scroll bars off, `setOffscreenPreRaster(true)` (always full screen: less checkerboarding on fast scroll), `RENDERER_PRIORITY_IMPORTANT`, the default hardware layer (no `setLayerType` anywhere), `android:hardwareAccelerated="true"` explicit. The display mode with the highest refresh rate at the current resolution is preferred (90/120 Hz phones run the springs natively; the spring loop is time-based).
- **Navy base**: window, WebView, status bar, navigation bar and splash use `#070A14`, so there is no black flash around the page.
- **`setTextZoom(100)`**: the layout is built on a fixed type scale and the vitals rings break with system font scaling. Known accessibility trade-off for the demo: Android's font size setting does not enlarge text inside Eva.
- User agent ends with `EvaAndroid/0.3.2`.

## Other behaviour

- JavaScript, DOM storage and autoplay without a gesture are on. File access is off, and no JavaScript bridge is exposed.
- File chooser: `<input type=file accept=image/* capture>` opens a chooser with the **camera** (photo, or video up to 60 s) and the gallery or files. Captures go to the app's private cache through its own content provider and are deleted after a day.
- `navigator.vibrate` works (VIBRATE permission), for alerts.
- Links to other sites open in the browser. CSV and JSON downloads are handed to the browser too. **Report → print to PDF** doesn't work inside a WebView: print the report from the laptop.
- Dark splash with the Eva mark (the system splash on Android 12+). Black status and navigation bars, including Android 15+ edge to edge.
- No cloud backup: pins and settings stay on the device.
- QR scanning isn't built in (it would add a large dependency). Scan the hub URL with the phone's camera app, copy it and paste it into **Hub address**.

## Rebuild

Needs JDK 17+ (tested with OpenJDK 21) and the Android SDK packages `platforms;android-36` and `build-tools;36.0.0` (Android Studio, or the command-line tools' `sdkmanager`).

```bash
cd android
echo "sdk.dir=/path/to/android-sdk" > local.properties        # or export ANDROID_HOME
./gradlew testDebugUnitTest lintDebug assembleDebug            # → app/build/outputs/apk/debug/eva-v0.3.6-debug.apk

# signed release: keystore and passwords OUTSIDE the repo
EVA_SIGNING_PROPS=/secure/place/signing.properties ./gradlew lintRelease assembleRelease
#   → app/build/outputs/apk/release/eva-v0.3.6-release.apk (unsigned if EVA_SIGNING_PROPS is not set)
```

`signing.properties` holds `storeFile=`, `storePassword=`, `keyAlias=`, `keyPassword=`. The 0.3.1 release keystore (`eva-release.p12`, alias `eva`) and its password file were created outside the repo, both mode 600. **Back them up**: an update signed with a different key has to be installed after uninstalling the old app. `.gitignore` excludes build outputs, `local.properties` and `*.jks`, `*.keystore`, `*.p12` and `signing.properties`.

To make a new key: `keytool -genkeypair -keystore eva-release.p12 -storetype PKCS12 -alias eva -keyalg RSA -keysize 3072 -validity 10000`.

## Test results 0.3.6 (17/11/2026)

| check | result |
|---|---|
| JVM unit + Robolectric | **84 / 84 pass** (the bundle checks follow the new `replay.js`: a short toast and the recorded answer when Eva can't be reached; media plays without a user gesture in both demo modes) |
| `lintDebug`, `lintRelease` | 0 errors |
| `apksigner verify --print-certs` | release verifies (v2), signer SHA-256 `55:F1:78:EE:…:44:D7`, the same key as 0.3.1–0.3.5 |
| `aapt2 dump badging` | `tech.sanjox.eva` 0.3.6/306; 188 files under `assets/replay/` |
| Headless Chromium, WebView user agent, the bundle served at the asset-host and site origins, live API, fake media | **37 / 38**: launcher (9 views, band auto-detect, `?mode`, deep links), plain http and `file://` have no API, "Tap to talk to Eva", chat, mic → stt → chat, Eva speaks (live tts answered 204 in English during the run, so the browser voice at 0.95; a mocked WAV from tts plays through `Audio` from a `blob:` URL at both origins), photo and video → vision, offline → toast + recorded answer, the band's Eva page. The one failure: the site-origin photo late in the sequential run got no answer from the vision endpoint; the same request answered on its own |

## Test results 0.3.5 (15/11/2026)

| check | result |
|---|---|
| JVM unit + Robolectric | **84 / 84 pass**. New since 0.3.4: the demo microphone (audio granted to the demo origin only, camera and other origins denied, the Android permission asked first, offline on the asset host), `isDemoMediaOrigin`, the bundled `replay.js` picks the absolute hosted API only on the asset host, the bundled launcher uses the app's exact wristband rule (compared with `app/templates/head.html`) and has the band views, demo modes keep the orientation free and the WebView at device width |
| `lintDebug`, `lintRelease` | 0 errors |
| `apksigner verify --print-certs` | release verifies (v2), signer SHA-256 `55:F1:78:EE:…:44:D7`, the same key as 0.3.1–0.3.4 |
| `aapt2 dump badging` | `tech.sanjox.eva` 0.3.5/305; `RECORD_AUDIO`, `CAMERA`; 188 files under `assets/replay/` |
| Headless Chromium, WebView user agent, the replay served at the asset-host origin and at the site origin, live Vercel API, fake media (a spoken WAV as the microphone) | 33 / 34 in the full run: launcher at 412 × 915 (9 views), 640 × 360 lands in the band view full screen with no chrome, `?mode` overrides, deep links; from both origins: chat, mic → stt → chat, Eva speaks (WAV), photo → vision, offline → toast + captured answer; band page: starts on Eva, chip and mic answer in her bubble, arrows. The one failure was the site-origin video in the sequential run (an error answer from the endpoint after several calls); the same request passed on its own and from the asset origin |
| QR | the download page's QR decodes to `https://eva-wearmeds.vercel.app/download/` (jsQR, from a screenshot) |
| Emulator | none on the build machine |

## Test results 0.3.4 (15/11/2026)

| check | result |
|---|---|
| JVM unit + Robolectric | **78 / 78 pass**. New: `AppModeTest` (9: saved ids, online → offline without network, TOFU only for the hub, start pages, the online and offline allow-lists incl. look-alike hosts, other ports, cleartext, user info and `..`), `DemoModeRobolectricTest` (14: three modes on setup, remembered choice, upgrade from a hub-only install, online demo URL and settings, allow-list and external links, **no trust prompt or pin for the demo host** while the hub still asks, fallback to the bundle on a main-frame error and with no network at launch, mic/camera denied in demos, the bundle served from assets with the right MIME types, the long-press menu) |
| `lintDebug`, `lintRelease` | 0 errors |
| `apksigner verify --print-certs` | release verifies (v2), signer SHA-256 `55:F1:78:EE:…:44:D7`, the same key as 0.3.1–0.3.3 |
| `aapt2 dump badging` | `tech.sanjox.eva` 0.3.4/304, min 26, target 36; 154 files under `assets/replay/` |
| Replay in Chromium with the WebView user agent | bundled copy and hosted site at 412 × 915 and 640 × 360: no page errors; `window.parent.document.documentElement.lang` read from a screen frame returns `es` after choosing ES (no throw); bundled chat goes to the captured answer with **0** `/api` calls; hosted chat makes 1 call and shows Eva's live reply |
| Emulator | none on the build machine (no `/dev/kvm`, no system image) |

## Test results 0.3.3 (15/11/2026)

Wrapper changes: the version (303 / 0.3.3, UA `EvaAndroid/0.3.3`) and the native screens' colours, now the landing's palette (black, `#141414` raised, `#2a2a2a` lines, `#9b9996` grey, orange `#f28a1a`, alarm red `#ff2a1f`). The copy and palette changes of the screens themselves live in the hub.

| check | result |
|---|---|
| JVM unit + Robolectric | **55 / 55 pass** |
| `lintDebug`, `lintRelease` | 0 errors |
| `apksigner verify --print-certs` | release verifies (v2), signer SHA-256 `55:F1:78:EE:…:44:D7`, the same key as 0.3.1 and 0.3.2 |
| `aapt2 dump badging` | `tech.sanjox.eva` 0.3.3/303, min 26, target 36 |

## Test results 0.3.2 (15/11/2026)

| check | result |
|---|---|
| JVM unit + Robolectric | **55 / 55 pass**. New: `EvaStatusTest` (3: toast text per status, nothing when the hub did not say, the value `evaluateJavascript` returns), `SeatTest` wristband cases (`/app/wrist`, `mode=wristband`, `?seat=wrist` is not a passenger seat), Robolectric: Wristband opens `/app/wrist?seat=4B&mode=wristband` locked landscape, the status request runs after the first page, WebView settings (UA `EvaAndroid/0.3.2`, text zoom 100, no overscroll, no scroll bars, orientation full user). `setOffscreenPreRaster` is not recorded by Robolectric's shadow, so it is checked by reading the code |
| `lintDebug`, `lintRelease` | 0 errors |
| `apksigner verify --print-certs` | release verifies (v2), signer SHA-256 `55:F1:78:EE:…:44:D7`, the same key as 0.3.1 |
| `aapt2 dump badging` | `tech.sanjox.eva` 0.3.2/302, min 26, target 36 |

## Test results 0.3.1 (15/11/2026)

There's no `/dev/kvm` on the build machine, so **no emulator run**. Instead:

| check | result |
|---|---|
| JVM unit tests: `HubConfigTest` (15), `SeatTest` (3), `PinPolicyTest` (8), `FingerprintsTest` (3), `MediaPolicyTest` (3) | **32 / 32 pass**: address parsing, seat URLs, full-URL paste, IPv6, port and host validation, private ranges, cleartext policy, same-origin, the pinning decision (exhaustive: PROCEED only with same host, trust-only errors and an equal pin), fingerprint format (FIPS 180-2 vector), mic and camera grant rules |
| Robolectric (Android 14 framework, `MainActivityRobolectricTest`) | **18 / 18 pass**: first run opens setup; the setup form saves and opens the hub; public http is refused with a message; pinned cert is shown and forgotten; WebView settings and seat URL; keep-screen-on on and off; cleartext 403; navigation in-app vs browser; native error screen, Retry and Change hub; **real X.509 certificates through `onReceivedSslError`**: the fingerprint equals `openssl`, ask → Trust pins and proceeds (queued subresources too), Don't trust cancels, pinned cert proceeds silently (even with a name mismatch), changed cert refused, other host and expired cert refused; mic and camera granted to the hub origin only, runtime permission first; Back at the root offers Leave and Change hub |
| Mutation check | breaking the pin comparison (always proceed) fails 3 tests |
| `lintDebug`, `lintRelease` | 0 errors. 2 warnings, both "newer version available" for test-only libraries (Robolectric, Mockito) |
| `apksigner verify` | release: verifies (v2), signer `CN=Eva, OU=Sanjo-X`. Debug: verifies, Android Debug key |
| `aapt2 dump badging` | `tech.sanjox.eva` 0.3.1/301, min 26, target 36, label Eva, launcher `MainActivity`. Permissions: INTERNET, ACCESS_NETWORK_STATE, RECORD_AUDIO, MODIFY_AUDIO_SETTINGS, CAMERA, VIBRATE. Camera and microphone not required. allowBackup false, network security config set, provider not exported |
| Hub probe (`eva demo --tls` on 9280/9281) | the URLs the app builds answer 200: `/app?seat=4B`, `3B`, `1B`, `/dash`, `/station`. The app's fingerprint code on the hub's real rcgen certificate equals `openssl`. Validity 1975 to 4096, so no date error |

Run them again: `cd android && ./gradlew testDebugUnitTest lintDebug` (reports in `app/build/reports/`).

## Manual checklist (real phone, before the demo)

Hub: `eva demo --tls` on the laptop. Phone: same Wi-Fi.

- [ ] Install the release APK (unknown sources). The launcher shows the Eva icon (round or squircle mask, character fully visible). With Android 13+ themed icons on, the monochrome mark shows.
- [ ] Cold start: black splash with the Eva mark, then **Connect to the ship hub**.
- [ ] Enter `<laptop-ip>:8080`, https, Ana · 4B, Connect → **Trust this hub?** with a fingerprint. It matches `openssl x509 -noout -fingerprint -sha256 -in app/data/certs/eva-cert.pem` → Trust → Ana's home ("Good afternoon, Ana", green Mesh dot).
- [ ] Kill and reopen the app: it goes straight to Ana's home, with no prompt.
- [ ] Eva tab → hold the mic: Android asks for the microphone → allow → speech is transcribed (or the browser dictation fallback runs). Second time: no prompt.
- [ ] Eva tab → photo: the chooser offers the camera and files. Take a photo → it uploads and Eva answers. Same for a short video.
- [ ] Dev panel (tap the clock 5 times) → Cardiac arrest on 4B: the phone vibrates and turns red. The screen stays on with no touches for 2+ minutes.
- [ ] Back on Ana's home → **Leave Eva?** dialog. Change hub → Leo · 3B → Leo's home. Long-press the launcher icon → **Hub settings** opens.
- [ ] Hub settings → Crew dashboard → `/dash` loads (landscape works with no reload). Station → `/station`.
- [ ] Stop `eva` on the laptop → reopen or Retry → native **Can't reach the ship hub** screen (no Chrome error page). Start `eva` → Retry → back in.
- [ ] Wi-Fi off → **No network** screen.
- [ ] Recreate the hub certificate (`rm -r app/data/certs`, restart with `--tls`) → **Hub certificate changed** with both fingerprints. Hub settings → Forget → Connect → new prompt.
- [ ] Hub without `--tls`, app on http → everything except voice works. The setup screen explains why.
- [ ] Hub settings → turn off "Keep the screen on" → the screen times out as usual.
- [ ] Android 15/16 phone: content isn't under the status bar or gesture bar, and the keyboard doesn't cover the Eva chat box.

### Smoothness checklist (0.3.2)

- [ ] Android System WebView / Chrome updated (≥ 111 for view transitions; older falls back to spring pushes).
- [ ] Battery saver off (it caps the display at 60 Hz and throttles timers); "Remove animations" off unless testing reduced motion.
- [ ] Home: heart ring beats at the shown bpm, numbers roll on change, tab indicator slides, no stutter when scrolling Band/Journal.
- [ ] Tile → detail morph plays (or a clean push on an old WebView). Band flick spins with inertia and haptic detents.
- [ ] Dev panel → cardiac arrest: red takeover + long vibration; CPR metronome haptic on each beat; text readable at arm's length.
- [ ] About → Smooth mode shows "Auto (Full)" on a mid-range phone; switching to Lite removes the glass blur and stays fluid.
- [ ] Tab bar clears the gesture bar; keyboard does not cover the Eva composer.
- [ ] Rotate the phone: the wristband UI appears (face, swipe pages, Help needs a 2 s hold) and goes back upright. *Open as → Wristband (landscape)* stays landscape.
- [ ] On connect, a toast says "Eva online" or "Eva: on-board rules (the hub has no key)".
