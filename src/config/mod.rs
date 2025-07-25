pub mod loader;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

pub use loader::ConfigLoader;

/// Supported proxy types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProxyType {
    Shadowsocks,
    #[serde(rename = "ss")]
    ShadowsocksShort,
    VMess,
    VLESS,
    Trojan,
    Hysteria,
    Hysteria2,
    #[serde(rename = "wireguard")]
    WireGuard,
    Socks5,
    #[serde(rename = "socks")]
    Socks,
    Http,
    Https,
    // Add support for newer/custom proxy types
    #[serde(rename = "anytls")]
    AnyTLS,
}

impl FromStr for ProxyType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "shadowsocks" | "ss" => Ok(ProxyType::Shadowsocks),
            "vmess" => Ok(ProxyType::VMess),
            "vless" => Ok(ProxyType::VLESS),
            "trojan" => Ok(ProxyType::Trojan),
            "hysteria" => Ok(ProxyType::Hysteria),
            "hysteria2" => Ok(ProxyType::Hysteria2),
            "wireguard" | "wg" => Ok(ProxyType::WireGuard),
            "socks5" | "socks" => Ok(ProxyType::Socks5),
            "http" => Ok(ProxyType::Http),
            "https" => Ok(ProxyType::Https),
            "anytls" => Ok(ProxyType::AnyTLS),
            _ => Err(format!("Unknown proxy type: {}", s)),
        }
    }
}

impl std::fmt::Display for ProxyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProxyType::Shadowsocks | ProxyType::ShadowsocksShort => write!(f, "Shadowsocks"),
            ProxyType::VMess => write!(f, "VMess"),
            ProxyType::VLESS => write!(f, "VLESS"),
            ProxyType::Trojan => write!(f, "Trojan"),
            ProxyType::Hysteria => write!(f, "Hysteria"),
            ProxyType::Hysteria2 => write!(f, "Hysteria2"),
            ProxyType::WireGuard => write!(f, "WireGuard"),
            ProxyType::Socks5 | ProxyType::Socks => write!(f, "SOCKS5"),
            ProxyType::Http => write!(f, "HTTP"),
            ProxyType::Https => write!(f, "HTTPS"),
            ProxyType::AnyTLS => write!(f, "AnyTLS"),
        }
    }
}

/// Main proxy configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub proxy_type: ProxyType,
    pub server: String,
    #[serde(deserialize_with = "deserialize_port")]
    pub port: u16,
    #[serde(flatten)]
    pub config: ProxyParameters,
}

/// Proxy parameters that vary by protocol type
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProxyParameters {
    // Common TLS settings
    pub tls: Option<bool>,
    #[serde(rename = "skip-cert-verify")]
    pub skip_cert_verify: Option<bool>,
    pub sni: Option<String>,

    // Authentication
    pub username: Option<String>,
    pub password: Option<String>,
    pub uuid: Option<String>,

    // Shadowsocks specific
    pub cipher: Option<String>,
    pub plugin: Option<String>,
    #[serde(rename = "plugin-opts")]
    pub plugin_opts: Option<HashMap<String, serde_yaml::Value>>,

    // VMess/VLESS specific
    #[serde(rename = "alterId")]
    pub alter_id: Option<u32>,
    pub security: Option<String>,
    pub flow: Option<String>,

    // Transport options
    pub network: Option<String>,
    #[serde(rename = "ws-opts")]
    pub ws_opts: Option<HashMap<String, serde_yaml::Value>>,
    #[serde(rename = "grpc-opts")]
    pub grpc_opts: Option<HashMap<String, serde_yaml::Value>>,
    #[serde(rename = "h2-opts")]
    pub h2_opts: Option<HashMap<String, serde_yaml::Value>>,

    // Hysteria specific
    pub protocol: Option<String>,
    #[serde(default, deserialize_with = "deserialize_string_or_number")]
    pub up: Option<String>,
    #[serde(default, deserialize_with = "deserialize_string_or_number")]
    pub down: Option<String>,
    pub auth: Option<String>,
    #[serde(rename = "auth-str")]
    pub auth_str: Option<String>,
    #[serde(rename = "ca-str")]
    pub ca_str: Option<String>,

    // Additional common fields
    pub udp: Option<bool>,
    pub tfo: Option<bool>,
    #[serde(rename = "client-fingerprint")]
    pub client_fingerprint: Option<String>,

    // Hysteria2 specific fields (ports field for port ranges)
    pub ports: Option<String>,

    // Trojan/TLS specific fields
    #[serde(rename = "alpn")]
    pub alpn: Option<Vec<String>>,
    #[serde(rename = "fingerprint")]
    pub fingerprint: Option<String>,

    // Connection optimization
    pub mptcp: Option<bool>,
    #[serde(rename = "ip-version")]
    pub ip_version: Option<String>,
    #[serde(rename = "interface-name")]
    pub interface_name: Option<String>,
    #[serde(rename = "routing-mark")]
    pub routing_mark: Option<u32>,
    #[serde(rename = "dialer-proxy")]
    pub dialer_proxy: Option<String>,

    // SMUX configuration
    pub smux: Option<HashMap<String, serde_yaml::Value>>,

    // Catch-all for unknown fields
    #[serde(flatten)]
    pub extra: HashMap<String, serde_yaml::Value>,
}

/// TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    pub enabled: bool,
    #[serde(rename = "skip-cert-verify")]
    pub skip_cert_verify: bool,
    pub server_name: Option<String>,
    pub alpn: Option<Vec<String>>,
}

/// Transport configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportConfig {
    #[serde(rename = "type")]
    pub transport_type: String,
    pub path: Option<String>,
    pub host: Option<String>,
    pub headers: Option<HashMap<String, String>>,
}

/// Root configuration structure for Clash config files
#[derive(Debug, Serialize, Deserialize)]
pub struct ClashConfig {
    pub proxies: Vec<ProxyConfig>,
    #[serde(rename = "proxy-providers")]
    pub proxy_providers: Option<HashMap<String, serde_yaml::Value>>,
    #[serde(flatten)]
    pub other: HashMap<String, serde_yaml::Value>,
}

/// Custom deserializer for port field that can be either string or number
fn deserialize_port<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;

    struct PortVisitor;

    impl<'de> Visitor<'de> for PortVisitor {
        type Value = u16;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a port number as string or integer")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            v.parse::<u16>()
                .map_err(|_| de::Error::custom(format!("invalid port string: {}", v)))
        }

        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if v >= 0 && v <= u16::MAX as i64 {
                Ok(v as u16)
            } else {
                Err(de::Error::custom(format!("port out of range: {}", v)))
            }
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if v <= u16::MAX as u64 {
                Ok(v as u16)
            } else {
                Err(de::Error::custom(format!("port out of range: {}", v)))
            }
        }
    }

    deserializer.deserialize_any(PortVisitor)
}

/// Custom deserializer for fields that can be either string or number
fn deserialize_string_or_number<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;

    struct StringOrNumberVisitor;

    impl<'de> Visitor<'de> for StringOrNumberVisitor {
        type Value = Option<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string or number")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(v.to_string()))
        }

        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(v.to_string()))
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(v.to_string()))
        }

        fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(v.to_string()))
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }
    }

    deserializer.deserialize_any(StringOrNumberVisitor)
}
