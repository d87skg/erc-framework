# 企业部署手册

## 环境要求
- Docker & Docker Compose
- 一个 AI API Key（DeepSeek / OpenAI / Gemini / 本地模型均可）

## 部署步骤
1. 克隆代码：git clone https://github.com/your-org/erc-framework.git
2. 配置环境变量：cp .env.example .env，编辑 .env 填入真实配置
3. 启动服务：docker compose up -d
4. 验证：curl http://localhost:8080/health
5. 查看审计轨迹：浏览器打开 http://localhost:8082/trace-viewer.html

## 安全注意事项
- 生产环境务必设置强密码 ERC_API_TOKEN
- 不要将 .env 文件提交到版本控制
- 建议使用非 root 用户运行容器
