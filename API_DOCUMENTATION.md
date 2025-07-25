# Mihomo SpeedTest Rust - API 函数使用文档

## 模块结构概览

```
mihomo_speedtest_rs/
├── cli/           # CLI 相关模块
├── config/        # 配置加载和解析
├── core/          # 核心速度测试逻辑
├── network/       # 网络测试组件
└── output/        # 结果格式化和导出
```

## 核心 API

### 1. 配置模块 (`config`)

#### `ConfigLoader`

配置加载器，负责从各种源加载代理配置。

##### 构造函数

```rust
impl ConfigLoader {
    /// 创建新的配置加载器
    pub fn new() -> Self
}
```

##### 主要方法

```rust
/// 从路径加载配置（文件或 URL）
pub async fn load_from_path(&self, path: &str) -> Result<Vec<ProxyConfig>>

/// 从多个路径加载配置
pub async fn load_from_paths(&self, paths: &str) -> Result<Vec<ProxyConfig>>
```

**使用示例：**
```rust
use mihomo_speedtest_rs::config::ConfigLoader;

let loader = ConfigLoader::new();

// 从本地文件加载
let proxies = loader.load_from_path("config.yaml").await?;

// 从订阅 URL 加载
let proxies = loader.load_from_path("https://example.com/subscription").await?;

// 从多个源加载
let proxies = loader.load_from_paths("config1.yaml,https://sub1.com").await?;
```

**支持的格式：**
- 本地 YAML/JSON 文件
- HTTP(S) 订阅 URL
- Base64 编码的订阅内容
- 各种代理 URL 格式（ss://、trojan://、vmess:// 等）

#### `ProxyConfig`

代理配置结构体。

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub name: String,                    // 代理名称
    pub proxy_type: ProxyType,          // 代理类型
    pub server: String,                 // 服务器地址
    pub port: u16,                      // 端口
    pub config: ProxyParameters,        // 代理参数
}
```

#### `ProxyType`

支持的代理类型枚举。

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProxyType {
    Shadowsocks,    // Shadowsocks
    VMess,          // VMess
    VLESS,          // VLESS
    Trojan,         // Trojan
    Hysteria,       // Hysteria
    Hysteria2,      // Hysteria2
    WireGuard,      // WireGuard
    Socks5,         // SOCKS5
    Http,           // HTTP
    Https,          // HTTPS
    AnyTLS,         // AnyTLS (新增)
}
```

**协议支持状态：**
- ✅ **完全支持**: `Http`, `Https`, `Socks5` - 支持完整代理功能
- 🔧 **基础支持**: `Hysteria2`, `AnyTLS`, `Trojan`, `Shadowsocks`, `VMess`, `VLESS`, `WireGuard` - 支持配置解析和连通性测试

#### `ProxyParameters`

