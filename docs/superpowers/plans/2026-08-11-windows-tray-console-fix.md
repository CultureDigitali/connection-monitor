# Windows Tray and Console Fix Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Pubblicare Connection Monitor 0.3.2 per Windows senza console nera e con quattro indicatori reali nella notification area.

**Architecture:** Il binario release Windows usa il sottosistema GUI. La tray condivisa crea sempre quattro icone; solo macOS abilita i titoli testuali, mentre Windows usa icone colorate e tooltip numerici. La guida chiarisce il comportamento del menu `^` controllato da Windows.

**Tech Stack:** Rust 2021, Tauri 2, JavaScript ES modules, Node test runner, GitHub Actions, NSIS, WiX.

## Global Constraints

- Windows 10/11 x64.
- Esattamente quattro indicatori: download, upload, qualita e dati.
- Nessuna quinta icona-logo.
- Nessuna console nelle build release Windows; console disponibile in debug.
- Windows puo spostare le icone nel menu nascosto e l'app non tenta di forzarne il pin.
- Installer non firmati con Authenticode.
- Release finale `v0.3.2` con `.exe`, `.msi`, DMG e metadati updater.

---

### Task 1: Eliminare la console Windows

**Files:**
- Modify: `tests/platform-config.test.mjs`
- Modify: `src-tauri/src/main.rs`

**Interfaces:**
- Consumes: entry point `connection_monitor_lib::run()`.
- Produces: binario release Windows con `windows_subsystem = "windows"`.

- [ ] **Step 1: Scrivere il test statico fallente**

Aggiungere a `tests/platform-config.test.mjs`:

```js
test("Windows release binary uses the GUI subsystem", async () => {
  const main = await readFile("src-tauri/src/main.rs", "utf8");
  assert.match(
    main,
    /#!\[cfg_attr\(all\(not\(debug_assertions\), target_os = "windows"\), windows_subsystem = "windows"\)\]/,
  );
});
```

- [ ] **Step 2: Verificare il rosso**

Run: `node --test tests/platform-config.test.mjs`

Expected: FAIL nel test `Windows release binary uses the GUI subsystem` per attributo assente.

- [ ] **Step 3: Applicare il fix minimo all'entry point**

Portare `src-tauri/src/main.rs` a:

```rust
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

fn main() {
    connection_monitor_lib::run()
}
```

- [ ] **Step 4: Verificare il verde**

Run: `node --test tests/platform-config.test.mjs && cargo check --manifest-path src-tauri/Cargo.toml`

Expected: test e compilazione completati con exit 0.

- [ ] **Step 5: Committare**

```bash
git add tests/platform-config.test.mjs src-tauri/src/main.rs
git commit -m "fix: hide Windows console in release builds"
```

### Task 2: Ripristinare quattro indicatori Windows

**Files:**
- Modify: `src-tauri/src/tray.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Produces: `INDICATOR_IDS: [&str; 4]` e `tray_titles_for_os(os: &str) -> bool`.
- Consumes: `indicator_icon`, `format_rate_compact`, `format_data_compact`, `ColorPrefs` e `ConnectionStats` esistenti.

- [ ] **Step 1: Sostituire il test della tray singola con test fallenti sui quattro indicatori**

In `src-tauri/src/tray.rs`, sostituire `windows_uses_one_status_tray` e `windows_tooltip_contains_all_four_live_values` con:

```rust
#[test]
fn windows_uses_four_icons_without_text_titles() {
    assert_eq!(INDICATOR_IDS, [DOWNLOAD_ID, UPLOAD_ID, QUALITY_ID, DATA_ID]);
    assert!(!tray_titles_for_os("windows"));
}

