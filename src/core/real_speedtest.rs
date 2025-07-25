use crate::Result;
use crate::config::ProxyConfig;
use crate::core::mihomo_runner::MihomoRunner;
use crate::core::{SpeedTestConfig, SpeedTestResult};
use chrono::Utc;
use std::time::Duration;
use tracing::{debug, info, warn};

/// Real proxy speed tester that uses mihomo process
pub struct RealSpeedTester {
    mihomo_runner: MihomoRunner,
    config: SpeedTestConfig,
}

impl RealSpeedTester {
    /// Create a new real speed tester
    pub fn new(mihomo_runner: MihomoRunner, config: SpeedTestConfig) -> Self {
        Self {
            mihomo_runner,
            config,
        }
    }

    /// Start mihomo and run speed tests
    pub async fn test_proxies(&mut self, proxies: &[ProxyConfig]) -> Result<Vec<SpeedTestResult>> {
        info!("Starting real proxy speed tests with mihomo process");

        // Generate and start mihomo with configuration
        let mihomo_config = self.mihomo_runner.generate_config(proxies)?;
        self.mihomo_runner.start(&mihomo_config).await?;

        let mut results = Vec::new();

        for proxy in proxies {
            info!("Testing proxy: {}", proxy.name);
            let result = self.test_single_proxy(proxy).await;
            results.push(result);
        }

        // Stop mihomo process
        if let Err(e) = self.mihomo_runner.stop() {
            warn!("Failed to stop mihomo process: {}", e);
        }

        Ok(results)
    }

    /// Test a single proxy through mihomo
    async fn test_single_proxy(&mut self, proxy: &ProxyConfig) -> SpeedTestResult {
        let start_time = Utc::now();

        // Switch mihomo to use this proxy
        if let Err(e) = self.mihomo_runner.switch_proxy(&proxy.name).await {
            return SpeedTestResult {
                proxy_name: proxy.name.clone(),
                proxy_type: proxy.proxy_type.clone(),
                latency: None,
                jitter: None,
                packet_loss: 1.0,
                download_speed: 0.0,
                upload_speed: 0.0,
                download_time: None,
                upload_time: None,
                error: Some(format!("Failed to switch proxy: {}", e)),
                timestamp: start_time,
            };
        }

        // Wait a moment for proxy to be ready
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Test latency using mihomo's built-in delay test
        let (latency, jitter, packet_loss) = match self.test_latency_through_mihomo(proxy).await {
            Ok(result) => result,
            Err(e) => {
                return SpeedTestResult {
                    proxy_name: proxy.name.clone(),
                    proxy_type: proxy.proxy_type.clone(),
                    latency: None,
                    jitter: None,
                    packet_loss: 1.0,
                    download_speed: 0.0,
                    upload_speed: 0.0,
                    download_time: None,
                    upload_time: None,
                    error: Some(format!("Latency test failed: {}", e)),
                    timestamp: start_time,
                };
            }
        };

        // Check if latency exceeds threshold
        if let Some(max_latency) = self.config.max_latency {
            if let Some(avg_latency) = latency {
                if avg_latency > max_latency {
                    return SpeedTestResult {
                        proxy_name: proxy.name.clone(),
                        proxy_type: proxy.proxy_type.clone(),
                        latency,
                        jitter,
                        packet_loss,
                        download_speed: 0.0,
                        upload_speed: 0.0,
                        download_time: None,
                        upload_time: None,
                        error: Some(format!(
                            "Latency {} exceeds threshold {:?}",
                            avg_latency.as_millis(),
                            max_latency.as_millis()
                        )),
                        timestamp: start_time,
                    };
                }
            }
        }

        // If fast mode is enabled, skip bandwidth tests
        if self.config.fast_mode {
            return SpeedTestResult {
                proxy_name: proxy.name.clone(),
                proxy_type: proxy.proxy_type.clone(),
                latency,
                jitter,
                packet_loss,
                download_speed: 0.0,
                upload_speed: 0.0,
                download_time: None,
                upload_time: None,
                error: None,
                timestamp: start_time,
            };
        }

        // Test bandwidth through mihomo proxy
        let (download_speed, download_time, upload_speed, upload_time, bandwidth_error) =
            self.test_bandwidth_through_mihomo().await;

        SpeedTestResult {
            proxy_name: proxy.name.clone(),
            proxy_type: proxy.proxy_type.clone(),
            latency,
            jitter,
            packet_loss,
            download_speed,
            upload_speed,
            download_time,
            upload_time,
            error: bandwidth_error,
            timestamp: start_time,
        }
    }

