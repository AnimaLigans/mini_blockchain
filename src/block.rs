use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};
use secp256k1::{Secp256k1, SecretKey, PublicKey};
use rand::Rng;

// ========== BLOCK ==============
// Основная единица цепочки - хранит транзакции, хэш и данные о целостности
#[derive(Debug)]
pub struct Block {
    pub index: u32,                          // Номер блока в цепочке (0, 1, 2...)
    pub timestamp: u64,                      // Время создания блока (секунды с 1970)
    pub transactions: Vec<Transaction>,      // Список транзакций в блоке
    pub prev_hash: String,                   // Хэш предыдущего блока - связывает цепь
    pub hash: String,                        // Хэш текущего блока (64 символа)
    pub nonce: u32,                          // Число для Proof of Work майнинга
}

impl Block {
    // Создание нового блока с транзакциями
    pub fn new(index: u32, transactions: Vec<Transaction>, prev_hash: String) -> Block {
        // Получаем текущее время
        let now = SystemTime::now();
        let since_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
        let timestamp = since_epoch.as_secs();

        // Устанавливаем параметры майнинга
        let difficulty = 2;  // Нужны 2 нуля в начале хэша
        let mut nonce = 0;
        let mut hash = Self::compute_hash(index, timestamp, &transactions, &prev_hash, nonce);

        // Ищем nonce, при котором хэш начинается с нулей (Proof of Work)
        while !hash.starts_with(&"0".repeat(difficulty as usize)) {
            nonce += 1;
            hash = Self::compute_hash(index, timestamp, &transactions, &prev_hash, nonce);
        }

        // Собираем готовый блок
        Block {
            index,
            timestamp,
            transactions,
            prev_hash,
            hash,
            nonce,
        }
    }

    // Создание первого блока цепочки (Genesis)
    pub fn genesis() -> Self {
        // Genesis имеет специальную транзакцию
        let genesis_tx = Transaction::new(
            "GENESIS".to_string(),
            "GENESIS".to_string(),
            0.0,
            "genesis_signature".to_string(),
            "genesis_key".to_string(),
        );
        Self::new(0, vec![genesis_tx], "0".repeat(64))
    }

    // Вычисление SHA-256 хэша из всех данных блока
    pub fn compute_hash(
        index: u32,
        timestamp: u64,
        transactions: &Vec<Transaction>,
        prev_hash: &str,
        nonce: u32,
    ) -> String {
        // Собираем все транзакции в одну строку
        let tx_data = transactions
            .iter()
            .map(|tx| format!("{}->{}:{}", tx.from, tx.to, tx.amount))
            .collect::<Vec<String>>()
            .join("|");
        
        // Объединяем все поля в единую строку для хэширования
        let input = format!("{}|{}|{}|{}|{}", index, timestamp, tx_data, prev_hash, nonce);
        
        // Хэшируем через SHA-256
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        let bytes = hasher.finalize();
        hex::encode(bytes)  // Преобразуем в 64 символа (hex)
    }

    // Проверка валидности блока относительно предыдущего
    pub fn is_valid(&self, prev: &Block) -> bool {
        // Проверка 1: индекс должен расти на 1
        if self.index != prev.index + 1 { return false; }
        
        // Проверка 2: prev_hash должен совпадать с хэшем предыдущего блока
        if self.prev_hash != prev.hash { return false; }
        
        // Проверка 3: все транзакции в блоке должны быть валидны
        for tx in &self.transactions {
            if !tx.is_valid() { return false; }
        }
        
        // Проверка 4: хэш должен соответствовать данным блока
        let expected = Self::compute_hash(
            self.index,
            self.timestamp,
            &self.transactions,
            &self.prev_hash,
            self.nonce,
        );
        if self.hash != expected { return false; }
        
        // Проверка 5: Proof of Work - хэш должен начинаться с нулей
        let difficulty = 2;
        if !self.hash.starts_with(&"0".repeat(difficulty as usize)) { return false; }
        
        true  // Все проверки пройдены - блок валиден
    }
}

// ========== BLOCKCHAIN ==============
// Цепочка всех блоков, управление добавлением и валидацией
pub struct Blockchain {
    pub chain: Vec<Block>,          // Все блоки в порядке
    pub difficulty: u32,            // Сложность майнинга
    pub mempool: MemPool,           // Пул ожидающих транзакций
}

impl Blockchain {
    // Создание нового блокчейна
    pub fn new() -> Blockchain {
        let mut blockchain = Blockchain {
            chain: Vec::new(),
            difficulty: 2,
            mempool: MemPool::new(),
        };
        // Добавляем первый (genesis) блок
        let genesis = Block::genesis();
        blockchain.chain.push(genesis);

        blockchain
    }

    // Добавление новой транзакции в очередь (MemPool)
    pub fn add_transaction(&mut self, tx: Transaction) -> bool {
        self.mempool.add_transaction(tx)
    }

