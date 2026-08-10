use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::image::Image as TauriImage;
use tauri::{AppHandle, Emitter, Manager, State};

mod i18n;
mod monitor;
mod stats;

use i18n::{translate, I18n, Language};
use monitor::bandwidth::BandwidthMonitor;
use monitor::notify::{Notification, Notifier};
use monitor::ping::{do_ping, PingMonitor};
use monitor::speedtest::{run_speed_test, SpeedTestResult};
use monitor::wifi::WifiMonitor;
use stats::ConnectionStats;

const CONFIG_DIR_NAME: &str = "connection-monitor";
const CONFIG_FILE_NAME: &str = "lang.conf";
const COLORS_FILE_NAME: &str = "colors.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorPrefs {
    pub download: String,
    pub upload: String,
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
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(152);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(211);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(153);
    (r, g, b)
}

fn is_connection_active(
    latency_ms: Option<f64>,
    packet_loss: f64,
    total_download_mb: f64,
    total_upload_mb: f64,
    has_wifi: bool,
    download_mbps: f64,
    upload_mbps: f64,
) -> bool {
    latency_ms.is_some()
        || packet_loss < 100.0
        || total_download_mb > 0.0
        || total_upload_mb > 0.0
        || has_wifi
        || download_mbps > 0.0
        || upload_mbps > 0.0
}

fn format_tray_title(
    download_mbps: f64,
    upload_mbps: f64,
    quality_score: u8,
    is_connected: bool,
) -> String {
    if !is_connected {
        return "OFFLINE".to_string();
    }

    let download = if download_mbps >= 1000.0 {
        format!("{:.1}k", download_mbps / 1000.0)
    } else {
        format!("{download_mbps:.1}")
    };
    let upload = if upload_mbps >= 1000.0 {
        format!("{:.1}k", upload_mbps / 1000.0)
    } else {
        format!("{upload_mbps:.1}")
    };

    format!("↓{download} ↑{upload} •{quality_score}")
}

fn generate_tray_icon(
    download_mbps: f64,
    upload_mbps: f64,
    quality_score: u8,
    prefs: &ColorPrefs,
) -> TauriImage<'static> {
    use image::codecs::png::PngEncoder;
    use image::ExtendedColorType;
    use image::ImageEncoder;

    let width: u32 = 24;
    let height: u32 = 24;
    let mut img = image::RgbaImage::new(width, height);

    let (dl_r, dl_g, dl_b) = if download_mbps > 0.0 {
        hex_to_rgb(&prefs.download)
    } else {
        (150, 150, 150)
    };

    let (ul_r, ul_g, ul_b) = if upload_mbps > 0.0 {
        hex_to_rgb(&prefs.upload)
    } else {
        (150, 150, 150)
    };

    let q_color = if quality_score >= 90 {
        hex_to_rgb(&prefs.quality_excellent)
    } else if quality_score >= 75 {
        hex_to_rgb(&prefs.quality_good)
    } else if quality_score >= 50 {
        hex_to_rgb(&prefs.quality_fair)
    } else if quality_score >= 25 {
        hex_to_rgb(&prefs.quality_poor)
    } else {
        hex_to_rgb(&prefs.quality_critical)
    };

    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let px = x as i32;
        let py = y as i32;

        if is_in_triangle(px, py, 5, 5, 10, 2, 17, 5) {
            pixel[0] = ul_r;
            pixel[1] = ul_g;
            pixel[2] = ul_b;
            pixel[3] = 255;
        } else if is_in_triangle(px, py, 7, 19, 17, 10, 13, 20) {
            pixel[0] = dl_r;
            pixel[1] = dl_g;
            pixel[2] = dl_b;
            pixel[3] = 255;
        } else if is_in_circle(px, py, 12, 12, 4) {
            let dist_sq = (px - 12) * (px - 12) + (py - 12) * (py - 12);
            if dist_sq <= 16 {
                pixel[0] = q_color.0;
                pixel[1] = q_color.1;
                pixel[2] = q_color.2;
                pixel[3] = 255;
            } else {
                pixel[3] = 0;
            }
        } else {
            pixel[3] = 0;
        }
    }

    let mut bytes: Vec<u8> = Vec::new();
    let writer = std::io::Cursor::new(&mut bytes);
    let encoder = PngEncoder::new(writer);
    let raw = img.as_raw();
    let ok = encoder
        .write_image(raw, width, height, ExtendedColorType::Rgba8)
        .is_ok();

    if ok {
        return match TauriImage::from_bytes(&bytes) {
            Ok(icon) => icon,
            Err(_) => load_default_icon(),
        };
    }
    load_default_icon()
}

