# ERC Framework 安全修复自审查报告

## 修复概述

本次修复基于安全审计报告，针对9个关键安全问题进行了修复。

## 修复清单

### ✅ F-01: SSRF白名单（Critical）
**文件**: `erc-gateway/src/proxy.rs`
**修复内容**:
- 添加 `ALLOWED_UPSTREAM_HOSTS` 环境变量支持
- 实现 `validate_upstream_url()` 函数验证上游URL
- 在代理请求前验证URL是否在白名单内
- 拒绝非白名单域名的请求

**验证**: cargo check通过

---

### ✅ F-02: 限流竞态（Medium）
**文件**: `erc-gateway/src/main.rs`
**修复内容**:
- 使用SQLite事务包裹限流检查和计数更新
- 添加 `BEGIN IMMEDIATE` 获取写锁
- 使用 `UPDATE ... RETURNING` 原子获取更新后值
- 事务提交确保原子性

**验证**: cargo check通过

---

### ✅ F-03: Panic消除（Medium）
**文件**: `erc-core/src/receipt.rs`, `erc-store/src/lib.rs`, `erc-gateway/src/recorder.rs`
**修复内容**:
- `canonical_bytes()`: 返回 `Result<Vec<u8>, String>` 替代直接unwrap
- `to_json()`: 返回 `Result<String, String>` 替代直接unwrap
- `bincode::serialize()`: 使用map_err转换错误
- `serde_json::to_string()`: 使用unwrap_or_else优雅降级
- `db.query_row()`: 使用unwrap_or_else优雅降级

**验证**: cargo check通过

---

### ✅ F-04: 请求体大小限制（Medium）
**文件**: `erc-gateway/src/proxy.rs`
**修复内容**:
- 添加 `MAX_REQUEST_BODY_SIZE` 常量 (10MB)
- 在读取请求体前检查Content-Length
- 读取后验证实际大小
- 超限返回413 Payload Too Large

**验证**: cargo check通过

---

### ✅ F-05: 上游响应验证（Medium）
**文件**: `erc-gateway/src/proxy.rs`
**修复内容**:
- 添加 `MAX_RESPONSE_BODY_SIZE` 常量 (50MB)
- 实现 `validate_upstream_response()` 函数
- 验证响应体大小
- 验证JSON结构合法性
- 检查是否包含意外字段（sql, exec, command, eval, script）

**验证**: cargo check通过

---

### ✅ F-06: 数据库文件权限（Medium）
**文件**: `erc-gateway/src/main.rs`
**修复内容**:
- 在Windows上使用 `set_permissions` 设置只读
- 在Unix上使用 `Command::new("chmod")` 设置0600权限
- 权限设置失败时打印警告但不终止

**验证**: cargo check通过

---

### ✅ F-07: 构建锁版本（Medium）
**文件**: `build.ps1`
**修复内容**:
- 添加 `--locked` 标志到cargo build命令
- 确保依赖版本与Cargo.lock一致
- 防止供应链攻击

**验证**: cargo check --release --locked通过

---

### ✅ F-08: Nonce原子递增（Medium）
**文件**: `erc-gateway/src/recorder.rs`
**修复内容**:
- 使用SQLite UPSERT实现原子递增
- `INSERT ... ON CONFLICT DO UPDATE SET nonce = nonce + 1 RETURNING nonce`
- 无需应用层锁，数据库保证原子性

**验证**: cargo check通过

---

### ✅ F-09: bincode固定整数编码（Low）
**文件**: `erc-core/src/receipt.rs`
**修复内容**:
- 创建 `bincode_config()` 函数
- 使用 `with_fixint_encoding()` 固定整数编码
- 使用 `with_little_endian()` 小端序
- 使用 `with_limit()` 设置大小限制
- 所有bincode调用使用统一配置

**验证**: cargo check通过

---

## 编译验证

```bash
cargo check --release --locked
```

**结果**: ✅ 编译成功，无错误

```
Checking erc-gateway v0.1.0 (D:\erc-project\erc-gateway)
    Finished `release` profile [optimized] target(s) in 0.80s
```

## 依赖更新

- 添加 `url = "2.5"` 到 erc-gateway/Cargo.toml
- 添加 `tracing = "0.1"` 到 erc-core/Cargo.toml
- 添加 `tracing = "0.1"` 到 erc-store/Cargo.toml
- 更新 tokio v1.52.2 -> v1.52.3
- 更新 hashbrown v0.17.0 -> v0.17.1
- 更新 cc v1.2.61 -> v1.2.62

## 遗留问题

以下问题需要后续处理：

1. **Stripe Webhook签名验证** - 需要Stripe集成后实现
2. **Webhook重放攻击防护** - 需要Stripe集成后实现
3. **SaaS管理后台JWT认证** - 需要管理后台开发时实现
4. **Token过期/刷新机制** - 需要后续增强
5. **并发限流测试** - 建议编写100并发测试用例
6. **cargo audit定期执行** - 建议集成到CI/CD

## 安全建议

1. 生产部署时使用专用用户 `erc-gateway`
2. 生产环境设置数据库文件权限为600
3. 日志审查：禁止打印prompt/response原文
4. 定期运行 `cargo audit` 检查CVE
5. 使用环境变量注入Stripe Public Key，不硬编码

## 签名

- 审查日期: 2026-05-11
- 审查工具: Cline AI Security Auditor
- 修复版本: v0.1.0-patch1