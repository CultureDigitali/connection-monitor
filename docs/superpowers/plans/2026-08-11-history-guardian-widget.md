# History, Guardian and Floating Widget Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver a correct four-value floating widget, 30-day local statistics, Connection Guardian, Internet Replay, Connection Streak, contextual tooltips, and an in-app guide.

**Architecture:** The existing one-second monitor remains the only live data source. Focused Rust history units aggregate minute buckets, detect incidents, persist a versioned atomic JSON document, and expose range queries; separate JavaScript modules render the dedicated floating entry point and the main Statistics experience.

**Tech Stack:** Rust 2021, Tauri 2, Tokio, Serde JSON, Chrono, vanilla JavaScript modules, HTML Canvas, Vite, Node test runner, CSS.

## Global Constraints

- All processing and storage stay on the Mac.
- No account, cloud service, telemetry, or new background network destination.
- Retain historical data for 30 rolling days.
- Preserve Italian, English, Spanish, and French support.
- Keep the native-looking, sober macOS visual language.
- Reuse the current canvas renderer instead of adding a chart framework.
- A local calendar day is reliable only with availability at least 99% and average quality at least 75.
- Version the feature release as `0.3.0`.

---

### Task 1: Historical domain, minute aggregation and streaks

**Files:**
- Create: `src-tauri/src/history/mod.rs`
- Create: `src-tauri/src/history/model.rs`
- Create: `src-tauri/src/history/recorder.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Consumes: `ConnectionStats` snapshots and Unix timestamps.
- Produces: `MinuteBucket`, `HistorySummary`, `StreakSummary`, and `HistoryRecorder::record(timestamp, &stats) -> Option<MinuteBucket>`.
- Produces: `summarize(buckets, incidents, range_start, range_end) -> HistorySummary` and `calculate_streak(days, today) -> StreakSummary`.

- [ ] **Step 1: Write failing aggregation tests**

Add Rust tests that feed two online samples into one minute and assert averaged quality, bandwidth, ping, availability, byte deltas, minimum quality, and sample count. Add a boundary test proving that a sample in the next minute finalizes the previous bucket.

```rust
let mut recorder = HistoryRecorder::new();
assert!(recorder.record(60, &online_stats(80, 10.0, 2.0, 100.0, 40.0)).is_none());
assert!(recorder.record(61, &online_stats(60, 20.0, 4.0, 101.0, 42.0)).is_none());
let bucket = recorder.record(120, &online_stats(90, 1.0, 1.0, 102.0, 43.0)).unwrap();
assert_eq!(bucket.sample_count, 2);
assert_eq!(bucket.average_quality, 70.0);
assert_eq!(bucket.minimum_quality, 60);
assert_eq!(bucket.availability, 1.0);
assert_eq!(bucket.downloaded_mb, 1.0);
assert_eq!(bucket.uploaded_mb, 2.0);
```

- [ ] **Step 2: Verify RED**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --lib history::recorder::tests`

Expected: FAIL because the history module and recorder do not exist.

- [ ] **Step 3: Implement immutable history models and recorder**

Define serializable structs with explicit schema fields. Aggregate finite metric values only, represent unavailable measurements with `Option<f64>`, compute traffic from monotonic session-total deltas, and clamp negative deltas to zero after app restarts.

- [ ] **Step 4: Write failing summary and streak tests**

Test empty summaries, a mixed online/offline range, a reliable day at exactly 99% and 75 points, an unreliable day below either boundary, exclusion of the current partial day, and current/best sequences across calendar gaps.

```rust
let days = vec![
    DailyReliability::complete("2026-08-07", 0.995, 82.0),
    DailyReliability::complete("2026-08-08", 0.990, 75.0),
    DailyReliability::complete("2026-08-09", 0.980, 90.0),
    DailyReliability::complete("2026-08-10", 1.000, 95.0),
];
let streak = calculate_streak(&days, "2026-08-11");
assert_eq!(streak.current, 1);
assert_eq!(streak.best, 2);
```

- [ ] **Step 5: Implement summaries and streak calculation**

Compute weighted averages by sample count, total traffic, range availability, min quality, incident count, complete-day reliability, current streak, best streak, next milestone from `[3, 7, 14, 30]`, and current-day progress separately.

