use crate::Result;
use crate::config::ProxyConfig;
use crate::network::NetworkTester;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, info, warn};

/// Type alias for progress callback
pub type ProgressCallback = Box<dyn Fn(&SpeedTestResult) + Send + Sync>;

/// Configuration for speed testing
#[derive(Debug, Clone)]
pub struct SpeedTestConfig {
    pub server_url: String,
    pub download_timeout: Duration, // 下载超时时间
    pub upload_timeout: Duration,   // 上传超时时间
    pub concurrent: usize,
    pub download_size: usize,
    pub upload_size: usize,
    pub max_latency: Option<Duration>,
    pub min_download_speed: Option<f64>,
    pub min_upload_speed: Option<f64>,
    pub fast_mode: bool,
}

impl Default for SpeedTestConfig {
    fn default() -> Self {
        Self {
            server_url: "https://speed.cloudflare.com".to_string(),
            download_timeout: Duration::from_secs(10), // 下载超时10秒
            upload_timeout: Duration::from_secs(30),   // 上传超时30秒
            concurrent: 4,
            download_size: 50 * 1024 * 1024, // 50MB
            upload_size: 20 * 1024 * 1024,   // 20MB
            max_latency: Some(Duration::from_millis(800)),
            min_download_speed: Some(5.0 * 1024.0 * 1024.0), // 5MB/s
            min_upload_speed: Some(2.0 * 1024.0 * 1024.0),   // 2MB/s
            fast_mode: false,
        }
    }
}

/// Result of a speed test for a single proxy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedTestResult {
    pub proxy_name: String,
    pub proxy_type: crate::config::ProxyType,
    pub latency: Option<Duration>,
    pub jitter: Option<Duration>,
    pub packet_loss: f64,
    pub download_speed: f64, // bytes per second
    pub upload_speed: f64,   // bytes per second
    pub download_time: Option<Duration>,
    pub upload_time: Option<Duration>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl SpeedTestResult {
    /// Create a new failed result
    pub fn failed(proxy_name: String, proxy_type: crate::config::ProxyType, error: String) -> Self {
        Self {
            proxy_name,
            proxy_type,
            latency: None,
            jitter: None,
            packet_loss: 100.0,
            download_speed: 0.0,
            upload_speed: 0.0,
            download_time: None,
            upload_time: None,
            error: Some(error),
            timestamp: Utc::now(),
        }
    }

    /// Format latency for display
    pub fn format_latency(&self) -> String {
        match self.latency {
            Some(latency) => format!("{}ms", latency.as_millis()),
            None => "Failed".to_string(),
        }
    }

    /// Format download speed for display
    pub fn format_download_speed(&self) -> String {
        if self.download_speed > 0.0 {
            let mbps = self.download_speed / (1024.0 * 1024.0);
            format!("{mbps:.2} MB/s")
        } else {
            "Failed".to_string()
        }
    }

    /// Format upload speed for display
    pub fn format_upload_speed(&self) -> String {
        if self.upload_speed > 0.0 {
            let mbps = self.upload_speed / (1024.0 * 1024.0);
            format!("{mbps:.2} MB/s")
        } else {
            "Failed".to_string()
        }
    }

    /// Check if the test was successful
    pub fn is_successful(&self) -> bool {
        self.error.is_none() && self.latency.is_some()
    }
}

/// Main speed testing engine
pub struct SpeedTester {
    config: SpeedTestConfig,
    network_tester: NetworkTester,
}

impl SpeedTester {
    /// Create a new speed tester with the given configuration
    pub fn new(config: SpeedTestConfig) -> Self {
        let network_tester = NetworkTester::new(
            config.server_url.clone(),
            config.download_timeout,
            config.upload_timeout,
        );
        Self {
            config,
            network_tester,
        }
    }

    /// Test a single proxy
    pub async fn test_proxy(&self, proxy: &ProxyConfig) -> Result<SpeedTestResult> {
        info!("Testing proxy: {}", proxy.name);

        let start_time = Utc::now();

        // Test latency first
        let latency_result = match self.network_tester.test_latency(proxy, 6).await {
            Ok(result) => result,
            Err(e) => {
                warn!("Latency test failed for {}: {}", proxy.name, e);
                return Ok(SpeedTestResult::failed(
                    proxy.name.clone(),
                    proxy.proxy_type.clone(),
                    format!("Latency test failed: {e}"),
                ));
            }
        };

        // If fast mode is enabled, only test latency
        if self.config.fast_mode {
            return Ok(SpeedTestResult {
                proxy_name: proxy.name.clone(),
                proxy_type: proxy.proxy_type.clone(),
                latency: Some(latency_result.avg_latency),
                jitter: Some(latency_result.jitter),
                packet_loss: latency_result.packet_loss,
                download_speed: 0.0,
                upload_speed: 0.0,
                download_time: None,
                upload_time: None,
                error: None,
                timestamp: start_time,
            });
        }

        // Test download speed
        let download_result = if self.config.download_size > 0 {
            match self
                .network_tester
                .test_download(proxy, self.config.download_size, self.config.concurrent)
                .await
            {
                Ok(result) => Some(result),
                Err(e) => {
                    debug!("Download test failed for {}: {}", proxy.name, e);
                    None
                }
            }
        } else {
            None
        };

        // Test upload speed
        let upload_result = if self.config.upload_size > 0 {
            match self
                .network_tester
                .test_upload(proxy, self.config.upload_size)
                .await
            {
                Ok(result) => Some(result),
                Err(e) => {
                    debug!("Upload test failed for {}: {}", proxy.name, e);
                    None
                }
            }
        } else {
            None
        };

        Ok(SpeedTestResult {
            proxy_name: proxy.name.clone(),
            proxy_type: proxy.proxy_type.clone(),
            latency: Some(latency_result.avg_latency),
            jitter: Some(latency_result.jitter),
            packet_loss: latency_result.packet_loss,
            download_speed: download_result.as_ref().map_or(0.0, |r| r.speed),
            upload_speed: upload_result.as_ref().map_or(0.0, |r| r.speed),
            download_time: download_result.as_ref().map(|r| r.duration),
            upload_time: upload_result.as_ref().map(|r| r.duration),
            error: None,
            timestamp: start_time,
        })
    }

    /// Test multiple proxies with optional progress callback
    pub async fn test_proxies(
        &self,
        proxies: Vec<ProxyConfig>,
        callback: Option<ProgressCallback>,
    ) -> Result<Vec<SpeedTestResult>> {
        let mut results = Vec::with_capacity(proxies.len());

        info!("Starting speed test for {} proxies", proxies.len());

        for (index, proxy) in proxies.iter().enumerate() {
            debug!(
                "Testing proxy {}/{}: {}",
                index + 1,
                proxies.len(),
                proxy.name
            );

            let result = self.test_proxy(proxy).await?;

            if let Some(ref callback) = callback {
                callback(&result);
            }

            results.push(result);
        }

        info!("Completed testing {} proxies", results.len());
        Ok(results)
    }

    /// Test multiple proxies concurrently
    pub async fn test_proxies_concurrent(
        &self,
        proxies: Vec<ProxyConfig>,
        max_concurrent: usize,
    ) -> Result<Vec<SpeedTestResult>> {
        use futures::stream::{StreamExt, iter};

        let results = iter(proxies)
            .map(|proxy| async move { self.test_proxy(&proxy).await })
            .buffer_unordered(max_concurrent)
            .collect::<Vec<_>>()
            .await;

        // Convert Vec<Result<T>> to Result<Vec<T>>
        results.into_iter().collect()
    }
}
