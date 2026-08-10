use crate::i18n::{translate, Language};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub title: String,
    pub body: String,
    pub subtitle: Option<String>,
}

pub struct Notifier {
    last_quality: Option<i32>,
    last_connection_state: Option<bool>,
}

impl Notifier {
    pub fn new() -> Self {
        Self {
            last_quality: None,
            last_connection_state: None,
        }
    }

    pub fn check_quality(
        &mut self,
        quality_score: i32,
        quality_label_key: &str,
        lang: Language,
    ) -> Option<Notification> {
        if let Some(prev_score) = self.last_quality {
            let diff = quality_score - prev_score;
            if diff.abs() >= 2 {
                let is_better = diff > 0;
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
                self.last_quality = Some(quality_score);
                return Some(Notification {
                    title,
                    body,
                    subtitle: None,
                });
            }
        } else {
            self.last_quality = Some(quality_score);
        }

        None
    }

    pub fn check_connection(&mut self, connected: bool, lang: Language) -> Option<Notification> {
        match self.last_connection_state {
            Some(prev) => {
                if prev && !connected {
                    self.last_connection_state = Some(false);
                    return Some(Notification {
                        title: translate(lang, "notif_conn_lost_title"),
                        body: translate(lang, "notif_conn_lost_body"),
                        subtitle: None,
                    });
                }
                if !prev && connected {
                    self.last_connection_state = Some(true);
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
}