- [ ] **Step 6: Verify GREEN and commit**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --lib history`

Expected: all history model and recorder tests pass.

```bash
git add src-tauri/src/history src-tauri/src/lib.rs
git commit -m "feat: aggregate local connection history"
```

### Task 2: Connection Guardian incident engine

**Files:**
- Create: `src-tauri/src/history/guardian.rs`
- Modify: `src-tauri/src/history/mod.rs`
- Test: unit tests inside `src-tauri/src/history/guardian.rs`

**Interfaces:**
- Consumes: timestamped `ConnectionStats`.
- Produces: `GuardianEngine::evaluate(timestamp, &stats) -> GuardianTransition`.
- Produces: `Incident { id, kind, started_at, ended_at, lowest_quality, issue_key, value, unit, recommendation_key }`.

- [ ] **Step 1: Write failing Guardian tests**

Test immediate offline opening, no degraded incident after two critical samples, degraded opening on the third, recovery only after three healthy samples, offline upgrading an active degradation, lowest-score tracking, dominant-issue tracking, and no duplicate incident while state is unchanged.

```rust
let mut guardian = GuardianEngine::new();
assert!(matches!(guardian.evaluate(1, &critical_stats()), GuardianTransition::None));
assert!(matches!(guardian.evaluate(2, &critical_stats()), GuardianTransition::None));
assert!(matches!(guardian.evaluate(3, &critical_stats()), GuardianTransition::Opened(_)));
assert!(matches!(guardian.evaluate(4, &healthy_stats()), GuardianTransition::None));
assert!(matches!(guardian.evaluate(5, &healthy_stats()), GuardianTransition::None));
assert!(matches!(guardian.evaluate(6, &healthy_stats()), GuardianTransition::Closed(_)));
```

- [ ] **Step 2: Verify RED**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --lib history::guardian::tests`

Expected: FAIL because `GuardianEngine` is absent.

- [ ] **Step 3: Implement the state machine**

Use `Healthy`, `PendingDegraded`, `Degraded`, and `Offline` internal states. Treat online scores below 25 as critical, open outage immediately, require three consecutive critical samples for degradation, require three consecutive online scores of at least 50 to recover, and preserve only measured diagnostic evidence.

- [ ] **Step 4: Verify GREEN and commit**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --lib history::guardian::tests`

Expected: every Guardian transition test passes.

```bash
git add src-tauri/src/history/guardian.rs src-tauri/src/history/mod.rs
git commit -m "feat: detect local connection incidents"
```

### Task 3: Atomic store, rolling retention and Tauri history API

**Files:**
- Create: `src-tauri/src/history/store.rs`
- Modify: `src-tauri/src/history/mod.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Produces: `HistoryStore::load(path)`, `append_bucket`, `apply_transition`, `query_range`, and `save_atomic`.
- Produces Tauri commands: `get_history(range: String) -> Result<HistoryResponse, String>` and `get_replay(day: String) -> Result<ReplayResponse, String>`.
- Consumes: finalized buckets and Guardian transitions from Tasks 1-2.

- [ ] **Step 1: Write failing store tests**

Use a unique directory below `std::env::temp_dir()` and test schema round-trip, pruning strictly older than 30 days, active-incident persistence, closed-incident replacement, range filtering, malformed JSON backup, and absence of partially written destination files.

```rust
let mut store = HistoryStore::load(&path, now).unwrap();
store.append_bucket(bucket_at(now - 31 * DAY_SECONDS));
store.append_bucket(bucket_at(now));
store.save_atomic().unwrap();
let restored = HistoryStore::load(&path, now).unwrap();
assert_eq!(restored.document().buckets.len(), 1);
```

- [ ] **Step 2: Verify RED**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --lib history::store::tests`

Expected: FAIL because `HistoryStore` is absent.

- [ ] **Step 3: Implement versioned atomic persistence**

Store schema version `1` in the application local-data directory as `history.json`. Serialize to `history.json.tmp`, sync, and rename over the destination. Rename malformed or unsupported documents to `history.corrupt-<timestamp>.json`, then initialize an empty document. Prune buckets and closed incidents older than 30 days.

- [ ] **Step 4: Integrate recording independently of windows**

Add shared recorder, Guardian, and store state during Tauri setup. Feed every monitor snapshot into recorder and Guardian before emitting UI events. Persist finalized minute buckets and every incident transition. Do not read window visibility in the recording path.

- [ ] **Step 5: Add range and replay commands**

Accept only `24h`, `7d`, or `30d`. Convert each to an exact start timestamp, return summary plus buckets and incidents, validate replay days as `YYYY-MM-DD`, and return localized keys rather than backend prose.

- [ ] **Step 6: Verify GREEN and commit**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --lib history`

Expected: all aggregation, Guardian, storage, retention, query, and streak tests pass.

```bash
git add src-tauri/src/history src-tauri/src/lib.rs
git commit -m "feat: persist and query 30-day history"
```

### Task 4: Dedicated floating widget entry point

**Files:**
- Create: `src/floating.html`
- Create: `src/floating.js`
- Create: `src/floating-model.js`
- Create: `src/floating.css`
- Create: `tests/floating-model.test.mjs`
- Modify: `vite.config.js`
- Modify: `src-tauri/tauri.conf.json`

