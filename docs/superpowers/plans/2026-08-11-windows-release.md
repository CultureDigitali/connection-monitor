# Connection Monitor Windows Release Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Pubblicare Connection Monitor 0.3.1 per Windows 10/11 x64 come installer NSIS `.exe` e WiX `.msi`, mantenendo macOS ARM64 e l’updater in una release GitHub unica.

**Architecture:** Il core resta condiviso. Configurazione, tray e Wi-Fi hanno adattatori di piattaforma selezionati a compile time; GitHub Actions compila e testa nativamente su macOS e Windows e aggiorna la stessa release Tauri.

**Tech Stack:** Tauri 2, Rust 2021, `windows-sys` Native Wi-Fi API, Vanilla JS/Vite, GitHub Actions, NSIS, WiX.

## Global Constraints

- Windows minimo: Windows 10 x64; supportato anche Windows 11 x64.
- Artefatti Windows obbligatori: installer NSIS `.exe`, installer WiX `.msi`, updater e firma.
- Gli installer Windows non usano Authenticode in questa versione.
- macOS ARM64 continua a produrre DMG e updater.
- Storico locale schema 1 invariato e nessun dato inviato fuori dal computer.
- La notification area Windows usa una sola icona ufficiale; macOS conserva quattro indicatori.
- La release finale è `v0.3.1` e nasce come bozza.

---

### Task 1: Versione e configurazioni Tauri di piattaforma

**Files:**
- Create: `tests/platform-config.test.mjs`
- Create: `src-tauri/tauri.macos.conf.json`
- Create: `src-tauri/tauri.windows.conf.json`
- Modify: `src-tauri/tauri.conf.json`
- Modify: `package.json`
- Modify: `package-lock.json`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/Cargo.lock`

**Interfaces:**
- Produces: configurazione comune 0.3.1; bundle macOS `app,dmg`; bundle Windows `nsis,msi`; dipendenza Windows Native Wi-Fi isolata per target.

- [ ] **Step 1: Scrivere il test di configurazione fallente**

Creare un test Node che legga i JSON/TOML reali e verifichi versione 0.3.1, assenza di opzioni macOS dal file comune e target dei due file di piattaforma:

```js
test("platform configs isolate macOS and Windows bundles", async () => {
  const common = JSON.parse(await readFile("src-tauri/tauri.conf.json", "utf8"));
  const mac = JSON.parse(await readFile("src-tauri/tauri.macos.conf.json", "utf8"));
  const windows = JSON.parse(await readFile("src-tauri/tauri.windows.conf.json", "utf8"));

  assert.equal(common.version, "0.3.1");
  assert.equal(common.app.macOSPrivateApi, undefined);
  assert.deepEqual(mac.bundle.targets, ["app", "dmg"]);
  assert.deepEqual(windows.bundle.targets, ["nsis", "msi"]);
});
```

- [ ] **Step 2: Eseguire il test e verificare RED**

Run: `node --test tests/platform-config.test.mjs`

Expected: FAIL perché i file specifici non esistono e la versione è 0.3.0.

- [ ] **Step 3: Separare le configurazioni e aggiornare la versione**

Il file comune conserva finestre, CSP, icone, descrizioni e updater. Spostare `macOSPrivateApi`, categoria e firma ad-hoc in `tauri.macos.conf.json`:

```json
{
  "app": { "macOSPrivateApi": true },
  "bundle": {
    "targets": ["app", "dmg"],
    "category": "public.app-category.utilities",
    "macOS": { "minimumSystemVersion": "11.0", "signingIdentity": "-" }
  }
}
```

Creare `tauri.windows.conf.json`:

```json
{
  "bundle": {
    "targets": ["nsis", "msi"],
    "windows": {
      "nsis": { "installerIcon": "icons/icon.ico" },
      "wix": { "language": "en-US" }
    }
  }
}
```

Portare `package.json`, lockfile, `Cargo.toml`, `Cargo.lock` e `tauri.conf.json` a 0.3.1. In `Cargo.toml` aggiungere solo su Windows:

```toml
[target.'cfg(target_os = "windows")'.dependencies]
windows-sys = { version = "0.61", features = ["Win32_Foundation", "Win32_NetworkManagement_WiFi"] }
```

Rimuovere `macos-private-api` dalla dipendenza Tauri comune e abilitarla solo in `[target.'cfg(target_os = "macos")'.dependencies]`, così Windows non compila API private Apple.

- [ ] **Step 4: Verificare GREEN e configurazioni risolte**

Run: `node --test tests/platform-config.test.mjs && npm run tauri -- build --help >/dev/null`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add tests/platform-config.test.mjs package.json package-lock.json src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/tauri.conf.json src-tauri/tauri.macos.conf.json src-tauri/tauri.windows.conf.json
git commit -m "build: configure Windows installers"
```

