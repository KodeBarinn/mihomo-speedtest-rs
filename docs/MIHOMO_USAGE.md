# Mihomo 真实测速功能使用指南

## 功能对比

### 标准模式（兼容调试）
```bash
# 直连测试 - 测试的是服务器的直连性能，不是代理性能
cargo run -- --config clash/config.yaml --standard --fast
```
**局限性**：
- 仅 HTTP/SOCKS 适合直接测试
- 对于 Shadowsocks、VMess、Trojan 等复杂协议，不再伪装成真实代理测速
- 无法体现真实的代理转发性能

### 默认功能 (Mihomo 模式)
```bash
# 真实代理测试 - 通过mihomo进程测试真实的代理性能
cargo run -- --config clash/config.yaml --fast
```
**优势**：
- 测试**真实的代理转发性能**
- Mihomo/Clash YAML 代理描述原样交给 mihomo 验证，协议类型不再由 Rust 白名单限制
- 获得实际用户体验的测试结果
- 仅监听 `127.0.0.1`，使用随机端口和临时目录，不启用 TUN/DNS 接管，不修改系统代理

## 新增命令行选项

### 基本使用
```bash
# mihomo 真实测速是默认行为；该参数兼容保留
--use-mihomo

# 启用旧标准模式（仅 HTTP/SOCKS 调试）
--standard

# 启用并发隔离 lane，每个 lane 一个 mihomo 进程
--isolated-lanes --max-concurrent 4

# 指定 mihomo 二进制文件路径（可选，默认自动查找）
--mihomo-binary /path/to/mihomo

# 配置 mihomo API 端口（默认 0，随机空闲端口）
--mihomo-api-port 0

# 配置 mihomo 代理端口（默认 0，随机空闲端口）
--mihomo-proxy-port 0

# 配置 mihomo 临时会话目录的父目录（默认系统临时目录）
--mihomo-config-dir /tmp
```

## 使用示例

### 1. 基本真实测速
```bash
# 使用 mihomo 进行真实代理测试
cargo run -- --config clash/config.yaml --fast

# 完整测试（包含带宽测试）
cargo run -- --config clash/config.yaml
```

### 2. 自定义 mihomo 配置
```bash
# 指定自定义 mihomo 二进制和端口
cargo run -- --config clash/config.yaml \
  --mihomo-binary /usr/local/bin/mihomo \
  --mihomo-api-port 9091 \
  --mihomo-proxy-port 7891
```

### 3. 隔离 lane 并发
```bash
cargo run -- --config clash/config.yaml \
  --isolated-lanes \
  --max-concurrent 4
```

### 4. 与原有功能对比
```bash
# 原有直连测试
cargo run -- --config clash/config.yaml --standard --fast --json > direct_test.json

# 新的真实代理测试  
cargo run -- --config clash/config.yaml --fast --json > real_test.json

# 对比结果
diff direct_test.json real_test.json
```

## 测试流程说明

### Mihomo 模式下的测试流程：

1. **启动 mihomo 进程**
   - 自动生成临时配置文件
   - 启动 mihomo 进程并等待就绪
   - 通过 API 检查健康状态

2. **逐个测试代理**
   - 通过 mihomo API 切换到目标代理
   - 使用 mihomo 的内置延迟测试
   - 通过 mihomo 代理进行带宽测试
   - 获得真实的代理性能数据

3. **清理资源**
   - 自动停止 mihomo 进程
   - 清理临时配置文件

## 技术实现细节

### 代理支持情况
```
mihomo 模式（真实代理测试）：
✅ Shadowsocks - 真实代理转发
✅ VMess - 真实代理转发  
✅ VLESS - 真实代理转发
✅ Trojan - 真实代理转发
✅ Hysteria2 - 真实代理转发
✅ AnyTLS - 真实代理转发
✅ HTTP/HTTPS - 真实代理转发
✅ SOCKS5 - 真实代理转发

标准模式（兼容调试）：
✅ HTTP/HTTPS - 真实代理转发
✅ SOCKS5 - 真实代理转发
⚠️  其他协议 - 不支持真实代理测速，请使用默认 mihomo 模式
```

### 测试指标对比
```
指标类型          标准模式        Mihomo模式
延迟测试         HTTP/SOCKS代理   真实代理延迟
抖动/丢包        HTTP/SOCKS代理   通过代理测试
下载速度         HTTP/SOCKS代理   代理转发速度
上传速度         HTTP/SOCKS代理   代理转发速度
真实可用性       HTTP/SOCKS       mihomo 支持的所有协议
```

## 安装 mihomo

### macOS (Homebrew)
```bash
brew install mihomo
```

### 手动安装
```bash
# 下载最新版本
wget https://github.com/MetaCubeX/mihomo/releases/latest/download/mihomo-darwin-amd64.gz
gunzip mihomo-darwin-amd64.gz
chmod +x mihomo-darwin-amd64
sudo mv mihomo-darwin-amd64 /usr/local/bin/mihomo
```

### 验证安装
```bash
mihomo -v
```

## 故障排除

### 常见问题

1. **mihomo 二进制未找到**
```
错误: Mihomo binary not found
解决: 安装 mihomo 或使用 --mihomo-binary 指定路径
```

2. **端口占用**
```
错误: Timeout waiting for mihomo to start
解决: 默认会随机选择空闲端口；如需固定端口，可使用 --mihomo-api-port 和 --mihomo-proxy-port 指定其他端口
```

3. **权限问题**
```
错误: Permission denied
解决: 确保有权限访问配置目录和执行 mihomo
```

## 性能建议

- **快速测试**: 使用 `--fast` 只测试延迟
- **默认顺序**: 默认单 mihomo 进程顺序切换 selector，优先稳定性
- **隔离并发**: 使用 `--isolated-lanes --max-concurrent N` 启用多 mihomo 进程并发
- **资源清理**: 程序会自动清理临时文件和进程
- **端口配置**: 默认随机端口可避免与现有服务冲突

这个新功能让您能够获得真正的用户体验测试结果，而不仅仅是服务器的基础连通性测试。
