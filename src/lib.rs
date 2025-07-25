//! # Mihomo SpeedTest Rust
//!
//! A fast and accurate Clash/Mihomo proxy speedtest tool.
//!
//! This library provides functionality to test proxy servers for latency, download speed,
//! and upload speed. It supports various proxy protocols including Shadowsocks, VMess,
//! Trojan, and standard HTTP/SOCKS5 proxies.

pub mod cli;
pub mod config;
pub mod core;
pub mod network;
pub mod output;

// Re-export commonly used types for convenience
pub use config::{ProxyConfig, ProxyParameters, ProxyType};
pub use core::{SpeedTestConfig, SpeedTestResult, SpeedTester};
pub use network::{BandwidthResult, LatencyResult};

/// Result type used throughout the library
pub type Result<T> = anyhow::Result<T>;
