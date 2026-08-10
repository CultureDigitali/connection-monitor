use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedTestResult {
    pub download_mbps: f64,
    pub latency_ms: f64,
    pub success: bool,
    pub error: Option<String>,
}

const SPEEDTEST_HOST: &str = "speedtest.tele2.net";
const SPEEDTEST_FALLBACK_IP: &str = "23.111.9.70";
const MAX_DOWNLOAD_BYTES: u64 = 10_000_000;
const MAX_DOWNLOAD_SECS: u64 = 15;

pub async fn run_speed_test() -> SpeedTestResult {
    let start = Instant::now();

    let host = if let Ok(ips) = dns_lookup::lookup_host(SPEEDTEST_HOST) {
        match ips.into_iter().next() {
            Some(ip) => ip.to_string(),
            None => SPEEDTEST_FALLBACK_IP.to_string(),
        }
    } else {
        SPEEDTEST_FALLBACK_IP.to_string()
    };

    let addr = format!("{}:80", host);

    let connect_start = Instant::now();
    match tokio::time::timeout(Duration::from_secs(5), TcpStream::connect(&addr)).await {
        Ok(Ok(mut stream)) => {
            let connect_time = connect_start.elapsed().as_secs_f64() * 1000.0;

            let request = format!(
                "GET /10MB.zip HTTP/1.0\r\nHost: {}\r\nConnection: close\r\n\r\n",
                SPEEDTEST_HOST
            );
            use tokio::io::AsyncWriteExt;
            if stream.write_all(request.as_bytes()).await.is_err() {
                return SpeedTestResult {
                    download_mbps: 0.0,
                    latency_ms: 0.0,
                    success: false,
                    error: Some("Request failed".to_string()),
                };
            }

            let download_start = Instant::now();
            let mut total_bytes = 0u64;
            let mut header_end = false;
            let mut buffer = [0u8; 8192];
            let mut accumulated: Vec<u8> = Vec::with_capacity(8192);
            let mut header_buf_consumed = 0;

            loop {
                if download_start.elapsed().as_secs() > MAX_DOWNLOAD_SECS {
                    break;
                }
                if total_bytes >= MAX_DOWNLOAD_BYTES {
                    break;
                }

                let read_result = tokio::time::timeout(
                    Duration::from_secs(5),
                    stream.read(&mut buffer),
                ).await;

                match read_result {
                    Ok(Ok(0)) => break,
                    Ok(Ok(n)) => {
                        let chunk = &buffer[..n];
                        if !header_end {
                            accumulated.extend_from_slice(chunk);
                            if let Some(idx) = find_header_end(&accumulated[header_buf_consumed..]) {
                                let abs_idx = header_buf_consumed + idx;
                                let body_start = abs_idx + 4;
                                if accumulated.len() > body_start {
                                    total_bytes = (accumulated.len() - body_start) as u64;
                                }
                                header_end = true;
                            }
                            header_buf_consumed = accumulated.len();
                        } else {
                            total_bytes += n as u64;
                        }
                    }
                    Ok(Err(_)) => break,
                    Err(_) => break,
                }
            }

            let elapsed = download_start.elapsed().as_secs_f64();
            if elapsed > 0.0 && total_bytes > 0 {
                let mbps = (total_bytes as f64 * 8.0) / elapsed / 1_000_000.0;
                SpeedTestResult {
                    download_mbps: mbps,
                    latency_ms: connect_time,
                    success: true,
                    error: None,
                }
            } else {
                SpeedTestResult {
                    download_mbps: 0.0,
                    latency_ms: connect_time,
                    success: false,
                    error: Some("No data received".to_string()),
                }
            }
        }
        Ok(Err(_)) => SpeedTestResult {
            download_mbps: 0.0,
            latency_ms: start.elapsed().as_secs_f64() * 1000.0,
            success: false,
            error: Some("Connection failed".to_string()),
        },
        Err(_) => SpeedTestResult {
            download_mbps: 0.0,
            latency_ms: start.elapsed().as_secs_f64() * 1000.0,
            success: false,
            error: Some("Connection timeout".to_string()),
        },
    }
}

fn find_header_end(data: &[u8]) -> Option<usize> {
    if data.len() < 4 {
        return None;
    }
    data.windows(4).position(|w| w == b"\r\n\r\n")
}
