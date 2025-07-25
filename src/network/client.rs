use crate::Result;
use crate::config::{ProxyConfig, ProxyType};
use crate::network::{BandwidthResult, BandwidthTester, LatencyResult, LatencyTester};
use std::time::Duration;
use tracing::{debug, warn};

/// HTTP client configured for proxy usage
#[derive(Clone)]
pub struct ProxyClient {
    client: reqwest::Client,
    proxy_config: ProxyConfig,
}

impl ProxyClient {
    /// Create a new proxy client
    pub fn new(proxy_config: ProxyConfig, timeout: Duration) -> Result<Self> {
        let mut client_builder = reqwest::Client::builder()
            .timeout(timeout)
            .danger_accept_invalid_certs(true) // For testing purposes
            .no_proxy(); // We'll handle proxy ourselves

        // Configure proxy based on type
        let client = match &proxy_config.proxy_type {
            ProxyType::Http | ProxyType::Https => {
                debug!(
                    "Setting up HTTP/HTTPS proxy: {}:{}",
                    proxy_config.server, proxy_config.port
                );
                let proxy_url = format!("http://{}:{}", proxy_config.server, proxy_config.port);
                let proxy = reqwest::Proxy::http(&proxy_url)?;

                if let (Some(username), Some(password)) =
                    (&proxy_config.config.username, &proxy_config.config.password)
                {
                    let proxy = proxy.basic_auth(username, password);
                    client_builder = client_builder.proxy(proxy);
                } else {
                    client_builder = client_builder.proxy(proxy);
                }

                client_builder.build()?
            }
            ProxyType::Socks5 | ProxyType::Socks => {
                debug!(
                    "Setting up SOCKS5 proxy: {}:{}",
                    proxy_config.server, proxy_config.port
                );
                let proxy_url = format!("socks5://{}:{}", proxy_config.server, proxy_config.port);
                let proxy = reqwest::Proxy::http(&proxy_url)?;

                if let (Some(username), Some(password)) =
                    (&proxy_config.config.username, &proxy_config.config.password)
                {
                    let proxy = proxy.basic_auth(username, password);
                    client_builder = client_builder.proxy(proxy);
                } else {
                    client_builder = client_builder.proxy(proxy);
                }

                client_builder.build()?
            }
            ProxyType::Trojan => {
                debug!(
                    "Trojan proxy detected: {}:{}",
                    proxy_config.server, proxy_config.port
                );
                warn!(
                    "Trojan protocol requires special client implementation - using direct connection for basic connectivity test"
                );
                // For now, fall back to direct connection but log the configuration
                debug!(
                    "Trojan config - password: {:?}, network: {:?}, skip-cert-verify: {:?}",
                    proxy_config.config.password.is_some(),
                    proxy_config.config.network,
                    proxy_config.config.skip_cert_verify
                );
                client_builder.build()?
            }
            ProxyType::Hysteria | ProxyType::Hysteria2 => {
                debug!(
                    "Hysteria2 proxy detected: {}:{}",
                    proxy_config.server, proxy_config.port
                );
                warn!(
                    "Hysteria2 protocol requires special client implementation - using direct connection for basic connectivity test"
                );
                // Log the configuration parameters
                debug!(
                    "Hysteria2 config - password: {:?}, ports: {:?}, skip-cert-verify: {:?}",
                    proxy_config.config.password.is_some(),
                    proxy_config.config.ports,
                    proxy_config.config.skip_cert_verify
                );
                client_builder.build()?
            }
            ProxyType::AnyTLS => {
                debug!(
                    "AnyTLS proxy detected: {}:{}",
                    proxy_config.server, proxy_config.port
                );
                warn!(
                    "AnyTLS protocol requires special client implementation - using direct connection for basic connectivity test"
                );
                // Log the configuration parameters
                debug!(
                    "AnyTLS config - password: {:?}, client-fingerprint: {:?}, udp: {:?}, tfo: {:?}",
                    proxy_config.config.password.is_some(),
                    proxy_config.config.client_fingerprint,
                    proxy_config.config.udp,
                    proxy_config.config.tfo
                );
                client_builder.build()?
            }
            ProxyType::Shadowsocks | ProxyType::ShadowsocksShort => {
                debug!(
                    "Shadowsocks proxy detected: {}:{}",
                    proxy_config.server, proxy_config.port
                );
                warn!(
                    "Shadowsocks protocol requires special client implementation - using direct connection for basic connectivity test"
                );
                debug!(
                    "Shadowsocks config - cipher: {:?}, password: {:?}",
                    proxy_config.config.cipher,
                    proxy_config.config.password.is_some()
                );
                client_builder.build()?
            }
            ProxyType::VMess => {
                debug!(
                    "VMess proxy detected: {}:{}",
                    proxy_config.server, proxy_config.port
                );
                warn!(
                    "VMess protocol requires special client implementation - using direct connection for basic connectivity test"
                );
                debug!(
                    "VMess config - uuid: {:?}, security: {:?}, alter_id: {:?}",
                    proxy_config.config.uuid.is_some(),
                    proxy_config.config.security,
                    proxy_config.config.alter_id
                );
                client_builder.build()?
            }
            ProxyType::VLESS => {
                debug!(
                    "VLESS proxy detected: {}:{}",
                    proxy_config.server, proxy_config.port
                );
                warn!(
                    "VLESS protocol requires special client implementation - using direct connection for basic connectivity test"
                );
                debug!(
                    "VLESS config - uuid: {:?}, flow: {:?}",
                    proxy_config.config.uuid.is_some(),
                    proxy_config.config.flow
                );
                client_builder.build()?
            }
            ProxyType::WireGuard => {
                debug!(
                    "WireGuard proxy detected: {}:{}",
                    proxy_config.server, proxy_config.port
                );
                warn!(
                    "WireGuard protocol requires special client implementation - using direct connection for basic connectivity test"
                );
                client_builder.build()?
            }
        };

        Ok(Self {
            client,
            proxy_config,
        })
    }

