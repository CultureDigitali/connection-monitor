use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStats {
    pub download_mbps: f64,
    pub upload_mbps: f64,
    pub ping_ms: f64,
    pub jitter_ms: f64,
    pub packet_loss: f64,
    pub wifi_ssid: Option<String>,
    pub wifi_signal: Option<i32>,
    pub wifi_quality: String,
    pub dns_ms: Option<f64>,
    pub quality_score: u8,
    pub quality_score_i32: i32,
    pub quality_label_key: String,
    pub uptime_seconds: u64,
    pub bandwidth_history: Vec<(u64, f64, f64)>,
    pub is_connected: bool,
    pub language: String,
    pub total_download_mb: f64,
    pub total_upload_mb: f64,
}

impl ConnectionStats {
    pub fn calculate_quality(
        ping_ms: f64,
        jitter_ms: f64,
        packet_loss: f64,
        wifi_signal: Option<i32>,
    ) -> (u8, i32, String) {
        let mut score: i32 = 100;

        if ping_ms > 0.0 {
            if ping_ms < 20.0 {
                score -= 0;
            } else if ping_ms < 50.0 {
                score -= 10;
            } else if ping_ms < 100.0 {
                score -= 25;
            } else if ping_ms < 200.0 {
                score -= 40;
            } else {
                score -= 55;
            }
        }

        if jitter_ms < 5.0 {
            score -= 0;
        } else if jitter_ms < 15.0 {
            score -= 8;
        } else if jitter_ms < 30.0 {
            score -= 18;
        } else {
            score -= 30;
        }

        score -= ((packet_loss * 5.0) as i32).min(50);

        if let Some(dbm) = wifi_signal {
            if dbm >= -50 {
                score -= 0;
            } else if dbm >= -60 {
                score -= 5;
            } else if dbm >= -70 {
                score -= 15;
            } else if dbm >= -80 {
                score -= 25;
            } else {
                score -= 40;
            }
        }

        let score_u8 = score.clamp(0, 100) as u8;

        let label_key = match score_u8 {
            90..=100 => "quality_excellent".to_string(),
            75..=89 => "quality_good".to_string(),
            50..=74 => "quality_fair".to_string(),
            25..=49 => "quality_poor".to_string(),
            _ => "quality_critical".to_string(),
        };

        (score_u8, score, label_key)
    }
}
