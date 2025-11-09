mod block;
use crate::block::{Blockchain, Transaction, Wallet};

fn main() {
    // Создаём новый блокчейн (автоматически создаёт genesis блок)
    let mut blockchain = Blockchain::new();
    println!("Blockchain initialized\n");

    // Создаём три кошелька (каждый с приватным и публичным ключом)
    println!("--- Creating wallets ---");
    let wallet1 = Wallet::new();
    let wallet2 = Wallet::new();
    let wallet3 = Wallet::new();

    // Выводим адреса кошельков (первые 10 символов публичного ключа)
    println!("Wallet 1 address: {}", wallet1.get_address());
    println!("Wallet 2 address: {}", wallet2.get_address());
    println!("Wallet 3 address: {}", wallet3.get_address());

    // Создаём и подписываем транзакции приватными ключами
    println!("\n--- Creating and signing transactions ---");
    
    // Транзакция 1: Wallet1 отправляет 10 единиц Wallet2
    let tx1_data = format!("{}->{}:{}", wallet1.get_address(), wallet2.get_address(), 10.0);
    let tx1_sig = wallet1.sign_transaction(&tx1_data);
    let tx1 = Transaction::new(
        wallet1.get_address(),
        wallet2.get_address(),
        10.0,
        tx1_sig,
        wallet1.public_key.clone(),
    );
    
    // Транзакция 2: Wallet2 отправляет 5 единиц Wallet3
    let tx2_data = format!("{}->{}:{}", wallet2.get_address(), wallet3.get_address(), 5.0);
    let tx2_sig = wallet2.sign_transaction(&tx2_data);
    let tx2 = Transaction::new(
        wallet2.get_address(),
        wallet3.get_address(),
        5.0,
        tx2_sig,
        wallet2.public_key.clone(),
    );

    println!("Transaction 1: {} -> {} (10 units)", wallet1.get_address(), wallet2.get_address());
    println!("Transaction 2: {} -> {} (5 units)", wallet2.get_address(), wallet3.get_address());

    // Добавляем транзакции в MemPool (очередь ожидания)
    println!("\n--- Adding to mempool ---");
    blockchain.add_transaction(tx1);
    blockchain.add_transaction(tx2);
    println!("Transactions added to mempool");

    // Майним первый блок с этими транзакциями
    println!("\n--- Mining block 1 ---");
    blockchain.mine_block();

    // Проверяем, что цепочка целостна
    println!("\n--- Checking chain validity ---");
    let is_valid = blockchain.is_chain_valid();
    println!("Chain valid: {}\n", is_valid);

    // Выводим все блоки и их транзакции
    println!("--- All blocks and transactions ---");
    for (i, block) in blockchain.chain.iter().enumerate() {
        println!("Block {}:", i);
        println!("  Hash: {}...", &block.hash[0..16]);
        println!("  Transactions: {}", block.transactions.len());
        for (j, tx) in block.transactions.iter().enumerate() {
            println!("    Tx {}: {} -> {} ({} units)", j + 1, tx.from, tx.to, tx.amount);
            println!("    Signature: {}...", &tx.signature[0..16]);
        }
    }
}

