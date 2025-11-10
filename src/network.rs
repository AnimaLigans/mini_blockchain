// ================= NETWORK MODULE ===========
// Управление Р2Р сетями между узлами  

use std::net::{TspListener, TcpStream};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use crate::Block::Blockchain;

// узел в Р2Р сети - представляет собой один компьютер в блокчейне
pub struct Node{
    pub id: String,                                 // имя узла любое
    pub blockchain: Arc<Mutex<Blockchain>>,         // блокчейн защищён мьютексом для потокобезопасности
    pub peers: Arc<Mutex<Vec<String>>>,             // список подключённых соседей по типу "127.0.0.1:3001"
}

impl Node{
    // создание нового узла
    pub fn new(id:String) -> Self {
        Node{
            id,
            blockchain: Ark::new(Mutex::new(blockchain::new())),
            peers: Arc::new(Mutex::new(Vec::new())),
        }

    }
}
 
// запуск TSP cервера - узел начинает слушать входящие подключения 
pub fn start_server(&self, port: u16) {
    // привязываемся к locahost на заданном порту
    let listener = TspListener::bind(format!("127.0.0:{}", port)) 
    .expect("Filed to bind to port");
println!("Node {} listening to port {}", self.id, port);

let blockchain_clone = Arc::clone(&self.blockchain);

// бесконечный цикл : слушаем входящих клиентов 
for stream in listener.incoming() {
    match stream{
        Ok(mut stream) => {
        // новый клиент подключился
        let blockchain = Arc::clone(&blockchain_clone);
        println!("peer connected to {}", self.id);

        // обрабатываем гостя в отдельном потоке
        thread::spawn(move || {
            handle_client(&mut stream, &blockchain);
        });
        }
            Err(e) => {
                println!("connection error: {}", e);
            }
    }
}
}
// подключаемся к другому узлу(соседу)
pub fn connect_to_peer(&self, peer_addr: &str) {
    match TcpStream::connect(peer_addr) {
        Ok(mut stream) => {
            println!("{} conected to peer; {}", self.id, peer_addr );

            // добавляем соседа в список 
            self.peers.lock().unwrap().push(peer_addr.to_string());

            // отправляем информацию о нашей цепочке соседу
            let blockchain = self.blockchain.lock().unwrap();
            let chain_size = blockchain.chain.len();
            let message = format!("CHAIN_SIZE:{}", chain_size);

            stream.write_all(message.as.bytes()).ok();
                  println!(" Sent chain info to {}", peer_addr);
            }
            Err(e) => {
                println!(" {} failed to connect to {}: {}", self.id, peer_addr, e);
            }
        }
    }

    // Получить информацию о своём узле
    pub fn get_node_info(&self) -> String {
        let blockchain = self.blockchain.lock().unwrap();
        format!("Node: {} | Blocks: {} | Chain valid: {} | Peers: {}", 
            self.id, 
            blockchain.chain.len(),
            blockchain.is_chain_valid(),
            self.peers.lock().unwrap().len()
        )
    }


// Обработка входящего подключения от соседа
fn handle_client(stream: &mut TcpStream, blockchain: &Arc<Mutex<Blockchain>>) {
    let mut buffer = [0; 512];  // Буфер для получения данных
    
    match stream.read(&mut buffer) {
        Ok(n) if n > 0 => {
            // Получили сообщение от соседа
            let message = String::from_utf8_lossy(&buffer[..n]);
            println!(" Message from peer: {}", message);
            
            // Отправляем ответ
            let blockchain_guard = blockchain.lock().unwrap();
            let response = format!("OK|BLOCKS:{}", blockchain_guard.chain.len());
            drop(blockchain_guard);
            
            stream.write_all(response.as_bytes()).ok();
        }
        Ok(_) => {
            println!(" Peer disconnected");
        }
        Err(e) => {
            println!(" Read error: {}", e);
        }
    }
}

        
    
