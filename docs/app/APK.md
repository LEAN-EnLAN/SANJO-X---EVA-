# Eva for Android (APK)

Phase 3 deliverable **03 · Final · Mobile APK, tested**.

The APK is a small native wrapper (Java, no Capacitor/Cordova, no AndroidX) around the **ship hub**: the laptop on the local network that runs `eva demo`. It doesn't contain the server. It opens `http(s)://<hub>:8080/app?seat=…` in a full-screen WebView and adds what a browser tab can't: a hub and seat picker, trust-on-first-use certificate pinning, mic and camera grants for the hub only, a native "hub unreachable" screen, and keep-screen-on.

| | |
|---|---|
| Package | `tech.sanjox.eva` (debug: `tech.sanjox.eva.debug`, can be installed alongside) |
| Version | 0.3.1 (code 301) |
| SDK | minSdk 26 (Android 8.0), targetSdk / compileSdk 36 (Android 16) |
| Size | release 384 KB, debug 405 KB |
| Signing | release: own keystore (RSA 3072, v2 scheme), cert SHA-256 `55:F1:78:EE:7A:AE:7E:AD:80:83:5D:AA:59:EE:47:0A:5E:D7:B9:29:00:FD:19:4A:B7:7F:42:50:58:88:44:D7` |
| Source | `android/` (Gradle wrapper 8.14.7, AGP 8.13.2) |

## Install

1. Copy `eva-v0.3.1.apk` to the phone (USB, chat, or `adb install eva-v0.3.1.apk`).
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
- **Open as**: Ana · 4B, Leo · 3B, Tomás · 1B, **Crew dashboard** (`/dash`) or **Station** (`/station`).
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
./gradlew testDebugUnitTest lintDebug assembleDebug            # → app/build/outputs/apk/debug/eva-v0.3.1-debug.apk

# signed release: keystore and passwords OUTSIDE the repo
EVA_SIGNING_PROPS=/secure/place/signing.properties ./gradlew lintRelease assembleRelease
#   → app/build/outputs/apk/release/eva-v0.3.1-release.apk (unsigned if EVA_SIGNING_PROPS is not set)
```

`signing.properties` holds `storeFile=`, `storePassword=`, `keyAlias=`, `keyPassword=`. The 0.3.1 release keystore (`eva-release.p12`, alias `eva`) and its password file were created outside the repo, both mode 600. **Back them up**: an update signed with a different key has to be installed after uninstalling the old app. `.gitignore` excludes build outputs, `local.properties` and `*.jks`, `*.keystore`, `*.p12` and `signing.properties`.

To make a new key: `keytool -genkeypair -keystore eva-release.p12 -storetype PKCS12 -alias eva -keyalg RSA -keysize 3072 -validity 10000`.

## Test results (15/11/2026)

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
