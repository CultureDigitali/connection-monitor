use serde::{Deserialize, Serialize};

use crate::stats::{ConnectionStats, ConnectionStatus, QualityIssue};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IncidentKind {
    Degraded,
    Offline,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Incident {
    pub id: String,
    pub kind: IncidentKind,
    pub started_at: i64,
    pub ended_at: Option<i64>,
    pub lowest_quality: Option<u8>,
    pub issue_key: Option<String>,
    pub value: Option<f64>,
    pub unit: Option<String>,
    pub recommendation_key: Option<String>,
    pub penalty: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GuardianTransition {
    None,
    Opened(Incident),
    Updated(Incident),
    Closed(Incident),
}

#[derive(Default)]
pub struct GuardianEngine {
    active: Option<Incident>,
    pending_critical_count: u8,
    pending_started_at: Option<i64>,
    recovery_count: u8,
}

impl GuardianEngine {
    pub fn new() -> Self {
        Self::default()
    }

    fn strongest_issue(stats: &ConnectionStats) -> Option<&QualityIssue> {
        stats.quality_issues.iter().max_by_key(|issue| issue.penalty)
    }

    fn degraded_incident(&self, timestamp: i64, stats: &ConnectionStats) -> Incident {
        let issue = Self::strongest_issue(stats);
        Incident {
            id: format!("incident-{timestamp}"),
            kind: IncidentKind::Degraded,
            started_at: self.pending_started_at.unwrap_or(timestamp),
            ended_at: None,
            lowest_quality: Some(stats.quality_score),
            issue_key: issue.map(|value| value.key.clone()),
            value: issue.map(|value| value.value),
            unit: issue.map(|value| value.unit.clone()),
            recommendation_key: issue.map(|value| value.recommendation_key.clone()),
            penalty: issue.map(|value| value.penalty).unwrap_or(0),
        }
    }

    fn offline_incident(timestamp: i64) -> Incident {
        Incident {
            id: format!("incident-{timestamp}"),
            kind: IncidentKind::Offline,
            started_at: timestamp,
            ended_at: None,
            lowest_quality: None,
            issue_key: Some("offline".to_string()),
            value: None,
            unit: None,
            recommendation_key: Some("recommendation_offline".to_string()),
            penalty: 100,
        }
    }

    fn update_evidence(active: &mut Incident, stats: &ConnectionStats) -> bool {
        let mut changed = false;
        if active
            .lowest_quality
            .map(|quality| stats.quality_score < quality)
            .unwrap_or(true)
        {
            active.lowest_quality = Some(stats.quality_score);
            changed = true;
        }
        if let Some(issue) = Self::strongest_issue(stats) {
            if issue.penalty > active.penalty {
                active.issue_key = Some(issue.key.clone());
                active.value = Some(issue.value);
                active.unit = Some(issue.unit.clone());
                active.recommendation_key = Some(issue.recommendation_key.clone());
                active.penalty = issue.penalty;
                changed = true;
            }
        }
        changed
    }

    pub fn evaluate(&mut self, timestamp: i64, stats: &ConnectionStats) -> GuardianTransition {
        if stats.connection_status == ConnectionStatus::Offline {
            self.pending_critical_count = 0;
            self.pending_started_at = None;
            self.recovery_count = 0;
            if let Some(active) = self.active.as_mut() {
                if active.kind != IncidentKind::Offline {
                    active.kind = IncidentKind::Offline;
                    active.issue_key = Some("offline".to_string());
                    active.value = None;
                    active.unit = None;
                    active.recommendation_key = Some("recommendation_offline".to_string());
                    active.penalty = 100;
                    return GuardianTransition::Updated(active.clone());
                }
                return GuardianTransition::None;
            }
            let incident = Self::offline_incident(timestamp);
            self.active = Some(incident.clone());
            return GuardianTransition::Opened(incident);
        }

        let critical = stats.connection_status == ConnectionStatus::Online && stats.quality_score < 25;
        let healthy = stats.connection_status == ConnectionStatus::Online && stats.quality_score >= 50;

        if critical {
            self.recovery_count = 0;
            if let Some(active) = self.active.as_mut() {
                let changed = Self::update_evidence(active, stats);
                return if changed {
                    GuardianTransition::Updated(active.clone())
                } else {
                    GuardianTransition::None
                };
            }
            if self.pending_critical_count == 0 {
                self.pending_started_at = Some(timestamp);
            }
            self.pending_critical_count += 1;
            if self.pending_critical_count >= 3 {
                let incident = self.degraded_incident(timestamp, stats);
                self.active = Some(incident.clone());
                self.pending_critical_count = 0;
                self.pending_started_at = None;
                return GuardianTransition::Opened(incident);
            }
            return GuardianTransition::None;
        }

        self.pending_critical_count = 0;
        self.pending_started_at = None;
        if self.active.is_some() && healthy {
            self.recovery_count += 1;
            if self.recovery_count >= 3 {
                self.recovery_count = 0;
                let mut closed = self.active.take().expect("active incident checked");
                closed.ended_at = Some(timestamp);
                return GuardianTransition::Closed(closed);
            }
        } else {
            self.recovery_count = 0;
        }
        GuardianTransition::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::{ConnectionStats, ConnectionStatus, QualityIssue};

    fn issue(key: &str, penalty: i32) -> QualityIssue {
        QualityIssue {
            key: key.to_string(),
            severity: "critical".to_string(),
            penalty,
            value: penalty as f64,
            unit: "ms".to_string(),
            recommendation_key: format!("recommendation_{key}"),
        }
    }

    fn online_stats(score: u8, issues: Vec<QualityIssue>) -> ConnectionStats {
        ConnectionStats {
            connection_status: ConnectionStatus::Online,
            is_connected: true,
            quality_score: score,
            quality_score_i32: i32::from(score),
            quality_issues: issues,
            ..ConnectionStats::default()
        }
    }

    fn critical_stats() -> ConnectionStats {
        online_stats(20, vec![issue("latency", 40)])
    }

    fn healthy_stats() -> ConnectionStats {
        online_stats(80, Vec::new())
    }

    fn offline_stats() -> ConnectionStats {
        ConnectionStats {
            connection_status: ConnectionStatus::Offline,
            ..ConnectionStats::default()
        }
    }

    #[test]
    fn degradation_opens_and_closes_only_after_three_consecutive_samples() {
        let mut guardian = GuardianEngine::new();
        assert!(matches!(guardian.evaluate(1, &critical_stats()), GuardianTransition::None));
        assert!(matches!(guardian.evaluate(2, &critical_stats()), GuardianTransition::None));
        assert!(matches!(guardian.evaluate(3, &critical_stats()), GuardianTransition::Opened(_)));
        assert!(matches!(guardian.evaluate(4, &healthy_stats()), GuardianTransition::None));
        assert!(matches!(guardian.evaluate(5, &healthy_stats()), GuardianTransition::None));
        let GuardianTransition::Closed(closed) = guardian.evaluate(6, &healthy_stats()) else {
            panic!("third healthy sample must close the incident");
        };
        assert_eq!(closed.ended_at, Some(6));
        assert_eq!(closed.kind, IncidentKind::Degraded);
    }

    #[test]
    fn offline_opens_immediately_without_duplicate_events() {
        let mut guardian = GuardianEngine::new();
        let GuardianTransition::Opened(opened) = guardian.evaluate(10, &offline_stats()) else {
            panic!("offline must open immediately");
        };
        assert_eq!(opened.kind, IncidentKind::Offline);
        assert!(matches!(guardian.evaluate(11, &offline_stats()), GuardianTransition::None));
    }

    #[test]
    fn offline_upgrades_an_active_degradation_instead_of_duplicating_it() {
        let mut guardian = GuardianEngine::new();
        guardian.evaluate(20, &critical_stats());
        guardian.evaluate(21, &critical_stats());
        let GuardianTransition::Opened(degraded) = guardian.evaluate(22, &critical_stats()) else {
            panic!("degradation should be open");
        };
        let GuardianTransition::Updated(upgraded) = guardian.evaluate(23, &offline_stats()) else {
            panic!("offline must update the current incident");
        };
        assert_eq!(upgraded.id, degraded.id);
        assert_eq!(upgraded.started_at, 20);
        assert_eq!(upgraded.kind, IncidentKind::Offline);
    }

    #[test]
    fn active_incident_tracks_the_lowest_score_and_strongest_evidence() {
        let mut guardian = GuardianEngine::new();
        guardian.evaluate(30, &critical_stats());
        guardian.evaluate(31, &critical_stats());
        guardian.evaluate(32, &critical_stats());
        let worse = online_stats(8, vec![issue("packet_loss", 50)]);
        let GuardianTransition::Updated(updated) = guardian.evaluate(33, &worse) else {
            panic!("stronger evidence must update the incident");
        };
        assert_eq!(updated.lowest_quality, Some(8));
        assert_eq!(updated.issue_key.as_deref(), Some("packet_loss"));
        assert_eq!(updated.recommendation_key.as_deref(), Some("recommendation_packet_loss"));
    }
}
