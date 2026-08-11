# Connection Monitor Diagnostics and Info Design

## Goal

Explain every connection quality judgment with measurable causes and useful actions, use the official application logo, and replace the dated Credits tab with a polished Info section.

## Quality diagnosis

The existing score remains authoritative. Its calculation will return a breakdown for latency, jitter, packet loss, and Wi-Fi signal so the explanation cannot disagree with the score.

Each issue contains a stable key, measured value, severity, score penalty, and recommendation key. Issues are ordered by penalty and severity. The compact view shows the two most important causes and the first practical recommendation. It must describe causes as probable rather than certain.

Offline and connecting states receive dedicated explanations. When no problem is detected, the panel confirms that the measured values are within healthy limits.

## Widget layout

The redundant star strip becomes a compact diagnostic card beneath the metrics. It displays:

- `Perché`: up to two primary causes;
- `Cosa fare`: one prioritized recommendation;
- a Details control that expands the complete metric breakdown.

The monitor content becomes vertically scrollable only when needed, preserving the current 340 by 540 window. Closing the widget remains an explicit action.

## Official identity

The generic Wi-Fi symbol in the header is replaced with the existing official application icon from `src-tauri/icons/128x128.png`. Vite imports the original asset directly so the header and packaged application cannot drift to different logos.

## Info section

The Credits tab is renamed Info in all supported languages. Its content uses the official logo, app name and runtime version as the visual focus, followed by compact cards for developer, Culture Digitali, useful links, and license. Long contact prose is reduced and spacing, typography, and link treatments follow the monitor's glass interface.

## Localization and accessibility

All diagnostic text, recommendations, Info labels, and controls are translated into Italian, English, Spanish, and French. The Details control exposes its expanded state and the official logo has an accessible application-name label.

## Testing

Rust tests prove score penalties and issue ordering for latency, jitter, packet loss, Wi-Fi, healthy, connecting, and offline states. JavaScript tests prove compact selection and translated rendering data. The full Rust and Node suites, production build, signed macOS bundle, installed version, and runtime process are verified before completion.
