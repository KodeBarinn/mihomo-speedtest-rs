use crate::Result;
use crate::network::{ProxyClient, ZeroReader};
use futures::future::try_join_all;
use std::time::{Duration, Instant};
use tracing::debug;

/// Result of bandwidth testing
#[derive(Debug, Clone)]
pub struct BandwidthResult {
    pub bytes: usize,
    pub duration: Duration,
    pub speed: f64, // bytes per second
}

impl BandwidthResult {
    /// Create a new bandwidth result
    pub fn new(bytes: usize, duration: Duration) -> Self {
        let speed = if duration.as_secs_f64() > 0.0 {
            bytes as f64 / duration.as_secs_f64()
        } else {
            0.0
        };

        Self {
            bytes,
            duration,
            speed,
        }
    }

    /// Get speed in MB/s
    pub fn speed_mbps(&self) -> f64 {
        self.speed / (1024.0 * 1024.0)
    }
}

/// Bandwidth tester for measuring download and upload speeds
pub struct BandwidthTester {
    client: ProxyClient,
    server_url: String,
}

impl BandwidthTester {
    /// Create a new bandwidth tester
    pub fn new(client: ProxyClient, server_url: String) -> Self {
        Self { client, server_url }
    }

    /// Test download speed with concurrent connections
    pub async fn test_download(&self, size: usize, concurrent: usize) -> Result<BandwidthResult> {
        debug!(
            "Starting download test: {} bytes with {} concurrent connections",
            size, concurrent
        );

        let chunk_size = size / concurrent;
        let mut tasks = Vec::new();

        let start = Instant::now();

        // Create concurrent download tasks
        for i in 0..concurrent {
            let client = self.client.clone();
            let server_url = self.server_url.clone();
            let actual_chunk_size = if i == concurrent - 1 {
                // Last chunk gets any remaining bytes
                size - (chunk_size * (concurrent - 1))
            } else {
                chunk_size
            };

            tasks.push(tokio::spawn(async move {
                Self::download_chunk(&client, &server_url, actual_chunk_size).await
            }));
        }

        // Wait for all downloads to complete
        let results = try_join_all(tasks).await?;
        let total_duration = start.elapsed();

        // Calculate total bytes downloaded
        let total_bytes: usize = results
            .iter()
            .map(|r| r.as_ref().map_or(0, |cr| cr.bytes))
            .sum();

        // Count successful downloads for average duration calculation
        let successful_results: Vec<_> = results.into_iter().filter_map(|r| r.ok()).collect();

        if successful_results.is_empty() {
            return Err(anyhow::anyhow!("All download chunks failed"));
        }

        debug!(
            "Download completed: {} bytes in {:?} ({:.2} MB/s)",
            total_bytes,
            total_duration,
            total_bytes as f64 / (1024.0 * 1024.0) / total_duration.as_secs_f64()
        );

        Ok(BandwidthResult::new(total_bytes, total_duration))
    }

    /// Test upload speed
    pub async fn test_upload(&self, size: usize) -> Result<BandwidthResult> {
        debug!("Starting upload test: {} bytes", size);

        let url = format!("{}/__up", self.server_url);
        let data = ZeroReader::new(size);

        let start = Instant::now();
        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/octet-stream")
            .body(data)
            .send()
            .await?;

        let duration = start.elapsed();
        debug!("Upload response status: {}", response.status());
        debug!("Upload response headers: {:?}", response.headers());

        if !response.status().is_success() {
            let status = response.status();
            // Try to read response body for error details
            match response.text().await {
                Ok(body) => {
                    debug!("Upload failed with status {}, body: {}", status, body);
                    return Err(anyhow::anyhow!(
                        "Upload failed with status: {}, body: {}",
                        status,
                        body
                    ));
                }
                Err(_) => {
                    return Err(anyhow::anyhow!(
                        "Upload failed with status: {}",
                        status
                    ));
                }
            }
        }

        debug!(
            "Upload completed: {} bytes in {:?} ({:.2} MB/s)",
            size,
            duration,
            size as f64 / (1024.0 * 1024.0) / duration.as_secs_f64()
        );

        Ok(BandwidthResult::new(size, duration))
    }

    /// Download a single chunk
    async fn download_chunk(
        client: &ProxyClient,
        server_url: &str,
        size: usize,
    ) -> Result<ChunkResult> {
        let url = format!("{server_url}/__down?bytes={size}");
        let _start = Instant::now();

        let response = client.get(&url).await?;
        debug!("Download chunk response status: {}", response.status());
        debug!("Download chunk response headers: {:?}", response.headers());
        
        if !response.status().is_success() {
            let status = response.status();
            // Try to read response body for error details
            match response.text().await {
                Ok(body) => {
                    debug!("Download chunk failed with status {}, body: {}", status, body);
                    return Err(anyhow::anyhow!(
                        "Download chunk failed with status: {}, body: {}",
                        status,
                        body
                    ));
                }
                Err(_) => {
                    return Err(anyhow::anyhow!(
                        "Download chunk failed with status: {}",
                        status
                    ));
                }
            }
        }

        match response.bytes().await {
            Ok(bytes) => {
                debug!("Download chunk successfully received {} bytes", bytes.len());
                Ok(ChunkResult { bytes: bytes.len() })
            }
            Err(e) => {
                debug!("Download chunk failed to decode response body: {}", e);
                Err(anyhow::anyhow!("Download chunk failed to decode response body: {}", e))
            }
        }
    }
}

/// Result of downloading a single chunk
#[derive(Debug)]
struct ChunkResult {
    bytes: usize,
}
