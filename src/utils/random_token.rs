use rand::RngExt;
use sha2::{Digest, Sha256};

pub fn generate_token() -> String {
    let mut rng = rand::rng();
    let token: Vec<u8> = (0..64).map(|_| rng.random::<u8>()).collect();
    hex::encode(token)
}

pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}
