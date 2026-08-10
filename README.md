# Connection Monitor

A macOS menu bar application that monitors your internet connection quality, bandwidth, and network statistics in real-time.

![Platform](https://img.shields.io/badge/platform-macOS-lightgrey)
![License](https://img.shields.io/badge/license-MIT-green)

## Features

- **Real-time bandwidth monitoring** - Download and upload speeds in your menu bar
- **Connection quality score** - Computed from latency, jitter, packet loss, and WiFi signal
- **Color-coded tray icon** - Visual indicator of connection health
- **Historical statistics** - Daily and weekly connection quality trends
- **Multiple languages** - English, Italiano, Espanol, Francais
- **Privacy focused** - All data stays on your machine, no telemetry

## Download

Download the latest release from the [Releases](https://github.com/culturedigitali/connection-monitor/releases) page.

## Building from Source

### Prerequisites

- [Node.js](https://nodejs.org/) v18+
- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- [Xcode Command Line Tools](https://developer.apple.com/xcode/resources/)

### Setup

```bash
git clone https://github.com/culturedigitali/connection-monitor.git
cd connection-monitor
npm install
```

### Development

```bash
npm run tauri dev
```

### Build Release

```bash
npm run tauri build
```

The built `.dmg` will be in `src-tauri/target/release/bundle/dmg/`.

## Network Connections

This app makes outbound connections to:

- `1.1.1.1` (Cloudflare DNS) - latency measurement
- `speedtest.tele2.net` - bandwidth measurement

No personal data is collected or transmitted. All monitoring data is stored locally on your Mac.

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Credits

Developed by **Luigi Strazzullo** for **Culture Digitali Srl**

### Culture Digitali Srl

Startup innovativa premiata da OVH, Amazon, MiBac e Google for Startups, specializzata in digitalizzazione, formazione e consulenza digitale.

- **Website:** [culturedigitali.eu](https://culturedigitali.eu)
- **Email:** [info@culturedigitali.eu](mailto:info@culturedigitali.eu)
- **Phone:** (+39) 081 180 88 248
- **Address:** Via Coroglio 57D, 80124 Napoli
- **P.IVA:** 09465861210
- **LinkedIn:** [Culture Digitali](https://linkedin.com/company/culture-digitali)

### Luigi Strazzullo

Amministratore e Fondatore di Culture Digitali Srl, consulente e formatore con oltre 15 anni di esperienza nel digitale.

- **LinkedIn:** [linkedin.com/in/luigistrazzullo](https://linkedin.com/in/luigistrazzullo)
- **Website:** [luigistrazzullo.it](https://luigistrazzullo.it)

---

Made with love in Naples, Italy
