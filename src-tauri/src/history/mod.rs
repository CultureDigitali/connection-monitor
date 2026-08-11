pub mod guardian;
pub mod model;
pub mod recorder;
pub mod store;

use chrono::{Duration, Local, NaiveDate};
use serde::{Deserialize, Serialize};

use guardian::Incident;
use model::{HistorySummary, MinuteBucket, StreakSummary};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryResponse {
    pub range: String,
    pub started_at: i64,
    pub ended_at: i64,
    pub summary: HistorySummary,
    pub buckets: Vec<MinuteBucket>,
    pub incidents: Vec<Incident>,
    pub streak: StreakSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayResponse {
    pub day: String,
    pub started_at: i64,
    pub ended_at: i64,
    pub buckets: Vec<MinuteBucket>,
    pub incidents: Vec<Incident>,
}

pub fn range_start(range: &str, now: i64) -> Option<i64> {
    match range {
        "24h" => Some(now - 86_400),
        "7d" => Some(now - 7 * 86_400),
        "30d" => Some(now - 30 * 86_400),
        _ => None,
    }
}

pub fn day_bounds(day: &str) -> Option<(i64, i64)> {
    let date = NaiveDate::parse_from_str(day, "%Y-%m-%d").ok()?;
    let start = date
        .and_hms_opt(0, 0, 0)?
        .and_local_timezone(Local)
        .earliest()?;
    let next = (date + Duration::days(1))
        .and_hms_opt(0, 0, 0)?
        .and_local_timezone(Local)
        .earliest()?;
    Some((start.timestamp(), next.timestamp() - 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepted_ranges_have_exact_rolling_boundaries() {
        let now = 2_000_000_000;
        assert_eq!(range_start("24h", now), Some(now - 86_400));
        assert_eq!(range_start("7d", now), Some(now - 7 * 86_400));
        assert_eq!(range_start("30d", now), Some(now - 30 * 86_400));
        assert_eq!(range_start("all", now), None);
    }

    #[test]
    fn replay_day_rejects_invalid_calendar_input() {
        assert!(day_bounds("2026-08-11").is_some());
        assert!(day_bounds("2026-02-30").is_none());
        assert!(day_bounds("11-08-2026").is_none());
    }
}
