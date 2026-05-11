这份就是整合了你刚才确认的所有内容（项目简介、快速开始、架构图、文档导航、FAQ），重新梳理后的完整 `README.md`，逻辑顺序和层级都已经调整好了，可以直接复制使用。

```markdown
# ERC Framework — AI 审计网关与责任协议

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Security Audit](https://img.shields.io/badge/security-audited-green.svg)](AUDIT_REPORT.md)

**只需替换一行 `base_url`，为你的 AI 应用生成不可篡改的审计轨迹。**

ERC 是一个跨模型的 AI 执行责任基础设施。它不绑定任何单一模型提供商，以透明代理的方式自动记录每一笔 AI 调用的完整上下文，生成密码学可验证的执行回执（ExecutionReceipt），并提供独立的查询和验证接口。

## 核心特性

- **一行代码接入**：只需把 AI API 的 `base_url` 改为 ERC Gateway 地址，无需修改业务代码。
- **不可篡改的审计轨迹**：每次 AI 调用自动生成 `ExecutionReceipt`，包含 `actor`、`custody`、`policy`、数字签名等完整责任信息。
- **独立可验证**：任何第三方都可以使用 `3-Line Audit` 离线验证回执的真实性，无需信任 ERC 服务端。
- **多模型支持**：兼容 DeepSeek、OpenAI、Gemini 等主流 AI API，以及本地部署的模型（如 Ollama、vLLM）。
- **轻量级自托管**：提供 Docker 一键部署，数据保留在本地，完全由你掌控。

## 系统架构

```
                          ┌─────────────────────────────────┐
                          │        AI API 客户端             │
                          │  (OpenAI SDK / LangChain / …)   │
                          └──────────────┬──────────────────┘
                                         │
                               base_url = ERC Gateway
                                         │
                                         ▼
┌────────────────────────────────────────────────────────────────┐
│                      ERC Gateway (8080)                        │
│  • 透明代理，不改动请求/响应                                     │
│  • 自动录制 ExecutionEvent                                     │
│  • 自动生成并签名 ExecutionReceipt                              │
│  • IP 限流（SQLite 持久化）                                     │
│  • SSRF 白名单防护                                             │
└────────────────────────┬───────────────────────────────────────┘
                         │
                         ▼
┌────────────────────────────────────────────────────────────────┐
│                      ERC Store (SQLite)                        │
│  • 持久化 Event 流 (events 表)                                  │
│  • 持久化 Receipt (receipts 表)                                 │
│  • 原子 Nonce 分配 (nonce_counters 表)                         │
└────────────────────────┬───────────────────────────────────────┘
                         │
                         ▼
┌────────────────────────────────────────────────────────────────┐
│                    ERC Query API (8082)                        │
│  • RESTful 查询接口                                             │
│    GET /api/v1/receipts?id=...                                 │
│    GET /api/v1/traces/causality?id=...                         │
│    GET /api/v1/executions/events?id=...                        │
│  • API Token 认证                                              │
│  • CORS 严格化                                                 │
│  • 内置 Trace Viewer 静态页面                                   │
└────────────────────────┬───────────────────────────────────────┘
                         │
                         ▼
┌────────────────────────────────────────────────────────────────┐
│                     审计 / 合规 / 开发                          │
│  • 3-Line Audit 离线验证                                       │
│  • 审计报告导出 (规划中)                                        │
│  • LangChain Adapter (规划中)                                   │
└────────────────────────────────────────────────────────────────┘
```

## 快速开始

### 1. 前置要求
- Docker & Docker Compose
- 一个 AI API Key（DeepSeek / OpenAI / Gemini / 本地模型均可）

### 2. 获取代码
```bash
git clone https://github.com/your-org/erc-framework.git
cd erc-framework
```

### 3. 配置环境变量
```bash
cp .env.example .env
# 编辑 .env，填入你的 API Key 和审计 Token
```

### 4. 一键启动
```bash
docker compose up -d
```

### 5. 验证服务
```bash
curl http://localhost:8080/health   # 应返回 "ERC Gateway: OK"
curl http://localhost:8082/health   # 应返回 "OK"
```

### 6. 发送第一个 AI 请求并查看审计轨迹
```bash
# 发送请求
curl -X POST http://localhost:8080/v1/messages \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer sk-your-api-key" \
  -d '{"model":"deepseek-chat","messages":[{"role":"user","content":"Hello"}]}'

