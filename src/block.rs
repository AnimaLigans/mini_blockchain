use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct Block {
    pub index: u32,
    pub timestamp: u64,
    pub data: String,
    pub prev_hash: String,
    pub hash: String,
    pub nonce: u32,
}
impl Block {
    pub fn new(index: u32, data: String, prev_hash: String) -> Block {
        let now = SystemTime::now();
        let since_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
        let timestamp = since_epoch.as_secs();

        let nonce = 0;
        let hash = Self::compute_hash(index, timestamp, &data, &prev_hash, nonce);

        Block {
            index,
            timestamp,
            data,
            prev_hash,
            hash,
            nonce,
        }
    }
    pub fn genesis() -> Self {
        Self::new(0, "GenesisBlock".to_string(), "0".repeat(64))
    }
    pub fn compute_hash(
        index: u32,
        timestamp: u64,
        data: &str,
        prev_hash: &str,
        nonce: u32,
    ) -> String {
        let input = format!("{}|{}|{}|{}|{}", index, timestamp, data, prev_hash, nonce);

        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        let bytes = hasher.finalize();
        hex::encode(bytes)
    }
    pub fn is_valid(&self, prev: &Block) -> bool {
        if self.index != prev.index + 1 {
            return false;
        }

        if self.prev_hash != prev.hash {
            return false;
        }

        let expected = Self::compute_hash(
            self.index,
            self.timestamp,
            &self.data,
            &self.prev_hash,
            self.nonce,
        );
        if self.hash != expected {
            return false;
        }

        true
    }
}
