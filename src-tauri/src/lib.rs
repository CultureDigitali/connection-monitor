use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_updater::UpdaterExt;

mod i18n;
mod history;
mod monitor;
mod stats;
mod tray;

use i18n::{translate, I18n, Language};
use monitor::bandwidth::BandwidthMonitor;
use monitor::notify::{Notification, Notifier};
use monitor::ping::{do_ping, PingMonitor};
use monitor::speedtest::{run_speed_test, SpeedTestResult};
use monitor::wifi::WifiMonitor;
use stats::{ConnectionStats, ConnectionStatus};
use tray::{format_data_compact, format_rate_compact, indicator_icon, IndicatorKind};

const CONFIG_DIR_NAME: &str = "connection-monitor";
const CONFIG_FILE_NAME: &str = "lang.conf";
const COLORS_FILE_NAME: &str = "colors.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ColorPrefs {
    pub download: String,
    pub upload: String,
    pub data: String,
    pub quality_excellent: String,
    pub quality_good: String,
    pub quality_fair: String,
    pub quality_poor: String,
    pub quality_critical: String,
}

impl Default for ColorPrefs {
    fn default() -> Self {
        ColorPrefs {
            download: "#34d399".to_string(),
            upload: "#60a5fa".to_string(),
            data: "#a78bfa".to_string(),
            quality_excellent: "#34d399".to_string(),
            quality_good: "#a78bfa".to_string(),
            quality_fair: "#fbbf24".to_string(),
            quality_poor: "#fb923c".to_string(),
            quality_critical: "#f87171".to_string(),
        }
    }
}

fn config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join(CONFIG_DIR_NAME))
}

fn load_language() -> Language {
    if let Some(dir) = config_dir() {
        let path = dir.join(CONFIG_FILE_NAME);
        if let Ok(mut file) = std::fs::File::open(&path) {
            let mut content = String::new();
            if file.read_to_string(&mut content).is_ok() {
                let code = content.trim();
                return Language::from_code(code);
            }
        }
    }
    Language::En
}

fn save_language(lang: Language) {
    if let Some(dir) = config_dir() {
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join(CONFIG_FILE_NAME);
        if let Ok(mut file) = std::fs::File::create(&path) {
            let _ = file.write_all(lang.code().as_bytes());
        }
    }
}

fn load_colors() -> ColorPrefs {
    if let Some(dir) = config_dir() {
        let path = dir.join(COLORS_FILE_NAME);
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(prefs) = serde_json::from_str::<ColorPrefs>(&content) {
                return prefs;
            }
        }
    }
    ColorPrefs::default()
}

fn save_colors(prefs: &ColorPrefs) {
    if let Some(dir) = config_dir() {
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join(COLORS_FILE_NAME);
        if let Ok(content) = serde_json::to_string(prefs) {
            if let Ok(mut file) = std::fs::File::create(&path) {
                let _ = file.write_all(content.as_bytes());
            }
        }
    }
}

fn hex_to_rgb(hex: &str) -> (u8, u8, u8) {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return (152, 211, 153);
    }
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(152);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(211);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(153);
    (r, g, b)
}

#[tauri::command]
fn get_language(state: State<'_, AppState>) -> String {
    state.i18n.get().code().to_string()
}

#[tauri::command]
fn change_language(app: AppHandle, lang_code: String, state: State<'_, AppState>) -> String {
    let lang = Language::from_code(&lang_code);
    state.i18n.set(lang);
    save_language(lang);
    let _ = app.emit("language-changed", lang.code());
    lang.code().to_string()
}

#[tauri::command]
fn get_languages() -> Vec<(String, String)> {
    [Language::En, Language::It, Language::Es, Language::Fr]
        .iter()
        .map(|lang| (lang.code().to_string(), lang.display_name().to_string()))
        .collect()
}

#[tauri::command]
fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
fn t(key: String, state: State<'_, AppState>) -> String {
    if key.len() > 64 || key.chars().any(|c| !c.is_ascii_alphanumeric() && c != '_') {
        return String::new();
    }
    state.i18n.t(&key)
}

