//! forensics/hash.rs — SHA3-512 chained hash calculation.

use sha3::{Digest, Sha3_512};

pub const GENESIS_HASH: &str = "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";

pub fn hash_record(payload: &str, prev_hash: &str) -> String {
    let mut hasher = Sha3_512::new();
    hasher.update(prev_hash.as_bytes());
    hasher.update(payload.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genesis_hash_len() {
        assert_eq!(GENESIS_HASH.len(), 128);
    }
}
