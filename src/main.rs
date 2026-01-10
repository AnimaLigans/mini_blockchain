mod block;
mod network;

use block::{Transaction, Wallet};
use network::Node;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

fn main() {
    println!(
        "\n\
         ╔═════════════════════════════════════╗\n\
         ║  🔗 BLOCKCHAIN P2P NETWORK v0.2.0  ║\n\
         ║     Educational Implementation     ║\n\
         ╚═════════════════════════════════════╝\n"
    );

    let node_role = std::env::args().nth(1).unwrap_or("alice".to_string());

    let (node_id, my_port, peer_addr) = match node_role.as_str() {
        "bob" => ("Bob", 3001u16, "127.0.0.1:3000"),
        "charlie" => ("Charlie", 3002u16, "127.0.0.1:3000"),
        _ => ("Alice", 3000u16, "127.0.0.1:3001"),
    };

    let node = Node::new(node_id.to_string());
    println!("✅ Created node: {}", node_id);

    // Запускаем сервер
    let node_for_server = node.clone_node();
    thread::spawn(move || {
        node_for_server.start_server(my_port);
    });

    thread::sleep(Duration::from_millis(1000));

    // Создаём кошельки
    println!("\n📝 Creating wallets...");
    let wallet1 = Wallet::new();
    let wallet2 = Wallet::new();
    println!("Wallet 1 address: {}", wallet1.get_address());
    println!("Wallet 2 address: {}", wallet2.get_address());

    // Создаём и подписываем транзакцию
    println!("\n💳 Creating transaction...");
    let amount = 50.0;
    let tx_data = format!("{}->{}:{}", wallet1.get_address(), wallet2.get_address(), amount);
    let signature = wallet1.sign_transaction(&tx_data);

    let tx = Transaction::new(
        wallet1.get_address(),
        wallet2.get_address(),
        amount,
        signature,
        wallet1.public_key.clone(),
    );

    // Добавляем в блокчейн и майним
    {
        let mut bc = node.blockchain.lock().unwrap();
        bc.add_transaction(tx.clone());

        println!("\n⛏️  Mining block...");
        bc.mine_block();

        println!("Chain validation: {}", bc.is_chain_valid());
        println!("Total blocks: {}", bc.chain.len());

        println!("\n💰 Balances:");
        println!(
            "  {} -> {}",
            wallet1.get_address(),
            bc.get_balance(&wallet1.get_address())
        );
        println!(
            "  {} -> {}",
            wallet2.get_address(),
            bc.get_balance(&wallet2.get_address())
        );
    }

    // Выводим информацию узла
    println!("\n{}", node.get_node_info());
    println!("Listening on 127.0.0.1:{}\n", my_port);

    // Подключаемся к пиру
    println!("🔗 Attempting to connect to peer: {}", peer_addr);
    thread::sleep(Duration::from_millis(500));

    for attempt in 1..=5 {
        if node.connect_to_peer(peer_addr) {
            println!("✅ Successfully connected to peer!\n");
            break;
        } else {
            println!("Attempt {}/5 - retrying in 2 seconds...", attempt);
            thread::sleep(Duration::from_secs(2));
        }
    }

    // Broadcast блок и транзакцию
    if let Some(last_block) = node.blockchain.lock().unwrap().chain.last().cloned() {
        node.broadcast_block(&last_block);
    }
    node.broadcast_transaction(&tx);

    println!("\n✅ Node [{}] is ready!", node_id);
    println!("═══════════════════════════════════════");
    println!("Commands:");
    println!("  Type 'mine' to mine a new block");
    println!("  Type 'tx'   to create transaction");
    println!("  Type 'info' to show node info");
    println!("  Type 'quit' to exit");
    println!("═══════════════════════════════════════\n");

    // Интерактивный режим
    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let command = input.trim();

        match command {
            "mine" => {
                let mut bc = node.blockchain.lock().unwrap();
                if bc.mempool.size() > 0 {
                    bc.mine_block();
                } else {
                    println!("⚠️  No transactions to mine");
                }
            }

            "tx" => {
                let wallet = Wallet::new();
                let recipient = Wallet::new();
                let amount = 10.0;

                let tx_data = format!(
                    "{}->{}:{}",
                    wallet.get_address(),
                    recipient.get_address(),
                    amount
                );
                let sig = wallet.sign_transaction(&tx_data);
                let tx = Transaction::new(
                    wallet.get_address(),
                    recipient.get_address(),
                    amount,
                    sig,
                    wallet.public_key,
                );

                let mut bc = node.blockchain.lock().unwrap();
                if bc.add_transaction(tx.clone()) {
                    println!("✅ Transaction added to mempool");
                    node.broadcast_transaction(&tx);
                }
            }

            "info" => {
                println!("\n{}", node.get_node_info());
                let bc = node.blockchain.lock().unwrap();
                println!("Status: {}\n", bc.chain_stats());
            }

            "quit" | "exit" => {
                println!("👋 Goodbye!");
                break;
            }

            _ => {
                println!("Unknown command. Type 'mine', 'tx', 'info', or 'quit'");
            }
        }
    }
}
