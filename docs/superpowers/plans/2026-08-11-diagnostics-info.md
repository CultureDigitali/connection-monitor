# Diagnostics and Info Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Explain connection quality with measured causes and actions, use the official app logo, and deliver a polished localized Info section.

**Architecture:** Rust remains the authority for score thresholds and returns a structured issue breakdown with every snapshot. A focused JavaScript presenter turns that breakdown into compact and expanded localized models. The existing official PNG is imported directly by Vite for both header and Info views.

**Tech Stack:** Rust 2021, Tauri 2.11, vanilla JavaScript modules, Vite 5, Node test runner, CSS.

## Global Constraints

- Compact diagnosis is always visible; full details are expandable.
- Explanations say probable cause and never claim proof beyond measured data.
- Score, penalties, causes, and ordering come from one Rust calculation.
- Official logo source is `src-tauri/icons/128x128.png`.
- All new text supports Italian, English, Spanish, and French.
- Current 340 by 540 macOS window remains usable through conditional vertical scrolling.
- Version becomes `0.2.1`.

---

### Task 1: Authoritative quality breakdown

**Files:**
- Modify: `src-tauri/src/stats.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Produces: `QualityIssue { key, severity, penalty, value, unit, recommendation_key }`.
- Produces: `ConnectionStats.quality_issues: Vec<QualityIssue>`.
- Produces: `ConnectionStats::assess_quality(...) -> QualityAssessment`.

- [ ] **Step 1: Write failing Rust tests**

Add tests proving that 220 ms latency creates a 55-point `latency` issue, 35 ms jitter creates a 30-point `jitter` issue, 6% loss creates a 30-point `packet_loss` issue, -82 dBm creates a 40-point `wifi` issue, healthy values create no issues, and issues are ordered by descending penalty.

- [ ] **Step 2: Verify RED**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --lib stats::tests::quality_breakdown`

Expected: compilation fails because `assess_quality` and `QualityIssue` are absent.

- [ ] **Step 3: Implement the breakdown**

Create `QualityIssue` and `QualityAssessment`. Move each existing threshold penalty into one assessment pass, append issues only when the penalty is positive, derive severity from the penalty, sort by descending penalty, and preserve the existing score labels. Populate `quality_issues` in connecting, offline, default, and online snapshots.

- [ ] **Step 4: Verify GREEN**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --lib stats::tests`

Expected: every stats test passes.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/stats.rs src-tauri/src/lib.rs
git commit -m "feat: explain connection quality score"
```

### Task 2: Diagnostic presenter and localized widget

**Files:**
- Create: `src/diagnostics.js`
- Create: `tests/diagnostics.test.mjs`
- Modify: `src/index.html`
- Modify: `src/main.js`
- Modify: `src/i18n.js`
- Modify: `src/styles.css`

**Interfaces:**
- Consumes: `ConnectionStats.connection_status` and `quality_issues`.
- Produces: `buildDiagnosticModel(stats, translate) -> { summary, recommendation, rows, state }`.

- [ ] **Step 1: Write failing JavaScript tests**

Test that online issues show only the two highest penalties in the summary, the first issue supplies the recommendation, expanded rows preserve every issue, connecting/offline have dedicated guidance, and an empty issue list reports healthy measurements.

- [ ] **Step 2: Verify RED**

Run: `node --test tests/diagnostics.test.mjs`

Expected: module-not-found failure for `src/diagnostics.js`.

- [ ] **Step 3: Implement the pure presenter**

Map stable issue and recommendation keys through the supplied translation function. Format latency and jitter as milliseconds, packet loss as percent, Wi-Fi as dBm, and include the exact score penalty in expanded rows.

- [ ] **Step 4: Build the diagnostic card**

Replace the star strip with `Perché`, `Cosa fare`, and an accessible Details button. Render the model on every stats update, toggle the complete metric list with `aria-expanded`, and make the monitor tab scroll only when expanded content exceeds the available height.

