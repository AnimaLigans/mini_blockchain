mod block;
use crate::block::Block;

fn main() {
    let genesis = Block::genesis();
    println!("{:?}", genesis);
    println!("nonce = {}", genesis.nonce);
}
