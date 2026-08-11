use tauri::image::Image;

use crate::stats::{ConnectionStats, ConnectionStatus};
use crate::{hex_to_rgb, ColorPrefs};

pub const DOWNLOAD_ID: &str = "tray-download";
pub const UPLOAD_ID: &str = "tray-upload";
pub const QUALITY_ID: &str = "tray-quality";
pub const DATA_ID: &str = "tray-data";
pub const WINDOWS_STATUS_ID: &str = "tray-status";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrayMode {
    MultiIndicator,
    SingleStatus,
}

pub fn tray_mode_for_os(os: &str) -> TrayMode {
    if os == "windows" {
        TrayMode::SingleStatus
    } else {
        TrayMode::MultiIndicator
    }
}

#[derive(Clone, Copy)]
pub enum IndicatorKind {
    Download,
    Upload,
    Quality(u8),
    Data,
}

pub fn format_rate_compact(mbps: f64) -> String {
    if mbps >= 1_000.0 {
        format!("{:.2}G", mbps / 1_000.0)
    } else if mbps >= 1.0 {
        format!("{mbps:.1}M")
    } else {
        format!("{:.0}K", mbps * 1_000.0)
    }
}

pub fn format_data_compact(megabytes: f64) -> String {
    if megabytes >= 1_000_000.0 {
        format!("{:.2} TB", megabytes / 1_000_000.0)
    } else if megabytes >= 1_000.0 {
        format!("{:.2} GB", megabytes / 1_000.0)
    } else {
        format!("{megabytes:.0} MB")
    }
}

pub fn windows_tooltip(stats: &ConnectionStats) -> String {
    let quality = if stats.connection_status == ConnectionStatus::Online {
        format!("{}/100", stats.quality_score)
    } else {
        "—".to_string()
    };
    format!(
        "Connection Monitor\n↓ {}  ↑ {}  Qualità {}  Dati {}",
        format_rate_compact(stats.download_mbps),
        format_rate_compact(stats.upload_mbps),
        quality,
        format_data_compact(stats.total_download_mb + stats.total_upload_mb),
    )
}

fn color_for(kind: IndicatorKind, prefs: &ColorPrefs) -> (u8, u8, u8) {
    let value = match kind {
        IndicatorKind::Download => &prefs.download,
        IndicatorKind::Upload => &prefs.upload,
        IndicatorKind::Data => &prefs.data,
        IndicatorKind::Quality(score) if score >= 90 => &prefs.quality_excellent,
        IndicatorKind::Quality(score) if score >= 75 => &prefs.quality_good,
        IndicatorKind::Quality(score) if score >= 50 => &prefs.quality_fair,
        IndicatorKind::Quality(score) if score >= 25 => &prefs.quality_poor,
        IndicatorKind::Quality(_) => &prefs.quality_critical,
    };
    hex_to_rgb(value)
}

fn set_pixel(pixels: &mut [u8], x: i32, y: i32, color: (u8, u8, u8)) {
    if !(0..18).contains(&x) || !(0..18).contains(&y) {
        return;
    }
    let index = ((y * 18 + x) * 4) as usize;
    pixels[index] = color.0;
    pixels[index + 1] = color.1;
    pixels[index + 2] = color.2;
    pixels[index + 3] = 255;
}

fn draw_arrow(pixels: &mut [u8], up: bool, color: (u8, u8, u8)) {
    for y in 3..15 {
        for x in 8..10 {
            set_pixel(pixels, x, y, color);
        }
    }
    let tip_y = if up { 2 } else { 15 };
    for offset in 0..5 {
        let y = if up { tip_y + offset } else { tip_y - offset };
        for x in (8 - offset)..=(9 + offset) {
            set_pixel(pixels, x, y, color);
        }
    }
}

fn draw_quality(pixels: &mut [u8], color: (u8, u8, u8)) {
    for y in 1..17 {
        for x in 1..17 {
            let dx = x - 9;
            let dy = y - 9;
            let distance = dx * dx + dy * dy;
            if (36..=64).contains(&distance) {
                set_pixel(pixels, x, y, color);
            }
        }
    }
    for y in 7..11 {
        for x in 7..11 {
            set_pixel(pixels, x, y, color);
        }
    }
}

fn draw_data(pixels: &mut [u8], color: (u8, u8, u8)) {
    for y in 4..14 {
        for x in 3..15 {
            set_pixel(pixels, x, y, color);
        }
    }
    for x in 5..13 {
        set_pixel(pixels, x, 2, color);
        set_pixel(pixels, x, 15, color);
    }
    for x in 3..15 {
        set_pixel(pixels, x, 3, color);
        set_pixel(pixels, x, 14, color);
    }
    for x in 5..13 {
        set_pixel(pixels, x, 6, (255, 255, 255));
    }
}

pub fn indicator_icon(kind: IndicatorKind, prefs: &ColorPrefs) -> Image<'static> {
    let mut pixels = vec![0; 18 * 18 * 4];
    let color = color_for(kind, prefs);
    match kind {
        IndicatorKind::Download => draw_arrow(&mut pixels, false, color),
        IndicatorKind::Upload => draw_arrow(&mut pixels, true, color),
        IndicatorKind::Quality(_) => draw_quality(&mut pixels, color),
        IndicatorKind::Data => draw_data(&mut pixels, color),
    }
    Image::new_owned(pixels, 18, 18)
}

#[cfg(test)]
mod tests {
    use super::{
        format_data_compact, format_rate_compact, indicator_icon, tray_mode_for_os,
        windows_tooltip, IndicatorKind, TrayMode,
    };
    use crate::stats::{ConnectionStats, ConnectionStatus};
    use crate::ColorPrefs;

    #[test]
    fn formats_adaptive_rates() {
        assert_eq!(format_rate_compact(0.2), "200K");
        assert_eq!(format_rate_compact(1.7), "1.7M");
        assert_eq!(format_rate_compact(1_200.0), "1.20G");
    }

    #[test]
    fn formats_session_data() {
        assert_eq!(format_data_compact(500.0), "500 MB");
        assert_eq!(format_data_compact(1_500.0), "1.50 GB");
        assert_eq!(format_data_compact(1_500_000.0), "1.50 TB");
    }

    #[test]
    fn creates_small_colored_icons() {
        let icon = indicator_icon(IndicatorKind::Download, &ColorPrefs::default());
        assert_eq!((icon.width(), icon.height()), (18, 18));
        assert!(icon.rgba().chunks_exact(4).any(|pixel| pixel[3] > 0));
        assert!(icon
            .rgba()
            .chunks_exact(4)
            .any(|pixel| pixel[1] > pixel[0] && pixel[1] > pixel[2]));
    }

    #[test]
    fn windows_uses_one_status_tray() {
        assert_eq!(tray_mode_for_os("windows"), TrayMode::SingleStatus);
        assert_eq!(tray_mode_for_os("macos"), TrayMode::MultiIndicator);
    }

    #[test]
    fn windows_tooltip_contains_all_four_live_values() {
        let mut stats = ConnectionStats::default();
        stats.connection_status = ConnectionStatus::Online;
        stats.download_mbps = 1.2;
        stats.upload_mbps = 0.4;
        stats.quality_score = 87;
        stats.total_download_mb = 1_000.0;
        stats.total_upload_mb = 536.0;

        let tooltip = windows_tooltip(&stats);
        assert!(tooltip.contains("↓ 1.2M"));
        assert!(tooltip.contains("↑ 400K"));
        assert!(tooltip.contains("87/100"));
        assert!(tooltip.contains("1.54 GB"));
    }
}
