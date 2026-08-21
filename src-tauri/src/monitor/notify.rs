use crate::i18n::{translate, Language};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub title: String,
    pub body: String,
    pub subtitle: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NotificationPrefs {
    pub enabled: bool,
    pub connection_enabled: bool,
    pub quality_enabled: bool,
    pub quality_cooldown_secs: u64,
    pub connection_cooldown_secs: u64,
    pub quality_threshold: i32,
}

impl Default for NotificationPrefs {
    fn default() -> Self {
        Self {
            enabled: true,
            connection_enabled: true,
            quality_enabled: false,
            quality_cooldown_secs: 900,
            connection_cooldown_secs: 300,
            quality_threshold: 10,
        }
    }
}

pub struct Notifier {
    last_quality: Option<i32>,
    last_connection_state: Option<bool>,
    last_quality_notification: Option<Instant>,
    last_connection_notification: Option<Instant>,
}

impl Notifier {
    pub fn new() -> Self {
        Self {
            last_quality: None,
            last_connection_state: None,
            last_quality_notification: None,
            last_connection_notification: None,
        }
    }

    pub fn check_quality(
        &mut self,
        quality_score: i32,
        quality_label_key: &str,
        lang: Language,
        prefs: &NotificationPrefs,
    ) -> Option<Notification> {
        let prev_score = self.last_quality;
        if prev_score.is_none() {
            self.last_quality = Some(quality_score);
            return None;
        }
        let prev = prev_score.unwrap();
        self.last_quality = Some(quality_score);

        if !prefs.enabled || !prefs.quality_enabled {
            return None;
        }

        let diff = (quality_score - prev).abs();
        if diff < prefs.quality_threshold.max(1) {
            return None;
        }

        if let Some(last) = self.last_quality_notification {
            if last.elapsed() < Duration::from_secs(prefs.quality_cooldown_secs) {
                return None;
            }
        }

        let is_better = quality_score > prev;
        let label = translate(lang, quality_label_key);
        let (title, body) = if is_better {
            let t = translate(lang, "notif_improved_title");
            let b = translate(lang, "notif_quality_body");
            (t, format!("{} {}", b, label))
        } else {
            let t = translate(lang, "notif_degraded_title");
            let b = translate(lang, "notif_quality_body");
            (t, format!("{} {}", b, label))
        };
        self.last_quality_notification = Some(Instant::now());
        Some(Notification {
            title,
            body,
            subtitle: None,
        })
    }

    pub fn check_connection(
        &mut self,
        connected: bool,
        lang: Language,
        prefs: &NotificationPrefs,
    ) -> Option<Notification> {
        match self.last_connection_state {
            Some(prev) => {
                if prev == connected {
                    return None;
                }
                self.last_connection_state = Some(connected);

                if !prefs.enabled || !prefs.connection_enabled {
                    return None;
                }

                if let Some(last) = self.last_connection_notification {
                    if last.elapsed() < Duration::from_secs(prefs.connection_cooldown_secs) {
                        return None;
                    }
                }

                self.last_connection_notification = Some(Instant::now());
                if prev && !connected {
                    return Some(Notification {
                        title: translate(lang, "notif_conn_lost_title"),
                        body: translate(lang, "notif_conn_lost_body"),
                        subtitle: None,
                    });
                }
                if !prev && connected {
                    return Some(Notification {
                        title: translate(lang, "notif_conn_restored_title"),
                        body: translate(lang, "notif_conn_restored_body"),
                        subtitle: None,
                    });
                }
            }
            None => {
                self.last_connection_state = Some(connected);
            }
        }
        None
    }

    #[cfg(test)]
    pub fn set_last_quality_notification(&mut self, instant: Instant) {
        self.last_quality_notification = Some(instant);
    }

