pub mod loader;

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_yaml::{Mapping, Value};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

pub use loader::ConfigLoader;

/// Supported proxy types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProxyType {
    Shadowsocks,
    ShadowsocksShort,
    VMess,
    VLESS,
    Trojan,
    Hysteria,
    Hysteria2,
    WireGuard,
    Socks5,
    Socks,
    Http,
    Https,
    // Add support for newer/custom proxy types
    AnyTLS,
    Unknown(String),
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
            _ => Ok(ProxyType::Unknown(s.to_string())),
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
            ProxyType::Unknown(proxy_type) => write!(f, "{proxy_type}"),
        }
    }
}

impl ProxyType {
    pub fn as_mihomo_type(&self) -> &str {
        match self {
            ProxyType::Shadowsocks => "shadowsocks",
            ProxyType::ShadowsocksShort => "ss",
            ProxyType::VMess => "vmess",
            ProxyType::VLESS => "vless",
            ProxyType::Trojan => "trojan",
            ProxyType::Hysteria => "hysteria",
            ProxyType::Hysteria2 => "hysteria2",
            ProxyType::WireGuard => "wireguard",
            ProxyType::Socks5 => "socks5",
            ProxyType::Socks => "socks",
            ProxyType::Http => "http",
            ProxyType::Https => "https",
            ProxyType::AnyTLS => "anytls",
            ProxyType::Unknown(proxy_type) => proxy_type.as_str(),
        }
    }

    pub fn supports_standard_mode(&self) -> bool {
        matches!(
            self,
            ProxyType::Http | ProxyType::Https | ProxyType::Socks5 | ProxyType::Socks
        )
    }
}

impl Serialize for ProxyType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_mihomo_type())
    }
}

impl<'de> Deserialize<'de> for ProxyType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ProxyTypeVisitor;

        impl Visitor<'_> for ProxyTypeVisitor {
            type Value = ProxyType;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a proxy type string")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                value.parse().map_err(de::Error::custom)
            }
        }

        deserializer.deserialize_str(ProxyTypeVisitor)
    }
}

/// Main proxy configuration structure
#[derive(Debug, Clone)]
pub struct ProxyConfig {
    pub name: String,
    pub proxy_type: ProxyType,
    pub server: String,
    pub port: u16,
    pub config: ProxyParameters,
    raw: Option<Mapping>,
}

impl ProxyConfig {
    pub fn new(
        name: String,
        proxy_type: ProxyType,
        server: String,
        port: u16,
        config: ProxyParameters,
    ) -> Self {
        Self {
            name,
            proxy_type,
            server,
            port,
            config,
            raw: None,
        }
    }

    pub fn raw_mapping(&self) -> Mapping {
        let mut mapping = if let Some(raw) = &self.raw {
            raw.clone()
        } else {
            match serde_yaml::to_value(&self.config) {
                Ok(Value::Mapping(mapping)) => mapping,
                _ => Mapping::new(),
            }
        };

        mapping.insert(
            Value::String("name".to_string()),
            Value::String(self.name.clone()),
        );
        let type_key = Value::String("type".to_string());
        if !mapping.contains_key(&type_key) {
            mapping.insert(
                type_key,
                Value::String(self.proxy_type.as_mihomo_type().to_string()),
            );
        }

        if !self.server.is_empty() {
            mapping.insert(
                Value::String("server".to_string()),
                Value::String(self.server.clone()),
            );
        }

        if self.port != 0 {
            mapping.insert(
                Value::String("port".to_string()),
                Value::Number(self.port.into()),
            );
        }

        mapping
    }
}

impl Serialize for ProxyConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.raw_mapping().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ProxyConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = Mapping::deserialize(deserializer)?;
        let name = value_as_string(raw.get(Value::String("name".to_string())))
            .unwrap_or_else(|| "Unnamed".to_string());
        let proxy_type = value_as_string(raw.get(Value::String("type".to_string())))
            .unwrap_or_else(|| "unknown".to_string())
            .parse()
            .map_err(de::Error::custom)?;
        let server =
            value_as_string(raw.get(Value::String("server".to_string()))).unwrap_or_default();
        let port = match raw.get(Value::String("port".to_string())) {
            Some(value) => parse_port_value(value).map_err(de::Error::custom)?,
            None => 0,
        };
        let config = serde_yaml::from_value(Value::Mapping(raw.clone())).unwrap_or_default();

        Ok(Self {
            name,
            proxy_type,
            server,
            port,
            config,
            raw: Some(raw),
        })
    }
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

fn value_as_string(value: Option<&Value>) -> Option<String> {
    match value {
        Some(Value::String(value)) => Some(value.clone()),
        Some(Value::Number(value)) => Some(value.to_string()),
        Some(Value::Bool(value)) => Some(value.to_string()),
        _ => None,
    }
}

fn parse_port_value(value: &Value) -> Result<u16, String> {
    match value {
        Value::Number(number) => {
            let port = number
                .as_u64()
                .ok_or_else(|| format!("invalid port number: {number}"))?;
            if port <= u16::MAX as u64 {
                Ok(port as u16)
            } else {
                Err(format!("port out of range: {port}"))
            }
        }
        Value::String(value) => value
            .parse::<u16>()
            .map_err(|_| format!("invalid port string: {value}")),
        _ => Err("port must be a string or number".to_string()),
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_unknown_proxy_type_and_preserves_raw_fields() {
        let yaml = r#"
proxies:
  - name: Mystery
    type: future-protocol
    server: example.com
    port: "443"
    custom-field:
      enabled: true
      value: 7
"#;

        let config: ClashConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.proxies.len(), 1);
        assert_eq!(
            config.proxies[0].proxy_type,
            ProxyType::Unknown("future-protocol".to_string())
        );

        let serialized = serde_yaml::to_value(&config.proxies[0]).unwrap();
        let mapping = serialized.as_mapping().unwrap();
        assert_eq!(
            mapping.get(Value::String("type".to_string())).unwrap(),
            &Value::String("future-protocol".to_string())
        );
        assert!(
            mapping
                .get(Value::String("custom-field".to_string()))
                .is_some()
        );
    }

    #[test]
    fn round_trips_modern_mihomo_proxy_types_without_protocol_whitelist() {
        let yaml = r#"
proxies:
  - name: hy2
    type: hysteria2
    server: hy.example.com
    port: 443
    password: secret
  - name: any
    type: anytls
    server: any.example.com
    port: 443
    password: secret
  - name: vless
    type: vless
    server: vless.example.com
    port: 443
    uuid: 00000000-0000-0000-0000-000000000000
    flow: xtls-rprx-vision
  - name: tuic
    type: tuic
    server: tuic.example.com
    port: 443
    uuid: 00000000-0000-0000-0000-000000000000
    password: secret
  - name: wg
    type: wireguard
    server: wg.example.com
    port: 51820
    private-key: abc
"#;

        let config: ClashConfig = serde_yaml::from_str(yaml).unwrap();
        let rendered = serde_yaml::to_string(&config).unwrap();

        for proxy_type in ["hysteria2", "anytls", "vless", "tuic", "wireguard"] {
            assert!(rendered.contains(&format!("type: {proxy_type}")));
        }
    }
}
