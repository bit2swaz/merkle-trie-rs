use clap::{Parser, Subcommand};
use merkle_trie_rs::trie::EthTrie;

#[derive(Parser, Debug)]
#[command(name = "merkle-trie-rs")]
#[command(about = "a merkle patricia trie implementation in rust", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Insert {
        key: String,
        value: String,
    },
    Get {
        key: String,
    },
    Proof {
        key: String,
    },
    Demo,
}

fn main() {
    let args = Args::parse();

    match args.command {
        Commands::Insert { key, value } => {
            let mut trie = EthTrie::new();
            trie.insert(key.as_bytes(), value.as_bytes());
            
            println!("inserted: '{}' => '{}'", key, value);
            println!("root hash: {}", hex::encode(trie.root_hash()));
            println!();
            trie.print_tree();
        }
        Commands::Get { key } => {
            let mut trie = EthTrie::new();
            trie.insert(key.as_bytes(), b"demo_value");
            
            match trie.get(key.as_bytes()) {
                Some(value) => {
                    println!("found: '{}' => '{}'", key, String::from_utf8_lossy(&value));
                }
                None => {
                    println!("key '{}' not found in trie", key);
                }
            }
        }
        Commands::Proof { key } => {
            let mut trie = EthTrie::new();
            trie.insert(key.as_bytes(), b"demo_value");
            
            let proof = trie.get_proof(key.as_bytes());
            let root_hash = trie.root_hash();
            
            println!("generating proof for key: '{}'", key);
            println!("root hash: {}", hex::encode(root_hash));
            println!("proof has {} nodes:", proof.len());
            
            for (i, node) in proof.iter().enumerate() {
                println!("  node {}: {} bytes (hex: {})", 
                    i, 
                    node.len(), 
                    hex::encode(node));
            }
            
            println!();
            match EthTrie::verify_proof(&root_hash, key.as_bytes(), &proof) {
                Some(value) => {
                    println!("proof verified successfully");
                    println!("  value: '{}'", String::from_utf8_lossy(&value));
                }
                None => {
                    println!("proof verification failed");
                }
            }
        }
        Commands::Demo => {
            println!("=== merkle patricia trie demo ===\n");
            
            let mut trie = EthTrie::new();
            
            println!("1. inserting keys...");
            let entries = vec![
                ("dog", "puppy"),
                ("do", "verb"),
                ("doge", "coin"),
                ("horse", "stallion"),
            ];
            
            for (key, value) in &entries {
                trie.insert(key.as_bytes(), value.as_bytes());
                println!("   inserted: '{}' => '{}'", key, value);
            }
            
            println!("\n2. trie structure:");
            println!("   root hash: {}\n", hex::encode(trie.root_hash()));
            trie.print_tree();
            
            println!("\n3. retrieving values...");
            for (key, expected_value) in &entries {
                match trie.get(key.as_bytes()) {
                    Some(value) => {
                        println!("   get('{}') => '{}'", key, String::from_utf8_lossy(&value));
                        assert_eq!(value, expected_value.as_bytes());
                    }
                    None => {
                        println!("   get('{}') => not found ", key);
                    }
                }
            }
            
            println!("\n4. generating and verifying proofs...");
            let root_hash = trie.root_hash();
            
            for (key, _) in &entries {
                let proof = trie.get_proof(key.as_bytes());
                match EthTrie::verify_proof(&root_hash, key.as_bytes(), &proof) {
                    Some(value) => {
                        println!(
                            "   proof for '{}': {} nodes, verified (value: '{}')",
                            key,
                            proof.len(),
                            String::from_utf8_lossy(&value)
                        );
                    }
                    None => {
                        println!("   proof for '{}': verification failed", key);
                    }
                }
            }
            
            println!("\n5. testing non-existent key...");
            let missing_key = "cat";
            match trie.get(missing_key.as_bytes()) {
                Some(_) => println!("   get('{}') => found (unexpected)", missing_key),
                None => println!("   get('{}') => not found", missing_key),
            }
            
            println!("\n=== demo complete ===");
        }
    }
}