fn load_default_icon() -> TauriImage<'static> {
    use image::codecs::png::PngEncoder;
    use image::ExtendedColorType;
    use image::ImageEncoder;

    let mut img = image::RgbaImage::new(24, 24);
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let dx = x as i32 - 12;
        let dy = y as i32 - 12;
        if dx * dx + dy * dy <= 100 {
            pixel[0] = 96;
            pixel[1] = 165;
            pixel[2] = 250;
            pixel[3] = 255;
        } else {
            pixel[3] = 0;
        }
    }

    let mut bytes: Vec<u8> = Vec::new();
    let writer = std::io::Cursor::new(&mut bytes);
    let encoder = PngEncoder::new(writer);
    if encoder
        .write_image(img.as_raw(), 24, 24, ExtendedColorType::Rgba8)
        .is_ok()
    {
        if let Ok(icon) = TauriImage::from_bytes(&bytes) {
            return icon;
        }
    }
    TauriImage::from_bytes(&[])
        .unwrap_or_else(|_| TauriImage::from_path("icons/32x32.png").unwrap())
}

fn is_in_triangle(px: i32, py: i32, x1: i32, y1: i32, x2: i32, y2: i32, x3: i32, y3: i32) -> bool {
    let d = (y2 - y3) * (x1 - x3) + (x3 - x2) * (y1 - y3);
    if d == 0 {
        return false;
    }
    let a = ((y2 - y3) as f64 * (px - x3) as f64 + (x3 - x2) as f64 * (py - y3) as f64) / d as f64;
    let b = ((y3 - y1) as f64 * (px - x3) as f64 + (x1 - x3) as f64 * (py - y3) as f64) / d as f64;
    let c = 1.0 - a - b;
    a >= 0.0 && b >= 0.0 && c >= 0.0
}