- [ ] **Step 5: Add all translations**

Add keys for reason/action/details, healthy/connecting/offline messages, four measured issue types, four recommendations, metric penalty text, and collapse control in English, Italian, Spanish, and French.

- [ ] **Step 6: Verify GREEN**

Run: `node --test tests/*.test.mjs`

Expected: all JavaScript tests pass.

- [ ] **Step 7: Commit**

```bash
git add src/diagnostics.js tests/diagnostics.test.mjs src/index.html src/main.js src/i18n.js src/styles.css
git commit -m "feat: add actionable quality diagnostics"
```

### Task 3: Official logo and Info refresh

**Files:**
- Modify: `src/index.html`
- Modify: `src/main.js`
- Modify: `src/i18n.js`
- Modify: `src/styles.css`

**Interfaces:**
- Consumes: Vite URL import from `../src-tauri/icons/128x128.png`.
- Produces: official logo in header and Info hero; localized Info tab.

- [ ] **Step 1: Add a failing asset assertion**

Extend `tests/diagnostics.test.mjs` to assert the exported official logo URL helper references `src-tauri/icons/128x128.png` and that the Info label key exists in each language.

- [ ] **Step 2: Verify RED**

Run: `node --test tests/diagnostics.test.mjs`

Expected: failure because the logo helper and new Info translations do not exist.

- [ ] **Step 3: Replace generic artwork**

Import the official icon through Vite, assign it to every `[data-app-logo]` image, replace the header Wi-Fi SVG and old Credits SVG, and add accessible alt text.

- [ ] **Step 4: Restyle and rename Info**

Rename the tab and content semantics from Credits to Info. Use an official-logo hero, version badge, concise developer and Culture Digitali cards, compact link pills, and a quiet license footer. Remove long duplicated contact copy.

- [ ] **Step 5: Verify production build**

Run: `npm run build`

Expected: Vite emits the official PNG asset and completes without warnings.

- [ ] **Step 6: Commit**

```bash
git add src/index.html src/main.js src/i18n.js src/styles.css tests/diagnostics.test.mjs
git commit -m "feat: refresh app identity and info"
```

### Task 4: Version, package, install, and verify

**Files:**
- Modify: `package.json`
- Modify: `package-lock.json`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/Cargo.lock`
- Modify: `src-tauri/tauri.conf.json`
- Replace: `/Applications/Connection Monitor.app`
- Replace: `/Users/luigimacmini/Desktop/Connection Monitor/ULTIMA VERSIONE/Connection Monitor.app`
- Create: `/Users/luigimacmini/Desktop/Connection Monitor/ULTIMA VERSIONE/Connection Monitor_0.2.1_aarch64.dmg`

**Interfaces:**
- Produces: installed and delivery version `0.2.1`.

- [ ] **Step 1: Bump every package version**

Set npm, Cargo, and Tauri versions to `0.2.1`; update lockfiles using their native package tools.

- [ ] **Step 2: Run the complete verification suite**

Run Rust tests, Node tests, `npm run build`, `cargo check`, YAML parsing, and `git diff --check`. Every command must exit zero without warnings.

- [ ] **Step 3: Build signed artifacts**

Read the updater private key and password from their existing macOS Keychain entries. Build `app`, `dmg`, updater archive, and updater signature for `aarch64-apple-darwin` with ad-hoc local signing because no Developer ID identity is installed.

- [ ] **Step 4: Validate and install**

Verify the app signature and DMG checksum. Stop the installed process, move version `0.2.0` into a timestamped recoverable backup, install identical app copies, copy the `0.2.1` DMG, and launch only `/Applications/Connection Monitor.app`.

- [ ] **Step 5: Verify runtime and commit**

Confirm version `0.2.1`, installed process path, identical app copies, green full suites, and clean worktree. Commit all version metadata and retain `fix/tray-widget-connection` without pushing or merging.
