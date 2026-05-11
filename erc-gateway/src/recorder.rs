use erc_core::receipt::{
    ExecutionReceipt, Actor, AgentInfo, ModelInfo, ActionInfo, CustodyChain,
};
use erc_core::event::{
    ExecutionEvent, Actor as EventActor, ActorType as EventActorType,
};
use rusqlite::Connection;
use serde_json::Value;
use std::sync::Mutex;
use sha2::{Sha256, Digest};

pub struct Recorder {
    db: Mutex<Connection>,
    agent_id: String,
}

impl Recorder {
    pub fn new(db_path: &str, agent_id: &str) -> Self {
        let conn = Connection::open(db_path)
            .unwrap_or_else(|e| {
                tracing::error!("无法打开 SQLite 数据库 {}: {}", db_path, e);
                std::process::exit(1);
            });

        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                execution_id TEXT NOT NULL,
                trace_id TEXT NOT NULL,
                event_type TEXT NOT NULL,
                payload TEXT NOT NULL,
                timestamp INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS receipts (
                receipt_id TEXT PRIMARY KEY,
                execution_id TEXT NOT NULL,
                payload TEXT NOT NULL,
                timestamp INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS nonces (
                agent_id TEXT PRIMARY KEY,
                nonce INTEGER NOT NULL DEFAULT 0
            );
        ")
        .unwrap_or_else(|e| {
            tracing::error!("无法创建数据库表: {}", e);
            std::process::exit(1);
        });

        Self {
            db: Mutex::new(conn),
            agent_id: agent_id.to_string(),
        }
    }

    pub fn record_execution(
        &self,
        request: &Value,
        response: &Value,
    ) -> ExecutionReceipt {
        let execution_id = uuid::Uuid::new_v4().to_string();
        let trace_id = uuid::Uuid::new_v4().to_string();
        let timestamp = chrono::Utc::now().timestamp();

        // 提取 prompt 和 response 文本
        let prompt_text = request["messages"].to_string();
        let response_text = response["content"].to_string();

        let prompt_hash = format!("sha256:{}", hex::encode(
            Sha256::digest(prompt_text.as_bytes())
        ));
        let response_hash = format!("sha256:{}", hex::encode(
            Sha256::digest(response_text.as_bytes())
        ));

        // 记录事件: execution.started
        let started_event = ExecutionEvent::Started {
            execution_id: execution_id.clone(),
            trace_id: trace_id.clone(),
            parent_execution_id: None,
            actor: EventActor {
                actor_type: EventActorType::Agent,
                id: self.agent_id.clone(),
                delegated_by: None,
                session_id: Some(uuid::Uuid::new_v4().to_string()),
            },
            timestamp,
        };

        // 记录事件: llm.call.completed
        let completed_event = ExecutionEvent::LlmCallCompleted {
            execution_id: execution_id.clone(),
            model: "claude-3-opus".to_string(),
            prompt_hash: prompt_hash.clone(),
            response_hash: response_hash.clone(),
            duration_ms: 0,
        };

        // 写入 events 表
        {
            let db = self.db.lock().unwrap_or_else(|e| {
                tracing::error!("数据库锁被污染: {}", e);
                std::process::exit(1);
            });
            db.execute(
                "INSERT INTO events (execution_id, trace_id, event_type, payload, timestamp)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![
                    execution_id,
                    trace_id,
                    "execution.started",
                    serde_json::to_string(&started_event).unwrap_or_else(|e| {
                        tracing::error!("序列化started_event失败: {}", e);
                        String::from("{}")
                    }),
                    timestamp,
                ],
            )
            .ok();
            db.execute(
                "INSERT INTO events (execution_id, trace_id, event_type, payload, timestamp)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![
                    execution_id,
                    trace_id,
                    "llm.call.completed",
                    serde_json::to_string(&completed_event).unwrap_or_else(|e| {
                        tracing::error!("序列化completed_event失败: {}", e);
                        String::from("{}")
                    }),
                    timestamp,
                ],
            )
            .ok();
        }

        // 构建 Receipt
        let actor = Actor {
            actor_type: "agent".to_string(),
            id: self.agent_id.clone(),
            delegated_by: None,
            session_id: Some(uuid::Uuid::new_v4().to_string()),
        };
        let agent = AgentInfo {
            id: self.agent_id.clone(),
            agent_type: "llm".to_string(),
            organization_id: None,
            public_key: String::new(),
        };
        let model = ModelInfo {
            provider: "anthropic".to_string(),
            name: "claude-3-opus".to_string(),
            version: "20240229".to_string(),
            parameters: serde_json::json!({}),
        };
        let action = ActionInfo {
            kind: "llm.call".to_string(),
            prompt: prompt_hash,
            response: response_hash,
            tools_called: vec![],
            decision: None,
        };
        let custody = CustodyChain {
            previous_receipt_hash: None,
            derived_from: None,
            transferred_to: None,
        };

        // 获取原子递增的nonce
        let nonce: u64 = {
            let db = self.db.lock().unwrap_or_else(|e| {
                tracing::error!("数据库锁被污染: {}", e);
                std::process::exit(1);
            });
            db.query_row(
                "INSERT INTO nonces (agent_id, nonce)
                 VALUES (?1, 1)
                 ON CONFLICT(agent_id) DO UPDATE SET nonce = nonce + 1
                 RETURNING nonce",
                [&self.agent_id],
                |row| row.get(0),
            ).unwrap_or_else(|e| {
                tracing::error!("获取nonce失败: {}", e);
                0
            })
        };

        let receipt = ExecutionReceipt::new(
            &execution_id,
            &trace_id,
            actor,
            agent,
            model,
            action,
            nonce,
            custody,
        );

        // 写入 receipts 表
        {
            let db = self.db.lock().unwrap_or_else(|e| {
                tracing::error!("数据库锁被污染: {}", e);
                std::process::exit(1);
            });
            let receipt_json = receipt.to_json().unwrap_or_else(|e| {
                tracing::error!("序列化receipt失败: {}", e);
                String::from("{}")
            });
            db.execute(
                "INSERT INTO receipts (receipt_id, execution_id, payload, timestamp)
                 VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![
                    receipt.receipt_id,
                    execution_id,
                    receipt_json,
                    timestamp,
                ],
            )
            .ok();
        }

        receipt
    }
}