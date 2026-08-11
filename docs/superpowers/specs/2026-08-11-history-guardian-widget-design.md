# Widget, Historical Statistics and Local Engagement Design

**Date:** 2026-08-11  
**Status:** Approved  
**Product:** Connection Monitor for macOS

## Goal

Fix the broken floating widget and add a private, local 30-day connection history that powers Connection Guardian, Internet Replay, Connection Streak, contextual tooltips, and an in-app guide.

## Product Constraints

- All processing and storage stay on the Mac.
- No account, cloud service, telemetry, or new background network destination.
- Retain historical data for 30 rolling days.
- Preserve Italian, English, Spanish, and French support.
- Keep the native-looking, sober macOS visual language.
- Reuse the current canvas renderer instead of adding a chart framework.

## Floating Widget

The `floating` Tauri window receives a dedicated document and controller instead of loading the full main panel. It displays four compact indicators:

1. current download rate;
2. current upload rate;
3. quality score and status color;
4. data transferred during the current app session.

The widget is always on top, draggable from its background, and can be closed without stopping monitoring. Every indicator has a localized tooltip. It consumes the existing `get_bandwidth` command and `stats-update` event, so it cannot create a second monitoring loop.

## Historical Storage

A focused Rust history module owns persistence and querying. It stores versioned JSON data below the application's local data directory using atomic replace-on-save writes.

Every minute it creates one aggregate containing:

- timestamp and connectivity ratio;
- average and minimum quality score;
- average download and upload rates;
- average ping, jitter, packet loss, and optional Wi-Fi signal;
- downloaded and uploaded megabytes during the bucket;
- sample count.

Buckets older than 30 days are pruned during writes and startup. A malformed file is moved aside as a recoverable backup; monitoring continues with a clean store. The frontend receives only range-limited summaries through Tauri commands, never the storage file directly.

## Statistics Tab

The main panel gains a third tab named **Statistics** between Monitor and Info. It supports fixed ranges of 24 hours, 7 days, and 30 days.

The view contains:

- summary cards for average quality, availability, number of interruptions, and transferred data;
- a canvas timeline with quality, download/upload, and ping display modes;
- incident markers that open a concise explanation panel;
- a Replay scrubber for moving through the selected day;
- a Connection Streak card with the current and best reliable-day streak.

Empty history, partially collected ranges, offline periods, and corrupted-store recovery use explicit localized states rather than blank charts.

## Connection Guardian

Guardian converts live samples into meaningful local incidents. An offline sample opens an outage immediately. A degraded incident opens only after three consecutive critical samples, avoiding transient false alarms. An incident closes after three consecutive healthy samples.

Each incident records:

- start, end, duration, and type;
- lowest quality score;
- dominant quality issue from the existing diagnostic model;
- representative metric value and unit;
- localized recommendation key.

Only one active incident exists at a time. A transition from degradation to offline upgrades the active incident rather than duplicating it. Guardian adds timeline evidence and explanations; it does not claim certainty about ISP or router fault.

## Internet Replay

Replay is a local time cursor over a selected day. Moving it updates the visible quality, bandwidth, latency, data, and active incident at that moment. Incident markers are keyboard accessible and can jump the cursor to the event start. Replay uses already aggregated buckets and performs no additional probing.

## Connection Streak

A local calendar day is reliable when availability is at least 99% and average quality is at least 75. Only complete days count toward streaks; the current partial day is shown as progress but does not extend or break a streak.

The interface shows current streak, personal best, and understated milestones at 3, 7, 14, and 30 reliable days. Milestones are informative, not competitive, and never leave the device.

## Tooltips and Guide

Controls, metrics, chart modes, widget values, Guardian labels, Replay, and Streak receive localized, keyboard-accessible tooltips. Native `title` text is insufficient: a shared tooltip component provides consistent timing and positioning while preserving `aria-label` or `aria-describedby` semantics.

The first launch after this update opens a short three-step guide:

1. understand live values and quality;
2. use the floating widget and tray indicators;
3. read history, incidents, Replay, and Streak.

The guide is dismissible, remembers completion locally, and remains available from the Info tab. It does not block monitoring.

## Components and Data Flow

- `monitor` produces live `ConnectionStats` once.
- `HistoryRecorder` aggregates samples into minute buckets.
- `GuardianEngine` evaluates the same samples and writes incident transitions.
- `HistoryStore` persists buckets, incidents, and streak metadata atomically.
- Tauri query commands return range summaries and day replay data.
- The main Statistics controller renders summaries, chart, incidents, Replay, and Streak.
- The floating controller renders only its four live indicators.

The recording path never depends on either window being visible.

## Error Handling

- Failed history writes are logged and retried at the next bucket without stopping monitoring.
- Invalid or future schema versions are preserved as backups and never overwritten silently.
- Query failures produce a localized unavailable state in Statistics.
- Missing data is represented as gaps, not zero values.
- Closing either window removes its listeners; reopening does not duplicate subscriptions.

## Testing and Acceptance

Automated Rust tests cover aggregation, rolling retention, atomic recovery behavior, Guardian hysteresis and transitions, incident summaries, reliable-day qualification, streak calculation, and range queries.

JavaScript tests cover statistics formatting and range models, replay selection, tooltip behavior, guide completion, translations, and detection of the dedicated floating entry point.

Acceptance requires:

- clicking Widget opens a legible dedicated four-value widget;
- no clipped main-panel content appears in the floating window;
- history continues while all windows are hidden;
- 24-hour, 7-day, and 30-day views handle real and empty data;
- incidents explain what happened and suggest an action;
- Replay and Streak follow the rules above;
- all four languages contain no missing UI keys;
- Rust tests, JavaScript tests, production build, and Tauri release build pass;
- the newly built app is installed, launched, and visually checked on the computer.

