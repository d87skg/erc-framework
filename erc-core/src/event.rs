use serde::{Serialize, Deserialize};
#[allow(dead_code)]

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum ExecutionEvent {
    #[serde(rename = "execution.started")]
    Started {
        execution_id: String,
        trace_id: String,
        parent_execution_id: Option<String>,
        actor: Actor,
        timestamp: i64,
    },
    #[serde(rename = "llm.call.completed")]
    LlmCallCompleted {
        execution_id: String,
        model: String,
        prompt_hash: String,
        response_hash: String,
        duration_ms: u64,
    },
    #[serde(rename = "tool.call.completed")]
    ToolCallCompleted {
        execution_id: String,
        tool_name: String,
        input_hash: String,
        output_hash: String,
    },
    #[serde(rename = "execution.completed")]
    Completed {
        execution_id: String,
        receipt_id: String,
        timestamp: i64,
    },
    #[serde(rename = "policy.denied")]
    PolicyDenied {
        execution_id: String,
        rule: String,
        reason: String,
    },
    #[serde(rename = "approval.granted")]
    ApprovalGranted {
        execution_id: String,
        approver: Actor,
        timestamp: i64,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Actor {
    pub actor_type: ActorType,
    pub id: String,
    pub delegated_by: Option<String>,
    pub session_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ActorType {
    Human,
    Agent,
    System,
}