**Interfaces:**
- Consumes: `get_bandwidth`, `stats-update`, `formatRate`, `formatData`, and translations.
- Produces: `buildFloatingModel(stats, translate) -> { download, upload, quality, data, state, tooltips }`.
- Produces: a Vite `floating.html` bundle loaded only by the `floating` window.

- [ ] **Step 1: Write failing floating-model tests**

Assert separate download/upload formatting, summed session traffic, online quality score, connecting/offline labels, quality CSS state, and four localized tooltip keys.

```javascript
const model = buildFloatingModel({
  download_mbps: 1.5,
  upload_mbps: 0.25,
  total_download_mb: 10,
  total_upload_mb: 2,
  quality_score: 84,
  connection_status: 'online',
}, (key) => key);
assert.equal(model.download.value, '1.5');
assert.equal(model.upload.value, '250');
assert.equal(model.data.value, '12 MB');
assert.equal(model.quality.value, '84');
```

- [ ] **Step 2: Verify RED**

Run: `node --test tests/floating-model.test.mjs`

Expected: FAIL because `floating-model.js` is absent.

- [ ] **Step 3: Implement model and dedicated document**

Render four equal indicators with distinct green, blue, score-dependent, and purple accents. Use `data-tauri-drag-region` on the background, tabular numerals, compact units, localized accessible labels, and one stats subscription removed on unload.

- [ ] **Step 4: Configure both Vite entry points and Tauri URL**

Set Rollup inputs to `src/index.html` and `src/floating.html`. Set the `floating` window URL to `floating.html`, size to `336x78`, keep it non-resizable and always-on-top, and preserve hide-on-close behavior.

- [ ] **Step 5: Verify GREEN and commit**

Run: `node --test tests/floating-model.test.mjs && npm run build`

Expected: tests pass and `dist/floating.html` exists with its own script and stylesheet assets.

```bash
git add src/floating.html src/floating.js src/floating-model.js src/floating.css tests/floating-model.test.mjs vite.config.js src-tauri/tauri.conf.json
git commit -m "fix: deliver a dedicated floating widget"
```

### Task 5: Statistics, Internet Replay and Connection Streak UI

**Files:**
- Create: `src/history-model.js`
- Create: `src/history-chart.js`
- Create: `tests/history-model.test.mjs`
- Modify: `src/index.html`
- Modify: `src/main.js`
- Modify: `src/styles.css`

**Interfaces:**
- Consumes: `get_history` and `get_replay` responses from Task 3.
- Produces: `buildHistoryView(response, translate)`, `selectReplayPoint(buckets, timestamp)`, and `HistoryChart.render(canvas, model, mode, cursor)`.

- [ ] **Step 1: Write failing history-model tests**

Test empty and partial ranges, summary formatting, gaps remaining gaps instead of zeroes, nearest replay-point selection, incident selection, streak milestone progress, and dominant incident recommendation mapping.

```javascript
assert.equal(selectReplayPoint([{ timestamp: 100 }, { timestamp: 160 }], 145).timestamp, 160);
assert.equal(selectReplayPoint([], 145), null);
```

- [ ] **Step 2: Verify RED**

Run: `node --test tests/history-model.test.mjs`

Expected: FAIL because the history model does not exist.

- [ ] **Step 3: Implement pure view models and canvas renderer**

Create range button models for 24 hours, 7 days, and 30 days; format quality, availability, incidents, and traffic; preserve missing samples as canvas gaps. Add quality, bandwidth, and ping modes, incident markers, keyboard-focusable incident controls, and a Replay cursor.

- [ ] **Step 4: Build the Statistics tab**

Add `Statistics` between Monitor and Info. Render four summary cards, range selector, chart mode selector, incident explanation card, Replay slider with current metrics, and the restrained current/best Streak card. Fetch only when Statistics opens or its range changes; refresh the active 24-hour view after a finalized bucket event.

- [ ] **Step 5: Verify GREEN and commit**

Run: `node --test tests/history-model.test.mjs && npm run build`

Expected: history tests pass and the production frontend builds.

```bash
git add src/history-model.js src/history-chart.js tests/history-model.test.mjs src/index.html src/main.js src/styles.css
git commit -m "feat: add statistics replay and streak views"
```

### Task 6: Localized tooltips and in-app guide

**Files:**
- Create: `src/tooltips.js`
- Create: `src/guide.js`
- Create: `tests/tooltips.test.mjs`
- Create: `tests/guide.test.mjs`
- Modify: `src/index.html`
- Modify: `src/main.js`
- Modify: `src/floating.js`
- Modify: `src/i18n.js`
- Modify: `src/styles.css`
- Modify: `src/floating.css`
- Modify: `tests/translations.test.mjs`

**Interfaces:**
- Produces: `bindTooltips(root, translate) -> cleanup`.
- Produces: `GuideState(storage, version)` with `shouldOpen`, `complete`, and `reset`.
- Consumes: elements with `data-tooltip-key` and local storage key `cm-guide-version`.