fn is_in_circle(px: i32, py: i32, _cx: i32, _cy: i32, radius: i32) -> bool {
    let dx = px - 15;
    let dy = py - 12;
    dx * dx + dy * dy <= radius * radius + radius
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
    let lang = state.i18n.get();

    let bw = state.bandwidth.lock();
    let (dl, ul) = bw.get_rolling_speed();
    let history = bw
        .get_history()
        .iter()
        .map(|s| (s.timestamp, s.download_mbps, s.upload_mbps))
        .collect();
    drop(bw);

    let ping_monitor = state.ping.lock();
    let ping_result = ping_monitor.compute_stats();
    drop(ping_monitor);

    let wifi = WifiMonitor::get_wifi_info();
    let quality = ConnectionStats::calculate_quality(
        ping_result.avg_latency,
        ping_result.jitter_ms,
        ping_result.packet_loss,
        wifi.signal_dbm,
    );

    let total_bytes_result = {
        let bw2 = state.bandwidth.lock();
        let res = bw2.measure();
        (res.total_download_mb, res.total_upload_mb)
    };

    let is_connected = is_connection_active(
        ping_result.latency_ms,
        ping_result.packet_loss,
        total_bytes_result.0,
        total_bytes_result.1,
        wifi.ssid.is_some(),
        dl,
        ul,
    );

    let elapsed = state.start_time.elapsed().as_secs();
    let wifi_quality_label = wifi
        .signal_dbm
        .map(|s| WifiMonitor::signal_quality(s).to_string())
        .unwrap_or_else(|| "N/A".to_string());

    ConnectionStats {
        download_mbps: dl,
        upload_mbps: ul,
        ping_ms: ping_result.avg_latency,
        jitter_ms: ping_result.jitter_ms,
        packet_loss: ping_result.packet_loss,
        wifi_ssid: wifi.ssid,
        wifi_signal: wifi.signal_dbm,
        wifi_quality: wifi_quality_label,
        dns_ms: None,
        quality_score: quality.0,
        quality_score_i32: quality.1,
        uptime_seconds: elapsed,
        bandwidth_history: history,
        is_connected,
        language: lang.code().to_string(),
        quality_label_key: quality.2,
        total_download_mb: total_bytes_result.0,
        total_upload_mb: total_bytes_result.1,
    }
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

            let (dl, ul, total_dl, total_ul, quality_score, is_connected, quality_label_key) = {
                let bw = state.bandwidth.lock();
                let (d, u) = bw.get_rolling_speed();
                let res = bw.measure();
                drop(bw);

                let ping_monitor = state.ping.lock();
                let ping_result = ping_monitor.compute_stats();
                drop(ping_monitor);

                let wifi = WifiMonitor::get_wifi_info();
                let quality = ConnectionStats::calculate_quality(
                    ping_result.avg_latency,
                    ping_result.jitter_ms,
                    ping_result.packet_loss,
                    wifi.signal_dbm,
                );

                let is_conn = is_connection_active(
                    ping_result.latency_ms,
                    ping_result.packet_loss,
                    res.total_download_mb,
                    res.total_upload_mb,
                    wifi.ssid.is_some(),
                    d,
                    u,
                );

                (
                    d,
                    u,
                    res.total_download_mb,
                    res.total_upload_mb,
                    quality.0,
                    is_conn,
                    quality.2,
                )
            };

            let _ = app.emit("stats-update", ());

            let colors = state.colors.lock();
            let icon = generate_tray_icon(dl, ul, quality_score, &colors);
            drop(colors);

            let tooltip = if !is_connected {
                translate(state.i18n.get(), "quality_disconnected")
            } else {
                let dl_val = if dl >= 1000.0 {
                    format!("{:.1}k", dl / 1000.0)
                } else {
                    format!("{:.1}", dl)
                };
                let ul_val = if ul >= 1000.0 {
                    format!("{:.1}k", ul / 1000.0)
                } else {
                    format!("{:.1}", ul)
                };
                format!(
                    "{} | \u{2193}{} \u{2191}{} MB/s | Score: {}",
                    translate(state.i18n.get(), "app_title"),
                    dl_val,
                    ul_val,
                    quality_score
                )
            };

            if let Some(tray) = app.tray_by_id("main-tray") {
                let _ = tray.set_icon(Some(icon));
                let _ =
                    tray.set_title(Some(format_tray_title(dl, ul, quality_score, is_connected)));
                let _ = tray.set_tooltip(Some(&tooltip));
            }

            let _ = app.emit(
                "tray-update",
                serde_json::json!({
                    "download": dl,
                    "upload": ul,
                    "totalDownloadMb": total_dl,
                    "totalUploadMb": total_ul,
                    "qualityScore": quality_score,
                    "isConnected": is_connected,
                    "qualityLabelKey": quality_label_key,
                }),
            );
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
                let was_connected = {
                    let stats = ping_monitor.compute_stats();
                    stats.avg_latency > 0.0 || stats.packet_loss < 100.0
                };
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

            let lang = state.i18n.get();
            let ping_monitor = state.ping.lock();
            let ping_result = ping_monitor.compute_stats();
            drop(ping_monitor);

            let wifi = WifiMonitor::get_wifi_info();
            let quality = ConnectionStats::calculate_quality(
                ping_result.avg_latency,
                ping_result.jitter_ms,
                ping_result.packet_loss,
                wifi.signal_dbm,
            );

            if let Some(notif) = state
                .notifier
                .lock()
                .check_quality(quality.1, &quality.2, lang)
            {
                send_notification(&app, &notif);
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let lang = load_language();
    let i18n = Arc::new(I18n::new(lang));
    let colors = Arc::new(Mutex::new(load_colors()));

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .setup({
            let i18n = i18n.clone();
            let colors = colors.clone();
            move |app| {
                let handle = app.handle().clone();

                let bandwidth = Arc::new(Mutex::new(BandwidthMonitor::new()));
                let ping = Arc::new(Mutex::new(PingMonitor::new()));
                let notifier = Arc::new(Mutex::new(Notifier::new()));

                app.manage(AppState {
                    bandwidth: bandwidth.clone(),
                    ping: ping.clone(),
                    notifier: notifier.clone(),
                    i18n: i18n.clone(),
                    colors: colors.clone(),
                    start_time: Instant::now(),
                });

                let start_time = app.state::<AppState>().start_time;
                let state_arc = Arc::new(AppState {
                    bandwidth,
                    ping,
                    notifier,
                    i18n,
                    colors,
                    start_time,
                });

                spawn_monitor_loop(handle.clone(), state_arc.clone());
                spawn_ping_loop(handle.clone(), state_arc.clone());
                spawn_quality_loop(handle.clone(), state_arc);

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
    let icon = generate_tray_icon(0.0, 0.0, 50, &ColorPrefs::default());

    let mut tray_builder = tauri::tray::TrayIconBuilder::with_id("main-tray")
        .tooltip("Connection Monitor")
        .title("…")
        .icon(icon);

    #[cfg(target_os = "macos")]
    {
        tray_builder = tray_builder.icon_as_template(false);
    }

    #[cfg(target_os = "macos")]
    let _tray = tray_builder
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

    #[cfg(not(target_os = "macos"))]
    let _tray = tray_builder.build(app)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{format_tray_title, is_connection_active};

    #[test]
    fn empty_first_sample_is_offline() {
        assert!(!is_connection_active(
            None, 100.0, 0.0, 0.0, false, 0.0, 0.0
        ));
    }

    #[test]
    fn successful_ping_is_online() {
        assert!(is_connection_active(
            Some(12.0),
            0.0,
            0.0,
            0.0,
            false,
            0.0,
            0.0
        ));
    }

    #[test]
    fn observed_traffic_is_online() {
        assert!(is_connection_active(None, 100.0, 0.0, 0.0, false, 0.1, 0.0));
    }

    #[test]
    fn connected_tray_title_is_readable() {
        assert_eq!(format_tray_title(0.0, 0.0, 50, true), "↓0.0 ↑0.0 •50");
    }

    #[test]
    fn offline_tray_title_is_explicit() {
        assert_eq!(format_tray_title(0.0, 0.0, 0, false), "OFFLINE");
    }
}