    // Майнинг нового блока с транзакциями из MemPool
    pub fn mine_block(&mut self) -> bool {
        let new_index = self.chain.len() as u32;
        let prev_block = &self.chain[self.chain.len() - 1];
        let prev_hash = prev_block.hash.clone();
        
        // Берём до 10 транзакций из MemPool
        let transactions = self.mempool.get_transactions(10);

        if transactions.is_empty() {
            println!("No transactions to mine");
            return false;
        }
        
        println!("Mining block {} with {} transactions...", new_index, transactions.len());
        let new_block = Block::new(new_index, transactions, prev_hash);
        println!("Block mined! nonce = {}", new_block.nonce);

        // Проверяем валидность перед добавлением
        if new_block.is_valid(prev_block) {
            self.chain.push(new_block);
            true
        } else {
            false
        }
    }

    // Проверка целой цепочки на валидность
    pub fn is_chain_valid(&self) -> bool {
        // Проходим по каждому блоку (начиная со второго)
        for i in 1..self.chain.len() {
            let current_block = &self.chain[i];
            let prev_block = &self.chain[i - 1];
            
            // Если хотя бы один блок невалиден - вся цепь сломана
            if !current_block.is_valid(prev_block) {
                return false;
            }
        }
        true  // Вся цепь целостна
    }
}

// ========== TRANSACTION ==============
// Транзакция - перевод денег с подписью и открытым ключом
#[derive(Debug, Clone)]
pub struct Transaction {
    pub from: String,           // Адрес отправителя (первые 10 символов публичного ключа)
    pub to: String,             // Адрес получателя
    pub amount: f64,            // Сумма
    pub timestamp: u64,         // Время транзакции
    pub signature: String,      // Цифровая подпись (ECDSA)
    pub public_key: String,     // Публичный ключ отправителя для проверки подписи
}

impl Transaction {
    // Создание новой транзакции с подписью
    pub fn new(from: String, to: String, amount: f64, signature: String, public_key: String) -> Transaction {
        let now = SystemTime::now();
        let since_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
        let timestamp = since_epoch.as_secs();
        
        Transaction { from, to, amount, timestamp, signature, public_key }
    }

    // Проверка транзакции на валидность
    pub fn is_valid(&self) -> bool {
        // Сумма должна быть положительной
        if self.amount <= 0.0 { return false; }
        
        // Адреса не должны быть пустыми
        if self.from.is_empty() || self.to.is_empty() { return false; }
        
        // Нельзя отправить самому себе
        if self.from == self.to { return false; }
        
        // Подпись и ключ обязательны
        if self.signature.is_empty() || self.public_key.is_empty() { return false; }
        
        true
    }
}

// ========== MEMPOOL ==============
// Пул неподтверждённых транзакций, ожидающих включения в блок
pub struct MemPool {
    pub transactions: Vec<Transaction>,
}

impl MemPool {
    // Создание пустого пула
    pub fn new() -> MemPool {
        MemPool { transactions: Vec::new() }
    }

    // Добавление новой транзакции в пул
    pub fn add_transaction(&mut self, tx: Transaction) -> bool {
        // Проверяем валидность перед добавлением
        if tx.is_valid() {
            self.transactions.push(tx);
            true
        } else {
            false
        }
    }

    // Получить первые N транзакций для блока
    pub fn get_transactions(&mut self, count: usize) -> Vec<Transaction> {
        let mut result = Vec::new();
        for _ in 0..count {
            if let Some(tx) = self.transactions.pop() {
                result.push(tx);
            }
        }
        result
    }

    // Очистить пул (после включения всех транзакций в блок)
    pub fn clear(&mut self) {
        self.transactions.clear();
    }
}

// ========== WALLET ==============
// Кошелёк - пара приватный/публичный ключ для подписания транзакций
pub struct Wallet {
    pub private_key: String,    // Приватный ключ (256 бит) - только ты знаешь
    pub public_key: String,     // Публичный ключ - твой адрес в сети
}

impl Wallet {
    // Создание нового кошелька с генерацией ключей
    pub fn new() -> Wallet {
        let secp = Secp256k1::new();
        let mut rng = rand::thread_rng();
        let mut secret_key_bytes = [0u8; 32];
        rng.fill(&mut secret_key_bytes);
        
        // Генерируем приватный ключ (256 бит)
        let secret_key = SecretKey::from_slice(&secret_key_bytes)
            .expect("Invalid secret key");
        
        // Вычисляем публичный ключ из приватного
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        
        // Кодируем в hex для хранения
        let private_key_hex = hex::encode(&secret_key_bytes);
        let public_key_hex = hex::encode(public_key.serialize());
        
        Wallet {
            private_key: private_key_hex,
            public_key: public_key_hex,
        }
    }
    
    // Получить адрес кошелька (первые 10 символов публичного ключа)
    pub fn get_address(&self) -> String {
        self.public_key[0..10].to_string()
    }
    
    // Подписать данные приватным ключом (ECDSA)
    pub fn sign_transaction(&self, tx_data: &str) -> String {
        let secp = Secp256k1::new();
        
        // Восстанавливаем приватный ключ
        let secret_key = SecretKey::from_slice(&hex::decode(&self.private_key).expect("Invalid key"))
            .expect("Invalid secret key");
        
        // Хэшируем данные транзакции
        let message = secp256k1::Message::from_slice(
            &Sha256::digest(tx_data.as_bytes())
        ).expect("Invalid message");
        
        // Подписываем хэш приватным ключом
        let signature = secp.sign_ecdsa(&message, &secret_key);
        hex::encode(signature.serialize_compact())
    }
}