代理参数结构体，包含各种协议的特定配置。

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProxyParameters {
    // 通用 TLS 设置
    pub tls: Option<bool>,
    pub skip_cert_verify: Option<bool>,
    pub sni: Option<String>,
    
    // 认证信息
    pub username: Option<String>,
    pub password: Option<String>,
    pub uuid: Option<String>,
    
    // Shadowsocks 特定
    pub cipher: Option<String>,
    pub plugin: Option<String>,
    pub plugin_opts: Option<HashMap<String, serde_yaml::Value>>,
    
    // VMess/VLESS 特定
    pub alter_id: Option<u32>,
    pub security: Option<String>,
    pub flow: Option<String>,
    
    // 传输配置
    pub network: Option<String>,
    pub ws_opts: Option<HashMap<String, serde_yaml::Value>>,
    pub grpc_opts: Option<HashMap<String, serde_yaml::Value>>,
    pub h2_opts: Option<HashMap<String, serde_yaml::Value>>,
    
    // Hysteria 特定
    pub protocol: Option<String>,
    pub up: Option<String>,
    pub down: Option<String>,
    pub auth: Option<String>,
    pub auth_str: Option<String>,
    pub ca_str: Option<String>,
    
    // 通用字段
    pub udp: Option<bool>,
    pub tfo: Option<bool>,
    pub client_fingerprint: Option<String>,
    pub ports: Option<String>,  // Hysteria2 端口范围
    
    // TLS/连接优化 (新增)
    pub alpn: Option<Vec<String>>,
    pub fingerprint: Option<String>,
    pub mptcp: Option<bool>,
    pub ip_version: Option<String>,
    pub interface_name: Option<String>,
    pub routing_mark: Option<u32>,
    pub dialer_proxy: Option<String>,
    pub smux: Option<HashMap<String, serde_yaml::Value>>,
    
    // 其他未知字段
    pub extra: HashMap<String, serde_yaml::Value>,
}
```

**协议特定字段说明：**

- **Hysteria2**: `password`, `ports`, `skip_cert_verify`
- **AnyTLS**: `password`, `client_fingerprint`, `udp`, `tfo`, `skip_cert_verify`
- **Trojan**: `password`, `network`, `udp`, `skip_cert_verify`
- **Shadowsocks**: `cipher`, `password`, `plugin`, `plugin_opts`
- **VMess**: `uuid`, `alter_id`, `security`, `network`
- **VLESS**: `uuid`, `flow`, `network`

### 2. 核心模块 (`core`)

#### `SpeedTester`

核心速度测试引擎。

##### 构造函数

```rust
impl SpeedTester {
    /// 创建新的速度测试器
    pub fn new(config: SpeedTestConfig) -> Self
}
```

##### 主要方法

```rust
/// 测试单个代理
pub async fn test_proxy(&self, proxy: &ProxyConfig) -> Result<SpeedTestResult>

/// 测试多个代理（顺序执行）
pub async fn test_proxies(
    &self,
    proxies: Vec<ProxyConfig>,
    callback: Option<ProgressCallback>,
) -> Result<Vec<SpeedTestResult>>

/// 测试多个代理（并发执行）
pub async fn test_proxies_concurrent(
    &self,
    proxies: Vec<ProxyConfig>,
    max_concurrent: usize,
) -> Result<Vec<SpeedTestResult>>
```

**使用示例：**
```rust
use mihomo_speedtest_rs::core::{SpeedTester, SpeedTestConfig};

// 创建配置
let config = SpeedTestConfig {
    server_url: "https://speed.cloudflare.com".to_string(),
    timeout: Duration::from_secs(5),
    fast_mode: false,
    ..Default::default()
};

// 创建测试器
let tester = SpeedTester::new(config);

// 测试单个代理
let result = tester.test_proxy(&proxy).await?;

// 测试多个代理
let results = tester.test_proxies(proxies, None).await?;

// 并发测试
let results = tester.test_proxies_concurrent(proxies, 5).await?;
```

#### `SpeedTestConfig`

速度测试配置结构体。

```rust
#[derive(Debug, Clone)]
pub struct SpeedTestConfig {
    pub server_url: String,              // 测试服务器 URL
    pub timeout: Duration,               // 超时时间
    pub concurrent: usize,               // 并发连接数
    pub download_size: usize,            // 下载测试大小
    pub upload_size: usize,              // 上传测试大小
    pub max_latency: Option<Duration>,   // 最大延迟过滤
    pub min_download_speed: Option<f64>, // 最小下载速度过滤
    pub min_upload_speed: Option<f64>,   // 最小上传速度过滤
    pub fast_mode: bool,                 // 快速模式（仅测试延迟）
}
```

#### `SpeedTestResult`

测试结果结构体。

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedTestResult {
    pub proxy_name: String,              // 代理名称
    pub proxy_type: ProxyType,           // 代理类型
    pub latency: Option<Duration>,       // 延迟
    pub jitter: Option<Duration>,        // 抖动
    pub packet_loss: f64,                // 丢包率
    pub download_speed: f64,             // 下载速度（字节/秒）
    pub upload_speed: f64,               // 上传速度（字节/秒）
    pub download_time: Option<Duration>, // 下载耗时
    pub upload_time: Option<Duration>,   // 上传耗时
    pub error: Option<String>,           // 错误信息
    pub timestamp: DateTime<Utc>,        // 时间戳
}
```

