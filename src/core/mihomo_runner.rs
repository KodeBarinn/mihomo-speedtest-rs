use crate::Result;
use crate::config::ProxyConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

/// Mihomo process manager for real proxy testing
pub struct MihomoRunner {
    config_dir: PathBuf,
    mihomo_binary: PathBuf,
    process: Option<Child>,
    api_port: u16,
    proxy_port: u16,
}

/// Mihomo configuration structure
#[derive(Debug, Serialize, Deserialize)]
pub struct MihomoConfig {
    #[serde(rename = "mixed-port")]
    pub mixed_port: u16,
    #[serde(rename = "allow-lan")]
    pub allow_lan: bool,
    pub mode: String,
    #[serde(rename = "log-level")]
    pub log_level: String,
    #[serde(rename = "external-controller")]
    pub external_controller: String,
    pub proxies: Vec<ProxyConfig>,
    #[serde(rename = "proxy-groups")]
    pub proxy_groups: Vec<ProxyGroup>,
    pub rules: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProxyGroup {
    pub name: String,
    #[serde(rename = "type")]
    pub group_type: String,
    pub proxies: Vec<String>,
    pub url: Option<String>,
    pub interval: Option<u32>,
}

/// Mihomo API response structures
#[derive(Debug, Deserialize)]
pub struct ProxyInfo {
    pub name: String,
    #[serde(rename = "type")]
    pub proxy_type: String,
    pub history: Vec<DelayHistory>,
    pub alive: bool,
}

#[derive(Debug, Deserialize)]
pub struct DelayHistory {
    pub time: String,
    pub delay: u32,
}

impl MihomoRunner {
    /// Create a new mihomo runner
    pub fn new<P: AsRef<Path>>(
        config_dir: P,
        mihomo_binary: Option<P>,
        api_port: u16,
        proxy_port: u16,
    ) -> Result<Self> {
        let config_dir = config_dir.as_ref().to_path_buf();

        // Try to find mihomo binary
        let mihomo_binary = if let Some(path) = mihomo_binary {
            path.as_ref().to_path_buf()
        } else {
            Self::find_mihomo_binary()?
        };

        // Create config directory if it doesn't exist
        std::fs::create_dir_all(&config_dir)?;

        Ok(Self {
            config_dir,
            mihomo_binary,
            process: None,
            api_port,
            proxy_port,
        })
    }

    /// Find mihomo binary in system PATH or common locations
    fn find_mihomo_binary() -> Result<PathBuf> {
        let common_names = ["mihomo", "clash", "clash-meta"];
        let common_paths = ["/usr/local/bin", "/usr/bin", "/opt/homebrew/bin", "./"];

        // First try system PATH
        for name in &common_names {
            if let Ok(output) = Command::new("which").arg(name).output() {
                if output.status.success() {
                    let path_str = String::from_utf8_lossy(&output.stdout);
                    let path = path_str.trim();
                    if !path.is_empty() {
                        info!("Found mihomo binary at: {}", path);
                        return Ok(PathBuf::from(path));
                    }
                }
            }
        }

        // Then try common paths
        for path in &common_paths {
            for name in &common_names {
                let full_path = PathBuf::from(path).join(name);
                if full_path.exists() && full_path.is_file() {
                    info!("Found mihomo binary at: {}", full_path.display());
                    return Ok(full_path);
                }
            }
        }

        Err(anyhow::anyhow!(
            "Mihomo binary not found. Please install mihomo or specify the path with --mihomo-binary"
        ))
    }

    /// Generate mihomo configuration for testing
    pub fn generate_config(&self, proxies: &[ProxyConfig]) -> Result<MihomoConfig> {
        let proxy_names: Vec<String> = proxies.iter().map(|p| p.name.clone()).collect();

        let config = MihomoConfig {
            mixed_port: self.proxy_port,
            allow_lan: false,
            mode: "rule".to_string(),
            log_level: "info".to_string(),
            external_controller: format!("127.0.0.1:{}", self.api_port),
            proxies: proxies.to_vec(),
            proxy_groups: vec![
                ProxyGroup {
                    name: "SpeedTest".to_string(),
                    group_type: "select".to_string(),
                    proxies: proxy_names.clone(),
                    url: None,
                    interval: None,
                },
                ProxyGroup {
                    name: "AutoTest".to_string(),
                    group_type: "url-test".to_string(),
                    proxies: proxy_names,
                    url: Some("http://www.gstatic.com/generate_204".to_string()),
                    interval: Some(300),
                },
            ],
            rules: vec!["MATCH,SpeedTest".to_string()],
        };

        Ok(config)
    }

    /// Write configuration to file
    pub fn write_config(&self, config: &MihomoConfig) -> Result<PathBuf> {
        let config_path = self.config_dir.join("speedtest-config.yaml");
        let config_yaml = serde_yaml::to_string(config)?;
        std::fs::write(&config_path, config_yaml)?;
        info!("Generated mihomo config at: {}", config_path.display());
        Ok(config_path)
    }