#[tauri::command]
fn get_colors(state: State<'_, AppState>) -> ColorPrefs {
    let colors = state.colors.lock();
    colors.clone()
}

#[tauri::command]
fn change_colors(colors: ColorPrefs, state: State<'_, AppState>) {
    save_colors(&colors);
    *state.colors.lock() = colors;
}

#[tauri::command]
fn get_bandwidth(state: State<'_, AppState>) -> ConnectionStats {
    state.latest_stats.lock().clone()
}

#[tauri::command]
fn toggle_floating_window(app: AppHandle) -> bool {
    if let Some(win) = app.get_webview_window("floating") {
        match win.is_visible() {
            Ok(true) => {
                let _ = win.hide();
                false
            }
            Ok(false) => {
                let _ = win.show();
                let _ = win.set_focus();
                true
            }
            Err(_) => false,
        }
    } else {
        false
    }
}

#[tauri::command]
fn show_main_window(app: AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.show();
        let _ = win.set_focus();
        position_window_near_tray(&win);
    }
}

#[tauri::command]
fn hide_main_window(app: AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.hide();
    }
}

#[tauri::command]
async fn speed_test(app: AppHandle) -> SpeedTestResult {
    let _ = app.emit("speed-test-start", ());
    let result = run_speed_test().await;
    let _ = app.emit("speed-test-done", &result);
    result
}

struct AppState {
    bandwidth: Arc<Mutex<BandwidthMonitor>>,
    ping: Arc<Mutex<PingMonitor>>,
    notifier: Arc<Mutex<Notifier>>,
    i18n: Arc<I18n>,
    colors: Arc<Mutex<ColorPrefs>>,
    latest_stats: Arc<Mutex<ConnectionStats>>,
    smoothed_quality: Arc<Mutex<Option<f64>>>,
    start_time: Instant,
}

fn position_window_near_tray(win: &tauri::WebviewWindow) {
    if let Ok(monitor) = win.current_monitor() {
        if let Some(monitor) = monitor {
            let screen_size = monitor.size();
            let scale = monitor.scale_factor();
            let x = (screen_size.width as f64 / scale) - 370.0;
            let _ = win.set_position(tauri::PhysicalPosition::new(x as i32, 28));
        }
    }
}

fn spawn_monitor_loop(app: AppHandle, state: Arc<AppState>) {
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;

            let stats = {
                let bw = state.bandwidth.lock();
                let res = bw.measure();
                let (download_mbps, upload_mbps) = bw.get_rolling_speed();
                let history = bw
                    .get_history()
                    .iter()
                    .map(|sample| (sample.timestamp, sample.download_mbps, sample.upload_mbps))
                    .collect();
                drop(bw);

                let ping_monitor = state.ping.lock();
                let ping_result = ping_monitor.compute_stats();
                let has_probe = ping_monitor.has_samples();
                drop(ping_monitor);

                let wifi = WifiMonitor::get_wifi_info();
                let connection_status = if !has_probe {
                    ConnectionStatus::Connecting
                } else if ping_result.latency_ms.is_some() {
                    ConnectionStatus::Online
                } else {
                    ConnectionStatus::Offline
                };
                let assessment = ConnectionStats::assess_quality(
                    ping_result.avg_latency,
                    ping_result.jitter_ms,
                    ping_result.packet_loss,
                    wifi.signal_dbm,
                );
                let (quality_score, quality_label_key, quality_issues) =
                    if connection_status == ConnectionStatus::Online {
                        let mut previous = state.smoothed_quality.lock();
                        let (score, smoothed) = stats::smooth_quality(*previous, assessment.score);
                        *previous = Some(smoothed);
                        (
                            score,
                            ConnectionStats::label_for_score(score).to_string(),
                            assessment.issues,
                        )
                    } else {
                        *state.smoothed_quality.lock() = None;
                        let label = if connection_status == ConnectionStatus::Connecting {
                            "quality_connecting"
                        } else {
                            "quality_disconnected"
                        };
                        (0, label.to_string(), Vec::new())
                    };

                ConnectionStats {
                    download_mbps,
                    upload_mbps,
                    ping_ms: ping_result.latency_ms.unwrap_or(0.0),
                    jitter_ms: ping_result.jitter_ms,
                    packet_loss: ping_result.packet_loss,
                    wifi_ssid: wifi.ssid,
                    wifi_signal: wifi.signal_dbm,
                    wifi_quality: wifi
                        .signal_dbm
                        .map(|signal| WifiMonitor::signal_quality(signal).to_string())
                        .unwrap_or_else(|| "N/A".to_string()),
                    dns_ms: None,
                    quality_score,
                    quality_score_i32: i32::from(quality_score),
                    quality_label_key,
                    quality_issues,
                    uptime_seconds: state.start_time.elapsed().as_secs(),
                    bandwidth_history: history,
                    is_connected: connection_status == ConnectionStatus::Online,
                    connection_status,
                    language: state.i18n.get().code().to_string(),
                    total_download_mb: res.total_download_mb,
                    total_upload_mb: res.total_upload_mb,
                }
            };

            *state.latest_stats.lock() = stats.clone();
            update_trays(&app, &stats, &state.colors.lock());
            let _ = app.emit("stats-update", &stats);
        }
    });
}

