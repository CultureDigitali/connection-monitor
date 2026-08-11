use crate::stats::{ConnectionStats, ConnectionStatus};

use super::model::MinuteBucket;

#[derive(Default)]
struct Average {
    sum: f64,
    count: u32,
}

impl Average {
    fn add(&mut self, value: f64) {
        if value.is_finite() {
            self.sum += value;
            self.count += 1;
        }
    }

    fn value(&self) -> Option<f64> {
        (self.count > 0).then(|| self.sum / f64::from(self.count))
    }
}

struct MinuteAccumulator {
    started_at: i64,
    sample_count: u32,
    online_count: u32,
    quality: Average,
    minimum_quality: Option<u8>,
    download: Average,
    upload: Average,
    ping: Average,
    jitter: Average,
    packet_loss: Average,
    wifi_signal: Average,
    downloaded_mb: f64,
    uploaded_mb: f64,
    last_download_total: f64,
    last_upload_total: f64,
}

impl MinuteAccumulator {
    fn new(started_at: i64, stats: &ConnectionStats) -> Self {
        let mut accumulator = Self {
            started_at,
            sample_count: 0,
            online_count: 0,
            quality: Average::default(),
            minimum_quality: None,
            download: Average::default(),
            upload: Average::default(),
            ping: Average::default(),
            jitter: Average::default(),
            packet_loss: Average::default(),
            wifi_signal: Average::default(),
            downloaded_mb: 0.0,
            uploaded_mb: 0.0,
            last_download_total: stats.total_download_mb,
            last_upload_total: stats.total_upload_mb,
        };
        accumulator.add_sample(stats, false);
        accumulator
    }

    fn add_sample(&mut self, stats: &ConnectionStats, include_traffic_delta: bool) {
        self.sample_count += 1;
        if stats.connection_status == ConnectionStatus::Online {
            self.online_count += 1;
            self.quality.add(f64::from(stats.quality_score));
            self.minimum_quality = Some(
                self.minimum_quality
                    .map(|value| value.min(stats.quality_score))
                    .unwrap_or(stats.quality_score),
            );
            self.download.add(stats.download_mbps);
            self.upload.add(stats.upload_mbps);
            if stats.ping_ms > 0.0 {
                self.ping.add(stats.ping_ms);
            }
            self.jitter.add(stats.jitter_ms);
            self.packet_loss.add(stats.packet_loss);
            if let Some(signal) = stats.wifi_signal {
                self.wifi_signal.add(f64::from(signal));
            }
        }

        if include_traffic_delta {
            let download_delta = stats.total_download_mb - self.last_download_total;
            let upload_delta = stats.total_upload_mb - self.last_upload_total;
            if download_delta.is_finite() && download_delta >= 0.0 {
                self.downloaded_mb += download_delta;
            }
            if upload_delta.is_finite() && upload_delta >= 0.0 {
                self.uploaded_mb += upload_delta;
            }
        }
        self.last_download_total = stats.total_download_mb;
        self.last_upload_total = stats.total_upload_mb;
    }

    fn finish(self) -> MinuteBucket {
        MinuteBucket {
            started_at: self.started_at,
            sample_count: self.sample_count,
            availability: if self.sample_count == 0 {
                0.0
            } else {
                f64::from(self.online_count) / f64::from(self.sample_count)
            },
            average_quality: self.quality.value(),
            minimum_quality: self.minimum_quality,
            average_download_mbps: self.download.value(),
            average_upload_mbps: self.upload.value(),
            average_ping_ms: self.ping.value(),
            average_jitter_ms: self.jitter.value(),
            average_packet_loss: self.packet_loss.value(),
            average_wifi_signal: self.wifi_signal.value(),
            downloaded_mb: self.downloaded_mb,
            uploaded_mb: self.uploaded_mb,
        }
    }
}

#[derive(Default)]
pub struct HistoryRecorder {
    current: Option<MinuteAccumulator>,
}