# 在浏览器中打开 Trace Viewer，输入响应中的 trace_id
# http://localhost:8082/trace-viewer.html
```

> 📖 详细的部署和配置说明请参阅 **[快速入门指南](QUICKSTART.md)**。

## 📚 文档导航

### 核心文档
- **[README](README.md)** — 项目概览、快速开始
- **[快速入门指南 (QUICKSTART)](QUICKSTART.md)** — 5 分钟部署教程

### 部署与配置
- **[Docker 部署向导 (deploy-wizard.html)](public/deploy-wizard.html)** — 离线可视化部署向导
- **[环境变量模板 (.env.example)](.env.example)** — 所有配置项说明
- **[Docker Compose 配置 (docker-compose.yml)](docker-compose.yml)** — 服务编排定义

### 治理与合规
- **[安全审计报告 (AUDIT_REPORT)](AUDIT_REPORT.md)** — 第三方安全审计结果
- **[功能验证清单 (FEATURE_VERIFICATION)](FEATURE_VERIFICATION.md)** — 核心功能逐项验证
- **[修复审查报告 (FIX_REVIEW)](FIX_REVIEW.md)** — 安全修复闭环证明
- **[企业部署手册 (enterprise-deployment)](enterprise-deployment.md)** — 风险透明表与合规映射
- **[Schema 冻结规则 (SCHEMA)](docs/SCHEMA.md)** — ERR/1.0 向后兼容承诺
- **[Event Model 规范 (EVENTS)](docs/EVENTS.md)** — IEM/1.0 事件语义定义

### 开发指南
- **[Rust 核心 (erc-core)](erc-core/)** — 签名、哈希、Receipt 结构
- **[Gateway (erc-gateway)](erc-gateway/)** — 透明代理与限流逻辑
- **[存储层 (erc-store)](erc-store/)** — SQLite 封装
- **[查询 API (erc-query-api)](erc-query-api/)** — RESTful 审计查询
- **[构建脚本 (build.ps1)](build.ps1)** — 纯净构建命令

## 常见问题 (FAQ)

### 部署时卡住或长时间无响应
通常是因为 Docker 网络或资源问题。尝试重启 Docker Desktop，然后重新执行 `docker compose up -d`。

### 构建时出现 "spurious network error" 或下载 crate 超时
Dockerfile 已内置国内 Cargo 镜像源，大多数情况可以稳定下载。如仍超时，可修改 Dockerfile 中的 `rsproxy.cn` 地址为其他可用镜像。

### 启动后 `curl http://localhost:8082/health` 返回空响应
检查 `docker compose logs query-api` 日志，如果看到 `Listening on 127.0.0.1:8082`，说明服务监听在了容器内部回环地址。请拉取最新代码，或手动将 `erc-query-api/src/main.rs` 中的绑定地址改为 `0.0.0.0:8082`。

### 如何离线使用（无外部网络）
将 `.env` 中的 `UPSTREAM_URL` 指向本地模型地址（如 `http://localhost:11434/v1/chat/completions`），并设置 `ALLOWED_UPSTREAM_HOSTS=localhost`，然后重建容器即可。

### 为什么我查不到审计记录？
确保每次 AI 请求都通过 Gateway (端口 8080) 发送，并且使用了正确的 `trace_id` 和 `ERC_API_TOKEN` 参数查询 Query API。

## 许可

本项目采用 [MIT License](LICENSE) 开源。核心协议与审计格式 (ERR/1.0) 永久冻结，确保生态兼容性。
```