##### 实用方法

```rust
impl SpeedTestResult {
    /// 创建失败结果
    pub fn failed(proxy_name: String, proxy_type: ProxyType, error: String) -> Self
    
    /// 格式化延迟显示
    pub fn format_latency(&self) -> String
    
    /// 格式化下载速度显示
    pub fn format_download_speed(&self) -> String
    
    /// 格式化上传速度显示
    pub fn format_upload_speed(&self) -> String
    
    /// 检查测试是否成功
    pub fn is_successful(&self) -> bool
}
```

### 3. 网络模块 (`network`)

#### `NetworkTester`

网络测试器，集成延迟和带宽测试。

```rust
impl NetworkTester {
    /// 创建新的网络测试器
    pub fn new(server_url: String, timeout: Duration) -> Self
    
    /// 测试延迟
    pub async fn test_latency(
        &self, 
        proxy: &ProxyConfig, 
        count: usize
    ) -> Result<LatencyResult>
    
    /// 测试下载速度
    pub async fn test_download(
        &self,
        proxy: &ProxyConfig,
        size: usize,
        concurrent: usize,
    ) -> Result<BandwidthResult>
    
    /// 测试上传速度
    pub async fn test_upload(
        &self,
        proxy: &ProxyConfig,
        size: usize,
    ) -> Result<BandwidthResult>
}
```

#### `ProxyClient`

代理客户端封装。

```rust
impl ProxyClient {
    /// 创建新的代理客户端
    pub fn new(proxy_config: ProxyConfig, timeout: Duration) -> Result<Self>
    
    /// 获取底层 HTTP 客户端
    pub fn client(&self) -> &reqwest::Client
    
    /// 获取代理配置
    pub fn proxy_config(&self) -> &ProxyConfig
    
    /// 发送 GET 请求
    pub async fn get(&self, url: &str) -> Result<reqwest::Response>
    
    /// 创建 POST 请求构建器
    pub fn post(&self, url: &str) -> reqwest::RequestBuilder
}
```

#### `LatencyResult`

延迟测试结果。

```rust
#[derive(Debug, Clone)]
pub struct LatencyResult {
    pub avg_latency: Duration,    // 平均延迟
    pub min_latency: Duration,    // 最小延迟
    pub max_latency: Duration,    // 最大延迟
    pub jitter: Duration,         // 抖动
    pub packet_loss: f64,         // 丢包率
    pub successful_pings: usize,  // 成功 ping 次数
    pub total_pings: usize,       // 总 ping 次数
}
```

#### `BandwidthResult`

带宽测试结果。

```rust
#[derive(Debug, Clone)]
pub struct BandwidthResult {
    pub speed: f64,               // 速度（字节/秒）
    pub duration: Duration,       // 耗时
    pub bytes_transferred: usize, // 传输字节数
}
```

### 4. 输出模块 (`output`)

#### `ResultFormatter`

结果格式化器。

```rust
impl ResultFormatter {
    /// 创建新的结果格式化器
    pub fn new(json_output: bool, use_colors: bool) -> Self
    
    /// 格式化结果
    pub fn format_results(&self, results: &[SpeedTestResult]) -> String
    
    /// 格式化摘要
    pub fn format_summary(&self, results: &[SpeedTestResult]) -> String
}
```

**使用示例：**
```rust
use mihomo_speedtest_rs::output::ResultFormatter;

// 创建格式化器
let formatter = ResultFormatter::new(false, true); // 表格输出，使用颜色

// 格式化结果
let output = formatter.format_results(&results);
println!("{}", output);

// 格式化摘要
let summary = formatter.format_summary(&results);
println!("{}", summary);
```

#### `ConfigExporter`

配置导出器。

