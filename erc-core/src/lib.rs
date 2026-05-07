use pyo3::prelude::*;
use ed25519_dalek::SigningKey;
use sha2::Sha256;
use sha2::Digest;

pub mod receipt;
pub mod event;
mod crypto;
mod merkle;

#[pyfunction]
fn generate_signed_receipt(
    prompt: &str,
    response: &str,
    model: &str,
    agent_id: &str,
    nonce: u64,
    secret_key: Vec<u8>,
) -> PyResult<(String, String)> {
    let actor = receipt::Actor {
        actor_type: "agent".to_string(),
        id: agent_id.to_string(),
        delegated_by: None,
        session_id: None,
    };
    let agent = receipt::AgentInfo {
        id: agent_id.to_string(),
        agent_type: "llm".to_string(),
        organization_id: None,
        public_key: String::new(),
    };
    let model_info = receipt::ModelInfo {
        provider: "unknown".to_string(),
        name: model.to_string(),
        version: "unknown".to_string(),
        parameters: serde_json::json!({}),
    };
    let action = receipt::ActionInfo {
        kind: "llm.call".to_string(),
        prompt: format!("sha256:{}", hex::encode(Sha256::digest(prompt.as_bytes()))),
        response: format!("sha256:{}", hex::encode(Sha256::digest(response.as_bytes()))),
        tools_called: vec![],
        decision: None,
    };
    let custody = receipt::CustodyChain {
        previous_receipt_hash: None,
        derived_from: None,
        transferred_to: None,
    };

    let mut receipt_obj = receipt::ExecutionReceipt::new(
        &uuid::Uuid::new_v4().to_string(),
        &uuid::Uuid::new_v4().to_string(),
        actor,
        agent,
        model_info,
        action,
        nonce,
        custody,
    );

    let canonical = receipt_obj.to_canonical_bytes();
    let sig = crypto::sign_and_clear(&canonical, secret_key)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;

    receipt_obj.security.signatures.push(receipt::SignatureEntry {
        signer: agent_id.to_string(),
        algorithm: "ed25519".to_string(),
        signature: hex::encode(&sig),
    });

    let json_data = receipt_obj.to_json();
    Ok((json_data, hex::encode(sig)))
}

#[pyfunction]
fn get_public_key(secret_key: Vec<u8>) -> PyResult<String> {
    if secret_key.len() != 32 {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>("私钥必须为32字节"));
    }
    let key_bytes: [u8; 32] = secret_key.as_slice().try_into()
        .map_err(|_| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("转换失败"))?;
    let sk = SigningKey::from_bytes(&key_bytes);
    Ok(hex::encode(sk.verifying_key().to_bytes()))
}

#[pyfunction]
fn verify_receipt_signature(
    json_data: &str,
    signature_hex: &str,
    public_key_hex: &str,
) -> PyResult<bool> {
    let sig_bytes = hex::decode(signature_hex)
        .map_err(|_| PyErr::new::<pyo3::exceptions::PyValueError, _>("无效签名"))?;
    let pub_bytes = hex::decode(public_key_hex)
        .map_err(|_| PyErr::new::<pyo3::exceptions::PyValueError, _>("无效公钥"))?;

    let receipt: receipt::ExecutionReceipt = serde_json::from_str(json_data)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("JSON解析失败: {}", e)))?;
    let canonical = receipt.to_canonical_bytes();

    crypto::verify(&canonical, &sig_bytes, &pub_bytes)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))
}

#[pymodule]
fn erc_core(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(generate_signed_receipt, m)?)?;
    m.add_function(wrap_pyfunction!(get_public_key, m)?)?;
    m.add_function(wrap_pyfunction!(verify_receipt_signature, m)?)?;
    Ok(())
}