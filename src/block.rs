#![allow(dead_code)]

use rand::Rng;
use secp256k1::{PublicKey, Secp256k1, SecretKey};
use sha2::{Digest, Sha256};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

// ========== TRANSACTION ==============
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: f64,
    pub timestamp: u64,
    pub signature: String,
    pub public_key: String,
}

impl Transaction {
    pub fn new(
        from: String,
        to: String,
        amount: f64,
        signature: String,
        public_key: String,
    ) -> Transaction {
        let now = SystemTime::now();
        let since_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
        let timestamp = since_epoch.as_secs();

        Transaction {
            from,
            to,
            amount,
            timestamp,
            signature,
            public_key,
        }
    }

    pub fn is_valid(&self) -> bool {
        if self.amount <= 0.0 {
            return false;
        }

        if self.from.is_empty() || self.to.is_empty() {
            return false;
        }

        if self.from == self.to {
            return false;
        }

        if self.signature.is_empty() || self.public_key.is_empty() {
            return false;
        }

        true
    }
}

// ========== BLOCK ==============
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Block {
    pub index: u32,
    pub timestamp: u64,
    pub transactions: Vec<Transaction>,
    pub prev_hash: String,
    pub hash: String,
    pub nonce: u32,
}

impl Block {
    pub fn new(index: u32, transactions: Vec<Transaction>, prev_hash: String) -> Block {
        let now = SystemTime::now();
        let since_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
        let timestamp = since_epoch.as_secs();

        let difficulty = 2;
        let mut nonce = 0;
        let mut hash = Self::compute_hash(index, timestamp, &transactions, &prev_hash, nonce);

        while !hash.starts_with(&"0".repeat(difficulty as usize)) {
            nonce += 1;
            hash = Self::compute_hash(index, timestamp, &transactions, &prev_hash, nonce);
        }

        Block {
            index,
            timestamp,
            transactions,
            prev_hash,
            hash,
            nonce,
        }
    }

    pub fn genesis() -> Self {
        let genesis_tx = Transaction::new(
            "GENESIS".to_string(),
            "GENESIS".to_string(),
            0.0,
            "genesis_signature".to_string(),
            "genesis_key".to_string(),
        );
        Self::new(0, vec![genesis_tx], "0".repeat(64))
    }

    pub fn compute_hash(
        index: u32,
        timestamp: u64,
        transactions: &[Transaction],
        prev_hash: &str,
        nonce: u32,
    ) -> String {
        let tx_data = transactions
            .iter()
            .map(|tx| format!("{}->{}:{}", tx.from, tx.to, tx.amount))
            .collect::<Vec<String>>()
            .join("|");

        let input = format!(
            "{}|{}|{}|{}|{}",
            index, timestamp, tx_data, prev_hash, nonce
        );

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

        for tx in &self.transactions {
            if !tx.is_valid() {
                return false;
            }
        }

        let expected = Self::compute_hash(
            self.index,
            self.timestamp,
            &self.transactions,
            &self.prev_hash,
            self.nonce,
        );
        if self.hash != expected {
            return false;
        }

        let difficulty = 2;
        if !self.hash.starts_with(&"0".repeat(difficulty as usize)) {
            return false;
        }

        true
    }
}

// ========== MEMPOOL ==============
#[derive(Clone, Debug)]
pub struct MemPool {
    pub transactions: Vec<Transaction>,
}

impl MemPool {
    pub fn new() -> MemPool {
        MemPool {
            transactions: Vec::new(),
        }
    }

    pub fn add_transaction(&mut self, tx: Transaction) -> bool {
        if tx.is_valid() {
            self.transactions.push(tx);
            true
        } else {
            false
        }
    }

    pub fn get_transactions(&mut self, count: usize) -> Vec<Transaction> {
        let mut result = Vec::new();
        for _ in 0..count {
            if let Some(tx) = self.transactions.pop() {
                result.push(tx);
            }
        }
        result
    }

    pub fn clear(&mut self) {
        self.transactions.clear();
    }

    pub fn size(&self) -> usize {
        self.transactions.len()
    }
}

// ========== BLOCKCHAIN ==============
#[derive(Clone)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub difficulty: u32,
    pub mempool: MemPool,
    pub target_block_time: u64,
    pub adjustment_interval: u32,
}

