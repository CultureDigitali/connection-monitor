use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;
use surge_ping::{Client, Config, PingIdentifier, PingSequence};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingResult {
    pub latency_ms: Option<f64>,
    pub jitter_ms: f64,
    pub packet_loss: f64,
    pub avg_latency: f64,
    pub min_latency: f64,
    pub max_latency: f64,
}

pub struct PingMonitor {
    history: Arc<Mutex<VecDeque<Option<f64>>>>,
    max_history: usize,
}

impl PingMonitor {
    pub fn new() -> Self {
        Self {
            history: Arc::new(Mutex::new(VecDeque::with_capacity(30))),
            max_history: 30,
        }
    }

    pub fn record_ping(&self, latency_ms: Option<f64>) {
        let mut history = self.history.lock();
        if history.len() >= self.max_history {
            history.pop_front();
        }
        history.push_back(latency_ms);
    }

    pub fn compute_stats(&self) -> PingResult {
        let history = self.history.lock();
        let successful: Vec<f64> = history.iter().filter_map(|x| *x).collect();

        let packet_loss = if history.is_empty() {
            0.0
        } else {
            let lost = history.iter().filter(|x| x.is_none()).count();
            (lost as f64 / history.len() as f64) * 100.0
        };

        if successful.is_empty() {
            return PingResult {
                latency_ms: None,
                jitter_ms: 0.0,
                packet_loss: 100.0,
                avg_latency: 0.0,
                min_latency: 0.0,
                max_latency: 0.0,
            };
        }

        let avg = successful.iter().sum::<f64>() / successful.len() as f64;
        let min = successful.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = successful.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        let jitter = if successful.len() >= 2 {
            let diffs: Vec<f64> = successful.windows(2).map(|w| (w[1] - w[0]).abs()).collect();
            diffs.iter().sum::<f64>() / diffs.len() as f64
        } else {
            0.0
        };

        PingResult {
            latency_ms: successful.last().copied(),
            jitter_ms: jitter,
            packet_loss,
            avg_latency: avg,
            min_latency: min,
            max_latency: max,
        }
    }

    #[allow(dead_code)]
    pub fn clear(&self) {
        let mut history = self.history.lock();
        history.clear();
    }
}

pub async fn do_ping(target: &str) -> Option<f64> {
    if let Some(latency) = do_icmp_ping(target).await {
        return Some(latency);
    }
    do_tcp_ping(target).await
}

async fn do_icmp_ping(target: &str) -> Option<f64> {
    let parsed_target: IpAddr = match target.parse() {
        Ok(ip) => ip,
        Err(_) => match dns_lookup::lookup_host(target) {
            Ok(ips) => ips.into_iter().next()?,
            Err(_) => return None,
        },
    };

    let config = Config::default();
    let client = Client::new(&config).ok()?;
    let payload = &[0u8; 56];
    let mut pinger = client.pinger(parsed_target, PingIdentifier(111)).await;
    pinger.timeout(Duration::from_secs(2));
    match pinger.ping(PingSequence(0), payload).await {
        Ok((_, duration)) => Some(duration.as_secs_f64() * 1000.0),
        Err(_) => None,
    }
}

async fn do_tcp_ping(target: &str) -> Option<f64> {
    use tokio::time::timeout;
    let start = std::time::Instant::now();
    let connect = tokio::net::TcpStream::connect((target, 443));
    match timeout(Duration::from_secs(3), connect).await {
        Ok(Ok(_stream)) => Some(start.elapsed().as_secs_f64() * 1000.0),
        _ => {
            let start = std::time::Instant::now();
            let connect = tokio::net::TcpStream::connect((target, 80));
            match timeout(Duration::from_secs(3), connect).await {
                Ok(Ok(_stream)) => Some(start.elapsed().as_secs_f64() * 1000.0),
                _ => None,
            }
        }
    }
}