fn spawn_ping_loop(app: AppHandle, state: Arc<AppState>) {
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(4)).await;

            let lang = state.i18n.get();
            let ping_result = do_ping("1.1.1.1").await;

            let connected_changed;
            {
                let monitor = state.ping.clone();
                let ping_monitor = monitor.lock();
                let was_connected = ping_monitor.compute_stats().latency_ms.is_some();
                ping_monitor.record_ping(ping_result);
                let is_now_connected = ping_result.is_some();
                connected_changed = was_connected != is_now_connected;
            }

            let _ = app.emit("ping-update", ());

            if connected_changed {
                if let Some(notif) = state
                    .notifier
                    .lock()
                    .check_connection(ping_result.is_some(), lang)
                {
                    send_notification(&app, &notif);
                }
            }
        }
    });
}

fn spawn_quality_loop(app: AppHandle, state: Arc<AppState>) {
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(10)).await;

            let stats = state.latest_stats.lock().clone();
            if stats.connection_status == ConnectionStatus::Online {
                if let Some(notif) = state.notifier.lock().check_quality(
                    stats.quality_score_i32,
                    &stats.quality_label_key,
                    state.i18n.get(),
                ) {
                    send_notification(&app, &notif);
                }
            }
        }
    });
}

fn send_notification(app: &AppHandle, notif: &Notification) {
    use tauri_plugin_notification::NotificationExt;
    let _ = app
        .notification()
        .builder()
        .title(&notif.title)
        .body(&notif.body)
        .show();
}

