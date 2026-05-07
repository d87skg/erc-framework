use sha2::{Sha256, Digest};

#[allow(dead_code)]
pub fn compute_merkle_root(leaves: &[Vec<u8>]) -> Vec<u8> {
    if leaves.is_empty() {
        return Sha256::digest(b"").to_vec();
    }
    let mut hashes: Vec<Vec<u8>> = leaves.iter()
        .map(|x| Sha256::digest(x).to_vec())
        .collect();

    while hashes.len() > 1 {
        let mut next = Vec::new();
        for chunk in hashes.chunks(2) {
            let mut hasher = Sha256::new();
            hasher.update(&chunk[0]);
            hasher.update(if chunk.len() > 1 { &chunk[1] } else { &chunk[0] });
            next.push(hasher.finalize().to_vec());
        }
        hashes = next;
    }
    hashes[0].clone()
}