---

### Task 2: Rilevamento Wi-Fi nativo Windows

**Files:**
- Modify: `src-tauri/src/monitor/wifi.rs`
- Test: unit tests nello stesso file

**Interfaces:**
- Produces: `WifiMonitor::get_wifi_info() -> WifiInfo` invariato.
- Produces: `wifi_info_from_windows_snapshot(snapshot: WindowsWifiSnapshot) -> WifiInfo`, funzione pura per conversione e test.
- Consumes: `WlanOpenHandle`, `WlanEnumInterfaces`, `WlanQueryInterface`, `WlanFreeMemory`, `WlanCloseHandle` da `windows-sys`.

- [ ] **Step 1: Scrivere i test fallenti della conversione Windows**

```rust
#[test]
fn windows_snapshot_maps_signal_and_rates() {
    let info = wifi_info_from_windows_snapshot(WindowsWifiSnapshot {
        ssid: b"Ufficio".to_vec(),
        signal_quality: 78,
        channel: Some(44),
        tx_rate_kbps: 866_000,
    });
    assert_eq!(info.ssid.as_deref(), Some("Ufficio"));
    assert_eq!(info.signal_dbm, Some(-61));
    assert_eq!(info.channel, Some(44));
    assert_eq!(info.transmit_rate, Some(866.0));
}

#[test]
fn windows_snapshot_rejects_invalid_ssid_and_clamps_signal() {
    let info = wifi_info_from_windows_snapshot(WindowsWifiSnapshot {
        ssid: vec![0xff, 0xfe],
        signal_quality: 120,
        channel: None,
        tx_rate_kbps: 0,
    });
    assert_eq!(info.ssid, None);
    assert_eq!(info.signal_dbm, Some(-50));
    assert_eq!(info.transmit_rate, None);
}
```

- [ ] **Step 2: Eseguire e verificare RED**

Run: `cargo test --manifest-path src-tauri/Cargo.toml monitor::wifi`

Expected: FAIL perché snapshot e funzione non esistono.

- [ ] **Step 3: Implementare la conversione pura**

Definire `WindowsWifiSnapshot`; convertire qualità 0–100 in dBm con `quality.clamp(0, 100) / 2 - 100`, usare esattamente i byte SSID dichiarati, sanitizzare il testo e convertire Kbps in Mbps. Nessun dato disponibile deve diventare zero fittizio.

- [ ] **Step 4: Implementare l’adattatore WLAN sotto `cfg(target_os = "windows")`**

Il flusso deve essere:

```text
WlanOpenHandle(2)
  -> WlanEnumInterfaces
  -> prima interfaccia con wlan_interface_state_connected
  -> WlanQueryInterface(wlan_intf_opcode_current_connection)
  -> WlanQueryInterface(wlan_intf_opcode_channel_number)
  -> WLAN_CONNECTION_ATTRIBUTES.wlanAssociationAttributes
  -> WindowsWifiSnapshot
```

Usare guardie RAII locali per chiamare sempre `WlanFreeMemory` e `WlanCloseHandle`, anche in caso di ritorno anticipato. Se l’API restituisce accesso negato, stato non valido o nessuna interfaccia, restituire `WifiInfo` con campi `None`.

- [ ] **Step 5: Verificare GREEN e regressioni Rust**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`

Expected: tutti i test PASS.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/monitor/wifi.rs src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "feat: read Windows WiFi status"
```

---

### Task 3: Tray nativa Windows con un solo indicatore

**Files:**
- Modify: `src-tauri/src/tray.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: unit tests in `src-tauri/src/tray.rs`

**Interfaces:**
- Produces: `TrayMode::{MultiIndicator, SingleStatus}`.
- Produces: `tray_mode_for_os(os: &str) -> TrayMode`.
- Produces: `windows_tooltip(stats: &ConnectionStats) -> String` con download, upload, qualità e dati.
- Consumes: `AppHandle::tray_by_id`, `TrayIcon::set_icon`, `TrayIcon::set_tooltip`.

- [ ] **Step 1: Scrivere i test fallenti del modello tray**

```rust
#[test]
fn windows_uses_one_status_tray() {
    assert_eq!(tray_mode_for_os("windows"), TrayMode::SingleStatus);
    assert_eq!(tray_mode_for_os("macos"), TrayMode::MultiIndicator);
}