    /// Test latency through mihomo's delay test and our own latency test
    async fn test_latency_through_mihomo(
        &mut self,
        proxy: &ProxyConfig,
    ) -> Result<(Option<Duration>, Option<Duration>, f64)> {
        // First try mihomo's built-in delay test
        match self.mihomo_runner.test_proxy_delay(&proxy.name, None).await {
            Ok(delay_ms) => {
                let latency = Duration::from_millis(delay_ms as u64);
                debug!("Mihomo delay test result: {}ms", delay_ms);

                // Also do our own detailed latency test for jitter calculation
                match self.detailed_latency_test().await {
                    Ok((_, jitter, packet_loss)) => Ok((Some(latency), jitter, packet_loss)),
                    Err(_) => {
                        // Fallback to mihomo result only
                        Ok((Some(latency), None, 0.0))
                    }
                }
            }
            Err(e) => {
                debug!("Mihomo delay test failed: {}, trying detailed test", e);
                // Fallback to detailed latency test
                self.detailed_latency_test().await
            }
        }
    }

    /// Detailed latency test through mihomo proxy
    async fn detailed_latency_test(&mut self) -> Result<(Option<Duration>, Option<Duration>, f64)> {
        let proxy_client = self
            .mihomo_runner
            .create_proxy_client(self.config.download_timeout)?;

        // Create a simple proxy config for the latency tester
        // We don't need the actual proxy details since we're going through mihomo
        let _dummy_proxy = ProxyConfig {
            name: "mihomo-proxy".to_string(),
            proxy_type: crate::config::ProxyType::Http,
            server: "127.0.0.1".to_string(),
            port: self.mihomo_runner.proxy_port(),
            config: Default::default(),
        };

        // Create custom latency tester that uses the mihomo proxy client
        let latency_tester = CustomLatencyTester::new(proxy_client, self.config.server_url.clone());
        let result = latency_tester.test_latency(6).await?;

        Ok((
            Some(result.avg_latency),
            Some(result.jitter),
            result.packet_loss,
        ))
    }

    /// Test bandwidth through mihomo proxy
    async fn test_bandwidth_through_mihomo(
        &mut self,
    ) -> (f64, Option<Duration>, f64, Option<Duration>, Option<String>) {
        // Use download timeout for download tests
        let download_client = match self
            .mihomo_runner
            .create_proxy_client(self.config.download_timeout)
        {
            Ok(client) => client,
            Err(e) => {
                return (
                    0.0,
                    None,
                    0.0,
                    None,
                    Some(format!("Failed to create proxy client: {}", e)),
                );
            }
        };

        // Use upload timeout for upload tests
        let upload_client = match self
            .mihomo_runner
            .create_proxy_client(self.config.upload_timeout)
        {
            Ok(client) => client,
            Err(e) => {
                return (
                    0.0,
                    None,
                    0.0,
                    None,
                    Some(format!("Failed to create upload proxy client: {}", e)),
                );
            }
        };

        // Create bandwidth testers
        let download_tester =
            CustomBandwidthTester::new(download_client, self.config.server_url.clone());
        let upload_tester =
            CustomBandwidthTester::new(upload_client, self.config.server_url.clone());

        // Test download
        let (download_speed, download_time) = match download_tester
            .test_download(self.config.download_size, self.config.concurrent)
            .await
        {
            Ok(result) => (result.speed, Some(result.duration)),
            Err(e) => {
                warn!("Download test failed: {}", e);
                (0.0, None)
            }
        };

        // Test upload
        let (upload_speed, upload_time) =
            match upload_tester.test_upload(self.config.upload_size).await {
                Ok(result) => (result.speed, Some(result.duration)),
                Err(e) => {
                    warn!("Upload test failed: {}", e);
                    (0.0, None)
                }
            };

        // Check speed thresholds
        let mut errors = Vec::new();

        if let Some(min_download) = self.config.min_download_speed {
            if download_speed < min_download {
                errors.push(format!(
                    "Download speed {:.2} MB/s below threshold {:.2} MB/s",
                    download_speed / 1_000_000.0,
                    min_download / 1_000_000.0
                ));
            }
        }

        if let Some(min_upload) = self.config.min_upload_speed {
            if upload_speed < min_upload {
                errors.push(format!(
                    "Upload speed {:.2} MB/s below threshold {:.2} MB/s",
                    upload_speed / 1_000_000.0,
                    min_upload / 1_000_000.0
                ));
            }
        }

        let error = if errors.is_empty() {
            None
        } else {
            Some(errors.join("; "))
        };

        (
            download_speed,
            download_time,
            upload_speed,
            upload_time,
            error,
        )
    }
}

