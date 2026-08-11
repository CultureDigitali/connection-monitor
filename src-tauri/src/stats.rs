use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionStatus {
    Connecting,
    Online,
    Offline,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QualityIssue {
    pub key: String,
    pub severity: String,
    pub penalty: i32,
    pub value: f64,
    pub unit: String,
    pub recommendation_key: String,
}

#[derive(Debug, Clone)]
pub struct QualityAssessment {
    pub score: u8,
    pub issues: Vec<QualityIssue>,
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
    pub quality_issues: Vec<QualityIssue>,
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
            quality_issues: Vec::new(),
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
    fn issue(key: &str, penalty: i32, value: f64, unit: &str) -> QualityIssue {
        let severity = match penalty {
            40.. => "critical",
            25..=39 => "poor",
            10..=24 => "fair",
            _ => "minor",
        };
        QualityIssue {
            key: key.to_string(),
            severity: severity.to_string(),
            penalty,
            value,
            unit: unit.to_string(),
            recommendation_key: format!("recommendation_{key}"),
        }
    }

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

    pub fn assess_quality(
        ping_ms: f64,
        jitter_ms: f64,
        packet_loss: f64,
        wifi_signal: Option<i32>,
    ) -> QualityAssessment {
        let mut score: i32 = 100;
        let mut issues = Vec::new();

        let latency_penalty = if ping_ms <= 0.0 || ping_ms < 20.0 {
            0
        } else if ping_ms < 50.0 {
            10
        } else if ping_ms < 100.0 {
            25
        } else if ping_ms < 200.0 {
            40
        } else {
            55
        };
        score -= latency_penalty;
        if latency_penalty > 0 {
            issues.push(Self::issue("latency", latency_penalty, ping_ms, "ms"));
        }

        let jitter_penalty = if jitter_ms < 5.0 {
            0
        } else if jitter_ms < 15.0 {
            8
        } else if jitter_ms < 30.0 {
            18
        } else {
            30
        };
        score -= jitter_penalty;
        if jitter_penalty > 0 {
            issues.push(Self::issue("jitter", jitter_penalty, jitter_ms, "ms"));
        }

        let loss_penalty = ((packet_loss * 5.0) as i32).min(50);
        score -= loss_penalty;
        if loss_penalty > 0 {
            issues.push(Self::issue("packet_loss", loss_penalty, packet_loss, "%"));
        }

        if let Some(signal) = wifi_signal {
            let wifi_penalty = if signal >= -50 {
                0
            } else if signal >= -60 {
                5
            } else if signal >= -70 {
                15
            } else if signal >= -80 {
                25
            } else {
                40
            };
            score -= wifi_penalty;
            if wifi_penalty > 0 {
                issues.push(Self::issue("wifi", wifi_penalty, f64::from(signal), "dBm"));
            }
        }

        let score_u8 = score.clamp(0, 100) as u8;
        issues.sort_by(|left, right| right.penalty.cmp(&left.penalty));

        QualityAssessment {
            score: score_u8,
            issues,
        }
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

    #[test]
    fn quality_breakdown_reports_critical_latency_penalty() {
        let assessment = ConnectionStats::assess_quality(220.0, 0.0, 0.0, None);
        let issue = &assessment.issues[0];
        assert_eq!(issue.key, "latency");
        assert_eq!(issue.penalty, 55);
        assert_eq!(issue.severity, "critical");
    }

    #[test]
    fn quality_breakdown_reports_jitter_penalty() {
        let assessment = ConnectionStats::assess_quality(10.0, 35.0, 0.0, None);
        let issue = &assessment.issues[0];
        assert_eq!(issue.key, "jitter");
        assert_eq!(issue.penalty, 30);
    }

    #[test]
    fn quality_breakdown_reports_packet_loss_penalty() {
        let assessment = ConnectionStats::assess_quality(10.0, 0.0, 6.0, None);
        let issue = &assessment.issues[0];
        assert_eq!(issue.key, "packet_loss");
        assert_eq!(issue.penalty, 30);
        assert_eq!(issue.unit, "%");
    }

    #[test]
    fn quality_breakdown_reports_weak_wifi_penalty() {
        let assessment = ConnectionStats::assess_quality(10.0, 0.0, 0.0, Some(-82));
        let issue = &assessment.issues[0];
        assert_eq!(issue.key, "wifi");
        assert_eq!(issue.penalty, 40);
        assert_eq!(issue.value, -82.0);
    }

    #[test]
    fn quality_breakdown_is_empty_for_healthy_measurements() {
        let assessment = ConnectionStats::assess_quality(10.0, 2.0, 0.0, Some(-45));
        assert_eq!(assessment.score, 100);
        assert!(assessment.issues.is_empty());
    }

    #[test]
    fn quality_breakdown_orders_largest_penalty_first() {
        let assessment = ConnectionStats::assess_quality(120.0, 35.0, 2.0, Some(-82));
        let penalties: Vec<i32> = assessment
            .issues
            .iter()
            .map(|issue| issue.penalty)
            .collect();
        assert_eq!(penalties, vec![40, 40, 30, 10]);
    }
}
