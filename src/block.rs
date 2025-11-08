use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

// ========== BLOCK (сначала блок!) ==============
#[derive(Debug)]
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
        );
        Self::new(0, vec![genesis_tx], "0".repeat(64))
    }

    pub fn compute_hash(
        index: u32,
        timestamp: u64,
        transactions: &Vec<Transaction>,
        prev_hash: &str,
        nonce: u32,
    ) -> String {
        let tx_data = transactions
            .iter()
            .map(|tx| format!("{}→{}:{}", tx.from, tx.to, tx.amount))
            .collect::<Vec<String>>()
            .join("|");
        let input = format!("{}|{}|{}|{}|{}", index, timestamp, tx_data, prev_hash, nonce);
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        let bytes = hasher.finalize();
        hex::encode(bytes)
    }

    pub fn is_valid(&self, prev: &Block) -> bool {
        if self.index != prev.index + 1 { return false; }
        if self.prev_hash != prev.hash { return false; }
        for tx in &self.transactions {
            if !tx.is_valid() { return false; }
        }
        let expected = Self::compute_hash(
            self.index,
            self.timestamp,
            &self.transactions,
            &self.prev_hash,
            self.nonce,
        );
        if self.hash != expected { return false; }
        let difficulty = 2;
        if !self.hash.starts_with(&"0".repeat(difficulty as usize)) { return false; }
        true
    }
}

// ========== BLOCKCHAIN ==============
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub difficulty: u32,
    pub mempool: MemPool,
}

impl Blockchain {
    pub fn new() -> Blockchain {
        let mut blockchain = Blockchain {
            chain: Vec::new(),
            difficulty: 2,
            mempool: MemPool::new(),
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
            println!("❌ Нет транзакций для майнинга!");
            return false;
        }
        println!("⛏️  Майнинг блока {} с {} транзакциями...", new_index, transactions.len());
        let new_block = Block::new(new_index, transactions, prev_hash);
        println!("✅ Блок смайнен! nonce = {}", new_block.nonce);

        if new_block.is_valid(prev_block) {
            self.chain.push(new_block);
            true
        } else {
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
}

// ========== TRANSACTION ==============
#[derive(Debug, Clone)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: f64,
    pub timestamp: u64,
}

impl Transaction {
    pub fn new(from: String, to: String, amount: f64) -> Transaction {
        let now = SystemTime::now();
        let since_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
        let timestamp = since_epoch.as_secs();
        Transaction { from, to, amount, timestamp }
    }

    pub fn is_valid(&self) -> bool {
        if self.amount <= 0.0 { return false; }
        if self.from.is_empty() || self.to.is_empty() { return false; }
        if self.from == self.to { return false; }
        true
    }
}

// ========== MEMPOOL ==============
pub struct MemPool {
    pub transactions: Vec<Transaction>,
}

impl MemPool {
    pub fn new() -> MemPool {
        MemPool { transactions: Vec::new() }
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
}
