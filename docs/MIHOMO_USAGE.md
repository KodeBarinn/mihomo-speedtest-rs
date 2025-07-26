# Mihomo 真实测速功能使用指南

## 功能对比

### 原有功能 (默认模式)
```bash
# 直连测试 - 测试的是服务器的直连性能，不是代理性能
cargo run -- --config clash/config.yaml --fast
```
**局限性**：
- 对于 Shadowsocks、VMess、Trojan 等复杂协议，只能测试**直连性能**
- 无法体现真实的代理转发性能
- 无法测试代理的实际可用性和稳定性

### 新增功能 (Mihomo 模式) 
```bash
# 真实代理测试 - 通过mihomo进程测试真实的代理性能
cargo run -- --config clash/config.yaml --use-mihomo --fast
```
**优势**：
- 测试**真实的代理转发性能**
- 所有协议都通过 mihomo 进程进行真实代理测试
- 获得实际用户体验的测试结果

## 新增命令行选项

### 基本使用
```bash
# 启用 mihomo 真实测速
--use-mihomo

# 指定 mihomo 二进制文件路径（可选，默认自动查找）
--mihomo-binary /path/to/mihomo

# 配置 mihomo API 端口（默认 19090）
--mihomo-api-port 19090

# 配置 mihomo 代理端口（默认 17890）
--mihomo-proxy-port 17890

# 配置 mihomo 临时配置目录（默认 ./mihomo-temp）
--mihomo-config-dir ./mihomo-temp
```

## 使用示例

### 1. 基本真实测速
```bash
# 使用 mihomo 进行真实代理测试
cargo run -- --config clash/config.yaml --use-mihomo --fast

# 完整测试（包含带宽测试）
cargo run -- --config clash/config.yaml --use-mihomo
```

### 2. 自定义 mihomo 配置
```bash
# 指定自定义 mihomo 二进制和端口
cargo run -- --config clash/config.yaml --use-mihomo \
  --mihomo-binary /usr/local/bin/mihomo \
  --mihomo-api-port 9091 \
  --mihomo-proxy-port 7891
```

### 3. 与原有功能对比
```bash
# 原有直连测试
cargo run -- --config clash/config.yaml --fast --json > direct_test.json

# 新的真实代理测试  
cargo run -- --config clash/config.yaml --use-mihomo --fast --json > real_test.json

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

原有模式（基础连通性测试）：
✅ HTTP/HTTPS - 真实代理转发
✅ SOCKS5 - 真实代理转发
⚠️  其他协议 - 直连测试（非真实代理）
```

### 测试指标对比
```
指标类型          原有模式        Mihomo模式
延迟测试         直连延迟        真实代理延迟
抖动/丢包        直连测试        通过代理测试  
下载速度         直连速度        代理转发速度
上传速度         直连速度        代理转发速度
真实可用性       部分协议        所有协议
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
解决: 使用 --mihomo-api-port 和 --mihomo-proxy-port 指定其他端口
```

3. **权限问题**
```
错误: Permission denied
解决: 确保有权限访问配置目录和执行 mihomo
```

## 性能建议

- **快速测试**: 使用 `--fast` 只测试延迟
- **并发限制**: mihomo 模式目前是顺序测试，确保稳定性
- **资源清理**: 程序会自动清理临时文件和进程
- **端口配置**: 避免与现有服务冲突

这个新功能让您能够获得真正的用户体验测试结果，而不仅仅是服务器的基础连通性测试。