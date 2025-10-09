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
        let time_now = SystemTime::now();
        let since_epoch = time_now
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let timestamp = since_epoch.as_secs();

        let hash = format!("{}-{}-{}-{}", index, timestamp, data, prev_hash);
        let nonce = 0;

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
        Self::new(0, "Genesis Block".to_string(), "0".repeat(64))
    }
}
