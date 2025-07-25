# Mihomo SpeedTest Rust - CLI 和函数使用文档

## 概述

Mihomo SpeedTest Rust 是一个快速、准确的 Clash/Mihomo 代理服务器速度测试工具。它支持多种代理协议，包括 Shadowsocks、VMess、Trojan、Hysteria2、AnyTLS 等，并提供详细的延迟、下载速度和上传速度测试。

## 支持的代理协议

### 完全支持（直接代理连接）
- ✅ **HTTP/HTTPS** - 支持基本认证，完整代理功能
- ✅ **SOCKS5** - 支持用户认证，完整代理功能

### 基础支持（连通性测试）
- ✅ **Hysteria2** - 支持配置解析和基础连通性测试
- ✅ **AnyTLS** - 支持配置解析和基础连通性测试  
- ✅ **Trojan** - 支持配置解析和基础连通性测试
- ✅ **Shadowsocks** - 支持配置解析和基础连通性测试
- ✅ **VMess** - 支持配置解析和基础连通性测试
- ✅ **VLESS** - 支持配置解析和基础连通性测试
- ✅ **WireGuard** - 支持配置解析和基础连通性测试

> **注意**: 基础支持协议目前使用直连进行连通性测试，可以获得基线延迟和基本网络性能指标。这些协议的完整代理功能实现正在开发中。

## 安装与运行

### 编译
```bash
cargo build --release
```

### 运行
```bash
cargo run -- [OPTIONS] --config <CONFIG_PATHS>
# 或
./target/release/mihomo-speedtest [OPTIONS] --config <CONFIG_PATHS>
```

## CLI 命令行参数

### 必需参数

#### `--config <CONFIG_PATHS>` / `-c <CONFIG_PATHS>`
指定配置文件路径或 HTTP(S) URL。

**支持的格式：**
- 本地文件路径：`clash/config.yaml`
- HTTP(S) 订阅地址：`https://example.com/subscription`
- Base64 编码的订阅内容
- 多个路径（逗号分隔）：`config1.yaml,config2.yaml`

**示例：**
```bash
# 本地配置文件
mihomo-speedtest --config clash/config.yaml

# 订阅地址
mihomo-speedtest --config https://example.com/subscription.txt

# 多个配置源
mihomo-speedtest --config "config.yaml,https://sub1.com,https://sub2.com"
```

### 过滤选项

#### `--filter <FILTER_REGEX>` / `-f <FILTER_REGEX>`
使用正则表达式过滤代理名称。

**默认值：** `.+`（匹配所有）

**示例：**
```bash
# 只测试包含 "香港" 的节点
mihomo-speedtest --config config.yaml --filter "香港"

# 测试美国和日本节点
mihomo-speedtest --config config.yaml --filter "(美国|日本)"

# 排除某些节点（使用负向先行断言）
mihomo-speedtest --config config.yaml --filter "^(?!.*测试).*"
```

#### `--block <BLOCK_KEYWORDS>` / `-b <BLOCK_KEYWORDS>`
根据关键词屏蔽代理（使用 `|` 分隔多个关键词）。

**示例：**
```bash
# 屏蔽包含 "测试" 或 "过期" 的节点
mihomo-speedtest --config config.yaml --block "测试|过期"

# 屏蔽免费节点
mihomo-speedtest --config config.yaml --block "免费|trial"
```

### 测试配置

#### `--server-url <SERVER_URL>`
指定速度测试服务器 URL。

**默认值：** `https://speed.cloudflare.com`

**示例：**
```bash
mihomo-speedtest --config config.yaml --server-url https://speed.example.com
```

#### `--download-size <DOWNLOAD_SIZE>`
设置下载测试的数据大小（字节）。

**默认值：** `52428800`（50MB）

**示例：**
```bash
# 使用 100MB 进行下载测试
mihomo-speedtest --config config.yaml --download-size 104857600
```

#### `--upload-size <UPLOAD_SIZE>`
设置上传测试的数据大小（字节）。

**默认值：** `20971520`（20MB）

**示例：**
```bash
# 使用 10MB 进行上传测试
mihomo-speedtest --config config.yaml --upload-size 10485760
```