#[test]
fn macos_keeps_text_titles_for_four_icons() {
    assert_eq!(INDICATOR_IDS.len(), 4);
    assert!(tray_titles_for_os("macos"));
}
```

Aggiornare gli import del modulo test per riferire `INDICATOR_IDS`, i quattro ID e `tray_titles_for_os`.

- [ ] **Step 2: Verificare il rosso**

Run: `cargo test --manifest-path src-tauri/Cargo.toml tray::tests::windows_uses_four_icons_without_text_titles`

Expected: FAIL per simboli `INDICATOR_IDS` e `tray_titles_for_os` assenti.

- [ ] **Step 3: Implementare il modello tray multipiattaforma minimo**

In `src-tauri/src/tray.rs`:

```rust
pub const DOWNLOAD_ID: &str = "tray-download";
pub const UPLOAD_ID: &str = "tray-upload";
pub const QUALITY_ID: &str = "tray-quality";
pub const DATA_ID: &str = "tray-data";
pub const INDICATOR_IDS: [&str; 4] = [DOWNLOAD_ID, UPLOAD_ID, QUALITY_ID, DATA_ID];

pub fn tray_titles_for_os(os: &str) -> bool {
    os == "macos"
}
```

Rimuovere `WINDOWS_STATUS_ID`, `TrayMode`, `tray_mode_for_os`, `windows_tooltip` e gli import `ConnectionStats`/`ConnectionStatus` non piu necessari nel file.

In `src-tauri/src/lib.rs`:

- importare `tray_titles_for_os` al posto di `tray_mode_for_os`, `windows_tooltip` e `TrayMode`;
- eliminare i rami `SingleStatus` da `build_tray` e `update_trays`;
- costruire sempre i quattro elementi `DOWNLOAD_ID`, `UPLOAD_ID`, `QUALITY_ID`, `DATA_ID`;
- chiamare `.title(title)` in `build_tray` e `set_title` in `update_trays` soltanto quando `tray_titles_for_os(std::env::consts::OS)` restituisce `true`;
- mantenere icona e tooltip aggiornati su entrambe le piattaforme;
- mantenere il click sinistro esistente su ciascuna icona.

Il builder deve seguire questa forma:

```rust
let show_titles = tray_titles_for_os(std::env::consts::OS);
let mut builder = tauri::tray::TrayIconBuilder::with_id(id)
    .tooltip("Connection Monitor")
    .icon(indicator_icon(kind, &prefs));
if show_titles {
    builder = builder.title(title);
}
```

L'aggiornamento deve proteggere il titolo allo stesso modo:

```rust
let show_titles = tray_titles_for_os(std::env::consts::OS);
// ...
if show_titles {
    let _ = item.set_title(Some(title));
}
```

- [ ] **Step 4: Verificare il verde**

Run: `cargo test --manifest-path src-tauri/Cargo.toml tray::tests && cargo check --manifest-path src-tauri/Cargo.toml`

Expected: tutti i test tray e la compilazione passano; nessun riferimento a `tray-status` resta nel sorgente.

- [ ] **Step 5: Committare**

```bash
git add src-tauri/src/tray.rs src-tauri/src/lib.rs
git commit -m "fix: show four indicators in Windows tray"
```

### Task 3: Aggiornare guida e versione 0.3.2

**Files:**
- Modify: `tests/translations.test.mjs`
- Modify: `src/i18n.js`
- Modify: `tests/platform-config.test.mjs`
- Modify: `package.json`
- Modify: `package-lock.json`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/Cargo.lock`
- Modify: `src-tauri/tauri.conf.json`
- Modify: `README.md`

**Interfaces:**
- Consumes: chiave esistente `guideStepWidgetBody` in tutte le lingue.
- Produces: guida al menu nascosto Windows e metadati coerenti `0.3.2`.

- [ ] **Step 1: Scrivere i test fallenti per guida e versione**

In `tests/translations.test.mjs` aggiungere:

```js
test("every language explains the Windows hidden tray menu", () => {
  for (const translations of Object.values(dictionary)) {
    assert.match(translations.guideStepWidgetBody, /Windows/);
    assert.match(translations.guideStepWidgetBody, /\^/);
  }
});
```

In `tests/platform-config.test.mjs` cambiare le asserzioni di versione da `0.3.1` a `0.3.2` e rinominare il relativo test.

- [ ] **Step 2: Verificare il rosso**

Run: `node --test tests/translations.test.mjs tests/platform-config.test.mjs`