    /// Start mihomo process with configuration
    pub async fn start(&mut self, config: &MihomoConfig) -> Result<()> {
        if self.process.is_some() {
            warn!("Mihomo process is already running");
            return Ok(());
        }

        let config_path = self.write_config(config)?;

        info!("Starting mihomo process...");
        debug!(
            "Command: {} -f {}",
            self.mihomo_binary.display(),
            config_path.display()
        );

        let mut child = Command::new(&self.mihomo_binary)
            .arg("-f")
            .arg(&config_path)
            .arg("-d")
            .arg(&self.config_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        // Wait for mihomo to start up
        let mut retries = 30; // 3 seconds with 100ms intervals
        while retries > 0 {
            if let Ok(Some(_)) = child.try_wait() {
                return Err(anyhow::anyhow!("Mihomo process exited unexpectedly"));
            }

            // Check if API is responding
            if self.check_api_health().await.is_ok() {
                info!("Mihomo API is ready at port {}", self.api_port);
                self.process = Some(child);
                return Ok(());
            }

            sleep(Duration::from_millis(100)).await;
            retries -= 1;
        }

        // Kill the child process if it's still running
        let _ = child.kill();
        Err(anyhow::anyhow!("Timeout waiting for mihomo to start"))
    }

    /// Stop mihomo process
    pub fn stop(&mut self) -> Result<()> {
        if let Some(mut process) = self.process.take() {
            info!("Stopping mihomo process...");
            process.kill()?;
            process.wait()?;
            info!("Mihomo process stopped");
        }
        Ok(())
    }

    /// Check if mihomo API is healthy
    async fn check_api_health(&self) -> Result<()> {
        let client = reqwest::Client::new();
        let url = format!("http://127.0.0.1:{}/", self.api_port);

        match client
            .get(&url)
            .timeout(Duration::from_millis(500))
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => Ok(()),
            Ok(_) => Err(anyhow::anyhow!("API returned non-success status")),
            Err(_) => Err(anyhow::anyhow!("API not responding")),
        }
    }

    /// Switch to a specific proxy
    pub async fn switch_proxy(&self, proxy_name: &str) -> Result<()> {
        let client = reqwest::Client::new();
        let url = format!("http://127.0.0.1:{}/proxies/SpeedTest", self.api_port);

        let mut body = HashMap::new();
        body.insert("name", proxy_name);

        let response = client
            .put(&url)
            .json(&body)
            .timeout(Duration::from_secs(5))
            .send()
            .await?;

        if response.status().is_success() {
            debug!("Switched to proxy: {}", proxy_name);
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Failed to switch proxy: {}",
                response.status()
            ))
        }
    }

    /// Get proxy information from mihomo API
    pub async fn get_proxy_info(&self, proxy_name: &str) -> Result<ProxyInfo> {
        let client = reqwest::Client::new();
        let url = format!("http://127.0.0.1:{}/proxies/{}", self.api_port, proxy_name);

        let response = client
            .get(&url)
            .timeout(Duration::from_secs(5))
            .send()
            .await?;

        if response.status().is_success() {
            let proxy_info: ProxyInfo = response.json().await?;
            Ok(proxy_info)
        } else {
            Err(anyhow::anyhow!(
                "Failed to get proxy info: {}",
                response.status()
            ))
        }
    }

    /// Test proxy delay using mihomo's built-in delay test
    pub async fn test_proxy_delay(&self, proxy_name: &str, url: Option<&str>) -> Result<u32> {
        let client = reqwest::Client::new();
        let test_url = url.unwrap_or("http://www.gstatic.com/generate_204");
        let api_url = format!(
            "http://127.0.0.1:{}/proxies/{}/delay?timeout=5000&url={}",
            self.api_port,
            proxy_name,
            urlencoding::encode(test_url)
        );

        let response = client
            .get(&api_url)
            .timeout(Duration::from_secs(10))
            .send()
            .await?;

        if response.status().is_success() {
            #[derive(Deserialize)]
            struct DelayResult {
                delay: u32,
            }

            let result: DelayResult = response.json().await?;
            Ok(result.delay)
        } else {
            Err(anyhow::anyhow!(
                "Failed to test proxy delay: {}",
                response.status()
            ))
        }
    }

    /// Get the proxy port for HTTP client configuration
    pub fn proxy_port(&self) -> u16 {
        self.proxy_port
    }

    /// Create an HTTP client configured to use mihomo proxy
    pub fn create_proxy_client(&self, timeout: Duration) -> Result<reqwest::Client> {
        let proxy_url = format!("http://127.0.0.1:{}", self.proxy_port);
        let proxy = reqwest::Proxy::http(&proxy_url)?;

        let client = reqwest::Client::builder()
            .proxy(proxy)
            .timeout(timeout)
            .danger_accept_invalid_certs(true)
            .build()?;

        Ok(client)
    }
}

impl Drop for MihomoRunner {
    fn drop(&mut self) {
        if let Err(e) = self.stop() {
            error!("Failed to stop mihomo process: {}", e);
        }
    }
}