#[test]
fn windows_tooltip_contains_all_four_live_values() {
    let mut stats = ConnectionStats::default();
    stats.connection_status = ConnectionStatus::Online;
    stats.download_mbps = 1.2;
    stats.upload_mbps = 0.4;
    stats.quality_score = 87;
    stats.total_download_mb = 1_000.0;
    stats.total_upload_mb = 536.0;
    let tooltip = windows_tooltip(&stats);
    assert!(tooltip.contains("↓ 1.2M"));
    assert!(tooltip.contains("↑ 400K"));
    assert!(tooltip.contains("87/100"));
    assert!(tooltip.contains("1.54 GB"));
}
```

- [ ] **Step 2: Eseguire e verificare RED**

Run: `cargo test --manifest-path src-tauri/Cargo.toml tray::tests`

Expected: FAIL perché modello e tooltip non esistono.

- [ ] **Step 3: Implementare il modello e separare la costruzione tray**

Mantenere `build_macos_trays` con i quattro ID esistenti. Aggiungere `WINDOWS_STATUS_ID = "tray-status"` e `build_windows_tray`, che usa l’icona ufficiale dell’app e lo stesso handler click del pannello.

```rust
#[cfg(target_os = "windows")]
fn build_platform_tray(app: &mut tauri::App) -> Result<(), Box<dyn Error>> {
    build_windows_tray(app)
}

#[cfg(not(target_os = "windows"))]
fn build_platform_tray(app: &mut tauri::App) -> Result<(), Box<dyn Error>> {
    build_macos_trays(app)
}
```

In `update_trays`, Windows aggiorna solo icona qualità e tooltip riassuntivo; macOS conserva icone, titoli e colori distinti.

- [ ] **Step 4: Verificare GREEN e suite Rust**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`

