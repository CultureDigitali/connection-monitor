# Tray, Widget and Connection Fix Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Keep the macOS widget open after click-release, expose readable live tray indicators, resolve the indefinite connecting state, and install the exact rebuilt app.

**Architecture:** Tauri remains the sole owner of window visibility. Small pure Rust helpers centralize connection detection and tray-title formatting so both runtime paths use one tested rule; the frontend owns only explicit close-button requests.

**Tech Stack:** Rust 2021, Tauri 2.11, Tokio, vanilla JavaScript, Node test runner, macOS application bundle.

## Global Constraints

- The widget closes only from its close button or a second tray-icon click.
- The menu bar always exposes a visible icon and readable live indicators.
- `Connecting...` must be replaced after the first completed sample.
- Modify only tray, monitor loop, and visual-state behavior.
- Install the same release bundle in `/Applications` and `Connection Monitor/ULTIMA VERSIONE`.

---

### Task 1: Persistent widget behavior

**Files:**
- Create: `src/window-visibility.js`
- Create: `tests/window-visibility.test.mjs`
- Modify: `src/main.js:82-128`

**Interfaces:**
- Produces: `bindWindowVisibility(closeButton: EventTarget, hideWindow: Function): void`
- Consumes: the existing `hide_main_window` Tauri command.

- [ ] **Step 1: Write the failing test**

```javascript
import test from 'node:test';
import assert from 'node:assert/strict';
import { bindWindowVisibility } from '../src/window-visibility.js';

test('ordinary document clicks do not hide the widget', () => {
  const closeButton = new EventTarget();
  const documentTarget = new EventTarget();
  let hideCount = 0;
  bindWindowVisibility(closeButton, () => { hideCount += 1; });
  documentTarget.dispatchEvent(new Event('click'));
  assert.equal(hideCount, 0);
});

test('the explicit close button hides the widget once', () => {
  const closeButton = new EventTarget();
  let hideCount = 0;
  bindWindowVisibility(closeButton, () => { hideCount += 1; });
  closeButton.dispatchEvent(new Event('click'));
  assert.equal(hideCount, 1);
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `node --test tests/window-visibility.test.mjs`
Expected: FAIL because `src/window-visibility.js` does not exist.

- [ ] **Step 3: Implement minimal behavior**

Create `bindWindowVisibility`, import it from `main.js`, bind the close button through it, and delete the document-level auto-hide listener.

- [ ] **Step 4: Run test to verify it passes**

Run: `node --test tests/window-visibility.test.mjs`
Expected: 2 passing tests.

### Task 2: Deterministic connection state and tray title

**Files:**
- Modify: `src-tauri/src/lib.rs:275-330,397-462,616-650`
- Test: unit tests inside `src-tauri/src/lib.rs`

**Interfaces:**
- Produces: `is_connection_active(Option<f64>, f64, f64, f64, bool, f64, f64) -> bool`
- Produces: `format_tray_title(f64, f64, u8, bool) -> String`
- Consumes: ping result, bandwidth totals, Wi-Fi presence, and rolling speeds.

- [ ] **Step 1: Write failing Rust unit tests**

Test literal cases: an empty first sample is offline; a successful ping is online; observed traffic is online; connected title is `↓0.0 ↑0.0 •50`; offline title is `OFFLINE`.

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --lib`
Expected: FAIL because the two helpers do not exist.

- [ ] **Step 3: Implement and integrate the helpers**

Use `is_connection_active` in `get_bandwidth` and `spawn_monitor_loop`. After every monitor sample, call both `tray.set_icon(Some(icon))` and `tray.set_title(Some(&format_tray_title(...)))`. Keep `icon_as_template(false)` so colored indicators remain visible. Keep tray toggling limited to left-button `MouseButtonState::Up`.

- [ ] **Step 4: Run Rust tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --lib`
Expected: all tests pass.

### Task 3: Build, install and runtime verification

**Files:**
- Replace: `Connection Monitor/ULTIMA VERSIONE/Connection Monitor.app`
- Replace: `Connection Monitor/ULTIMA VERSIONE/Connection Monitor_0.1.0_aarch64.dmg`
- Replace: `/Applications/Connection Monitor.app`

**Interfaces:**
- Consumes: passing source tree from Tasks 1-2.
- Produces: identical installed and delivery bundles.

- [ ] **Step 1: Run complete source verification**

Run: `node --test tests/*.test.mjs && cargo test --manifest-path src-tauri/Cargo.toml && npm run build`
Expected: exit 0 with no failing tests.

- [ ] **Step 2: Build the Apple Silicon release bundle**

Run: `npm run tauri build -- --target aarch64-apple-darwin`
Expected: exit 0 and fresh `.app` plus `.dmg` under `src-tauri/target/aarch64-apple-darwin/release/bundle`.

- [ ] **Step 3: Replace both copies safely**

Terminate only running `Connection Monitor` instances, copy the previous installed and delivery apps to a timestamped temporary backup, then copy the new bundle to both exact destinations.

- [ ] **Step 4: Verify artifacts and launched process**

Compare SHA-256 hashes of all three executables, launch `/Applications/Connection Monitor.app`, and confirm the running executable path resolves inside `/Applications/Connection Monitor.app`.

- [ ] **Step 5: Verify the original symptoms in the UI**

Check that the tray shows icon plus live title, click-release leaves the widget visible, the status leaves `Connecting...`, a second tray click hides it, and reopening plus the close button hides it.