```rust
impl ConfigExporter {
    /// 导出 Clash 配置文件
    pub async fn export_clash_config<P: AsRef<Path>>(
        results: &[SpeedTestResult],
        original_proxies: &[ProxyConfig],
        output_path: P,
    ) -> Result<()>
    
    /// 导出 JSON 结果
    pub async fn export_json<P: AsRef<Path>>(
        results: &[SpeedTestResult],
        output_path: P,
    ) -> Result<()>
    
    /// 使用统计信息重命名代理
    pub fn rename_proxies_with_stats(
        original_proxies: &[ProxyConfig],
        results: &[SpeedTestResult],
    ) -> Vec<ProxyConfig>
}
```

**使用示例：**
```rust
use mihomo_speedtest_rs::output::ConfigExporter;

// 导出成功的代理配置
ConfigExporter::export_clash_config(
    &results,
    &proxies,
    "output.yaml"
).await?;

// 导出 JSON 结果
ConfigExporter::export_json(&results, "results.json").await?;

// 重命名代理并导出
let renamed_proxies = ConfigExporter::rename_proxies_with_stats(&proxies, &results);
ConfigExporter::export_clash_config(&results, &renamed_proxies, "renamed.yaml").await?;
```

## 完整使用示例

### 基本库使用

```rust
use mihomo_speedtest_rs::{
    config::ConfigLoader,
    core::{SpeedTester, SpeedTestConfig},
    output::{ResultFormatter, ConfigExporter},
};
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. 加载配置
    let loader = ConfigLoader::new();
    let proxies = loader.load_from_path("config.yaml").await?;
    
    // 2. 创建测试配置
    let config = SpeedTestConfig {
        server_url: "https://speed.cloudflare.com".to_string(),
        timeout: Duration::from_secs(10),
        fast_mode: true, // 仅测试延迟
        ..Default::default()
    };
    
    // 3. 执行测试
    let tester = SpeedTester::new(config);
    let results = tester.test_proxies(proxies.clone(), None).await?;
    
    // 4. 过滤成功结果
    let successful_results: Vec<_> = results
        .into_iter()
        .filter(|r| r.is_successful())
        .collect();
    
    // 5. 格式化输出
    let formatter = ResultFormatter::new(false, true);
    println!("{}", formatter.format_results(&successful_results));
    
    // 6. 导出配置
    ConfigExporter::export_clash_config(
        &successful_results,
        &proxies,
        "filtered.yaml"
    ).await?;
    
    Ok(())
}
```

### 自定义进度回调

```rust
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

let counter = Arc::new(AtomicUsize::new(0));
let total = proxies.len();

let progress_callback = {
    let counter = counter.clone();
    Box::new(move |result: &SpeedTestResult| {
        let current = counter.fetch_add(1, Ordering::SeqCst) + 1;
        println!("测试进度: {}/{} - {}: {}", 
                current, total, result.proxy_name, 
                if result.is_successful() { "成功" } else { "失败" });
    })
};

let results = tester.test_proxies(proxies, Some(progress_callback)).await?;
```

### 并发测试示例

```rust
// 并发测试 5 个代理
let results = tester.test_proxies_concurrent(proxies, 5).await?;

// 处理结果
for result in &results {
    if result.is_successful() {
        println!("{}: {} - {}",
            result.proxy_name,
            result.format_latency(),
            result.format_download_speed()
        );
    } else {
        println!("{}: 测试失败 - {:?}",
            result.proxy_name,
            result.error
        );
    }
}
```

## 错误处理

所有异步函数都返回 `Result<T>` 类型，其中错误类型为 `anyhow::Error`。

```rust
use anyhow::Result;

// 错误处理示例
match loader.load_from_path("config.yaml").await {
    Ok(proxies) => println!("加载了 {} 个代理", proxies.len()),
    Err(e) => eprintln!("加载失败: {}", e),
}
```

## 类型导入

```rust
// 导入常用类型
use mihomo_speedtest_rs::{
    ProxyConfig, ProxyParameters, ProxyType,
    SpeedTestConfig, SpeedTestResult, SpeedTester,
    BandwidthResult, LatencyResult,
};
```

## 异步运行时要求

所有网络操作都是异步的，需要 Tokio 运行时：

```rust
#[tokio::main]
async fn main() {
    // 你的代码
}
```

或者：

```rust
fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        // 你的异步代码
    });
}
```