Expected: tutti i test PASS e nessuna regressione macOS.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/tray.rs src-tauri/src/lib.rs
git commit -m "feat: add native Windows tray behavior"
```

---

### Task 4: CI Windows e release multipiattaforma

**Files:**
- Create: `.github/workflows/windows.yml`
- Modify: `.github/workflows/release.yml`
- Modify: `tests/release-workflow.test.mjs`

**Interfaces:**
- Produces: CI Windows su PR e `workflow_dispatch` con test, check e bundle NSIS/MSI come workflow artifact.
- Produces: release matrix macOS ARM64 + Windows x64 sulla stessa tag.

- [ ] **Step 1: Estendere i test workflow e verificare RED**

Il test deve controllare `windows-latest`, target `x86_64-pc-windows-msvc`, entrambi i bundle, test JS/Rust prima della build e `updaterJsonPreferNsis: true`.

```js
assert.match(release, /windows-latest/);
assert.match(release, /x86_64-pc-windows-msvc/);
assert.match(release, /--bundles nsis,msi/);
assert.match(release, /updaterJsonPreferNsis: true/);
assert.match(windowsCi, /upload-artifact@v7/);
```

- [ ] **Step 2: Eseguire e verificare RED**

Run: `node --test tests/release-workflow.test.mjs`

Expected: FAIL perché workflow e matrice Windows non esistono.

- [ ] **Step 3: Creare la CI Windows**

`windows.yml` usa `windows-latest`, Node 24, Rust stable MSVC, `npm ci`, `node --test tests/*.test.mjs`, `cargo test`, `cargo check` e:

```powershell
npm run tauri build -- --target x86_64-pc-windows-msvc --bundles nsis,msi
```

Il comando build riceve `TAURI_SIGNING_PRIVATE_KEY` e `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` dai secret GitHub esistenti, necessari per generare updater e `.sig` anche senza Authenticode.

Caricare le directory `src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis` e `.../msi` come artifact `connection-monitor-windows-x64`.

- [ ] **Step 4: Convertire Release in matrice**

La matrice contiene:

```yaml
matrix:
  include:
    - platform: macos-latest
      target: aarch64-apple-darwin
      bundles: app,dmg
    - platform: windows-latest
      target: x86_64-pc-windows-msvc
      bundles: nsis,msi
```

Ogni job esegue test prima di `tauri-apps/tauri-action@v0`, passa le chiavi updater, usa `args: --target ${{ matrix.target }} --bundles ${{ matrix.bundles }}` e `updaterJsonPreferNsis: true`.

- [ ] **Step 5: Verificare GREEN e sintassi**

Run: `node --test tests/*.test.mjs && ruby -e 'require "yaml"; YAML.load_file(".github/workflows/windows.yml"); YAML.load_file(".github/workflows/release.yml")'`

Expected: tutti i test PASS e YAML valido.

- [ ] **Step 6: Commit**

```bash
git add .github/workflows/windows.yml .github/workflows/release.yml tests/release-workflow.test.mjs
git commit -m "ci: build Windows installers on GitHub"
```

---

### Task 5: Documentazione Windows e preflight locale

**Files:**
- Modify: `README.md`
- Create: `tests/readme.test.mjs`

**Interfaces:**
- Produces: istruzioni Windows, avviso SmartScreen e download `.exe`/`.msi` documentati.

- [ ] **Step 1: Scrivere il test fallente della documentazione**

Verificare che README contenga Windows 10/11, riferimenti `.exe`/`.msi`, avviso installer non firmato e badge Windows.

- [ ] **Step 2: Eseguire e verificare RED**

Run: `node --test tests/readme.test.mjs tests/platform-config.test.mjs`

Expected: FAIL per documentazione Windows mancante.

- [ ] **Step 3: Aggiornare README**

Documentare download consigliato `.exe`, alternativa `.msi`, SmartScreen → “Ulteriori informazioni” → “Esegui comunque”, requisiti Windows e parità funzionale. Non promettere firma Authenticode.

- [ ] **Step 4: Verificare GREEN e preflight completo**

Run:

```bash
node --test tests/*.test.mjs
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
npm run build
npm audit
git diff --check
```

Expected: zero fallimenti e zero vulnerabilità.

- [ ] **Step 5: Commit**

```bash
git add README.md tests/readme.test.mjs
git commit -m "docs: add Windows installation guidance"
```

---

### Task 6: Build GitHub, correzioni e pubblicazione 0.3.1

**Files:**
- Modify only if CI reveals a reproducible platform issue; every fix requires a failing regression test first.

**Interfaces:**
- Produces: PR GitHub, workflow Windows verde, merge in `main`, tag e release pubblica `v0.3.1`.

- [ ] **Step 1: Pubblicare il branch e aprire una PR bozza**

```bash
git push -u origin feat/windows-release
printf '%s\n' '## Summary' '- Windows 10/11 x64 support' '- NSIS EXE and WiX MSI installers' '- Native Windows Wi-Fi and tray behavior' '' '## Checks' '- JavaScript and Rust tests' '- Native Windows bundle build' | gh pr create --draft --base main --head feat/windows-release --title "Connection Monitor 0.3.1 for Windows" --body-file -
```

- [ ] **Step 2: Avviare e monitorare la build Windows**

```bash
gh workflow run windows.yml --ref feat/windows-release
RUN_ID=$(gh run list --workflow windows.yml --branch feat/windows-release --limit 1 --json databaseId --jq '.[0].databaseId')
gh run watch "$RUN_ID" --exit-status
```

Expected: test, `cargo check`, NSIS e MSI completati.

- [ ] **Step 3: Ispezionare gli artifact GitHub**

Scaricare in una directory temporanea; verificare presenza e dimensione non nulla di `.exe`, `.msi`, updater e `.sig`. Non eseguire binari Windows su macOS.

- [ ] **Step 4: Rendere pronta e unire la PR**

Solo con CI verde:

```bash
PR_NUMBER=$(gh pr view feat/windows-release --json number --jq '.number')
gh pr ready "$PR_NUMBER"
gh pr merge "$PR_NUMBER" --merge --delete-branch
```

- [ ] **Step 5: Creare tag e attendere la release bozza**

Aggiornare `main`, creare tag annotata `v0.3.1`, pubblicarla e monitorare il workflow Release fino alla conclusione.

- [ ] **Step 6: Verificare e pubblicare la release**

Controllare DMG, `.exe`, `.msi`, pacchetti updater, firme e `latest.json` con chiavi Darwin ARM64 e Windows x64/NSIS. Aggiornare le note con avviso SmartScreen, quindi rendere la release pubblica e “Latest”.

- [ ] **Step 7: Riallineare e verificare lo stato finale**

```bash
git switch main
git pull --ff-only origin main
git status -sb
gh release view v0.3.1
```

Expected: `main` sincronizzato, worktree pulito, PR merged e release pubblica.

## Riferimenti ufficiali

- Tauri platform-specific configuration: https://v2.tauri.app/develop/configuration-files/
- Tauri Windows installers: https://v2.tauri.app/distribute/windows-installer/
- Tauri GitHub Action: https://github.com/tauri-apps/tauri-action
- Microsoft Native Wi-Fi query API: https://learn.microsoft.com/windows/win32/api/wlanapi/nf-wlanapi-wlanqueryinterface
