<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="src-tauri/icons/128x128.png">
  <img src="src-tauri/icons/128x128.png" width="120" alt="Connection Monitor Logo">
</picture>

# Connection Monitor

**The beautiful, privacy-first internet connection monitor for macOS**

[![Stars](https://img.shields.io/github/stars/CultureDigitali/connection-monitor?style=flat&logo=github&color=yellow)](https://github.com/CultureDigitali/connection-monitor/stargazers)
[![Downloads](https://img.shields.io/github/downloads/CultureDigitali/connection-monitor/total?style=flat&logo=apple&color=blue)](https://github.com/CultureDigitali/connection-monitor/releases)
[![License](https://img.shields.io/badge/license-MIT-green?style=flat)](LICENSE)
[![Platform](https://img.shields.io/badge/macOS-11.0+-silver?style=flat&logo=apple)](https://www.apple.com/macos)
[![Release](https://img.shields.io/github/v/release/CultureDigitali/connection-monitor?style=flat&color=success)](https://github.com/CultureDigitali/connection-monitor/releases)
[![Build](https://img.shields.io/github/actions/workflow/status/CultureDigitali/connection-monitor/release.yml?branch=main&style=flat&logo=githubactions)](https://github.com/CultureDigitali/connection-monitor/actions)

[![Made with Rust](https://img.shields.io/badge/Rust-2021-orange?style=flat&logo=rust&logoColor=white)](https://www.rust-lang.org)
[![Made with Tauri](https://img.shields.io/badge/Tauri-2.0-blue?style=flat&logo=tauri&logoColor=white)](https://tauri.app)
[![Types](https://img.shields.io/badge/Vanilla-JS-yellow?style=flat&logo=javascript&logoColor=white)](https://developer.mozilla.org/en-US/docs/Web/JavaScript)

[Download](https://github.com/CultureDigitali/connection-monitor/releases) · [Features](#-features) · [Install](#-install) · [Screenshots](#-screenshots) · [Roadmap](#-roadmap) · [Credits](#-credits)

---

### Stop guessing. Know your connection.

Connection Monitor lives in your menu bar and tells you exactly how good your internet is — right now. Not tomorrow. Not after a speed test. **Right now.**

</div>

---

## ✨ Why you'll love it

<div align="center">

| | | | |
|:---:|:---:|:---:|:---:|
| **Beautiful** | **Private** | **Lightweight** | **Free** |
| Color-coded icon that changes with your quality | Zero telemetry. Zero cloud. Zero tracking | 5MB app. ~10MB RAM. Negligible CPU | 100% Open Source. MIT Licensed |

</div>

---

## 🚀 Features

### 📊 Real-time bandwidth monitoring
See your **download** and **upload** speeds live in the menu bar. No delays. No refresh buttons. Just open your Mac and look up.

### 🎯 Connection quality score (0–100)
We compute a single score from latency, jitter, packet loss, and WiFi signal. Know instantly if your connection is **excellent**, **good**, **fair**, **poor**, or **critical**.

### 🎨 Color-coded tray icon
The icon changes color based on your connection quality:
- 🟢 **Green** = Excellent (90–100)
- 🟢 **Light Green** = Good (75–89)
- 🟡 **Yellow** = Fair (50–74)
- 🟠 **Orange** = Poor (25–49)
- 🔴 **Red** = Critical (0–24)

### ⚡ Built-in speed test
Run a full speed test with one click. See your max download speed and latency.

### 📈 Bandwidth history chart
A live-updating chart shows the last 60 seconds of your bandwidth usage. Spot patterns. Find issues.

### 🌍 Multi-language
Available in **English**, **Italiano**, **Español**, and **Français**.

### 🔒 Privacy first
- No accounts
- No analytics
- No network calls except to your chosen speed test server
- All data stays on your Mac, always

---

## 📥 Install

### Option 1: Download (Recommended)

Grab the latest DMG from the [Releases](https://github.com/CultureDigitali/connection-monitor/releases) page.

**System Requirements:**
- macOS 11.0 (Big Sur) or later
- Apple Silicon (M1/M2/M3/M4)

### Option 2: Build from source

```bash
# Clone
git clone https://github.com/CultureDigitali/connection-monitor.git
cd connection-monitor

# Install dependencies
npm install

# Run in development mode
npm run tauri dev

# Build release
npm run tauri build
```

The built `.dmg` will be at `src-tauri/target/release/bundle/dmg/`.

---

## 📸 Screenshots

<div align="center">

### Menu Bar
```
┌─────────────────────────────────────────────────┐
│  📶  ↓ 45.2 MB/s  ↑ 12.1 MB/s  Score: 87/100 │
└─────────────────────────────────────────────────┘
```

### Main Panel
| Monitor View | Speed Test View |
|:---:|:---:|
| Download/Upload speeds with live chart | Full speed test with gauge |
| Quality score with star rating | Latency results |
| Ping, Jitter, Loss, WiFi metrics | Historical data |

</div>

---

## 🛠 Tech Stack

| Layer | Technology |
|:---|:---|
| **Backend** | Rust 2021 |
| **Framework** | Tauri 2.0 |
| **Frontend** | Vanilla JavaScript + HTML + CSS |
| **Ping** | surge-ping (ICMP) |
| **Image** | image crate (PNG encoding) |
| **System Info** | sysinfo crate |

---

## 📋 Roadmap

- [x] Real-time bandwidth monitoring
- [x] Quality score (latency + jitter + loss + signal)
- [x] Color-coded dynamic tray icon
- [x] Speed test integration
- [x] Multi-language support (EN, IT, ES, FR)
- [ ] Daily/weekly statistics view
- [ ] Connection alerts & notifications
- [ ] Intel Mac support
- [ ] Dark/Light mode toggle
- [ ] Customizable refresh interval
- [ ] Export data to CSV

See [Discussions](https://github.com/CultureDigitali/connection-monitor/discussions) for ideas and feedback.

---

## 🤝 Contributing

Contributions are welcome! If you have ideas or bug reports:

1. **Open an Issue** — bugs, feature requests, or questions
2. **Start a Discussion** — share ideas with the community
3. **Submit a PR** — bug fixes and features are appreciated

Please read our [Contributing Guidelines](CONTRIBUTING.md) first.

---

## ⭐ Star this repo

If you find this project useful, **give it a star**! It helps others discover the project and motivates continued development.

<div align="center">

[![Star History Chart](https://api.star-history.com/svg?repos=CultureDigitali/connection-monitor&type=Date)](https://star-history.com/#CultureDigitali/connection-monitor&Date)

</div>

---

## 🙏 Credits

Developed with love by **[Luigi Strazzullo](https://linkedin.com/in/luigistrazzullo)** for **[Culture Digitali Srl](https://culturedigitali.eu)**

<div align="center">

### Culture Digitali Srl

*Startup Innovativa* — Digitalizzazione, Formazione & Consulenza Digitale

Premiata da **OVH**, **Amazon**, **MiBac** e **Google for Startups**

| | |
|:---|:---|
| 🌐 [culturedigitali.eu](https://culturedigitali.eu) | 📧 [info@culturedigitali.eu](mailto:info@culturedigitali.eu) |
| 📞 (+39) 081 180 88 248 | 📍 Via Coroglio 57D, 80124 Napoli |
| 🏢 P.IVA: 09465861210 | 💼 [LinkedIn](https://linkedin.com/company/culture-digitali) |

</div>

---

## 📄 License

This project is licensed under the **MIT License** — see [LICENSE](LICENSE) for details.

---

<div align="center">

**Made with ❤️ in Naples, Italy**

[⬆ Back to top](#connection-monitor)

</div>
