use clap::Parser;
use merkle_trie_rs::nibbles::Nibbles;

#[derive(Parser, Debug)]
#[command(name = "merkle-trie-rs")]
#[command(about = "a merkle patricia trie implementation in rust", long_about = None)]
struct Args {
}

fn main() {
    let _args = Args::parse();
    println!("merkle trie rs - cli initialized");
    
    println!("{:?}", Nibbles::from_raw(&[0xAB], true));
}