Expected: FAIL per testo guida assente e metadati ancora `0.3.1`.

- [ ] **Step 3: Aggiornare la guida in quattro lingue**

Estendere ogni `guideStepWidgetBody` con una frase localizzata equivalente a: `On Windows, open ^ and enable the four indicators to keep them beside the clock.`

Correggere inoltre i quattro `guideStepHistoryBody` sostituendo i riferimenti esclusivi a `Mac` con `device`, `dispositivo`, `dispositivo` e `appareil`.

- [ ] **Step 4: Allineare tutti i metadati a 0.3.2**

Eseguire `npm version 0.3.2 --no-git-tag-version`, quindi aggiornare con patch mirata:

- `src-tauri/Cargo.toml`: `version = "0.3.2"`;
- `src-tauri/Cargo.lock`: package locale `connection-monitor` a `0.3.2`;
- `src-tauri/tauri.conf.json`: `"version": "0.3.2"`;
- `README.md`: link e testo latest coerenti con la release multipiattaforma, senza dichiarare firma Authenticode.

- [ ] **Step 5: Verificare il verde**

Run: `node --test tests/*.test.mjs && cargo test --manifest-path src-tauri/Cargo.toml && npm run build && npm audit --audit-level=high`

Expected: zero test falliti, build Vite riuscita, zero vulnerabilita high.

- [ ] **Step 6: Committare**

```bash
git add tests src/i18n.js package.json package-lock.json src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/tauri.conf.json README.md
git commit -m "release: prepare Connection Monitor 0.3.2"
```

### Task 4: Verificare e pubblicare la correzione

**Files:**
- Verify: `.github/workflows/windows.yml`
- Verify: `.github/workflows/release.yml`

**Interfaces:**
- Consumes: branch `fix/windows-tray-console` e workflow esistenti.
- Produces: `main`, tag `v0.3.2` e release pubblica con installer Windows corretti.

- [ ] **Step 1: Eseguire la verifica locale finale**

Run: `node --test tests/*.test.mjs && cargo test --manifest-path src-tauri/Cargo.toml && cargo check --manifest-path src-tauri/Cargo.toml && npm run build && npm audit --audit-level=high && git diff --check && git status --short`

Expected: 0 failure, 0 vulnerabilita high, worktree pulito.

- [ ] **Step 2: Pubblicare branch e Pull Request**

```bash
git push -u origin fix/windows-tray-console
gh pr create --base main --head fix/windows-tray-console --title "Fix Windows console and tray indicators" --body "Fixes the Windows console window and restores four live tray indicators."
```

- [ ] **Step 3: Attendere e verificare GitHub Actions Windows**

Ricavare il run con:

```bash
windows_run_id=$(gh run list --workflow Windows --branch fix/windows-tray-console --limit 1 --json databaseId --jq '.[0].databaseId')
gh run watch "$windows_run_id" --exit-status
```

Expected: test frontend, test Rust, check Rust, build NSIS/MSI e upload artifact tutti verdi.

Scaricare gli artifact con `gh run download` e verificare che esistano un `.exe` NSIS e un `.msi` x64 non vuoti.

- [ ] **Step 4: Integrare e verificare main**

Unire la PR approvata, aggiornare `main` e ripetere la suite completa del passo 1 sul commit integrato.

- [ ] **Step 5: Creare e controllare la release**

```bash
git tag -a v0.3.2 -m "Connection Monitor v0.3.2"
git push origin v0.3.2
release_run_id=$(gh run list --workflow Release --limit 1 --json databaseId,headBranch --jq '.[] | select(.headBranch == "v0.3.2") | .databaseId')
gh run watch "$release_run_id" --exit-status
```

Expected: job macOS e Windows verdi; bozza con DMG, `.exe`, `.msi`, firme updater e `latest.json` contenente piattaforme macOS e Windows.

- [ ] **Step 6: Pubblicare e verificare live**

Pubblicare la bozza come latest, con note che citano il fix console, i quattro indicatori e il limite del menu nascosto Windows. Verificare con `gh release view v0.3.2` che `isDraft` sia `false` e che tutti gli asset siano presenti.