#### `--timeout <TIMEOUT>`
设置每个测试的超时时间。

**默认值：** `5s`

**支持格式：** `5s`, `30s`, `2m`, `1h`

**示例：**
```bash
mihomo-speedtest --config config.yaml --timeout 10s
```

#### `--concurrent <CONCURRENT>`
设置测试时的并发连接数。

**默认值：** `4`

**示例：**
```bash
mihomo-speedtest --config config.yaml --concurrent 8
```

### 性能过滤

#### `--max-latency <MAX_LATENCY>`
过滤掉延迟超过指定值的代理。

**默认值：** `800ms`

**示例：**
```bash
mihomo-speedtest --config config.yaml --max-latency 500ms
```

#### `--min-download-speed <MIN_DOWNLOAD_SPEED>`
过滤掉下载速度低于指定值的代理（MB/s）。

**默认值：** `5`

**示例：**
```bash
mihomo-speedtest --config config.yaml --min-download-speed 10
```

#### `--min-upload-speed <MIN_UPLOAD_SPEED>`
过滤掉上传速度低于指定值的代理（MB/s）。

**默认值：** `2`

**示例：**
```bash
mihomo-speedtest --config config.yaml --min-upload-speed 5
```

### 运行模式

#### `--fast`
快速模式：仅测试延迟，跳过带宽测试。

**示例：**
```bash
mihomo-speedtest --config config.yaml --fast
```

#### `--max-concurrent <MAX_CONCURRENT>`
设置同时测试的代理数量。

**默认值：** `1`（顺序测试）

**示例：**
```bash
# 同时测试 5 个代理
mihomo-speedtest --config config.yaml --max-concurrent 5
```

### 输出选项

#### `--output <OUTPUT>` / `-o <OUTPUT>`
指定输出配置文件路径。

**示例：**
```bash
mihomo-speedtest --config config.yaml --output result.yaml
```

#### `--json` / `-j`
以 JSON 格式输出结果。

**示例：**
```bash
mihomo-speedtest --config config.yaml --json
```

#### `--verbose` / `-v`
启用详细输出。

**示例：**
```bash
mihomo-speedtest --config config.yaml --verbose
```

### 高级选项

#### `--rename`
使用位置和速度信息重命名节点。

**示例：**
```bash
mihomo-speedtest --config config.yaml --rename --output renamed.yaml
```

#### `--stash-compatible`
启用 Stash 兼容模式。

**示例：**
```bash
mihomo-speedtest --config config.yaml --stash-compatible
```

## 完整使用示例

### 基本用法
```bash
# 测试本地配置文件
mihomo-speedtest --config clash/config.yaml

# 测试订阅地址
mihomo-speedtest --config https://example.com/subscription
```

### 快速延迟测试
```bash
# 仅测试延迟
mihomo-speedtest --config config.yaml --fast

# 过滤低延迟节点
mihomo-speedtest --config config.yaml --fast --max-latency 200ms
```

### 完整速度测试
```bash
# 完整测试并输出到文件
mihomo-speedtest --config config.yaml \
  --output result.yaml \
  --min-download-speed 10 \
  --min-upload-speed 3
```

### 高性能测试
```bash
# 并发测试多个节点
mihomo-speedtest --config config.yaml \
  --max-concurrent 10 \
  --timeout 10s \
  --concurrent 8
```

### 自定义过滤
```bash
# 测试特定地区的高质量节点
mihomo-speedtest --config config.yaml \
  --filter "(香港|新加坡|日本)" \
  --block "测试|免费" \
  --max-latency 300ms \
  --min-download-speed 15
```

### JSON 输出
```bash
# 输出 JSON 格式结果
mihomo-speedtest --config config.yaml --json > results.json
```

## 支持的配置格式

### 1. Clash YAML 配置

#### Shadowsocks 配置
```yaml
proxies:
  - name: "香港节点1"
    type: ss
    server: example.com
    port: 443
    cipher: aes-256-gcm
    password: password123
```

