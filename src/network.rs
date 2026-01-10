use crate::block::{Block, Blockchain, Transaction};
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Clone)]
pub struct Node {
    pub id: String,
    pub blockchain: Arc<Mutex<Blockchain>>,
    pub peers: Arc<Mutex<Vec<String>>>,
}

impl Node {
    pub fn new(id: String) -> Self {
        Node {
            id,
            blockchain: Arc::new(Mutex::new(Blockchain::new())),
            peers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn clone_node(&self) -> Self {
        Node {
            id: self.id.clone(),
            blockchain: Arc::clone(&self.blockchain),
            peers: Arc::clone(&self.peers),
        }
    }

    pub fn start_server(&self, port: u16) {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
            .expect("Failed to bind to port");
        println!("🟢 Node [{}] listening on 127.0.0.1:{}", self.id, port);

        let blockchain_clone = Arc::clone(&self.blockchain);
        let id_clone = self.id.clone();

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let blockchain = Arc::clone(&blockchain_clone);
                    let node_id = id_clone.clone();
                    println!("📥 Incoming connection to Node [{}]", node_id);

                    thread::spawn(move || {
                        handle_client(stream, &blockchain, &node_id);
                    });
                }
                Err(e) => {
                    eprintln!("❌ Connection error: {}", e);
                }
            }
        }
    }

    pub fn connect_to_peer(&self, peer_addr: &str) -> bool {
        match TcpStream::connect(peer_addr) {
            Ok(mut stream) => {
                println!("🟢 Node [{}] connected to peer: {}", self.id, peer_addr);

                self.peers.lock().unwrap().push(peer_addr.to_string());

                let sync_request = json!({
                    "type": "SYNC_REQUEST",
                    "from": self.id,
                });

                if let Err(e) = stream.write_all(sync_request.to_string().as_bytes()) {
                    eprintln!("❌ Failed to send sync request: {}", e);
                    return false;
                }

                println!("📤 Sent SYNC_REQUEST to {}", peer_addr);

                let mut buffer = [0; 131072]; // 128KB буфер
                match stream.set_read_timeout(Some(Duration::from_secs(5))) {
                    Ok(_) => {
                        match stream.read(&mut buffer) {
                            Ok(n) if n > 0 => {
                                let response = String::from_utf8_lossy(&buffer[..n]);
                                if let Ok(data) = serde_json::from_str::<Value>(&response) {
                                    self.handle_sync_response(&data);
                                    return true;
                                }
                            }
                            _ => {
                                println!("⏱️  Timeout waiting for sync response");
                            }
                        }
                    }
                    Err(e) => eprintln!("❌ Timeout setup error: {}", e),
                }

                false
            }
            Err(e) => {
                println!(
                    "❌ Node [{}] failed to connect to {}: {}",
                    self.id, peer_addr, e
                );
                false
            }
        }
    }

    fn handle_sync_response(&self, data: &Value) {
        if let Some(chain_data) = data.get("chain").and_then(|v| v.as_array()) {
            let mut bc = self.blockchain.lock().unwrap();

            let blocks_before = bc.chain.len();

            for (idx, block_value) in chain_data.iter().enumerate() {
                if let Ok(block) = serde_json::from_value::<Block>(block_value.clone()) {
                    if idx == 0 {
                        // Проверяем, совпадает ли genesis блок
                        if bc.chain[0].hash != block.hash {
                            println!("⚠️  Genesis block mismatch - cannot sync");
                            return;
                        }
                    } else {
                        let prev_block = &bc.chain[bc.chain.len() - 1];
                        if block.is_valid(prev_block) {
                            bc.chain.push(block);
                        }
                    }
                }
            }

            let blocks_after = bc.chain.len();
            if blocks_after > blocks_before {
                println!(
                    "✅ Synced! Received {} new blocks. Total: {}",
                    blocks_after - blocks_before,
                    blocks_after
                );
            }
        }
    }

    pub fn broadcast_block(&self, block: &Block) {
        let msg = json!({
            "type": "NEW_BLOCK",
            "block": block
        });

        self.broadcast_to_peers(&msg.to_string());
    }

    pub fn broadcast_transaction(&self, tx: &Transaction) {
        let msg = json!({
            "type": "NEW_TRANSACTION",
            "transaction": tx
        });

        self.broadcast_to_peers(&msg.to_string());
    }

    fn broadcast_to_peers(&self, message: &str) {
        let peers = self.peers.lock().unwrap().clone();
        if peers.is_empty() {
            return;
        }

        for peer_addr in peers {
            if let Ok(mut stream) = TcpStream::connect(&peer_addr) {
                let _ = stream.write_all(message.as_bytes());
            }
        }
    }

    pub fn get_node_info(&self) -> String {
        let bc = self.blockchain.lock().unwrap();
        let peers = self.peers.lock().unwrap();

        format!(
            "╔══════════════════════════════╗\n\
             ║ Node: {:<18} ║\n\
             ║ Blocks: {:<20} ║\n\
             ║ Valid: {:<21} ║\n\
             ║ Peers: {:<21} ║\n\
             ║ Difficulty: {:<15} ║\n\
             ║ Mempool: {:<19} ║\n\
             ╚══════════════════════════════╝",
            self.id,
            bc.chain.len(),
            bc.is_chain_valid(),
            peers.len(),
            bc.difficulty,
            bc.mempool.size()
        )
    }
}

fn handle_client(
    mut stream: TcpStream,
    blockchain: &Arc<Mutex<Blockchain>>,
    node_id: &str,
) {
    let mut buffer = [0; 131072];

    match stream.read(&mut buffer) {
        Ok(n) if n > 0 => {
            let message = String::from_utf8_lossy(&buffer[..n]);

            if let Ok(data) = serde_json::from_str::<Value>(&message) {
                let msg_type = data.get("type").and_then(|v| v.as_str());

                match msg_type {
                    Some("SYNC_REQUEST") => {
                        let bc = blockchain.lock().unwrap();
                        let response = json!({
                            "type": "SYNC_RESPONSE",
                            "chain": bc.chain
                        });

                        println!(
                            "📤 Node [{}] sending chain with {} blocks",
                            node_id,
                            bc.chain.len()
                        );
                        let _ = stream.write_all(response.to_string().as_bytes());
                    }

                    Some("NEW_BLOCK") => {
                        if let Some(block_data) = data.get("block") {
                            if let Ok(block) = serde_json::from_value::<Block>(block_data.clone())
                            {
                                let mut bc = blockchain.lock().unwrap();
                                if bc.chain.len() > 0 {
                                    let prev = &bc.chain[bc.chain.len() - 1];
                                    if block.is_valid(prev) {
                                        bc.chain.push(block);
                                        println!("✅ Node [{}] added new block", node_id);
                                    }
                                }
                            }
                        }
                    }

                    Some("NEW_TRANSACTION") => {
                        if let Some(tx_data) = data.get("transaction") {
                            if let Ok(tx) =
                                serde_json::from_value::<Transaction>(tx_data.clone())
                            {
                                let mut bc = blockchain.lock().unwrap();
                                if bc.add_transaction(tx) {
                                    println!(
                                        "✅ Node [{}] added new transaction to mempool",
                                        node_id
                                    );
                                }
                            }
                        }
                    }

                    _ => {
                        println!("⚠️  Node [{}] received unknown message type", node_id);
                    }
                }
            }
        }
        Ok(_) => {
            println!("👋 Peer disconnected from Node [{}]", node_id);
        }
        Err(e) => {
            eprintln!("❌ Node [{}] read error: {}", node_id, e);
        }
    }
}