- [ ] **Step 1: Write failing tooltip, guide and translation tests**

Use `EventTarget` test elements to prove focus/mouse opening, Escape dismissal, cleanup, first-run guide visibility, persistence after completion, reopening from Info, and exact key parity across all four language dictionaries.

```javascript
const storage = new MapStorage();
const guide = new GuideState(storage, '0.3');
assert.equal(guide.shouldOpen(), true);
guide.complete();
assert.equal(new GuideState(storage, '0.3').shouldOpen(), false);
```

- [ ] **Step 2: Verify RED**

Run: `node --test tests/tooltips.test.mjs tests/guide.test.mjs tests/translations.test.mjs`

Expected: FAIL because the modules and required translation keys are absent.

- [ ] **Step 3: Implement accessible tooltips**

Create one reusable tooltip surface per document. Support pointer hover, keyboard focus, Escape, viewport clamping, `role="tooltip"`, `aria-describedby`, delayed show, immediate focus show, and listener cleanup. Apply keys to live metrics, chart modes, ranges, widget values, Guardian, Replay, Streak, close, language, speed test, and guide controls.

- [ ] **Step 4: Implement the three-step guide**

Build a non-blocking modal with official app logo and compact inline metric illustrations. Steps cover live quality, tray plus floating widget, and Statistics plus Guardian/Replay/Streak. Add Skip, Back, Next, Done, focus management, reduced-motion support, remembered completion, and a `Guide` button in Info that always reopens it.

- [ ] **Step 5: Add complete localization**

Add the same Statistics, Guardian, Replay, Streak, tooltip, empty/error, and guide keys in English, Italian, Spanish, and French. Remove the duplicate French `metricLoss` entry while preserving its value.

- [ ] **Step 6: Verify GREEN and commit**

Run: `node --test tests/*.test.mjs && npm run build`

Expected: all JavaScript tests pass, translation key sets match, and Vite builds both entry points.

```bash
git add src/tooltips.js src/guide.js tests/tooltips.test.mjs tests/guide.test.mjs src/index.html src/main.js src/floating.js src/i18n.js src/styles.css src/floating.css tests/translations.test.mjs
git commit -m "feat: add tooltips and an in-app guide"
```

### Task 7: Release 0.3.0, install and runtime verification

**Files:**
- Modify: `package.json`
- Modify: `package-lock.json`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/Cargo.lock`
- Modify: `src-tauri/tauri.conf.json`
- Modify: `README.md`
- Replace: `/Applications/Connection Monitor.app`
- Replace: `/Users/luigimacmini/Desktop/Connection Monitor/ULTIMA VERSIONE/Connection Monitor.app`
- Create: `/Users/luigimacmini/Desktop/Connection Monitor/ULTIMA VERSIONE/Connection Monitor_0.3.0_aarch64.dmg`

**Interfaces:**
- Produces: identical installed and delivery app bundles at version `0.3.0`.

- [ ] **Step 1: Bump and document version 0.3.0**

Set npm, Cargo, and Tauri versions to `0.3.0`, update native lockfiles, refresh the Info fallback version, and update the README features and roadmap to describe local 30-day Statistics, Guardian, Replay, Streak, tooltips, guide, and dedicated widget.

- [ ] **Step 2: Run complete source verification**

Run:

```bash
node --test tests/*.test.mjs
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
npm run build
npm audit --audit-level=high
git diff --check
```

Expected: every command exits zero, all tests pass, both frontend entry points build, and the audit reports no high-severity vulnerabilities.

- [ ] **Step 3: Build release artifacts**

Run: `npm run tauri build -- --target aarch64-apple-darwin`

Expected: fresh `.app` and `.dmg` exist below `src-tauri/target/aarch64-apple-darwin/release/bundle` and report version `0.3.0`.

- [ ] **Step 4: Install recoverably**

Stop only running Connection Monitor processes. Move the existing installed and delivery applications into timestamped backup names in their respective parent directories, copy the new app to both exact destinations, copy the new DMG to `ULTIMA VERSIONE`, and launch only `/Applications/Connection Monitor.app`.

- [ ] **Step 5: Verify the real application**

Confirm the executable path and version, inspect code signatures, compare executable SHA-256 hashes between build and both installed copies, then visually verify: tray colors and four indicators; persistent main panel; dedicated floating widget; Statistics ranges; empty/collected states; Guardian explanation; Replay controls; Streak card; tooltips; guide reopening; and Info branding.

- [ ] **Step 6: Commit release metadata**

```bash
git add package.json package-lock.json src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/tauri.conf.json src/index.html README.md
git commit -m "chore: release Connection Monitor 0.3.0"
```

Run `git status --short --branch` and retain the current branch without pushing or merging.
