# 使用示例指南

本指南展示如何使用 mihomo-speedtest-rs 进行各种测试场景。

## 基本使用

### 1. 快速测试 (标准模式)

```bash
# 测试所有代理的延迟
cargo run -- --config example-config.yaml --fast

# 测试完整性能 (延迟 + 带宽)
cargo run -- --config example-config.yaml
```

### 2. 真实代理测试 (Mihomo 模式)

```bash
# 安装 mihomo (macOS)
brew install mihomo

# 真实代理延迟测试
cargo run -- --config example-config.yaml --use-mihomo --fast

# 真实代理完整测试
cargo run -- --config example-config.yaml --use-mihomo
```

## 过滤和筛选

### 按地区过滤

```bash
# 只测试香港和新加坡节点
cargo run -- --config example-config.yaml --filter "(HK|SG)" --fast

# 排除测试节点
cargo run -- --config example-config.yaml --block "测试|Test" --fast
```

### 按协议类型过滤

```bash
# 只测试 Shadowsocks 节点
cargo run -- --config example-config.yaml --filter "SS-" --fast

# 测试新协议 (Hysteria2, AnyTLS)
cargo run -- --config example-config.yaml --filter "(Hysteria2|AnyTLS)" --fast

# 比较不同协议性能
cargo run -- --config example-config.yaml --filter "VMess" --use-mihomo --fast
cargo run -- --config example-config.yaml --filter "Trojan" --use-mihomo --fast
```

## 性能筛选

### 基于延迟筛选

```bash
# 只保留延迟低于 200ms 的节点
cargo run -- --config example-config.yaml --max-latency 200ms --output fast-nodes.yaml

# 使用 mihomo 模式获得真实延迟
cargo run -- --config example-config.yaml --use-mihomo --max-latency 200ms --output real-fast-nodes.yaml
```

### 基于速度筛选

```bash
# 筛选高速节点 (下载 > 50MB/s, 上传 > 20MB/s)
cargo run -- --config example-config.yaml \
  --min-download-speed 50 \
  --min-upload-speed 20 \
  --output high-speed-nodes.yaml

# 真实代理速度测试
cargo run -- --config example-config.yaml --use-mihomo \
  --min-download-speed 30 \
  --min-upload-speed 10 \
  --output real-high-speed-nodes.yaml
```

## 并发测试

```bash
# 并发测试 3 个节点 (仅标准模式支持)
cargo run -- --config example-config.yaml --max-concurrent 3

# mihomo 模式目前只支持顺序测试以确保稳定性
cargo run -- --config example-config.yaml --use-mihomo
```

## 输出格式

### JSON 输出

```bash
# 标准模式 JSON 输出
cargo run -- --config example-config.yaml --fast --json > standard-results.json

# mihomo 模式 JSON 输出
cargo run -- --config example-config.yaml --use-mihomo --fast --json > real-results.json

# 比较两种模式的结果
jq '.[] | {name: .proxy_name, latency_ms: (.latency.nanos / 1000000)}' standard-results.json
jq '.[] | {name: .proxy_name, latency_ms: (.latency.nanos / 1000000)}' real-results.json
```

### 表格输出

```bash
# 彩色表格输出
cargo run -- --config example-config.yaml --fast

# 详细信息输出
cargo run -- --config example-config.yaml --verbose
```

## 高级用法

### 自定义测试参数

```bash
# 自定义下载/上传大小 (MB 格式) 和并发数
cargo run -- --config example-config.yaml \
  --download-size 100 \
  --upload-size 50 \
  --timeout 10s \
  --concurrent 8

# 支持小数值，适合快速测试
cargo run -- --config example-config.yaml \
  --download-size 0.5 \
  --upload-size 1.5 \
  --fast

# mihomo 模式自定义端口
cargo run -- --config example-config.yaml --use-mihomo \
  --mihomo-api-port 9091 \
  --mihomo-proxy-port 7891 \
  --mihomo-config-dir ./custom-mihomo-temp
```

### 自定义测试服务器

```bash
# 使用其他测试服务器
cargo run -- --config example-config.yaml \
  --server-url https://speed.cloudflare.com \
  --fast

# 使用 mihomo 模式测试自定义服务器
cargo run -- --config example-config.yaml --use-mihomo \
  --server-url https://your-speed-server.com
```

### 批量测试脚本

```bash
#!/bin/bash

echo "开始批量测试..."

# 标准模式快速测试
echo "标准模式测试..."
cargo run -- --config example-config.yaml --fast --json > standard.json

# mihomo 模式真实测试
echo "mihomo 真实测试..."
cargo run -- --config example-config.yaml --use-mihomo --fast --json > real.json

# 生成对比报告
echo "生成对比报告..."
echo "标准模式结果:"
jq '.[] | select(.error == null) | {name: .proxy_name, latency_ms: (.latency.nanos / 1000000)}' standard.json

echo -e "\nmihomo 真实测试结果:"
jq '.[] | select(.error == null) | {name: .proxy_name, latency_ms: (.latency.nanos / 1000000)}' real.json

echo "测试完成！"
```

## 故障排除

### 常见问题

1. **mihomo 二进制未找到**
   ```bash
   # 手动指定 mihomo 路径
   cargo run -- --config example-config.yaml --use-mihomo \
     --mihomo-binary /usr/local/bin/mihomo
   ```

2. **端口冲突**
   ```bash
   # 使用不同端口
   cargo run -- --config example-config.yaml --use-mihomo \
     --mihomo-api-port 9091 --mihomo-proxy-port 7891
   ```

3. **配置文件错误**
   ```bash
   # 启用详细日志
   cargo run -- --config example-config.yaml --verbose
   
   # 检查配置文件格式
   cargo run -- --config example-config.yaml --fast --json | jq .
   ```

### 调试模式

```bash
# 启用调试日志
RUST_LOG=debug cargo run -- --config example-config.yaml --use-mihomo --verbose

# 保留 mihomo 临时文件用于调试
ls -la mihomo-temp/
cat mihomo-temp/speedtest-config.yaml
```

## 性能对比

### 延迟对比

```bash
# 测试延迟差异
echo "测试标准模式 vs mihomo 模式延迟差异..."

# 标准模式 (直连延迟)
cargo run -- --config example-config.yaml --filter "SS-" --fast --json > ss-standard.json

# mihomo 模式 (真实代理延迟)  
cargo run -- --config example-config.yaml --filter "SS-" --use-mihomo --fast --json > ss-real.json

# 对比结果
echo "标准模式延迟:"
jq '.[0].latency.nanos / 1000000' ss-standard.json

echo "真实代理延迟:"
jq '.[0].latency.nanos / 1000000' ss-real.json
```

这个指南涵盖了所有主要使用场景，帮助用户充分利用 mihomo-speedtest-rs 的功能。