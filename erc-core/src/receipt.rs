use serde::{Serialize, Deserialize};
use uuid::Uuid;


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExecutionReceipt {
    pub receipt_id: String,
    pub execution_id: String,
    pub trace_id: String,
    pub schema_version: String,

    pub actor: Actor,
    pub agent: AgentInfo,
    pub model: ModelInfo,
    pub action: ActionInfo,

    pub security: SecurityInfo,
    pub proof: ProofInfo,

    pub custody: CustodyChain,
    pub policy: PolicyInfo,

    pub extensions: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Actor {
    pub actor_type: String,
    pub id: String,
    pub delegated_by: Option<String>,
    pub session_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentInfo {
    pub id: String,
    pub agent_type: String,
    pub organization_id: Option<String>,
    pub public_key: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModelInfo {
    pub provider: String,
    pub name: String,
    pub version: String,
    pub parameters: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ActionInfo {
    pub kind: String,
    pub prompt: String,
    pub response: String,
    pub tools_called: Vec<String>,
    pub decision: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SecurityInfo {
    pub nonce: u64,
    pub signatures: Vec<SignatureEntry>,
    pub tee_attestation: Option<String>,
    pub timestamp_proof: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SignatureEntry {
    pub signer: String,
    pub algorithm: String,
    pub signature: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProofInfo {
    pub merkle_root: String,
    pub celestia_height: Option<u64>,
    pub celestia_namespace: String,
    pub celestia_tx_hash: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CustodyChain {
    pub previous_receipt_hash: Option<String>,
    pub derived_from: Option<String>,
    pub transferred_to: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PolicyInfo {
    pub risk_level: String,
    pub approvals_required: u32,
    pub compliance_tags: Vec<String>,
    pub jurisdiction: String,
}

impl ExecutionReceipt {
    pub fn new(
        execution_id: &str,
        trace_id: &str,
        actor: Actor,
        agent: AgentInfo,
        model: ModelInfo,
        action: ActionInfo,
        nonce: u64,
        custody: CustodyChain,
    ) -> Self {
        Self {
            receipt_id: Uuid::new_v4().to_string(),
            execution_id: execution_id.to_string(),
            trace_id: trace_id.to_string(),
            schema_version: "1.0.0".to_string(),
            actor,
            agent,
            model,
            action,
            security: SecurityInfo {
                nonce,
                signatures: vec![],
                tee_attestation: None,
                timestamp_proof: None,
            },
            proof: ProofInfo {
                merkle_root: String::new(),
                celestia_height: None,
                celestia_namespace: "0001".to_string(),
                celestia_tx_hash: None,
            },
            custody,
            policy: PolicyInfo {
                risk_level: "low".to_string(),
                approvals_required: 0,
                compliance_tags: vec![],
                jurisdiction: "unknown".to_string(),
            },
            extensions: serde_json::Value::Object(serde_json::Map::new()),
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).expect("Bincode serialization failed")
    }
}