/// Custom latency tester that works with mihomo proxy
struct CustomLatencyTester {
    client: reqwest::Client,
    server_url: String,
}

impl CustomLatencyTester {
    fn new(client: reqwest::Client, server_url: String) -> Self {
        Self { client, server_url }
    }

    async fn test_latency(&self, iterations: usize) -> Result<crate::network::LatencyResult> {
        let mut latencies = Vec::new();
        let mut failed_count = 0;

        for i in 0..iterations {
            let url = format!("{}/__down?bytes=0", self.server_url);
            let start = std::time::Instant::now();

            match self.client.get(&url).send().await {
                Ok(response) if response.status().is_success() => {
                    let latency = start.elapsed();
                    latencies.push(latency);
                    debug!("Ping {}: {}ms", i + 1, latency.as_millis());
                }
                Ok(response) => {
                    warn!("Ping {} failed with status: {}", i + 1, response.status());
                    failed_count += 1;
                }
                Err(e) => {
                    warn!("Ping {} failed: {}", i + 1, e);
                    failed_count += 1;
                }
            }

            // Small delay between pings
            if i < iterations - 1 {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }

        if latencies.is_empty() {
            return Err(anyhow::anyhow!("All ping attempts failed"));
        }

        // Calculate statistics
        let sum: Duration = latencies.iter().sum();
        let avg_latency = sum / latencies.len() as u32;

        let variance: f64 = latencies
            .iter()
            .map(|&x| {
                let diff = x.as_nanos() as f64 - avg_latency.as_nanos() as f64;
                diff * diff
            })
            .sum::<f64>()
            / latencies.len() as f64;

        let jitter = Duration::from_nanos(variance.sqrt() as u64);
        let packet_loss = failed_count as f64 / iterations as f64;

        Ok(crate::network::LatencyResult {
            avg_latency,
            jitter,
            packet_loss,
            min_latency: *latencies.iter().min().unwrap_or(&avg_latency),
            max_latency: *latencies.iter().max().unwrap_or(&avg_latency),
        })
    }
}

/// Custom bandwidth tester that works with mihomo proxy
struct CustomBandwidthTester {
    client: reqwest::Client,
    server_url: String,
}

impl CustomBandwidthTester {
    fn new(client: reqwest::Client, server_url: String) -> Self {
        Self { client, server_url }
    }

    async fn test_download(
        &self,
        size: usize,
        concurrent: usize,
    ) -> Result<crate::network::BandwidthResult> {
        let start = std::time::Instant::now();
        let chunk_size = size / concurrent;

        let mut tasks = Vec::new();
        for i in 0..concurrent {
            let client = self.client.clone();
            let url = format!("{}/__down?bytes={}", self.server_url, chunk_size);

            let task = tokio::spawn(async move {
                debug!("Starting download chunk {} of size {}", i + 1, chunk_size);
                match client.get(&url).send().await {
                    Ok(response) => {
                        let bytes = response.bytes().await?;
                        Ok(bytes.len())
                    }
                    Err(e) => Err(anyhow::anyhow!("Download chunk {} failed: {}", i + 1, e)),
                }
            });
            tasks.push(task);
        }

        let mut total_bytes = 0;
        for task in tasks {
            match task.await? {
                Ok(bytes) => total_bytes += bytes,
                Err(e) => return Err(e),
            }
        }

        let duration = start.elapsed();
        let speed = total_bytes as f64 / duration.as_secs_f64();

        Ok(crate::network::BandwidthResult {
            bytes: total_bytes,
            speed,
            duration,
        })
    }

    async fn test_upload(&self, size: usize) -> Result<crate::network::BandwidthResult> {
        let start = std::time::Instant::now();
        let url = format!("{}/__up", self.server_url);

        // Create dummy data
        let data = vec![0u8; size];

        debug!("Starting upload of {} bytes", size);
        let response = self.client.post(&url).body(data).send().await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Upload failed with status: {}",
                response.status()
            ));
        }

        let duration = start.elapsed();
        let speed = size as f64 / duration.as_secs_f64();

        Ok(crate::network::BandwidthResult {
            bytes: size,
            speed,
            duration,
        })
    }
}
