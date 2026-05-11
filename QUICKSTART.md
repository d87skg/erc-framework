# ERC Framework 快速入门

5 分钟内启动 ERC 审计框架，为你的 AI 应用添加不可篡改的审计轨迹。

## 前置要求

- Docker 和 Docker Compose 已安装
- 拥有一个 AI API Key（DeepSeek / OpenAI / Gemini 任选其一）

## 步骤一：获取代码

```bash
git clone https://github.com/your-org/erc-framework.git
cd erc-framework
```

## 步骤二：配置环境变量

```bash
cp .env.example .env
```

编辑 `.env` 文件，填入你的配置：

```bash
# 必填：你的 AI API Key
UPSTREAM_API_KEY=sk-your-actual-api-key

# 必填：设置一个审计 Token（用于访问查询 API）
ERC_API_TOKEN=your-secure-token-here

# 可选：允许的上游域名（默认 api.deepseek.com）
ALLOWED_UPSTREAM_HOSTS=api.deepseek.com
```

## 步骤三：一键启动

```bash
docker compose up -d
```

等待约 30 秒，让服务完全启动。

## 步骤四：验证运行

检查 Gateway 健康状态：

```bash
curl http://localhost:8080/health
# 应返回: "ERC Gateway: OK"
```

检查 Query API 健康状态：

```bash
curl http://localhost:8082/health
# 应返回: "OK"
```

## 步骤五：发送第一个 AI 请求

```bash
curl -X POST http://localhost:8080/v1/messages \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer sk-your-actual-api-key" \
  -d '{
    "model": "deepseek-chat",
    "messages": [
      {"role": "user", "content": "Hello, what is 1+1?"}
    ]
  }'
```

响应中会包含 `trace_id`，例如：`"trace_id": "abc123..."`

## 步骤六：查看审计轨迹

### 方式一：使用 Trace Viewer（推荐）

浏览器打开 [http://localhost:8082/trace-viewer.html](http://localhost:8082/trace-viewer.html)，输入上一步返回的 `trace_id`，即可查看完整的时间线。

### 方式二：使用 API

```bash
# 查询因果链
curl "http://localhost:8082/api/v1/traces/causality?id=YOUR_TRACE_ID&token=your-secure-token-here"

# 查询执行事件
curl "http://localhost:8082/api/v1/executions/events?id=YOUR_EXECUTION_ID&token=your-secure-token-here"

# 查询收据
curl "http://localhost:8082/api/v1/receipts?id=YOUR_RECEIPT_ID&token=your-secure-token-here"
```

## 离线使用（本地 AI 模型）

如果你使用本地部署的 AI 模型（如 Ollama、vLLM），只需修改 `.env`：

```bash
# 指向本地模型地址
UPSTREAM_URL=http://localhost:11434/v1/chat/completions

# 允许本地域名
ALLOWED_UPSTREAM_HOSTS=localhost
```

然后重启服务：

```bash
docker compose down
docker compose up -d
```

即可完全离线运行，无需任何外部 API。

## 常用命令

```bash
# 查看日志
docker compose logs -f

# 停止服务
docker compose down

# 重新构建并启动
docker compose up -d --build

# 查看运行状态
docker compose ps
```

## 下一步

- 阅读 [架构文档](docs/ARCHITECTURE.md) 了解 ERC 内部原理
- 查看 [API 参考](docs/API.md) 了解所有可用接口
- 集成 [Python SDK](erc-sdk-py/) 到你的应用

## 故障排除

### 服务无法启动

1. 检查端口是否被占用：`netstat -ano | findstr :8080`
2. 检查 Docker 是否运行：`docker info`
3. 查看详细日志：`docker compose logs gateway`

### 请求返回 401

确保 `Authorization` Header 中的 API Key 与上游服务一致。

### 无法查看审计轨迹

1. 确保 `ERC_API_TOKEN` 已设置
2. 确保使用正确的 `token` 参数访问 Query API