impl HistoryRecorder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&mut self, timestamp: i64, stats: &ConnectionStats) -> Option<MinuteBucket> {
        let minute = timestamp.div_euclid(60) * 60;
        match self.current.take() {
            None => {
                self.current = Some(MinuteAccumulator::new(minute, stats));
                None
            }
            Some(mut current) if current.started_at == minute => {
                current.add_sample(stats, true);
                self.current = Some(current);
                None
            }
            Some(current) => {
                let finished = current.finish();
                self.current = Some(MinuteAccumulator::new(minute, stats));
                Some(finished)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::{ConnectionStats, ConnectionStatus};

    fn online_stats(
        quality: u8,
        download_mbps: f64,
        upload_mbps: f64,
        total_download_mb: f64,
        total_upload_mb: f64,
    ) -> ConnectionStats {
        ConnectionStats {
            quality_score: quality,
            quality_score_i32: i32::from(quality),
            download_mbps,
            upload_mbps,
            ping_ms: 20.0,
            jitter_ms: 3.0,
            packet_loss: 0.5,
            wifi_signal: Some(-55),
            connection_status: ConnectionStatus::Online,
            is_connected: true,
            total_download_mb,
            total_upload_mb,
            ..ConnectionStats::default()
        }
    }

    #[test]
    fn finalizes_a_minute_with_hand_checked_aggregates() {
        let mut recorder = HistoryRecorder::new();
        assert!(recorder
            .record(60, &online_stats(80, 10.0, 2.0, 100.0, 40.0))
            .is_none());
        assert!(recorder
            .record(61, &online_stats(60, 20.0, 4.0, 101.0, 42.0))
            .is_none());

        let bucket = recorder
            .record(120, &online_stats(90, 1.0, 1.0, 102.0, 43.0))
            .expect("the previous minute should finalize");

        assert_eq!(bucket.started_at, 60);
        assert_eq!(bucket.sample_count, 2);
        assert_eq!(bucket.average_quality, Some(70.0));
        assert_eq!(bucket.minimum_quality, Some(60));
        assert_eq!(bucket.availability, 1.0);
        assert_eq!(bucket.average_download_mbps, Some(15.0));
        assert_eq!(bucket.average_upload_mbps, Some(3.0));
        assert_eq!(bucket.average_ping_ms, Some(20.0));
        assert_eq!(bucket.downloaded_mb, 1.0);
        assert_eq!(bucket.uploaded_mb, 2.0);
    }

    #[test]
    fn unavailable_metrics_stay_missing_and_offline_samples_reduce_availability() {
        let mut recorder = HistoryRecorder::new();
        let offline = ConnectionStats {
            connection_status: ConnectionStatus::Offline,
            total_download_mb: 4.0,
            total_upload_mb: 2.0,
            ..ConnectionStats::default()
        };

        recorder.record(180, &offline);
        let bucket = recorder
            .record(240, &online_stats(90, 2.0, 1.0, 5.0, 3.0))
            .unwrap();

        assert_eq!(bucket.availability, 0.0);
        assert_eq!(bucket.average_quality, None);
        assert_eq!(bucket.average_ping_ms, None);
        assert_eq!(bucket.downloaded_mb, 0.0);
        assert_eq!(bucket.uploaded_mb, 0.0);
    }

    #[test]
    fn session_counter_reset_never_creates_negative_traffic() {
        let mut recorder = HistoryRecorder::new();
        recorder.record(300, &online_stats(90, 1.0, 1.0, 50.0, 20.0));
        recorder.record(301, &online_stats(90, 1.0, 1.0, 1.0, 1.0));
        let bucket = recorder
            .record(360, &online_stats(90, 1.0, 1.0, 2.0, 2.0))
            .unwrap();

        assert_eq!(bucket.downloaded_mb, 0.0);
        assert_eq!(bucket.uploaded_mb, 0.0);
    }
}