    #[cfg(test)]
    pub fn set_last_connection_notification(&mut self, instant: Instant) {
        self.last_connection_notification = Some(instant);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::Language;
    use std::time::{Duration, Instant};

    fn prefs_all_on() -> NotificationPrefs {
        NotificationPrefs {
            enabled: true,
            connection_enabled: true,
            quality_enabled: true,
            quality_cooldown_secs: 0,
            connection_cooldown_secs: 0,
            quality_threshold: 2,
        }
    }

    #[test]
    fn quality_disabled_by_default_prevents_spam() {
        let mut n = Notifier::new();
        let prefs = NotificationPrefs::default();
        assert!(!prefs.quality_enabled);
        assert!(n
            .check_quality(80, "quality_good", Language::En, &prefs)
            .is_none());
        assert!(n
            .check_quality(40, "quality_poor", Language::En, &prefs)
            .is_none());
    }

    #[test]
    fn quality_notifies_only_when_threshold_crossed() {
        let mut n = Notifier::new();
        let prefs = prefs_all_on();
        assert!(n
            .check_quality(80, "quality_good", Language::En, &prefs)
            .is_none());
        // diff 1 < threshold 2 -> no notification
        assert!(n
            .check_quality(81, "quality_good", Language::En, &prefs)
            .is_none());
        // diff 10 >=2 -> notify
        assert!(n
            .check_quality(70, "quality_fair", Language::En, &prefs)
            .is_some());
    }

    #[test]
    fn quality_cooldown_blocks_second_notification() {
        let mut n = Notifier::new();
        let mut prefs = prefs_all_on();
        prefs.quality_cooldown_secs = 900;
        assert!(n
            .check_quality(80, "quality_good", Language::En, &prefs)
            .is_none());
        let first = n.check_quality(60, "quality_fair", Language::En, &prefs);
        assert!(first.is_some());
        // second within cooldown -> blocked even though diff large
        let second = n.check_quality(40, "quality_poor", Language::En, &prefs);
        assert!(second.is_none());
    }

    #[test]
    fn quality_cooldown_expires_allows_again() {
        let mut n = Notifier::new();
        let mut prefs = prefs_all_on();
        prefs.quality_cooldown_secs = 1;
        n.check_quality(80, "quality_good", Language::En, &prefs);
        n.check_quality(60, "quality_fair", Language::En, &prefs);
        // simulate 2 secs ago
        n.set_last_quality_notification(Instant::now() - Duration::from_secs(2));
        assert!(n
            .check_quality(40, "quality_poor", Language::En, &prefs)
            .is_some());
    }

    #[test]
    fn master_switch_disables_all_notifications() {
        let mut n = Notifier::new();
        let mut prefs = prefs_all_on();
        prefs.enabled = false;
        n.check_quality(80, "quality_good", Language::En, &prefs);
        assert!(n
            .check_quality(60, "quality_fair", Language::En, &prefs)
            .is_none());

        let mut n2 = Notifier::new();
        n2.check_connection(true, Language::En, &prefs_all_on());
        // need to prime connection state first
        let mut prefs_off = prefs_all_on();
        prefs_off.enabled = false;
        assert!(n2
            .check_connection(false, Language::En, &prefs_off)
            .is_none());
    }

    #[test]
    fn connection_cooldown_blocks_flapping() {
        let mut n = Notifier::new();
        let mut prefs = prefs_all_on();
        prefs.connection_cooldown_secs = 900;
        n.check_connection(true, Language::En, &prefs);
        assert!(n
            .check_connection(false, Language::En, &prefs)
            .is_some());
        n.set_last_connection_notification(Instant::now());
        // flapping back quickly should be blocked
        assert!(n
            .check_connection(true, Language::En, &prefs)
            .is_none());
    }

    #[test]
    fn connection_toggle_disables_only_connection() {
        let mut n = Notifier::new();
        let mut prefs = prefs_all_on();
        prefs.connection_enabled = false;
        prefs.quality_enabled = true;
        n.check_connection(true, Language::En, &prefs);
        assert!(n
            .check_connection(false, Language::En, &prefs)
            .is_none());
        // quality still works when connection disabled
        n.check_quality(80, "quality_good", Language::En, &prefs);
        assert!(n
            .check_quality(60, "quality_fair", Language::En, &prefs)
            .is_some());
    }

    #[test]
    fn prefs_default_serializes() {
        let prefs = NotificationPrefs::default();
        let json = serde_json::to_string(&prefs).unwrap();
        let restored: NotificationPrefs = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.enabled, true);
        assert_eq!(restored.quality_enabled, false);
    }
}
