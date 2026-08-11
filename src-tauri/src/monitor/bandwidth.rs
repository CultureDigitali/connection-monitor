use chrono::Local;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Instant;

const BYTES_PER_MB: f64 = 1_000_000.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthSample {
    pub timestamp: u64,
    pub download_mbps: f64,
    pub upload_mbps: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthResult {
    pub download_mbps: f64,
    pub upload_mbps: f64,
    pub total_download_mb: f64,
    pub total_upload_mb: f64,
}

pub struct BandwidthMonitor {
    history: Arc<Mutex<VecDeque<BandwidthSample>>>,
    last_check: Arc<Mutex<Option<(Instant, u64, u64)>>>,
    initial_totals: Arc<Mutex<Option<(u64, u64)>>>,
    max_history: usize,
}

fn session_megabytes(current: u64, baseline: u64) -> f64 {
    current.saturating_sub(baseline) as f64 / BYTES_PER_MB
}

impl BandwidthMonitor {
    pub fn new() -> Self {
        Self {
            history: Arc::new(Mutex::new(VecDeque::with_capacity(120))),
            last_check: Arc::new(Mutex::new(None)),
            initial_totals: Arc::new(Mutex::new(None)),
            max_history: 60,
        }
    }

    pub fn measure(&self) -> BandwidthResult {
        use sysinfo::Networks;

        let mut networks = Networks::new_with_refreshed_list();
        networks.refresh();

        let mut total_rx = 0u64;
        let mut total_tx = 0u64;

        for (_name, network) in &networks {
            total_rx += network.total_received();
            total_tx += network.total_transmitted();
        }

        let now = Instant::now();
        let timestamp = Local::now().timestamp_millis() as u64;
        let (initial_rx, initial_tx) = {
            let mut initial_totals = self.initial_totals.lock();
            *initial_totals.get_or_insert((total_rx, total_tx))
        };

        let (download_mbps, upload_mbps) = {
            let last_check = self.last_check.lock();
            match *last_check {
                Some((last_time, last_rx, last_tx)) => {
                    let elapsed = now.duration_since(last_time).as_secs_f64();
                    if elapsed > 0.0 {
                        let rx_diff = total_rx.saturating_sub(last_rx) as f64;
                        let tx_diff = total_tx.saturating_sub(last_tx) as f64;
                        let dl = (rx_diff * 8.0) / (elapsed * BYTES_PER_MB);
                        let ul = (tx_diff * 8.0) / (elapsed * BYTES_PER_MB);
                        (dl.max(0.0), ul.max(0.0))
                    } else {
                        (0.0, 0.0)
                    }
                }
                None => (0.0, 0.0),
            }
        };

        {
            let mut last_check = self.last_check.lock();
            *last_check = Some((now, total_rx, total_tx));
        }

        let sample = BandwidthSample {
            timestamp,
            download_mbps,
            upload_mbps,
        };

        {
            let mut history = self.history.lock();
            if history.len() >= self.max_history {
                history.pop_front();
            }
            history.push_back(sample);
        }

        BandwidthResult {
            download_mbps,
            upload_mbps,
            total_download_mb: session_megabytes(total_rx, initial_rx),
            total_upload_mb: session_megabytes(total_tx, initial_tx),
        }
    }

    pub fn get_history(&self) -> Vec<BandwidthSample> {
        let history = self.history.lock();
        history.iter().cloned().collect()
    }

    pub fn get_rolling_speed(&self) -> (f64, f64) {
        let history = self.history.lock();
        if history.is_empty() {
            return (0.0, 0.0);
        }
        let recent: Vec<_> = history.iter().rev().take(3).cloned().collect();
        let avg_dl: f64 = recent.iter().map(|s| s.download_mbps).sum::<f64>() / recent.len() as f64;
        let avg_ul: f64 = recent.iter().map(|s| s.upload_mbps).sum::<f64>() / recent.len() as f64;
        (avg_dl.max(0.0), avg_ul.max(0.0))
    }
}

#[cfg(test)]
mod tests {
    use super::session_megabytes;

    #[test]
    fn session_counter_starts_at_zero() {
        assert_eq!(session_megabytes(42_000_000, 42_000_000), 0.0);
    }

    #[test]
    fn session_counter_reports_only_the_delta() {
        assert_eq!(session_megabytes(43_500_000, 42_000_000), 1.5);
    }

    #[test]
    fn session_counter_never_underflows_after_interface_reset() {
        assert_eq!(session_megabytes(10, 42_000_000), 0.0);
    }
}
