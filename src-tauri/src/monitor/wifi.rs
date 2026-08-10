use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WifiInfo {
    pub ssid: Option<String>,
    pub signal_dbm: Option<i32>,
    pub channel: Option<u32>,
    pub noise_dbm: Option<i32>,
    pub transmit_rate: Option<f64>,
}

fn sanitize_ssid(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.len() > 64 {
        return None;
    }
    let sanitized: String = trimmed
        .chars()
        .filter(|&c| {
            (c.is_alphanumeric() || " .-_#()[]!".contains(c))
                && c != '\u{0000}'
        })
        .collect();
    if sanitized.is_empty() {
        None
    } else {
        Some(sanitized)
    }
}

pub struct WifiMonitor;

impl WifiMonitor {
    pub fn get_wifi_info() -> WifiInfo {
        #[cfg(target_os = "macos")]
        {
            let output = Command::new("/System/Library/PrivateFrameworks/Apple80211.framework/Versions/Current/Resources/airport")
                .arg("-I")
                .output();

            match output {
                Ok(result) if result.status.success() => {
                    let text = String::from_utf8_lossy(&result.stdout);
                    return parse_airport_output(&text);
                }
                _ => {}
            }

            let output2 = Command::new("networksetup")
                .args(["-getairportnetwork", "en0"])
                .output();

            match output2 {
                Ok(result) if result.status.success() => {
                    let text = String::from_utf8_lossy(&result.stdout);
                    let ssid = parse_networksetup_ssid(&text);
                    return WifiInfo {
                        ssid,
                        signal_dbm: None,
                        channel: None,
                        noise_dbm: None,
                        transmit_rate: None,
                    };
                }
                _ => {}
            }
        }

        WifiInfo {
            ssid: None,
            signal_dbm: None,
            channel: None,
            noise_dbm: None,
            transmit_rate: None,
        }
    }

    pub fn signal_quality(dbm: i32) -> &'static str {
        match dbm {
            dbm if dbm >= -50 => "Excellent",
            dbm if dbm >= -60 => "Good",
            dbm if dbm >= -70 => "Fair",
            dbm if dbm >= -80 => "Weak",
            _ => "Very Weak",
        }
    }
}

#[cfg(target_os = "macos")]
fn parse_airport_output(text: &str) -> WifiInfo {
    let mut ssid = None;
    let mut signal = None;
    let mut channel = None;
    let mut noise = None;
    let mut rate = None;

    for line in text.lines() {
        let line = line.trim();
        if let Some((key, val)) = line.split_once(':') {
            let key = key.trim();
            let val = val.trim();
            if key == "SSID" {
                ssid = sanitize_ssid(val);
            } else if key == "agrCtlRSSI" {
                signal = val.parse::<i32>().ok();
            } else if key == "agrCtlNoise" {
                noise = val.parse::<i32>().ok();
            } else if key == "channel" {
                channel = val.split(',').next().and_then(|c| c.parse::<u32>().ok());
            } else if key == "lastTxRate" {
                rate = val.parse::<f64>().ok();
            }
        }
    }

    WifiInfo {
        ssid,
        signal_dbm: signal,
        channel,
        noise_dbm: noise,
        transmit_rate: rate,
    }
}

#[cfg(target_os = "macos")]
fn parse_networksetup_ssid(text: &str) -> Option<String> {
    let text = text.trim();
    if text.contains("Current Wi-Fi Network:") || text.contains("You are not associated") {
        text.split(':').nth(1).and_then(|s| sanitize_ssid(s))
    } else {
        None
    }
}