    /// Get the underlying reqwest client
    pub fn client(&self) -> &reqwest::Client {
        &self.client
    }

    /// Get the proxy configuration
    pub fn proxy_config(&self) -> &ProxyConfig {
        &self.proxy_config
    }

    /// Make a GET request
    pub async fn get(&self, url: &str) -> Result<reqwest::Response> {
        debug!("Making GET request to: {}", url);
        Ok(self.client.get(url).send().await?)
    }

    /// Create a POST request builder
    pub fn post(&self, url: &str) -> reqwest::RequestBuilder {
        debug!("Creating POST request to: {}", url);
        self.client.post(url)
    }
}

/// Network tester that combines latency and bandwidth testing
pub struct NetworkTester {
    server_url: String,
    download_timeout: Duration,
    upload_timeout: Duration,
}

impl NetworkTester {
    /// Create a new network tester
    pub fn new(server_url: String, download_timeout: Duration, upload_timeout: Duration) -> Self {
        Self {
            server_url,
            download_timeout,
            upload_timeout,
        }
    }

    /// Test latency for a proxy
    pub async fn test_latency(
        &self,
        proxy: &ProxyConfig,
        iterations: usize,
    ) -> Result<LatencyResult> {
        let client = ProxyClient::new(proxy.clone(), self.download_timeout)?;
        let tester = LatencyTester::new(client, self.server_url.clone());
        tester.test_latency(iterations).await
    }

    /// Test download bandwidth for a proxy
    pub async fn test_download(
        &self,
        proxy: &ProxyConfig,
        size: usize,
        concurrent: usize,
    ) -> Result<BandwidthResult> {
        let client = ProxyClient::new(proxy.clone(), self.download_timeout)?;
        let tester = BandwidthTester::new(client, self.server_url.clone());
        tester.test_download(size, concurrent).await
    }

    /// Test upload bandwidth for a proxy
    pub async fn test_upload(&self, proxy: &ProxyConfig, size: usize) -> Result<BandwidthResult> {
        let client = ProxyClient::new(proxy.clone(), self.upload_timeout)?;
        let tester = BandwidthTester::new(client, self.server_url.clone());
        tester.test_upload(size).await
    }
}
