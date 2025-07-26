use super::parameters::ParameterTable;
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

    /// Download size in MB for testing (supports decimal like 0.5)
    #[arg(long = "download-size", default_value = "50", value_parser = parse_size_mb)]
    pub download_size: usize,

    /// Upload size in MB for testing (supports decimal like 0.5)
    #[arg(long = "upload-size", default_value = "20", value_parser = parse_size_mb)]
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

/// Parse size in MB from string (supports decimal values like 0.5)
fn parse_size_mb(s: &str) -> Result<usize, String> {
    // Parse as a floating-point number in MB
    let mb = s
        .parse::<f64>()
        .map_err(|e| format!("Invalid size format: {}", e))?;

    if mb < 0.0 {
        return Err("Size cannot be negative".to_string());
    }

    // Convert MB to bytes (1 MB = 1024 * 1024 bytes)
    let bytes = (mb * 1024.0 * 1024.0) as usize;
    Ok(bytes)
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

    /// Create a parameter table showing default vs current values
    pub fn create_parameter_table(&self) -> ParameterTable {
        let mut table = ParameterTable::new();

        // Basic configuration parameters
        table.add_string_param(
            "config",
            "Required",
            self.config_paths.as_ref().unwrap_or(&"None".to_string()),
            "Configuration file path or URL",
        );

        table.add_string_param(
            "filter-regex",
            ".+",
            &self.filter_regex,
            "Filter proxies by name using regex",
        );

        table.add_optional_string_param(
            "block-keywords",
            None,
            &self.block_keywords,
            "Block proxies by keywords",
        );

        // Network configuration
        table.add_string_param(
            "server-url",
            "https://speed.cloudflare.com",
            &self.server_url,
            "Speed test server URL",
        );

        table.add_numeric_param(
            "download-size",
            50_usize,
            self.download_size / (1024 * 1024), // Convert bytes back to MB for display
            "Download size in MB for testing",
        );

        table.add_numeric_param(
            "upload-size",
            20_usize,
            self.upload_size / (1024 * 1024), // Convert bytes back to MB for display
            "Upload size in MB for testing",
        );

        // Timeout configuration
        table.add_duration_param(
            "download-timeout",
            Duration::from_secs(10),
            self.download_timeout,
            "Download timeout",
        );

        table.add_duration_param(
            "upload-timeout",
            Duration::from_secs(30),
            self.upload_timeout,
            "Upload timeout",
        );

        table.add_optional_duration_param(
            "timeout",
            None,
            self.timeout,
            "Unified timeout (overrides individual timeouts)",
        );

        // Performance parameters
        table.add_numeric_param(
            "concurrent",
            4_usize,
            self.concurrent,
            "Number of concurrent connections",
        );

        table.add_numeric_param(
            "max-concurrent",
            1_usize,
            self.max_concurrent,
            "Maximum proxies to test concurrently",
        );

        // Filtering thresholds
        table.add_duration_param(
            "max-latency",
            Duration::from_millis(800),
            self.max_latency,
            "Maximum allowed latency",
        );

        table.add_numeric_param(
            "min-download-speed",
            5.0_f64,
            self.min_download_speed,
            "Minimum download speed (MB/s)",
        );

        table.add_numeric_param(
            "min-upload-speed",
            2.0_f64,
            self.min_upload_speed,
            "Minimum upload speed (MB/s)",
        );

        // Mode flags
        table.add_bool_param(
            "fast-mode",
            false,
            self.fast_mode,
            "Fast mode: only test latency",
        );

        table.add_bool_param(
            "rename-nodes",
            false,
            self.rename_nodes,
            "Rename nodes with location and speed info",
        );

        table.add_bool_param(
            "stash-compatible",
            false,
            self.stash_compatible,
            "Enable Stash compatibility mode",
        );

        // Output options
        table.add_bool_param(
            "json-output",
            false,
            self.json_output,
            "Output results in JSON format",
        );

        table.add_bool_param("verbose", false, self.verbose, "Verbose output");

        table.add_optional_string_param("output", None, &self.output, "Output config file path");

        // Mihomo configuration
        table.add_bool_param(
            "use-mihomo",
            false,
            self.use_mihomo,
            "Use mihomo process for real proxy testing",
        );

        table.add_optional_string_param(
            "mihomo-binary",
            None,
            &self.mihomo_binary,
            "Path to mihomo binary",
        );

        table.add_numeric_param(
            "mihomo-api-port",
            19090_u16,
            self.mihomo_api_port,
            "Mihomo API port",
        );

        table.add_numeric_param(
            "mihomo-proxy-port",
            17890_u16,
            self.mihomo_proxy_port,
            "Mihomo proxy port",
        );

        table.add_string_param(
            "mihomo-config-dir",
            "./mihomo-temp",
            &self.mihomo_config_dir,
            "Mihomo config directory",
        );

        table
    }
}
