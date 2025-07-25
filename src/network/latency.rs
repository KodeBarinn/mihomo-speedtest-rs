use crate::Result;
use crate::core::StatisticalAnalysis;
use crate::network::ProxyClient;
use std::time::{Duration, Instant};
use tracing::debug;

/// Result of latency testing
#[derive(Debug, Clone)]
pub struct LatencyResult {
    pub avg_latency: Duration,
    pub jitter: Duration,
    pub packet_loss: f64,
    pub min_latency: Duration,
    pub max_latency: Duration,
}

/// Latency tester for measuring round-trip time
pub struct LatencyTester {
    client: ProxyClient,
    server_url: String,
}

impl LatencyTester {
    /// Create a new latency tester
    pub fn new(client: ProxyClient, server_url: String) -> Self {
        Self { client, server_url }
    }

    /// Test latency with multiple iterations
    pub async fn test_latency(&self, iterations: usize) -> Result<LatencyResult> {
        let mut latencies = Vec::new();
        let mut failed_pings = 0;

        debug!("Starting latency test with {} iterations", iterations);

        for i in 0..iterations {
            // Small delay between pings to avoid overwhelming the server
            if i > 0 {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }

            let start = Instant::now();
            match self.ping_server().await {
                Ok(_) => {
                    let latency = start.elapsed();
                    latencies.push(latency);
                    debug!("Ping {}: {}ms", i + 1, latency.as_millis());
                }
                Err(e) => {
                    failed_pings += 1;
                    debug!("Ping {} failed: {}", i + 1, e);
                }
            }
        }

        Ok(self.calculate_result(latencies, failed_pings, iterations))
    }

    /// Send a ping to the server (minimal data transfer)
    async fn ping_server(&self) -> Result<()> {
        let url = format!("{}/__down?bytes=0", self.server_url);
        let response = self.client.get(&url).await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Server returned error: {}",
                response.status()
            ))
        }
    }

    /// Calculate latency statistics
    fn calculate_result(
        &self,
        latencies: Vec<Duration>,
        failed_pings: usize,
        total_pings: usize,
    ) -> LatencyResult {
        let packet_loss = StatisticalAnalysis::packet_loss_percentage(failed_pings, total_pings);

        if latencies.is_empty() {
            return LatencyResult {
                avg_latency: Duration::ZERO,
                jitter: Duration::ZERO,
                packet_loss,
                min_latency: Duration::ZERO,
                max_latency: Duration::ZERO,
            };
        }

        let avg_latency = StatisticalAnalysis::mean_duration(&latencies);
        let jitter = StatisticalAnalysis::std_deviation_duration(&latencies, avg_latency);
        let min_latency = *latencies.iter().min().unwrap();
        let max_latency = *latencies.iter().max().unwrap();

        LatencyResult {
            avg_latency,
            jitter,
            packet_loss,
            min_latency,
            max_latency,
        }
    }
}