#### Hysteria2 配置
```yaml
proxies:
  - name: "高速节点1"
    type: hysteria2
    server: hy2.example.com
    port: 8443
    ports: 21000-26000
    password: your_password
    skip-cert-verify: true
```

#### AnyTLS 配置
```yaml
proxies:
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

#### Trojan 配置
```yaml
proxies:
  - name: "Trojan节点1"
    type: trojan
    server: trojan.example.com
    port: 443
    password: your_password
    udp: true
    skip-cert-verify: true
    network: ws
```

#### VMess 配置
```yaml
proxies:
  - name: "VMess节点1"
    type: vmess
    server: vmess.example.com
    port: 443
    uuid: your-uuid-here
    alterId: 0
    cipher: auto
    tls: true
    skip-cert-verify: true
```

### 2. 订阅 URL 格式
支持以下代理 URL 格式：
- `ss://` - Shadowsocks
- `trojan://` - Trojan
- `vmess://` - VMess (Base64 编码的 JSON)
- `vless://` - VLESS
- `hysteria://` - Hysteria (基础支持)
- `socks5://` - SOCKS5

**注意**: Hysteria2 和 AnyTLS 协议通常通过 YAML 配置文件格式提供，订阅 URL 格式支持有限。

### 3. Base64 编码订阅
自动检测和解码 Base64 编码的订阅内容。

## 输出格式

### 表格输出（默认）
```
┌─────────────────┬──────────┬───────────────┬───────────────┬─────────────┐
│ Proxy Name      │ Latency  │ Download      │ Upload        │ Status      │
├─────────────────┼──────────┼───────────────┼───────────────┼─────────────┤
│ 香港节点1       │ 45ms     │ 25.6 MB/s     │ 12.3 MB/s     │ ✅ Success  │
│ 日本节点1       │ 78ms     │ 18.9 MB/s     │ 8.7 MB/s      │ ✅ Success  │
└─────────────────┴──────────┴───────────────┴───────────────┴─────────────┘
```

### JSON 输出
```json
[
  {
    "proxy_name": "香港节点1",
    "proxy_type": "Shadowsocks",
    "latency": "45ms",
    "jitter": "2ms",
    "packet_loss": 0.0,
    "download_speed": 26843545.6,
    "upload_speed": 12884901.888,
    "error": null,
    "timestamp": "2024-01-01T12:00:00Z"
  }
]
```

## 错误处理

### 常见错误和解决方案

1. **配置文件不存在**
   ```
   Error: Failed to read file config.yaml: No such file or directory
   ```
   解决：检查文件路径是否正确

2. **网络连接失败**
   ```
   Error: HTTP error 404: Not Found
   ```
   解决：检查订阅 URL 是否有效

3. **解析错误**
   ```
   Error: Failed to parse proxies section: missing field 'server'
   ```
   解决：检查配置文件格式是否正确

4. **代理连接失败**
   ```
   Latency test failed: Connection timeout
   ```
   解决：检查代理服务器是否可用，或增加超时时间

## 性能优化建议

1. **使用快速模式进行初步筛选**
   ```bash
   mihomo-speedtest --config config.yaml --fast --max-latency 300ms
   ```

2. **合理设置并发数量**
   ```bash
   # 根据系统性能调整
   mihomo-speedtest --config config.yaml --max-concurrent 5
   ```

3. **减少测试数据大小以加快测试**
   ```bash
   mihomo-speedtest --config config.yaml \
     --download-size 10485760 \
     --upload-size 5242880
   ```

4. **使用过滤器减少测试节点**
   ```bash
   mihomo-speedtest --config config.yaml \
     --filter "香港|新加坡" \
     --block "测试|过期"
   ```

## 注意事项

1. **资源使用**：并发测试会消耗更多网络带宽和系统资源
2. **测试准确性**：测试结果可能受网络状况和服务器负载影响
3. **配置兼容性**：支持标准 Clash 配置格式
4. **代理协议**：HTTP/HTTPS 和 SOCKS5 协议支持完整代理功能。其他协议（Hysteria2、AnyTLS、Trojan 等）目前进行基础连通性测试，使用直连获得基线性能指标