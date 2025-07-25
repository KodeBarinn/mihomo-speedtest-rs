# 更新日志 (CHANGELOG)

所有值得注意的项目更改都将记录在此文件中。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，
并且本项目遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

## [1.0.0] - 2025-07-26

🎉 **首次正式发布！**

### ✨ 核心功能

**mihomo-speedtest-rs** 是一个用 Rust 编写的高性能 Clash/Mihomo 代理测试工具，提供精确的延迟和带宽测试。

#### 🚀 双模式测试引擎
- **标准模式** - 直接连接测试，速度快，适用于基础连通性检查
- **Mihomo 模式** - 真实代理测试，通过 mihomo 进程获得准确的用户体验数据

#### 🌐 多协议支持
- **完全支持**: HTTP/HTTPS, SOCKS5（直接代理连接）
- **连通性测试**: Shadowsocks, VMess, VLESS, Trojan, Hysteria2, AnyTLS, WireGuard
- **真实测试（Mihomo 模式）**: 所有协议的完整代理转发测试

#### 📊 全面的性能指标
- **延迟测试**: 平均延迟、抖动、丢包率
- **带宽测试**: 下载和上传速度
- **统计分析**: 最小值、最大值、标准差

### 🛠️ 命令行功能

#### 基础参数
- `--config` - 支持本地 YAML 文件或订阅 URL
- `--filter` - 正则表达式过滤代理名称
- `--block` - 关键词屏蔽
- `--fast` - 快速模式（仅测试延迟）
- `--output` - 导出过滤后的配置文件
- `--json` - JSON 格式输出
- `--max-concurrent` - 控制并发测试数量

#### 🎯 灵活的超时控制
- `--timeout` - 统一设置下载和上传超时时间
- `--download-timeout` - 单独设置下载超时（默认: 10秒）
- `--upload-timeout` - 单独设置上传超时（默认: 30秒）
- `--max-latency` - 延迟过滤阈值（默认: 800毫秒）

**简化输入**: 支持纯数字输入，超时参数默认秒，延迟参数默认毫秒：
```bash
--timeout 15          # 15秒
--max-latency 500     # 500毫秒
```

#### 🔧 Mihomo 真实测试
- `--use-mihomo` - 启用 mihomo 真实代理测试模式
- `--mihomo-binary` - 指定 mihomo 二进制文件路径（支持自动检测）
- `--mihomo-api-port` - API 控制端口（默认: 19090）
- `--mihomo-proxy-port` - 代理监听端口（默认: 17890）
- `--mihomo-config-dir` - 临时配置目录

**端口冲突避免**: 使用非标准端口（19090/17890）避免与现有 mihomo 进程冲突

### 🏗️ 架构特性

#### 高性能异步设计
- 基于 Tokio 异步运行时
- 支持并发测试，可配置并发数量
- 智能资源管理和自动清理

#### 智能配置处理
- 支持 YAML 配置文件和 Base64 订阅
- 灵活的端口字段处理（支持字符串和数字格式）
- 智能代理协议检测

#### 可靠的进程管理
- 自动 mihomo 进程生命周期管理
- API 集成的代理切换和状态检测
- 自动临时文件和进程清理

### 📝 输出格式

#### 表格输出
- 彩色格式化表格
- 延迟、下载、上传速度展示
- 状态指示器

#### JSON 输出
- 完整的结构化数据
- 时间戳和详细错误信息
- 适合脚本集成和进一步处理

#### 配置导出
- 导出测试通过的代理配置
- 保持原始配置格式
- 支持速度和延迟过滤

### 🔍 使用示例

```bash
# 快速延迟测试
mihomo-speedtest --config config.yaml --fast

# 完整测试带过滤
mihomo-speedtest --config config.yaml --filter "香港|新加坡" --max-latency 300

# 真实代理测试（推荐）
mihomo-speedtest --config config.yaml --use-mihomo

# 自定义超时和导出
mihomo-speedtest --config config.yaml --timeout 20 --output fast-proxies.yaml

# JSON 输出用于脚本
mihomo-speedtest --config config.yaml --json > results.json
```

### 📦 技术栈

- **Rust** - 系统编程语言，保证性能和安全性
- **Tokio** - 异步运行时
- **Reqwest** - HTTP 客户端
- **Clap** - 命令行解析
- **Serde** - 序列化和反序列化
- **tracing** - 结构化日志

### 🎯 设计目标

- **性能优先**: Rust 原生性能，异步并发设计
- **准确性**: 真实代理测试模式提供准确的用户体验数据
- **易用性**: 直观的命令行界面和灵活的参数设置
- **可靠性**: 健壮的错误处理和资源管理
- **兼容性**: 支持所有主流代理协议和配置格式

---

这个版本标志着 mihomo-speedtest-rs 的首次正式发布，提供了完整、稳定、生产就绪的代理测试解决方案。