impl Blockchain {
    pub fn new() -> Blockchain {
        let mut blockchain = Blockchain {
            chain: Vec::new(),
            difficulty: 2,
            mempool: MemPool::new(),
            target_block_time: 10,
            adjustment_interval: 10,
        };
        let genesis = Block::genesis();
        blockchain.chain.push(genesis);

        blockchain
    }

    pub fn add_transaction(&mut self, tx: Transaction) -> bool {
        self.mempool.add_transaction(tx)
    }

    pub fn mine_block(&mut self) -> bool {
        let new_index = self.chain.len() as u32;
        let prev_block = &self.chain[self.chain.len() - 1];
        let prev_hash = prev_block.hash.clone();

        let transactions = self.mempool.get_transactions(10);

        if transactions.is_empty() {
            println!("⚠️  No transactions to mine");
            return false;
        }

        println!(
            "⛏️  Mining block {} with {} transactions...",
            new_index,
            transactions.len()
        );
        let new_block = Block::new(new_index, transactions, prev_hash);
        println!("✅ Block mined! Hash: {}, nonce = {}", &new_block.hash[0..16], new_block.nonce);

        if new_block.is_valid(prev_block) {
            self.chain.push(new_block);
            self.adjust_difficulty();
            true
        } else {
            println!("❌ Block validation failed!");
            false
        }
    }

    pub fn is_chain_valid(&self) -> bool {
        for i in 1..self.chain.len() {
            let current_block = &self.chain[i];
            let prev_block = &self.chain[i - 1];

            if !current_block.is_valid(prev_block) {
                return false;
            }
        }
        true
    }

    pub fn adjust_difficulty(&mut self) {
        if self.chain.len() < self.adjustment_interval as usize {
            return;
        }

        if self.chain.len() % self.adjustment_interval as usize != 0 {
            return;
        }

        let start_idx = self.chain.len() - self.adjustment_interval as usize;
        let end_idx = self.chain.len() - 1;

        let first_block = &self.chain[start_idx];
        let last_block = &self.chain[end_idx];

        let actual_time = last_block.timestamp - first_block.timestamp;
        let target_time = self.target_block_time * self.adjustment_interval as u64;

        if actual_time < target_time && actual_time > 0 {
            self.difficulty += 1;
            println!("📈 Difficulty increased to: {}", self.difficulty);
        } else if actual_time > target_time && self.difficulty > 1 {
            self.difficulty -= 1;
            println!("📉 Difficulty decreased to: {}", self.difficulty);
        }
    }

    pub fn get_balance(&self, address: &str) -> f64 {
        let mut balance = 0.0;

        for block in &self.chain {
            for tx in &block.transactions {
                if tx.from == address {
                    balance -= tx.amount;
                }
                if tx.to == address {
                    balance += tx.amount;
                }
            }
        }

        balance
    }

    pub fn chain_stats(&self) -> String {
        format!(
            "Blocks: {} | Valid: {} | Difficulty: {} | Mempool: {}",
            self.chain.len(),
            self.is_chain_valid(),
            self.difficulty,
            self.mempool.size()
        )
    }
}

// ========== WALLET ==============
#[derive(Clone)]
pub struct Wallet {
    pub private_key: String,
    pub public_key: String,
}

impl Wallet {
    pub fn new() -> Wallet {
        let secp = Secp256k1::new();
        let mut rng = rand::thread_rng();
        let mut secret_key_bytes = [0u8; 32];
        rng.fill(&mut secret_key_bytes);

        let secret_key = SecretKey::from_slice(&secret_key_bytes).expect("Invalid secret key");
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);

        let private_key_hex = hex::encode(&secret_key_bytes);
        let public_key_hex = hex::encode(public_key.serialize());

        Wallet {
            private_key: private_key_hex,
            public_key: public_key_hex,
        }
    }

    pub fn get_address(&self) -> String {
        self.public_key[0..10].to_string()
    }

    pub fn sign_transaction(&self, tx_data: &str) -> String {
        let secp = Secp256k1::new();

        let secret_key =
            SecretKey::from_slice(&hex::decode(&self.private_key).expect("Invalid key"))
                .expect("Invalid secret key");

        let message = secp256k1::Message::from_slice(&Sha256::digest(tx_data.as_bytes()))
            .expect("Invalid message");

        let signature = secp.sign_ecdsa(&message, &secret_key);
        hex::encode(signature.serialize_compact())
    }

    pub fn export_private_key(&self) -> String {
        self.private_key.clone()
    }

    pub fn export_public_key(&self) -> String {
        self.public_key.clone()
    }
}
