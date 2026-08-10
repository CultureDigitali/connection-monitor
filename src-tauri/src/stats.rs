use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionStatus {
    Connecting,
    Online,
    Offline,
}

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
    pub connection_status: ConnectionStatus,
    pub language: String,
    pub total_download_mb: f64,
    pub total_upload_mb: f64,
}

impl Default for ConnectionStats {
    fn default() -> Self {
        Self {
            download_mbps: 0.0,
            upload_mbps: 0.0,
            ping_ms: 0.0,
            jitter_ms: 0.0,
            packet_loss: 0.0,
            wifi_ssid: None,
            wifi_signal: None,
            wifi_quality: "N/A".to_string(),
            dns_ms: None,
            quality_score: 0,
            quality_score_i32: 0,
            quality_label_key: "quality_connecting".to_string(),
            uptime_seconds: 0,
            bandwidth_history: Vec::new(),
            is_connected: false,
            connection_status: ConnectionStatus::Connecting,
            language: "en".to_string(),
            total_download_mb: 0.0,
            total_upload_mb: 0.0,
        }
    }
}

pub fn smooth_quality(previous: Option<f64>, raw: u8) -> (u8, f64) {
    let smoothed = previous
        .map(|value| value * 0.75 + f64::from(raw) * 0.25)
        .unwrap_or(f64::from(raw));
    (smoothed.round().clamp(0.0, 100.0) as u8, smoothed)
}

impl ConnectionStats {
    pub fn label_for_score(score: u8) -> &'static str {
        match score {
            90..=100 => "quality_excellent",
            75..=89 => "quality_good",
            50..=74 => "quality_fair",
            25..=49 => "quality_poor",
            _ => "quality_critical",
        }
    }

    #[cfg(test)]
    pub fn offline(uptime_seconds: u64) -> Self {
        Self {
            connection_status: ConnectionStatus::Offline,
            quality_label_key: "quality_disconnected".to_string(),
            uptime_seconds,
            packet_loss: 100.0,
            ..Self::default()
        }
    }

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

        let label_key = Self::label_for_score(score_u8).to_string();

        (score_u8, score, label_key)
    }
}

#[cfg(test)]
mod tests {
    use super::{smooth_quality, ConnectionStats, ConnectionStatus};

    #[test]
    fn connection_status_serializes_for_the_frontend() {
        assert_eq!(
            serde_json::to_string(&ConnectionStatus::Connecting).unwrap(),
            "\"connecting\""
        );
        assert_eq!(
            serde_json::to_string(&ConnectionStatus::Online).unwrap(),
            "\"online\""
        );
        assert_eq!(
            serde_json::to_string(&ConnectionStatus::Offline).unwrap(),
            "\"offline\""
        );
    }

    #[test]
    fn initial_snapshot_is_connecting() {
        let stats = ConnectionStats::default();
        assert_eq!(stats.connection_status, ConnectionStatus::Connecting);
        assert!(!stats.is_connected);
    }

    #[test]
    fn first_quality_sample_is_not_delayed() {
        assert_eq!(smooth_quality(None, 80), (80, 80.0));
    }

    #[test]
    fn later_quality_samples_use_quarter_weight() {
        assert_eq!(smooth_quality(Some(80.0), 40), (70, 70.0));
    }

    #[test]
    fn offline_snapshot_has_zero_quality() {
        let stats = ConnectionStats::offline(12);
        assert_eq!(stats.connection_status, ConnectionStatus::Offline);
        assert_eq!(stats.quality_score, 0);
        assert_eq!(stats.uptime_seconds, 12);
    }

    #[test]
    fn smoothed_score_selects_the_display_label() {
        assert_eq!(ConnectionStats::label_for_score(90), "quality_excellent");
        assert_eq!(ConnectionStats::label_for_score(75), "quality_good");
        assert_eq!(ConnectionStats::label_for_score(50), "quality_fair");
        assert_eq!(ConnectionStats::label_for_score(25), "quality_poor");
        assert_eq!(ConnectionStats::label_for_score(24), "quality_critical");
    }
}
