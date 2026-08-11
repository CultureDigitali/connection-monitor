use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Write};
use std::path::PathBuf;

use super::guardian::{GuardianTransition, Incident};
use super::model::MinuteBucket;

const SCHEMA_VERSION: u32 = 1;
const RETENTION_SECONDS: i64 = 30 * 86_400;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoryDocument {
    pub schema_version: u32,
    pub buckets: Vec<MinuteBucket>,
    pub incidents: Vec<Incident>,
}

impl Default for HistoryDocument {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            buckets: Vec::new(),
            incidents: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoryRange {
    pub buckets: Vec<MinuteBucket>,
    pub incidents: Vec<Incident>,
}

pub struct HistoryStore {
    path: PathBuf,
    document: HistoryDocument,
}

impl HistoryStore {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            document: HistoryDocument::default(),
        }
    }

    pub fn load(path: PathBuf, now: i64) -> Result<Self, String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        if !path.exists() {
            return Ok(Self::new(path));
        }

        let parsed = File::open(&path)
            .map(BufReader::new)
            .map_err(|error| error.to_string())
            .and_then(|reader| {
                serde_json::from_reader::<_, HistoryDocument>(reader)
                    .map_err(|error| error.to_string())
            });
        match parsed {
            Ok(mut document) if document.schema_version == SCHEMA_VERSION => {
                Self::prune_document(&mut document, now);
                Ok(Self { path, document })
            }
            Ok(_) | Err(_) => {
                let backup = path
                    .parent()
                    .unwrap_or_else(|| std::path::Path::new("."))
                    .join(format!("history.corrupt-{now}.json"));
                std::fs::rename(&path, backup).map_err(|error| error.to_string())?;
                Ok(Self::new(path))
            }
        }
    }

    pub fn document(&self) -> &HistoryDocument {
        &self.document
    }

    fn prune_document(document: &mut HistoryDocument, now: i64) {
        let cutoff = now - RETENTION_SECONDS;
        document
            .buckets
            .retain(|bucket| bucket.started_at >= cutoff);
        document
            .incidents
            .retain(|incident| incident.ended_at.unwrap_or(incident.started_at) >= cutoff);
    }

    pub fn append_bucket(&mut self, bucket: MinuteBucket, now: i64) {
        self.document.buckets.push(bucket);
        self.document
            .buckets
            .sort_by_key(|bucket| bucket.started_at);
        Self::prune_document(&mut self.document, now);
    }

    pub fn apply_transition(&mut self, transition: GuardianTransition, now: i64) {
        let incident = match transition {
            GuardianTransition::None => return,
            GuardianTransition::Opened(incident)
            | GuardianTransition::Updated(incident)
            | GuardianTransition::Closed(incident) => incident,
        };
        if let Some(existing) = self
            .document
            .incidents
            .iter_mut()
            .find(|existing| existing.id == incident.id)
        {
            *existing = incident;
        } else {
            self.document.incidents.push(incident);
        }
        self.document
            .incidents
            .sort_by_key(|incident| incident.started_at);
        Self::prune_document(&mut self.document, now);
    }

    pub fn query_range(&self, start: i64, end: i64) -> HistoryRange {
        HistoryRange {
            buckets: self
                .document
                .buckets
                .iter()
                .filter(|bucket| bucket.started_at >= start && bucket.started_at <= end)
                .cloned()
                .collect(),
            incidents: self
                .document
                .incidents
                .iter()
                .filter(|incident| {
                    incident.started_at <= end && incident.ended_at.unwrap_or(end) >= start
                })
                .cloned()
                .collect(),
        }
    }

    pub fn save_atomic(&self) -> Result<(), String> {
        let temporary = self.path.with_extension("json.tmp");
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&temporary)
            .map_err(|error| error.to_string())?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer(&mut writer, &self.document).map_err(|error| error.to_string())?;
        writer.flush().map_err(|error| error.to_string())?;
        writer
            .get_ref()
            .sync_all()
            .map_err(|error| error.to_string())?;
        std::fs::rename(&temporary, &self.path).map_err(|error| error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::guardian::{GuardianTransition, Incident, IncidentKind};
    use crate::history::model::MinuteBucket;
    use std::time::{SystemTime, UNIX_EPOCH};

    const DAY: i64 = 86_400;

    fn test_path(name: &str) -> std::path::PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir()
            .join(format!(
                "connection-monitor-{name}-{}-{nonce}",
                std::process::id()
            ))
            .join("history.json")
    }

    fn bucket_at(timestamp: i64) -> MinuteBucket {
        MinuteBucket {
            started_at: timestamp,
            sample_count: 60,
            availability: 1.0,
            average_quality: Some(90.0),
            minimum_quality: Some(85),
            average_download_mbps: Some(5.0),
            average_upload_mbps: Some(1.0),
            average_ping_ms: Some(20.0),
            average_jitter_ms: Some(2.0),
            average_packet_loss: Some(0.0),
            average_wifi_signal: Some(-50.0),
            downloaded_mb: 2.0,
            uploaded_mb: 0.5,
        }
    }

    fn incident(id: &str, started_at: i64, ended_at: Option<i64>) -> Incident {
        Incident {
            id: id.to_string(),
            kind: IncidentKind::Degraded,
            started_at,
            ended_at,
            lowest_quality: Some(10),
            issue_key: Some("latency".to_string()),
            value: Some(200.0),
            unit: Some("ms".to_string()),
            recommendation_key: Some("recommendation_latency".to_string()),
            penalty: 55,
        }
    }

    #[test]
    fn round_trip_prunes_data_older_than_thirty_days() {
        let now = 2_000_000_000;
        let path = test_path("round-trip");
        let mut store = HistoryStore::load(path.clone(), now).unwrap();
        store.append_bucket(bucket_at(now - 31 * DAY), now);
        store.append_bucket(bucket_at(now), now);
        store.save_atomic().unwrap();

        let restored = HistoryStore::load(path.clone(), now).unwrap();
        assert_eq!(restored.document().schema_version, 1);
        assert_eq!(restored.document().buckets, vec![bucket_at(now)]);
        assert!(!path.with_extension("json.tmp").exists());
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn incident_updates_replace_the_same_record() {
        let now = 2_000_000_000;
        let path = test_path("incident");
        let mut store = HistoryStore::load(path.clone(), now).unwrap();
        let opened = incident("same", now - 10, None);
        store.apply_transition(GuardianTransition::Opened(opened.clone()), now);
        let mut closed = opened;
        closed.ended_at = Some(now);
        store.apply_transition(GuardianTransition::Closed(closed.clone()), now);

        assert_eq!(store.document().incidents, vec![closed]);
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn range_query_filters_buckets_and_overlapping_incidents() {
        let now = 2_000_000_000;
        let path = test_path("query");
        let mut store = HistoryStore::load(path.clone(), now).unwrap();
        store.append_bucket(bucket_at(now - 200), now);
        store.append_bucket(bucket_at(now - 60), now);
        store.apply_transition(
            GuardianTransition::Opened(incident("overlap", now - 90, None)),
            now,
        );

        let result = store.query_range(now - 120, now);
        assert_eq!(result.buckets, vec![bucket_at(now - 60)]);
        assert_eq!(result.incidents.len(), 1);
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn malformed_json_is_preserved_before_starting_clean() {
        let now = 2_000_000_000;
        let path = test_path("corrupt");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, b"{not-json").unwrap();

        let store = HistoryStore::load(path.clone(), now).unwrap();
        assert!(store.document().buckets.is_empty());
        assert!(path
            .parent()
            .unwrap()
            .join(format!("history.corrupt-{now}.json"))
            .exists());
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
}
