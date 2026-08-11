use serde::{Deserialize, Serialize};
use chrono::{Duration, NaiveDate};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MinuteBucket {
    pub started_at: i64,
    pub sample_count: u32,
    pub availability: f64,
    pub average_quality: Option<f64>,
    pub minimum_quality: Option<u8>,
    pub average_download_mbps: Option<f64>,
    pub average_upload_mbps: Option<f64>,
    pub average_ping_ms: Option<f64>,
    pub average_jitter_ms: Option<f64>,
    pub average_packet_loss: Option<f64>,
    pub average_wifi_signal: Option<f64>,
    pub downloaded_mb: f64,
    pub uploaded_mb: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistorySummary {
    pub average_quality: Option<f64>,
    pub minimum_quality: Option<u8>,
    pub availability: f64,
    pub average_download_mbps: Option<f64>,
    pub average_upload_mbps: Option<f64>,
    pub average_ping_ms: Option<f64>,
    pub average_jitter_ms: Option<f64>,
    pub average_packet_loss: Option<f64>,
    pub downloaded_mb: f64,
    pub uploaded_mb: f64,
    pub incident_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DailyReliability {
    pub date: String,
    pub availability: f64,
    pub average_quality: f64,
    pub is_complete: bool,
}

impl DailyReliability {
    pub fn complete(date: &str, availability: f64, average_quality: f64) -> Self {
        Self {
            date: date.to_string(),
            availability,
            average_quality,
            is_complete: true,
        }
    }

    pub fn partial(date: &str, availability: f64, average_quality: f64) -> Self {
        Self {
            date: date.to_string(),
            availability,
            average_quality,
            is_complete: false,
        }
    }

    pub fn is_reliable(&self) -> bool {
        self.availability >= 0.99 && self.average_quality >= 75.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StreakSummary {
    pub current: u32,
    pub best: u32,
    pub next_milestone: Option<u32>,
    pub today_reliable_so_far: Option<bool>,
}

fn weighted_average(
    buckets: &[MinuteBucket],
    value: impl Fn(&MinuteBucket) -> Option<f64>,
) -> Option<f64> {
    let (sum, weight) = buckets.iter().fold((0.0, 0_u64), |(sum, weight), bucket| {
        value(bucket)
            .map(|metric| {
                (
                    sum + metric * f64::from(bucket.sample_count),
                    weight + u64::from(bucket.sample_count),
                )
            })
            .unwrap_or((sum, weight))
    });
    (weight > 0).then(|| sum / weight as f64)
}

pub fn summarize(buckets: &[MinuteBucket], incident_count: usize) -> HistorySummary {
    let total_samples: u64 = buckets.iter().map(|bucket| u64::from(bucket.sample_count)).sum();
    let availability = if total_samples == 0 {
        0.0
    } else {
        buckets
            .iter()
            .map(|bucket| bucket.availability * f64::from(bucket.sample_count))
            .sum::<f64>()
            / total_samples as f64
    };

    HistorySummary {
        average_quality: weighted_average(buckets, |bucket| bucket.average_quality),
        minimum_quality: buckets.iter().filter_map(|bucket| bucket.minimum_quality).min(),
        availability,
        average_download_mbps: weighted_average(buckets, |bucket| bucket.average_download_mbps),
        average_upload_mbps: weighted_average(buckets, |bucket| bucket.average_upload_mbps),
        average_ping_ms: weighted_average(buckets, |bucket| bucket.average_ping_ms),
        average_jitter_ms: weighted_average(buckets, |bucket| bucket.average_jitter_ms),
        average_packet_loss: weighted_average(buckets, |bucket| bucket.average_packet_loss),
        downloaded_mb: buckets.iter().map(|bucket| bucket.downloaded_mb).sum(),
        uploaded_mb: buckets.iter().map(|bucket| bucket.uploaded_mb).sum(),
        incident_count,
    }
}

pub fn calculate_streak(days: &[DailyReliability], today: &str) -> StreakSummary {
    let today_date = NaiveDate::parse_from_str(today, "%Y-%m-%d").ok();
    let mut complete: Vec<(NaiveDate, bool)> = days
        .iter()
        .filter(|day| day.is_complete)
        .filter_map(|day| {
            NaiveDate::parse_from_str(&day.date, "%Y-%m-%d")
                .ok()
                .filter(|date| today_date.map(|today| *date < today).unwrap_or(true))
                .map(|date| (date, day.is_reliable()))
        })
        .collect();
    complete.sort_by_key(|(date, _)| *date);

    let mut best = 0_u32;
    let mut run = 0_u32;
    let mut previous_date: Option<NaiveDate> = None;
    for (date, reliable) in &complete {
        let consecutive = previous_date
            .map(|previous| *date == previous + Duration::days(1))
            .unwrap_or(false);
        run = if *reliable {
            if consecutive { run + 1 } else { 1 }
        } else {
            0
        };
        best = best.max(run);
        previous_date = Some(*date);
    }

    let mut current = 0_u32;
    if let Some(mut expected) = today_date.map(|date| date - Duration::days(1)) {
        for (date, reliable) in complete.iter().rev() {
            if *date != expected || !*reliable {
                break;
            }
            current += 1;
            expected -= Duration::days(1);
        }
    }

    let today_reliable_so_far = days
        .iter()
        .find(|day| day.date == today)
        .map(DailyReliability::is_reliable);
    let next_milestone = [3, 7, 14, 30]
        .into_iter()
        .find(|milestone| *milestone > current);

    StreakSummary {
        current,
        best,
        next_milestone,
        today_reliable_so_far,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bucket(quality: f64, availability: f64, samples: u32) -> MinuteBucket {
        MinuteBucket {
            started_at: 0,
            sample_count: samples,
            availability,
            average_quality: Some(quality),
            minimum_quality: Some(quality as u8),
            average_download_mbps: Some(10.0),
            average_upload_mbps: Some(2.0),
            average_ping_ms: Some(20.0),
            average_jitter_ms: Some(3.0),
            average_packet_loss: Some(0.5),
            average_wifi_signal: Some(-55.0),
            downloaded_mb: 4.0,
            uploaded_mb: 1.0,
        }
    }

    #[test]
    fn summary_weights_quality_and_availability_by_sample_count() {
        let summary = summarize(&[bucket(90.0, 1.0, 3), bucket(30.0, 0.0, 1)], 2);

        assert_eq!(summary.average_quality, Some(75.0));
        assert_eq!(summary.availability, 0.75);
        assert_eq!(summary.minimum_quality, Some(30));
        assert_eq!(summary.downloaded_mb, 8.0);
        assert_eq!(summary.uploaded_mb, 2.0);
        assert_eq!(summary.incident_count, 2);
    }

    #[test]
    fn empty_summary_preserves_missing_measurements() {
        let summary = summarize(&[], 0);
        assert_eq!(summary.average_quality, None);
        assert_eq!(summary.minimum_quality, None);
        assert_eq!(summary.average_ping_ms, None);
        assert_eq!(summary.availability, 0.0);
    }

    #[test]
    fn streak_uses_completed_days_and_both_reliability_thresholds() {
        let days = vec![
            DailyReliability::complete("2026-08-07", 0.995, 82.0),
            DailyReliability::complete("2026-08-08", 0.990, 75.0),
            DailyReliability::complete("2026-08-09", 0.980, 90.0),
            DailyReliability::complete("2026-08-10", 1.000, 95.0),
            DailyReliability::partial("2026-08-11", 1.000, 100.0),
        ];

        let streak = calculate_streak(&days, "2026-08-11");
        assert_eq!(streak.current, 1);
        assert_eq!(streak.best, 2);
        assert_eq!(streak.next_milestone, Some(3));
        assert_eq!(streak.today_reliable_so_far, Some(true));
    }

    #[test]
    fn missing_yesterday_breaks_the_current_streak() {
        let days = vec![DailyReliability::complete("2026-08-09", 1.0, 90.0)];
        let streak = calculate_streak(&days, "2026-08-11");
        assert_eq!(streak.current, 0);
        assert_eq!(streak.best, 1);
    }
}
