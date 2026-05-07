use ed25519_dalek::{SigningKey, Signer, Verifier, Signature};
use zeroize::Zeroize;

pub fn sign_and_clear(data: &[u8], mut secret_bytes: Vec<u8>) -> Result<Vec<u8>, String> {
    if secret_bytes.len() != 32 {
        secret_bytes.zeroize();
        return Err("Secret key must be 32 bytes".into());
    }

    let key_array: [u8; 32] = secret_bytes.as_slice()
        .try_into()
        .map_err(|_| "Array conversion failed".to_string())?;
    let signing_key = SigningKey::from_bytes(&key_array);
    let signature = signing_key.sign(data);

    secret_bytes.zeroize();
    Ok(signature.to_vec())
}

pub fn verify(data: &[u8], sig_bytes: &[u8], pub_key_bytes: &[u8]) -> Result<bool, String> {
    let public_key = ed25519_dalek::VerifyingKey::from_bytes(
        pub_key_bytes.try_into().map_err(|_| "Invalid public key length")?,
    )
    .map_err(|e| e.to_string())?;

    let signature = Signature::from_slice(sig_bytes).map_err(|e| e.to_string())?;
    Ok(public_key.verify(data, &signature).is_ok())
}