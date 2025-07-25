use clap::Parser;
use std::time::Duration;

/// Command line arguments for the mihomo speedtest tool
#[derive(Parser, Debug)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(about = env!("CARGO_PKG_DESCRIPTION"))]
#[command(author = env!("CARGO_PKG_AUTHORS"))]
#[command(version = env!("CARGO_PKG_VERSION"))]
pub struct Cli {
    /// Config file path or HTTP(S) URL
    #[arg(short = 'c', long = "config", required_unless_present_any = ["show_author", "show_about"])]
    pub config_paths: Option<String>,

    /// Filter proxies by name using regex
    #[arg(short = 'f', long = "filter", default_value = ".+")]
    pub filter_regex: String,

    /// Block proxies by keywords (use | to separate)
    #[arg(short = 'b', long = "block")]
    pub block_keywords: Option<String>,

    /// Speed test server URL
    #[arg(long = "server-url", default_value = "https://speed.cloudflare.com")]
    pub server_url: String,

    /// Download size in bytes for testing
    #[arg(long = "download-size", default_value = "52428800")]
    pub download_size: usize,

    /// Upload size in bytes for testing
    #[arg(long = "upload-size", default_value = "20971520")]
    pub upload_size: usize,

    /// Download timeout in seconds (or duration like "10s", "1m")
    #[arg(long = "download-timeout", default_value = "10", value_parser = parse_duration)]
    pub download_timeout: Duration,

    /// Upload timeout in seconds (or duration like "30s", "1m")
    #[arg(long = "upload-timeout", default_value = "30", value_parser = parse_duration)]
    pub upload_timeout: Duration,

    /// Set both download and upload timeout in seconds (overrides individual timeout settings)
    #[arg(long = "timeout", value_parser = parse_duration, help = "Set both download and upload timeout (overrides --download-timeout and --upload-timeout)")]
    pub timeout: Option<Duration>,

    /// Number of concurrent connections for testing
    #[arg(long = "concurrent", default_value = "4")]
    pub concurrent: usize,

    /// Output config file path
    #[arg(short = 'o', long = "output")]
    pub output: Option<String>,

    /// Filter out proxies with latency greater than this (milliseconds or duration like "800ms")
    #[arg(long = "max-latency", default_value = "800", value_parser = parse_latency_duration)]
    pub max_latency: Duration,

    /// Filter out proxies with download speed less than this (MB/s)
    #[arg(long = "min-download-speed", default_value = "5")]
    pub min_download_speed: f64,

    /// Filter out proxies with upload speed less than this (MB/s)
    #[arg(long = "min-upload-speed", default_value = "2")]
    pub min_upload_speed: f64,

    /// Fast mode: only test latency
    #[arg(long = "fast")]
    pub fast_mode: bool,

    /// Rename nodes with location and speed info
    #[arg(long = "rename")]
    pub rename_nodes: bool,

    /// Enable Stash compatibility mode
    #[arg(long = "stash-compatible")]
    pub stash_compatible: bool,

    /// Output results in JSON format
    #[arg(short = 'j', long = "json")]
    pub json_output: bool,

    /// Verbose output
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,

    /// Maximum number of proxies to test concurrently
    #[arg(long = "max-concurrent", default_value = "1")]
    pub max_concurrent: usize,

    /// Use mihomo process for real proxy testing
    #[arg(long = "use-mihomo")]
    pub use_mihomo: bool,

    /// Path to mihomo binary (auto-detect if not specified)
    #[arg(long = "mihomo-binary")]
    pub mihomo_binary: Option<String>,

    /// Mihomo API port
    #[arg(long = "mihomo-api-port", default_value = "19090")]
    pub mihomo_api_port: u16,

    /// Mihomo proxy port  
    #[arg(long = "mihomo-proxy-port", default_value = "17890")]
    pub mihomo_proxy_port: u16,

    /// Mihomo config directory
    #[arg(long = "mihomo-config-dir", default_value = "./mihomo-temp")]
    pub mihomo_config_dir: String,

    /// Show author information
    #[arg(long = "author", action = clap::ArgAction::SetTrue)]
    pub show_author: bool,

    /// Show about information
    #[arg(long = "about", action = clap::ArgAction::SetTrue)]
    pub show_about: bool,
}

/// Parse latency duration from either milliseconds (number) or duration string
fn parse_latency_duration(s: &str) -> Result<Duration, String> {
    // Try to parse as a number (milliseconds for latency)
    if let Ok(millis) = s.parse::<u64>() {
        return Ok(Duration::from_millis(millis));
    }

    // Fall back to humantime parsing for complex formats like "800ms", "1s"
    humantime::parse_duration(s).map_err(|e| e.to_string())
}

/// Parse duration from either seconds (number) or duration string
fn parse_duration(s: &str) -> Result<Duration, String> {
    // Try to parse as a number (seconds)
    if let Ok(seconds) = s.parse::<u64>() {
        return Ok(Duration::from_secs(seconds));
    }

    // Fall back to humantime parsing for complex formats like "1m30s"
    humantime::parse_duration(s).map_err(|e| e.to_string())
}

impl Cli {
    /// Convert CLI args to SpeedTestConfig
    pub fn to_speedtest_config(&self) -> crate::core::SpeedTestConfig {
        // Determine timeout values based on user input
        let (download_timeout, upload_timeout) = if let Some(timeout) = self.timeout {
            // User provided --timeout
            // Use it for both download and upload
            (timeout, timeout)
        } else {
            // No --timeout provided, use individual timeout settings
            (self.download_timeout, self.upload_timeout)
        };

        crate::core::SpeedTestConfig {
            server_url: self.server_url.clone(),
            download_timeout,
            upload_timeout,
            concurrent: self.concurrent,
            download_size: self.download_size,
            upload_size: self.upload_size,
            max_latency: Some(self.max_latency),
            min_download_speed: Some(self.min_download_speed * 1024.0 * 1024.0), // Convert MB/s to bytes/s
            min_upload_speed: Some(self.min_upload_speed * 1024.0 * 1024.0), // Convert MB/s to bytes/s
            fast_mode: self.fast_mode,
        }
    }
}