fn spawn_update_check(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_secs(15)).await;
        let Ok(updater) = app.updater() else {
            return;
        };
        let Ok(Some(update)) = updater.check().await else {
            return;
        };
        let _ = app.emit("update-available", update.version.to_string());
        if update.download_and_install(|_, _| {}, || {}).await.is_ok() {
            app.restart();
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let lang = load_language();
    let i18n = Arc::new(I18n::new(lang));
    let colors = Arc::new(Mutex::new(load_colors()));

    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .setup({
            let i18n = i18n.clone();
            let colors = colors.clone();
            move |app| {
                let handle = app.handle().clone();

                let bandwidth = Arc::new(Mutex::new(BandwidthMonitor::new()));
                let ping = Arc::new(Mutex::new(PingMonitor::new()));
                let notifier = Arc::new(Mutex::new(Notifier::new()));
                let mut initial_stats = ConnectionStats::default();
                initial_stats.language = i18n.get().code().to_string();
                let latest_stats = Arc::new(Mutex::new(initial_stats));
                let smoothed_quality = Arc::new(Mutex::new(None));
                let start_time = Instant::now();

                app.manage(AppState {
                    bandwidth: bandwidth.clone(),
                    ping: ping.clone(),
                    notifier: notifier.clone(),
                    i18n: i18n.clone(),
                    colors: colors.clone(),
                    latest_stats: latest_stats.clone(),
                    smoothed_quality: smoothed_quality.clone(),
                    start_time,
                });

                let state_arc = Arc::new(AppState {
                    bandwidth,
                    ping,
                    notifier,
                    i18n,
                    colors,
                    latest_stats,
                    smoothed_quality,
                    start_time,
                });

                spawn_monitor_loop(handle.clone(), state_arc.clone());
                spawn_ping_loop(handle.clone(), state_arc.clone());
                spawn_quality_loop(handle.clone(), state_arc);
                spawn_update_check(handle.clone());

                build_tray(app)?;

                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.hide();
                }
                if let Some(win) = app.get_webview_window("floating") {
                    let _ = win.hide();
                }

                Ok(())
            }
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_bandwidth,
            toggle_floating_window,
            show_main_window,
            hide_main_window,
            speed_test,
            get_language,
            change_language,
            get_languages,
            get_app_version,
            t,
            get_colors,
            change_colors
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn build_tray(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let prefs = ColorPrefs::default();
    let indicators = [
        (tray::DOWNLOAD_ID, IndicatorKind::Download, "0K"),
        (tray::UPLOAD_ID, IndicatorKind::Upload, "0K"),
        (tray::QUALITY_ID, IndicatorKind::Quality(0), "…"),
        (tray::DATA_ID, IndicatorKind::Data, "0 MB"),
    ];

    for (id, kind, title) in indicators {
        let mut builder = tauri::tray::TrayIconBuilder::with_id(id)
            .tooltip("Connection Monitor")
            .title(title)
            .icon(indicator_icon(kind, &prefs));

        #[cfg(target_os = "macos")]
        {
            builder = builder.icon_as_template(false);
        }

        builder
            .on_tray_icon_event(|tray, event| {
                if let tauri::tray::TrayIconEvent::Click {
                    button: tauri::tray::MouseButton::Left,
                    button_state: tauri::tray::MouseButtonState::Up,
                    ..
                } = event
                {
                    let app = tray.app_handle();
                    if let Some(win) = app.get_webview_window("main") {
                        let is_visible = win.is_visible().unwrap_or(false);
                        if is_visible {
                            let _ = win.hide();
                        } else {
                            let _ = win.show();
                            let _ = win.set_focus();
                            position_window_near_tray(&win);
                        }
                    }
                }
            })
            .build(app)?;
    }

    Ok(())
}

fn update_trays(app: &AppHandle, stats: &ConnectionStats, prefs: &ColorPrefs) {
    let values = [
        (
            tray::DOWNLOAD_ID,
            IndicatorKind::Download,
            format_rate_compact(stats.download_mbps),
            format!("Download: {:.2} Mbps", stats.download_mbps),
        ),
        (
            tray::UPLOAD_ID,
            IndicatorKind::Upload,
            format_rate_compact(stats.upload_mbps),
            format!("Upload: {:.2} Mbps", stats.upload_mbps),
        ),
        (
            tray::QUALITY_ID,
            IndicatorKind::Quality(stats.quality_score),
            if stats.connection_status == ConnectionStatus::Online {
                stats.quality_score.to_string()
            } else {
                "—".to_string()
            },
            translate(
                Language::from_code(&stats.language),
                &stats.quality_label_key,
            ),
        ),
        (
            tray::DATA_ID,
            IndicatorKind::Data,
            format_data_compact(stats.total_download_mb + stats.total_upload_mb),
            "Dati trasferiti in questa sessione".to_string(),
        ),
    ];

    for (id, kind, title, tooltip) in values {
        if let Some(item) = app.tray_by_id(id) {
            let _ = item.set_icon(Some(indicator_icon(kind, prefs)));
            let _ = item.set_title(Some(title));
            let _ = item.set_tooltip(Some(tooltip));
        }
    }
}
