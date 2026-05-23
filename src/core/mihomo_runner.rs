use crate::Result;
use crate::config::ProxyConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread::JoinHandle;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Mihomo process manager for real proxy testing
pub struct MihomoRunner {
    sandbox: TestSessionSandbox,
    mihomo_binary: PathBuf,
    process: Option<Child>,
    log_threads: Vec<JoinHandle<()>>,
}

/// Isolated filesystem and port sandbox for a single mihomo test session.
#[derive(Debug)]
pub struct TestSessionSandbox {
    config_dir: PathBuf,
    api_port: u16,
    proxy_port: u16,
    controller_secret: String,
}

/// Mihomo configuration structure
#[derive(Debug, Serialize, Deserialize)]
pub struct MihomoConfig {
    #[serde(rename = "mixed-port")]
    pub mixed_port: u16,
    #[serde(rename = "allow-lan")]
    pub allow_lan: bool,
    #[serde(rename = "bind-address")]
    pub bind_address: String,
    pub mode: String,
    #[serde(rename = "log-level")]
    pub log_level: String,
    #[serde(rename = "external-controller")]
    pub external_controller: String,
    pub secret: String,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
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
        // Try to find mihomo binary
        let mihomo_binary = if let Some(path) = mihomo_binary {
            path.as_ref().to_path_buf()
        } else {
            Self::find_mihomo_binary()?
        };

        let sandbox = TestSessionSandbox::new(config_dir, api_port, proxy_port)?;

        Ok(Self {
            sandbox,
            mihomo_binary,
            process: None,
            log_threads: Vec::new(),
        })
    }

    /// Find mihomo binary in system PATH or common locations
    fn find_mihomo_binary() -> Result<PathBuf> {
        let common_names = ["mihomo", "clash", "clash-meta"];
        let common_paths = ["/usr/local/bin", "/usr/bin", "/opt/homebrew/bin", "./"];

        // First try system PATH
        for name in &common_names {
            if let Ok(output) = Command::new("which").arg(name).output()
                && output.status.success()
            {
                let path_str = String::from_utf8_lossy(&output.stdout);
                let path = path_str.trim();
                if !path.is_empty() {
                    info!("Found mihomo binary at: {}", path);
                    return Ok(PathBuf::from(path));
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
            mixed_port: self.proxy_port(),
            allow_lan: false,
            bind_address: "127.0.0.1".to_string(),
            mode: "rule".to_string(),
            log_level: "info".to_string(),
            external_controller: format!("127.0.0.1:{}", self.api_port()),
            secret: self.sandbox.controller_secret.clone(),
            proxies: proxies.to_vec(),
            proxy_groups: vec![ProxyGroup {
                name: "SpeedTest".to_string(),
                group_type: "select".to_string(),
                proxies: proxy_names,
                url: None,
                interval: None,
            }],
            rules: vec!["MATCH,SpeedTest".to_string()],
        };

        Ok(config)
    }

    /// Write configuration to file
    pub fn write_config(&self, config: &MihomoConfig) -> Result<PathBuf> {
        let config_path = self.sandbox.config_dir.join("speedtest-config.yaml");
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
            .arg(&self.sandbox.config_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        self.consume_child_output(&mut child);

        // Wait for mihomo to start up
        let mut retries = 30; // 3 seconds with 100ms intervals
        while retries > 0 {
            if let Ok(Some(_)) = child.try_wait() {
                return Err(anyhow::anyhow!("Mihomo process exited unexpectedly"));
            }

            // Check if API is responding
            if self.check_api_health().await.is_ok() {
                info!("Mihomo API is ready at port {}", self.api_port());
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
            let _ = process.kill();
            process.wait()?;
            info!("Mihomo process stopped");
        }
        for handle in self.log_threads.drain(..) {
            let _ = handle.join();
        }
        Ok(())
    }

    /// Check if mihomo API is healthy
    async fn check_api_health(&self) -> Result<()> {
        let client = reqwest::Client::new();
        let url = format!("http://127.0.0.1:{}/", self.api_port());

        match client
            .get(&url)
            .bearer_auth(&self.sandbox.controller_secret)
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
        let url = format!("http://127.0.0.1:{}/proxies/SpeedTest", self.api_port());

        let mut body = HashMap::new();
        body.insert("name", proxy_name);

        let response = client
            .put(&url)
            .bearer_auth(&self.sandbox.controller_secret)
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
        let url = format!(
            "http://127.0.0.1:{}/proxies/{}",
            self.api_port(),
            proxy_name
        );

        let response = client
            .get(&url)
            .bearer_auth(&self.sandbox.controller_secret)
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
            self.api_port(),
            proxy_name,
            urlencoding::encode(test_url)
        );

        let response = client
            .get(&api_url)
            .bearer_auth(&self.sandbox.controller_secret)
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
        self.sandbox.proxy_port
    }

    /// Get the controller API port.
    pub fn api_port(&self) -> u16 {
        self.sandbox.api_port
    }

    /// Get the isolated config directory for this session.
    pub fn config_dir(&self) -> &Path {
        &self.sandbox.config_dir
    }

    fn consume_child_output(&mut self, child: &mut Child) {
        if let Some(stdout) = child.stdout.take() {
            self.log_threads.push(std::thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines().map_while(std::result::Result::ok) {
                    debug!("mihomo stdout: {}", line);
                }
            }));
        }

        if let Some(stderr) = child.stderr.take() {
            self.log_threads.push(std::thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines().map_while(std::result::Result::ok) {
                    debug!("mihomo stderr: {}", line);
                }
            }));
        }
    }

    /// Create an HTTP client configured to use mihomo proxy
    pub fn create_proxy_client(&self, timeout: Duration) -> Result<reqwest::Client> {
        let proxy_url = format!("http://127.0.0.1:{}", self.proxy_port());
        let proxy = reqwest::Proxy::http(&proxy_url)?;

        let client = reqwest::Client::builder()
            .proxy(proxy)
            .timeout(timeout)
            .danger_accept_invalid_certs(true)
            .build()?;

        Ok(client)
    }
}

