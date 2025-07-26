use crate::Result;
use crate::config::{ClashConfig, ProxyConfig, ProxyParameters, ProxyType};
use base64::{Engine as _, engine::general_purpose};
use regex::Regex;
use tracing::{debug, info, warn};

/// Configuration loader for Clash config files
pub struct ConfigLoader {
    client: reqwest::Client,
}

impl ConfigLoader {
    /// Create a new config loader
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap();

        Self { client }
    }

    /// Load configuration from path (file or URL)
    pub async fn load_from_path(&self, path: &str) -> Result<Vec<ProxyConfig>> {
        info!("Loading configuration from: {}", path);

        if path.starts_with("http://") || path.starts_with("https://") {
            self.load_from_url(path).await
        } else {
            self.load_from_file(path).await
        }
    }

    /// Load configuration from multiple paths
    pub async fn load_from_paths(&self, paths: &str) -> Result<Vec<ProxyConfig>> {
        let mut all_proxies = Vec::new();

        for path in paths.split(',') {
            let path = path.trim();
            if path.is_empty() {
                continue;
            }

            match self.load_from_path(path).await {
                Ok(mut proxies) => {
                    info!("Loaded {} proxies from {}", proxies.len(), path);
                    all_proxies.append(&mut proxies);
                }
                Err(e) => {
                    warn!("Failed to load from {}: {}", path, e);
                }
            }
        }

        info!("Total loaded proxies: {}", all_proxies.len());
        Ok(all_proxies)
    }

    /// Load from URL
    async fn load_from_url(&self, url: &str) -> Result<Vec<ProxyConfig>> {
        debug!("Fetching config from URL: {}", url);

        let response = self.client.get(url).send().await?;
        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "HTTP error {}: {}",
                response.status(),
                response.status().canonical_reason().unwrap_or("Unknown")
            ));
        }

        let content = response.text().await?;
        self.parse_config(&content)
    }

    /// Load from file
    async fn load_from_file(&self, path: &str) -> Result<Vec<ProxyConfig>> {
        debug!("Loading config from file: {}", path);

        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to read file {}: {}", path, e))?;

        self.parse_config(&content)
    }

    /// Parse configuration content
    fn parse_config(&self, content: &str) -> Result<Vec<ProxyConfig>> {
        // First try to decode as base64 (common for subscriptions)
        if let Ok(decoded_bytes) = general_purpose::STANDARD.decode(content.trim()) {
            if let Ok(decoded_content) = String::from_utf8(decoded_bytes) {
                debug!("Content appears to be base64 encoded, trying to parse decoded content");
                return self.parse_decoded_content(&decoded_content);
            }
        }

        // If not base64, try to parse directly
        self.parse_decoded_content(content)
    }

    /// Parse decoded content (could be YAML, JSON, or proxy list)
    fn parse_decoded_content(&self, content: &str) -> Result<Vec<ProxyConfig>> {
        // Try YAML parsing with proxy extraction first
        if let Ok(proxies) = self.extract_proxies_from_yaml(content) {
            if !proxies.is_empty() {
                debug!("Successfully extracted {} proxies from YAML", proxies.len());
                return Ok(proxies);
            }
        }

        // Try full YAML structure parsing
        match serde_yaml::from_str::<ClashConfig>(content) {
            Ok(config) => {
                debug!("Successfully parsed as complete YAML config");
                Ok(config.proxies)
            }
            Err(yaml_err) => {
                debug!("Full YAML parsing failed: {}", yaml_err);

                // Try JSON
                match serde_json::from_str::<ClashConfig>(content) {
                    Ok(config) => {
                        debug!("Successfully parsed as JSON");
                        Ok(config.proxies)
                    }
                    Err(json_err) => {
                        debug!("JSON parsing failed: {}", json_err);

                        // Try parsing as a simple proxy list or subscription format
                        self.parse_subscription_content(content)
                    }
                }
            }
        }
    }

    /// Extract proxies from YAML by parsing just the proxies section
    fn extract_proxies_from_yaml(&self, content: &str) -> Result<Vec<ProxyConfig>> {
        // Parse as generic YAML value first
        let yaml_value: serde_yaml::Value = serde_yaml::from_str(content)?;

        // Extract the 'proxies' field
        if let Some(proxies_value) = yaml_value.get("proxies") {
            // Try to deserialize the proxies array
            let proxies: Vec<ProxyConfig> = serde_yaml::from_value(proxies_value.clone())
                .map_err(|e| anyhow::anyhow!("Failed to parse proxies section: {}", e))?;
            Ok(proxies)
        } else {
            Err(anyhow::anyhow!("No 'proxies' field found in YAML"))
        }
    }

    /// Parse subscription content (various proxy URL formats)
    fn parse_subscription_content(&self, content: &str) -> Result<Vec<ProxyConfig>> {
        let mut proxies = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Try to parse different proxy URL formats
            match self.parse_proxy_url(line, line_num + 1) {
                Ok(proxy) => proxies.push(proxy),
                Err(e) => {
                    debug!("Failed to parse line {}: {} ({})", line_num + 1, e, line);
                    // Try legacy format as fallback
                    match self.parse_proxy_line(line, line_num + 1) {
                        Ok(proxy) => proxies.push(proxy),
                        Err(e2) => warn!(
                            "Failed to parse line {} in any format: {} / {} ({})",
                            line_num + 1,
                            e,
                            e2,
                            line
                        ),
                    }
                }
            }
        }

        if proxies.is_empty() {
            return Err(anyhow::anyhow!("No valid proxies found in configuration"));
        }

        Ok(proxies)
    }

    /// Parse a single proxy line (basic implementation)
    fn parse_proxy_line(&self, line: &str, _line_num: usize) -> Result<ProxyConfig> {
        // This is a basic implementation - in a full version, you'd want more sophisticated parsing
        // Format: name = type, server, port, ...

        if let Some(caps) = Regex::new(r"^([^=]+)=\s*(.+)$")?.captures(line) {
            let name = caps[1].trim().to_string();
            let config_part = &caps[2];

            // Simple comma-separated parsing
            let parts: Vec<&str> = config_part.split(',').map(|s| s.trim()).collect();

            if parts.len() < 3 {
                return Err(anyhow::anyhow!("Insufficient proxy parameters"));
            }

            let proxy_type = parts[0]
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid proxy type"))?;
            let server = parts[1].to_string();
            let port: u16 = parts[2]
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid port"))?;

            return Ok(ProxyConfig {
                name,
                proxy_type,
                server,
                port,
                config: Default::default(),
            });
        }

        Err(anyhow::anyhow!("Invalid proxy line format"))
    }

    /// Parse proxy URL in various formats (ss://, trojan://, vmess://, etc.)
    fn parse_proxy_url(&self, url: &str, _line_num: usize) -> Result<ProxyConfig> {
        if url.starts_with("ss://") {
            self.parse_shadowsocks_url(url)
        } else if url.starts_with("trojan://") {
            self.parse_trojan_url(url)
        } else if url.starts_with("vmess://") {
            self.parse_vmess_url(url)
        } else if url.starts_with("vless://") {
            self.parse_vless_url(url)
        } else if url.starts_with("hysteria://") {
            self.parse_hysteria_url(url)
        } else if url.starts_with("socks5://") || url.starts_with("socks://") {
            self.parse_socks_url(url)
        } else if url.starts_with("http://") || url.starts_with("https://") {
            // Don't parse HTTP URLs as proxy configs - they might be subscription URLs
            Err(anyhow::anyhow!("HTTP URLs are not proxy configurations"))
        } else {
            Err(anyhow::anyhow!("Unknown proxy URL format"))
        }
    }

    /// Parse Shadowsocks URL format: ss://method:password@server:port#name
    fn parse_shadowsocks_url(&self, url: &str) -> Result<ProxyConfig> {
        let url_without_scheme = url.strip_prefix("ss://").unwrap();

        // Split by # to get name
        let (config_part, name) = if let Some(hash_pos) = url_without_scheme.rfind('#') {
            let name = urlencoding::decode(&url_without_scheme[hash_pos + 1..])
                .map_err(|_| anyhow::anyhow!("Invalid URL encoding in name"))?;
            (&url_without_scheme[..hash_pos], name.to_string())
        } else {
            (url_without_scheme, "Shadowsocks".to_string())
        };

        // Try to decode base64 if the config part looks like base64
        let decoded_config =
            if let Ok(decoded_bytes) = general_purpose::STANDARD.decode(config_part) {
                if let Ok(decoded_str) = String::from_utf8(decoded_bytes) {
                    decoded_str
                } else {
                    config_part.to_string()
                }
            } else {
                config_part.to_string()
            };

        // Parse method:password@server:port
        if let Some(at_pos) = decoded_config.rfind('@') {
            let auth_part = &decoded_config[..at_pos];
            let server_part = &decoded_config[at_pos + 1..];

            // Parse server:port
            let (server, port) = if let Some(colon_pos) = server_part.rfind(':') {
                let server = server_part[..colon_pos].to_string();
                let port: u16 = server_part[colon_pos + 1..]
                    .parse()
                    .map_err(|_| anyhow::anyhow!("Invalid port in Shadowsocks URL"))?;
                (server, port)
            } else {
                return Err(anyhow::anyhow!("Missing port in Shadowsocks URL"));
            };

            // Parse method:password
            let (cipher, password) = if let Some(colon_pos) = auth_part.find(':') {
                let cipher = auth_part[..colon_pos].to_string();
                let password = auth_part[colon_pos + 1..].to_string();
                (cipher, password)
            } else {
                return Err(anyhow::anyhow!("Invalid auth format in Shadowsocks URL"));
            };

            let config = ProxyParameters {
                cipher: Some(cipher),
                password: Some(password),
                ..Default::default()
            };

            Ok(ProxyConfig {
                name,
                proxy_type: ProxyType::Shadowsocks,
                server,
                port,
                config,
            })
        } else {
            Err(anyhow::anyhow!("Invalid Shadowsocks URL format"))
        }
    }

    /// Parse Trojan URL format: trojan://password@server:port?params#name
    fn parse_trojan_url(&self, url: &str) -> Result<ProxyConfig> {
        let url_without_scheme = url.strip_prefix("trojan://").unwrap();

        // Split by # to get name
        let (config_part, name) = if let Some(hash_pos) = url_without_scheme.rfind('#') {
            let name = urlencoding::decode(&url_without_scheme[hash_pos + 1..])
                .map_err(|_| anyhow::anyhow!("Invalid URL encoding in name"))?;
            (&url_without_scheme[..hash_pos], name.to_string())
        } else {
            (url_without_scheme, "Trojan".to_string())
        };

        // Split by ? to get params
        let (auth_server_part, _params) = if let Some(question_pos) = config_part.find('?') {
            (
                &config_part[..question_pos],
                Some(&config_part[question_pos + 1..]),
            )
        } else {
            (config_part, None)
        };

        // Parse password@server:port
        if let Some(at_pos) = auth_server_part.rfind('@') {
            let password = auth_server_part[..at_pos].to_string();
            let server_part = &auth_server_part[at_pos + 1..];

            // Parse server:port
            let (server, port) = if let Some(colon_pos) = server_part.rfind(':') {
                let server = server_part[..colon_pos].to_string();
                let port: u16 = server_part[colon_pos + 1..]
                    .parse()
                    .map_err(|_| anyhow::anyhow!("Invalid port in Trojan URL"))?;
                (server, port)
            } else {
                return Err(anyhow::anyhow!("Missing port in Trojan URL"));
            };

            let config = ProxyParameters {
                password: Some(password),
                tls: Some(true),              // Trojan always uses TLS
                skip_cert_verify: Some(true), // Common default for testing
                ..Default::default()
            };

            Ok(ProxyConfig {
                name,
                proxy_type: ProxyType::Trojan,
                server,
                port,
                config,
            })
        } else {
            Err(anyhow::anyhow!("Invalid Trojan URL format"))
        }
    }

    /// Parse VMess URL format (base64 encoded JSON)
    fn parse_vmess_url(&self, url: &str) -> Result<ProxyConfig> {
        let url_without_scheme = url.strip_prefix("vmess://").unwrap();

        // VMess URLs are typically base64 encoded JSON
        let decoded_bytes = general_purpose::STANDARD
            .decode(url_without_scheme)
            .map_err(|_| anyhow::anyhow!("Invalid base64 in VMess URL"))?;
        let decoded_str = String::from_utf8(decoded_bytes)
            .map_err(|_| anyhow::anyhow!("Invalid UTF-8 in VMess URL"))?;

        // Parse as JSON
        let vmess_config: serde_json::Value = serde_json::from_str(&decoded_str)
            .map_err(|_| anyhow::anyhow!("Invalid JSON in VMess URL"))?;

        let name = vmess_config
            .get("ps")
            .and_then(|v| v.as_str())
            .unwrap_or("VMess")
            .to_string();

        let server = vmess_config
            .get("add")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing server in VMess config"))?
            .to_string();

        let port = vmess_config
            .get("port")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing port in VMess config"))?
            as u16;

        let uuid = vmess_config
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing UUID in VMess config"))?
            .to_string();

        let mut config = ProxyParameters {
            uuid: Some(uuid),
            ..Default::default()
        };

        if let Some(security) = vmess_config.get("scy").and_then(|v| v.as_str()) {
            config.security = Some(security.to_string());
        }

        if let Some(alter_id) = vmess_config.get("aid").and_then(|v| v.as_u64()) {
            config.alter_id = Some(alter_id as u32);
        }

        if let Some(network) = vmess_config.get("net").and_then(|v| v.as_str()) {
            config.network = Some(network.to_string());
        }

        if let Some(tls) = vmess_config.get("tls").and_then(|v| v.as_str()) {
            config.tls = Some(tls == "tls");
        }

        Ok(ProxyConfig {
            name,
            proxy_type: ProxyType::VMess,
            server,
            port,
            config,
        })
    }

    /// Parse VLESS URL format
    fn parse_vless_url(&self, _url: &str) -> Result<ProxyConfig> {
        // VLESS parsing would be similar to VMess but with different parameters
        Err(anyhow::anyhow!("VLESS URL parsing not yet implemented"))
    }

    /// Parse Hysteria URL format
    fn parse_hysteria_url(&self, _url: &str) -> Result<ProxyConfig> {
        // Hysteria parsing would have its own format
        Err(anyhow::anyhow!("Hysteria URL parsing not yet implemented"))
    }

    /// Parse SOCKS URL format
    fn parse_socks_url(&self, url: &str) -> Result<ProxyConfig> {
        let url_without_scheme = if url.starts_with("socks5://") {
            url.strip_prefix("socks5://").unwrap()
        } else {
            url.strip_prefix("socks://").unwrap()
        };

        // Parse [username:password@]server:port
        let (auth, server_part) = if let Some(at_pos) = url_without_scheme.rfind('@') {
            (
                Some(&url_without_scheme[..at_pos]),
                &url_without_scheme[at_pos + 1..],
            )
        } else {
            (None, url_without_scheme)
        };

        // Parse server:port
        let (server, port) = if let Some(colon_pos) = server_part.rfind(':') {
            let server = server_part[..colon_pos].to_string();
            let port: u16 = server_part[colon_pos + 1..]
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid port in SOCKS URL"))?;
            (server, port)
        } else {
            return Err(anyhow::anyhow!("Missing port in SOCKS URL"));
        };

        let mut config = ProxyParameters::default();

        if let Some(auth_part) = auth {
            if let Some(colon_pos) = auth_part.find(':') {
                config.username = Some(auth_part[..colon_pos].to_string());
                config.password = Some(auth_part[colon_pos + 1..].to_string());
            }
        }

        Ok(ProxyConfig {
            name: format!("SOCKS5-{server}"),
            proxy_type: ProxyType::Socks5,
            server,
            port,
            config,
        })
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}
