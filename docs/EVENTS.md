# ERC Event Model v1.0.0

## 冻结声明
此文档定义的 6 种事件类型从即日起冻结。禁止删除或修改已有事件类型的必填字段。新增事件类型仅允许追加。

## 执行生命周期事件
| 事件类型 | 必填字段 | 说明 |
|----------|----------|------|
| `execution.started` | execution_id, trace_id, actor, timestamp | 执行开始 |
| `execution.completed` | execution_id, receipt_id, timestamp | 执行完成，关联回执 |

## LLM 调用事件
| 事件类型 | 必填字段 | 说明 |
|----------|----------|------|
| `llm.call.completed` | execution_id, model, prompt_hash, response_hash, duration_ms | LLM 调用完成 |

## 工具调用事件
| 事件类型 | 必填字段 | 说明 |
|----------|----------|------|
| `tool.call.completed` | execution_id, tool_name, input_hash, output_hash | 工具调用完成 |

## 策略与审批事件
| 事件类型 | 必填字段 | 说明 |
|----------|----------|------|
| `policy.denied` | execution_id, rule, reason | 策略拒绝 |
| `approval.granted` | execution_id, approver, timestamp | 审批通过 |