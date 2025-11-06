mod block;
use crate::block::{Block, Blockchain, Transaction};

fn main() {
    let mut blockchain = Blockchain::new();
    println!("🔗 Genesis блок создан!\n");

    println!("─── Создание транзакций ───");
    let tx1 = Transaction::new("Alice".to_string(), "Bob".to_string(), 10.0);
    let tx2 = Transaction::new("Bob".to_string(), "Charlie".to_string(), 5.0);
    let tx3 = Transaction::new("Charlie".to_string(), "Alice".to_string(), 3.0);

    println!("✓ Транзакция 1: Alice → Bob (10 монет)");
    println!("✓ Транзакция 2: Bob → Charlie (5 монет)");
    println!("✓ Транзакция 3: Charlie → Alice (3 монеты)\n");

    println!("─── Добавление в MemPool ───");
    blockchain.add_transaction(tx1);
    blockchain.add_transaction(tx2);
    blockchain.add_transaction(tx3);
    println!("✓ Все транзакции добавлены в MemPool\n");

    println!("─── Майнинг блока 1 ───");
    blockchain.mine_block();
    println!();

    println!("─── Создание новых транзакций ───");
    let tx4 = Transaction::new("Alice".to_string(), "David".to_string(), 7.0);
    blockchain.add_transaction(tx4);
    println!("✓ Транзакция добавлена\n");

    println!("─── Майнинг блока 2 ───");
    blockchain.mine_block();
    println!();

    println!("─── Проверка цепочки ───");
    let is_valid = blockchain.is_chain_valid();
    println!("Цепочка валидна? {}\n", is_valid);

    println!("─── Все блоки и транзакции ───");
    for (i, block) in blockchain.chain.iter().enumerate() {
        println!("📦 Блок {}:", i);
        println!("   Hash: {}...", &block.hash[0..16]);
        println!("   Транзакции: {}", block.transactions.len());
        for (j, tx) in block.transactions.iter().enumerate() {
            println!("     Tx {}: {} → {} ({} монет)", j + 1, tx.from, tx.to, tx.amount);
        }
        println!();
    }
}