impl TestSessionSandbox {
    pub fn new<P: AsRef<Path>>(base_dir: P, api_port: u16, proxy_port: u16) -> Result<Self> {
        let base_dir = base_dir.as_ref();
        let root = if base_dir.as_os_str().is_empty() {
            std::env::temp_dir()
        } else {
            base_dir.to_path_buf()
        };

        let session_id = Uuid::new_v4().to_string();
        let config_dir = root.join(format!("mihomo-speedtest-{session_id}"));
        std::fs::create_dir_all(&config_dir)?;

        let api_port = select_loopback_port(api_port)?;
        let proxy_port = select_distinct_loopback_port(proxy_port, api_port)?;

        Ok(Self {
            config_dir,
            api_port,
            proxy_port,
            controller_secret: Uuid::new_v4().to_string(),
        })
    }
}

impl Drop for TestSessionSandbox {
    fn drop(&mut self) {
        if let Err(e) = std::fs::remove_dir_all(&self.config_dir) {
            debug!(
                "Failed to remove mihomo sandbox {}: {}",
                self.config_dir.display(),
                e
            );
        }
    }
}

impl Drop for MihomoRunner {
    fn drop(&mut self) {
        if let Err(e) = self.stop() {
            error!("Failed to stop mihomo process: {}", e);
        }
    }
}

fn select_loopback_port(requested: u16) -> Result<u16> {
    if requested != 0 && can_bind_loopback(requested) {
        return Ok(requested);
    }

    if requested != 0 {
        warn!(
            "Port {} is unavailable, selecting a random free port",
            requested
        );
    }

    let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0))?;
    Ok(listener.local_addr()?.port())
}

fn select_distinct_loopback_port(requested: u16, unavailable: u16) -> Result<u16> {
    if requested != unavailable {
        let selected = select_loopback_port(requested)?;
        if selected != unavailable {
            return Ok(selected);
        }
    }

    for _ in 0..32 {
        let selected = select_loopback_port(0)?;
        if selected != unavailable {
            return Ok(selected);
        }
    }

    Err(anyhow::anyhow!("failed to select distinct loopback port"))
}

fn can_bind_loopback(port: u16) -> bool {
    TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, port)).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ProxyConfig, ProxyType};

    fn runner_for_tests(base_dir: &Path, api_port: u16, proxy_port: u16) -> MihomoRunner {
        MihomoRunner {
            sandbox: TestSessionSandbox::new(base_dir, api_port, proxy_port).unwrap(),
            mihomo_binary: PathBuf::from("mihomo"),
            process: None,
            log_threads: Vec::new(),
        }
    }

    #[test]
    fn generated_config_is_loopback_selector_only_sandbox() {
        let temp_dir = tempfile::tempdir().unwrap();
        let runner = runner_for_tests(temp_dir.path(), 0, 0);
        let proxies = vec![ProxyConfig::new(
            "node".to_string(),
            ProxyType::Unknown("tuic".to_string()),
            "example.com".to_string(),
            443,
            Default::default(),
        )];

        let config = runner.generate_config(&proxies).unwrap();
        let yaml = serde_yaml::to_string(&config).unwrap();

        assert!(!config.allow_lan);
        assert_eq!(config.bind_address, "127.0.0.1");
        assert!(config.external_controller.starts_with("127.0.0.1:"));
        assert!(yaml.contains("mixed-port:"));
        assert!(yaml.contains("MATCH,SpeedTest"));
        assert!(!yaml.contains("url-test"));
        assert!(!yaml.contains("redir-port"));
        assert!(!yaml.contains("tproxy-port"));
        assert!(!yaml.contains("tun:"));
        assert!(!yaml.contains("listen:"));
        assert!(!yaml.contains("allow-lan: true"));
    }

    #[test]
    fn unavailable_requested_ports_are_replaced() {
        let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)).unwrap();
        let busy_port = listener.local_addr().unwrap().port();
        let temp_dir = tempfile::tempdir().unwrap();

        let sandbox = TestSessionSandbox::new(temp_dir.path(), busy_port, busy_port).unwrap();

        assert_ne!(sandbox.api_port, busy_port);
        assert_ne!(sandbox.proxy_port, busy_port);
        assert_ne!(sandbox.api_port, sandbox.proxy_port);
    }

    #[test]
    fn sandbox_directory_is_removed_on_drop() {
        let temp_dir = tempfile::tempdir().unwrap();
        let sandbox = TestSessionSandbox::new(temp_dir.path(), 0, 0).unwrap();
        let config_dir = sandbox.config_dir.clone();

        assert!(config_dir.exists());
        drop(sandbox);
        assert!(!config_dir.exists());
    }
}
