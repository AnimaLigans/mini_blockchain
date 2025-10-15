mod block;
use crate::block::Block;

fn main() {
    let genesis = Block::genesis();
    println!("hash len = {}", genesis.hash.len());
    println!("{:?}", genesis);

    let b1 = Block::new(1, "Tx1".to_string(), genesis.hash.clone());
    println!("{:?}", b1);
    println!("valid? {}", b1.is_valid(&genesis));
}
