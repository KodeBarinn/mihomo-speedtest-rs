# Mihomo SpeedTest Rust

[![Crates.io](https://img.shields.io/crates/v/mihomo-speedtest-rs.svg)](https://crates.io/crates/mihomo-speedtest-rs)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0)
[![Build Status](https://img.shields.io/github/actions/workflow/status/KodeBarinn/mihomo-speedtest-rs/ci.yml?branch=main)](https://github.com/KodeBarinn/mihomo-speedtest-rs/actions)

一个快速、准确的 Clash/Mihomo 代理服务器速度测试工具，使用 Rust 编写。

## 特性

- 🚀 **高性能**: 使用 Rust 和 Tokio 异步运行时，支持并发测试
- 📊 **全面测试**: 延迟、抖动、丢包率、下载/上传速度测试
- 🔥 **真实代理测试**: 支持通过 mihomo 进程进行真实的代理性能测试
- 🔧 **多格式支持**: 支持 YAML、JSON 配置文件和各种订阅格式
- 🌐 **协议支持**: Shadowsocks、VMess、Trojan、Hysteria、SOCKS5、HTTP 等
- 📈 **智能过滤**: 基于性能指标自动过滤节点
- 💾 **结果导出**: 支持导出过滤后的配置文件和测试结果
- 🎨 **美观输出**: 彩色表格和 JSON 格式输出

## 安装

### 通过 Cargo 安装（推荐）

从 [crates.io](https://crates.io/crates/mihomo-speedtest-rs) 直接安装最新版本：

```bash
cargo install mihomo-speedtest-rs
```

> **要求**: 需要安装 [Rust](https://rustup.rs/) 工具链

### 从源码编译

```bash
git clone https://github.com/KodeBarinn/mihomo-speedtest-rs.git
cd mihomo-speedtest-rs
cargo build --release
```

### 运行

```bash
# 通过 cargo install 安装后直接使用
mihomo-speedtest --help

# 或在开发环境中使用 cargo run
cargo run -- --help

# 或使用编译后的二进制文件
./target/release/mihomo-speedtest --help
```

## 快速开始

### 测试模式对比

本工具提供两种测试模式：

#### 1. 标准模式 (默认)
- **HTTP/SOCKS5**: 真实代理测试
- **其他协议**: 基础连通性测试（直连方式）
- **特点**: 快速测试，获得基线性能指标

#### 2. Mihomo 模式 (推荐)  
- **所有协议**: 通过 mihomo 进程进行真实代理测试
- **特点**: 获得真实用户体验的测试结果
- **要求**: 需要安装 mihomo 二进制文件

### 基本用法

```bash
# 标准模式测试
mihomo-speedtest --config clash/config.yaml

# 真实代理测试 (推荐)
mihomo-speedtest --config clash/config.yaml --use-mihomo

# 快速延迟测试
mihomo-speedtest --config config.yaml --fast

# 真实代理快速测试
mihomo-speedtest --config config.yaml --use-mihomo --fast

# 过滤高质量节点并导出
mihomo-speedtest --config config.yaml \
  --max-latency 300ms \
  --min-download-speed 10 \
  --output filtered.yaml
```

### Mihomo 模式使用

首先安装 mihomo：

```bash
# macOS
brew install mihomo

# 或手动下载
wget https://github.com/MetaCubeX/mihomo/releases/latest/download/mihomo-darwin-amd64.gz
gunzip mihomo-darwin-amd64.gz
chmod +x mihomo-darwin-amd64
sudo mv mihomo-darwin-amd64 /usr/local/bin/mihomo
```

然后使用 mihomo 模式进行真实测试：

```bash
# 基本真实测试
mihomo-speedtest --config config.yaml --use-mihomo --fast

# 自定义 mihomo 配置
mihomo-speedtest --config config.yaml --use-mihomo \
  --mihomo-binary /usr/local/bin/mihomo \
  --mihomo-api-port 9091 \
  --mihomo-proxy-port 7891

# 测试订阅地址
mihomo-speedtest --config https://example.com/subscription --use-mihomo
```

### 高级用法

```bash
# 并发测试多个节点
mihomo-speedtest --config config.yaml \
  --max-concurrent 5 \
  --filter "(香港|新加坡)" \
  --block "测试|过期"

# 自定义测试参数
mihomo-speedtest --config config.yaml \
  --server-url https://speed.cloudflare.com \
  --download-size 104857600 \
  --upload-size 52428800 \
  --timeout 10 \
  --concurrent 8

# JSON 输出
mihomo-speedtest --config config.yaml --json > results.json

# 测试特定协议类型
mihomo-speedtest --config config.yaml --filter "hysteria2|anytls" --fast

# 测试所有 Trojan 节点
mihomo-speedtest --config config.yaml --filter "trojan" --verbose
```

### 新协议测试示例

```bash
# 仅测试 Hysteria2 节点
mihomo-speedtest --config config.yaml --fast \
  --filter ".*hy2.*|.*hysteria2.*" \
  --max-latency 500ms

# 测试 AnyTLS 节点性能
mihomo-speedtest --config config.yaml \
  --filter ".*anytls.*" \
  --min-download-speed 0 \
  --json

# 对比不同协议延迟
mihomo-speedtest --config config.yaml --fast \
  --filter "(trojan|hysteria2|anytls)" \
  --json | jq '.[] | {name: .proxy_name, type: .proxy_type, latency_ms: (.latency.nanos / 1000000)}'
```

## CLI 文档

完整的 CLI 使用文档请查看 [CLI_DOCUMENTATION.md](CLI_DOCUMENTATION.md)。

### 主要参数

| 参数 | 描述 | 默认值 |
|------|------|--------|
| `--config` | 配置文件路径或 URL | - |
| `--filter` | 正则表达式过滤代理名称 | `.+` |
| `--block` | 屏蔽关键词（用\|分隔） | - |
| `--fast` | 快速模式（仅测试延迟） | `false` |
| `--timeout` | 统一设置下载和上传超时（秒） | - |
| `--download-timeout` | 下载超时时间（秒） | `10` |
| `--upload-timeout` | 上传超时时间（秒） | `30` |
| `--max-latency` | 最大延迟过滤（毫秒） | `800` |
| `--min-download-speed` | 最小下载速度（MB/s） | `5` |
| `--min-upload-speed` | 最小上传速度（MB/s） | `2` |
| `--max-concurrent` | 最大并发测试数 | `1` |
| `--output` | 输出文件路径 | - |
| `--json` | JSON 格式输出 | `false` |

#### 超时参数说明

超时参数支持多种设置方式：

```bash
# 统一设置下载和上传超时为 15 秒
mihomo-speedtest --config config.yaml --timeout 15

# 分别设置下载和上传超时
mihomo-speedtest --config config.yaml --download-timeout 10 --upload-timeout 45

# 支持时间单位（可选）
mihomo-speedtest --config config.yaml --timeout 1m30s --max-latency 500ms

# 纯数字默认单位：超时参数为秒，延迟参数为毫秒
mihomo-speedtest --config config.yaml --timeout 20 --max-latency 800
```

**参数优先级**：
- 如果指定了 `--timeout`，将同时设置下载和上传超时
- 如果同时指定了 `--timeout` 和 `--download-timeout`/`--upload-timeout`，`--timeout` 优先生效

### Mihomo 模式参数

| 参数 | 描述 | 默认值 |
|------|------|--------|
| `--use-mihomo` | 启用 mihomo 真实代理测试 | `false` |
| `--mihomo-binary` | mihomo 二进制文件路径 | 自动检测 |
| `--mihomo-api-port` | mihomo API 端口 | `19090` |
| `--mihomo-proxy-port` | mihomo 代理端口 | `17890` |
| `--mihomo-config-dir` | mihomo 配置目录 | `./mihomo-temp` |

## API 文档

作为库使用的完整 API 文档请查看 [API_DOCUMENTATION.md](API_DOCUMENTATION.md)。

### 基本库使用

```rust
use mihomo_speedtest_rs::{
    config::ConfigLoader,
    core::{SpeedTester, SpeedTestConfig, MihomoRunner, RealSpeedTester},
    output::{ResultFormatter, ConfigExporter},
};
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 加载配置
    let loader = ConfigLoader::new();
    let proxies = loader.load_from_path("config.yaml").await?;
    
    // 标准模式测试
    let config = SpeedTestConfig {
        fast_mode: true,
        timeout: Duration::from_secs(5),
        ..Default::default()
    };
    let tester = SpeedTester::new(config.clone());
    let results = tester.test_proxies(proxies.clone(), None).await?;
    
    // Mihomo 真实代理测试
    let mihomo_runner = MihomoRunner::new(
        "./mihomo-temp",
        None,  // 自动检测 mihomo
        19090,  // API 端口
        17890,  // 代理端口
    )?;
    let mut real_tester = RealSpeedTester::new(mihomo_runner, config);
    let real_results = real_tester.test_proxies(&proxies).await?;
    
    // 格式化输出
    let formatter = ResultFormatter::new(false, true);
    println!("标准测试结果:");
    println!("{}", formatter.format_results(&results));
    println!("\nMihomo 真实测试结果:");
    println!("{}", formatter.format_results(&real_results));
    
    Ok(())
}
```

### 核心类型

- `ConfigLoader`: 配置加载器
- `SpeedTester`: 标准速度测试引擎
- `RealSpeedTester`: Mihomo 真实代理测试引擎
- `MihomoRunner`: Mihomo 进程管理器
- `SpeedTestResult`: 测试结果
- `ProxyConfig`: 代理配置
- `ResultFormatter`: 结果格式化器
- `ConfigExporter`: 配置导出器

## 支持的代理协议

### 标准模式

#### 完全支持（真实代理连接）
- ✅ **HTTP/HTTPS** - 支持基本认证，完整代理功能
- ✅ **SOCKS5** - 支持用户认证，完整代理功能

#### 基础支持（连通性测试）
- ✅ **Hysteria2** - 支持配置解析和基础连通性测试
- ✅ **AnyTLS** - 支持配置解析和基础连通性测试  
- ✅ **Trojan** - 支持配置解析和基础连通性测试
- ✅ **Shadowsocks** - 支持配置解析和基础连通性测试
- ✅ **VMess** - 支持配置解析和基础连通性测试
- ✅ **VLESS** - 支持配置解析和基础连通性测试
- ✅ **WireGuard** - 支持配置解析和基础连通性测试

### Mihomo 模式 (推荐)

#### 完全支持（真实代理连接）
- ✅ **所有协议** - 通过 mihomo 进程进行真实代理测试
- ✅ **Shadowsocks** - 完整的代理转发和性能测试
- ✅ **VMess** - 完整的代理转发和性能测试
- ✅ **VLESS** - 完整的代理转发和性能测试
- ✅ **Trojan** - 完整的代理转发和性能测试
- ✅ **Hysteria2** - 完整的代理转发和性能测试
- ✅ **AnyTLS** - 完整的代理转发和性能测试
- ✅ **WireGuard** - 完整的代理转发和性能测试
- ✅ **HTTP/HTTPS** - 完整的代理转发和性能测试
- ✅ **SOCKS5** - 完整的代理转发和性能测试

> **推荐**: 使用 `--use-mihomo` 选项获得所有协议的真实用户体验测试结果。

## 支持的配置格式

### 1. Clash YAML 配置

```yaml
proxies:
  # Shadowsocks
  - name: "香港节点1"
    type: ss
    server: example.com
    port: 443
    cipher: aes-256-gcm
    password: password123
    
  # Trojan
  - name: "美国节点1"
    type: trojan
    server: us.example.com
    port: 443
    password: trojan_password
    network: ws
    skip-cert-verify: true
    
  # Hysteria2
  - name: "高速节点1"
    type: hysteria2
    server: hy2.example.com
    port: 8443
    ports: 21000-26000
    password: your_password
    skip-cert-verify: true
    
  # AnyTLS
  - name: "AnyTLS节点1"
    type: anytls
    server: anytls.example.com
    port: 4430
    client-fingerprint: chrome
    password: your_password
    udp: true
    tfo: true
    skip-cert-verify: true
```

### 2. 订阅 URL

支持以下格式的代理 URL：
- `ss://` - Shadowsocks
- `trojan://` - Trojan
- `vmess://` - VMess (Base64 编码的 JSON)
- `vless://` - VLESS
- `hysteria://` - Hysteria (基础支持)
- `socks5://` - SOCKS5

**注意**: Hysteria2 和 AnyTLS 等新协议主要通过 YAML 配置文件支持。

### 3. Base64 编码订阅

自动检测和解码 Base64 编码的订阅内容。

## 测试结果示例

### 表格输出

```
┌─────────────────┬──────────┬───────────────┬───────────────┬─────────────┐
│ Proxy Name      │ Latency  │ Download      │ Upload        │ Status      │
├─────────────────┼──────────┼───────────────┼───────────────┼─────────────┤
│ 香港节点1       │ 45ms     │ 25.6 MB/s     │ 12.3 MB/s     │ ✅ Success  │
│ 日本节点1       │ 78ms     │ 18.9 MB/s     │ 8.7 MB/s      │ ✅ Success  │
│ 美国节点1       │ 156ms    │ 32.1 MB/s     │ 15.6 MB/s     │ ✅ Success  │
└─────────────────┴──────────┴───────────────┴───────────────┴─────────────┘

📊 测试摘要:
✅ 成功: 3/5 (60%)
⚡ 平均延迟: 93ms
📈 平均下载速度: 25.5 MB/s
📤 平均上传速度: 12.2 MB/s
```

### JSON 输出

```json
[
  {
    "proxy_name": "香港节点1",
    "proxy_type": "ss",
    "latency": {"secs": 0, "nanos": 45000000},
    "jitter": {"secs": 0, "nanos": 2000000},
    "packet_loss": 0.0,
    "download_speed": 26843545.6,
    "upload_speed": 12884901.888,
    "error": null,
    "timestamp": "2025-01-01T12:00:00Z"
  },
  {
    "proxy_name": "高速节点1",
    "proxy_type": "hysteria2",
    "latency": {"secs": 0, "nanos": 35000000},
    "jitter": {"secs": 0, "nanos": 1500000},
    "packet_loss": 0.0,
    "download_speed": 0.0,
    "upload_speed": 0.0,
    "error": null,
    "timestamp": "2025-01-01T12:00:05Z"
  },
  {
    "proxy_name": "AnyTLS节点1", 
    "proxy_type": "anytls",
    "latency": {"secs": 0, "nanos": 40000000},
    "jitter": {"secs": 0, "nanos": 2500000},
    "packet_loss": 0.0,
    "download_speed": 0.0,
    "upload_speed": 0.0,
    "error": null,
    "timestamp": "2025-01-01T12:00:10Z"
  }
]
```

## 配置文件

### 支持的环境变量

- `MIHOMO_CONFIG`: 默认配置文件路径
- `MIHOMO_SERVER_URL`: 默认测试服务器 URL
- `MIHOMO_TIMEOUT`: 默认超时时间

### 配置文件示例

创建 `mihomo-config.toml`:

```toml
[default]
server_url = "https://speed.cloudflare.com"
timeout = "10s"
max_latency = "500ms"
min_download_speed = 10.0
min_upload_speed = 5.0

[filters]
include = ["香港", "新加坡", "日本"]
exclude = ["测试", "过期", "免费"]
```

## 性能优化建议

1. **使用快速模式进行初步筛选**
   ```bash
   mihomo-speedtest --config config.yaml --fast --max-latency 300ms
   ```

2. **合理设置并发数**
   ```bash
   # 根据网络环境调整并发数
   mihomo-speedtest --config config.yaml --max-concurrent 3
   ```

3. **减少测试数据量**
   ```bash
   mihomo-speedtest --config config.yaml \
     --download-size 10485760 \
     --upload-size 5242880
   ```

4. **使用过滤器**
   ```bash
   mihomo-speedtest --config config.yaml \
     --filter "(香港|新加坡)" \
     --block "测试|过期"
   ```

## 故障排除

### 常见问题

1. **代理连接失败**
   - 检查代理配置是否正确
   - 验证网络连接
   - 增加超时时间

2. **配置解析错误**
   - 检查 YAML/JSON 格式
   - 验证必需字段
   - 查看详细错误信息

3. **性能测试不准确**
   - 选择合适的测试服务器
   - 避免网络高峰期测试
   - 增加测试数据量

### 调试模式

```bash
# 启用详细输出
mihomo-speedtest --config config.yaml --verbose

# 设置日志级别
RUST_LOG=debug mihomo-speedtest --config config.yaml
```

## 开发

### 项目结构

```
src/
├── cli/           # CLI 模块
│   ├── args.rs    # 命令行参数
│   ├── progress.rs # 进度显示
│   └── mod.rs
├── config/        # 配置模块
│   ├── loader.rs  # 配置加载器
│   └── mod.rs     # 配置类型定义
├── core/          # 核心模块
│   ├── speedtest.rs     # 标准速度测试引擎
│   ├── real_speedtest.rs # Mihomo 真实代理测试引擎
│   ├── mihomo_runner.rs # Mihomo 进程管理器
│   ├── statistics.rs    # 统计分析
│   └── mod.rs
├── network/       # 网络模块
│   ├── client.rs  # 代理客户端
│   ├── latency.rs # 延迟测试
│   ├── bandwidth.rs # 带宽测试
│   └── mod.rs
├── output/        # 输出模块
│   ├── formatter.rs # 结果格式化
│   ├── export.rs  # 配置导出
│   └── mod.rs
├── lib.rs         # 库入口
└── main.rs        # CLI 入口
```

### 运行测试

```bash
cargo test
```

### 构建发布版本

```bash
cargo build --release
```

## 贡献

欢迎提交 Issue 和 Pull Request！

### 开发环境要求

- Rust 1.70+
- Tokio 异步运行时
- 网络连接（用于测试）

### 代码规范

```bash
# 格式化代码
cargo fmt

# 检查代码
cargo clippy

# 运行测试
cargo test
```

## 许可证

本项目采用 [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0) 许可证

## 致谢

- [Clash](https://github.com/Dreamacro/clash) - 代理工具参考
- [Tokio](https://tokio.rs/) - 异步运行时
- [Reqwest](https://github.com/seanmonstar/reqwest) - HTTP 客户端
- [Clap](https://github.com/clap-rs/clap) - 命令行解析

---

## 相关链接

- [CLI 使用文档](CLI_DOCUMENTATION.md)
- [API 函数文档](API_DOCUMENTATION.md)
- [Mihomo 真实测速使用指南](MIHOMO_USAGE.md)
- [GitHub 仓库](https://github.com/KodeBarinn/mihomo-speedtest-rs)
- [问题反馈](https://github.com/KodeBarinn/mihomo-speedtest-rs/issues)