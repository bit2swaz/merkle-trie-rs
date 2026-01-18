# merkle-trie-rs

my rust implementation of the modified merkle patricia trie as specified in the ethereum yellow paper. this data structure is used by ethereum to store account states, transaction receipts, and other blockchain data.

## what is this

the modified merkle patricia trie (mpt) is a hexary radix tree with cryptographic commitment. unlike standard merkle trees, it allows efficient insertion, deletion, and proof generation without rehashing the entire structure. the root hash changes deterministically with any data modification, providing o(1) verification of the entire dataset.

this implementation follows the ethereum specification exactly, including:

- nibble-based key encoding (4-bit units)
- hex-prefix encoding for path compression
- rlp serialization for all nodes
- keccak256 hashing with proper node reference handling
- merkle proof generation and verification

## features

- four node types: null, leaf, extension, and branch
- recursive insertion with automatic path splitting
- deterministic root hash calculation
- merkle proof generation for light client verification
- proof verification without trie reconstruction
- tree visualization for debugging
- command-line interface for testing
- comprehensive test suite (64 tests, 100% passing)

## installation

requires rust 1.70 or later.

```bash
git clone https://github.com/bit2swaz/merkle-trie-rs.git
cd merkle-trie-rs
cargo build --release
```

## usage

### command line

insert key-value pairs:
```bash
cargo run -- insert <key> <value>
```

retrieve values:
```bash
cargo run -- get <key>
```

generate merkle proofs:
```bash
cargo run -- proof <key>
```

run demonstration:
```bash
cargo run -- demo
```

### library api

```rust
use merkle_trie_rs::trie::EthTrie;

fn main() {
    let mut trie = EthTrie::new();
    
    // insert data
    trie.insert(b"do", b"verb");
    trie.insert(b"dog", b"puppy");
    trie.insert(b"doge", b"coin");
    
    // get root hash
    let root = trie.root_hash();
    println!("root: {}", hex::encode(root));
    
    // retrieve values
    if let Some(value) = trie.get(b"dog") {
        println!("found: {}", String::from_utf8_lossy(&value));
    }
    
    // generate proof
    let proof = trie.get_proof(b"dog").unwrap();
    
    // verify proof (static method)
    let verified = EthTrie::verify_proof(&root, b"dog", b"puppy", &proof);
    assert!(verified);
}
```

## architecture

### module structure

```
src/
├── lib.rs       - public api exports
├── main.rs      - cli interface
├── nibbles.rs   - nibble encoding and hex-prefix implementation
├── node.rs      - node enum with rlp serialization
└── trie.rs      - core trie operations
```

### node types

**null**: empty node, encoded as empty byte array

**leaf**: terminal node storing a value
- compact-encoded partial path
- value bytes

**extension**: path compression node
- compact-encoded shared path prefix
- reference to child node

**branch**: 16-way split node
- 16 child references (one per nibble)
- optional value for keys ending at branch

### hex-prefix encoding

compact encoding distinguishes node types and handles odd-length paths:

- flag 0: extension, even length
- flag 1: extension, odd length
- flag 2: leaf, even length
- flag 3: leaf, odd length

### node references

nodes are referenced differently based on rlp-encoded size:

- less than 32 bytes: embedded directly
- 32 bytes or more: replaced with keccak256 hash

this optimization reduces storage while maintaining merkle properties.

## testing

run all tests:
```bash
cargo test
```

run specific test module:
```bash
cargo test nibbles
cargo test node
cargo test trie
```

run with output:
```bash
cargo test -- --nocapture
```

### test coverage

- **nibbles.rs**: 17 tests for nibble conversion and hex-prefix encoding
- **node.rs**: 7 tests for rlp serialization of all node types
- **trie.rs**: 40 tests covering insertion, retrieval, proofs, and edge cases

### verification tests

the test suite includes verification of ethereum compatibility:

1. **empty root hash**: confirms empty trie produces the canonical ethereum empty root
2. **insertion determinism**: verifies that insertion order does not affect final root hash
3. **proof sharing**: validates that proofs can be verified independently of the original trie instance

## implementation details

### insertion algorithm

insertion follows these steps:

1. convert key to nibble path
2. traverse trie matching nibbles
3. handle node type cases:
   - **null**: create new leaf
   - **leaf**: update if path matches, otherwise split into branch
   - **extension**: continue if path matches, otherwise split at divergence
   - **branch**: recurse into appropriate child
4. recalculate hashes bottom-up during recursion unwind

the most complex case is splitting extension nodes when paths diverge partway through the shared prefix.

### proof generation

proofs are constructed by collecting rlp-encoded nodes along the path from root to leaf. a verifier can:

1. hash the first proof element to get the expected root
2. decode it to find the next node reference
3. hash subsequent proof elements and verify they match references
4. confirm the final leaf contains the expected value

this allows light clients to verify data without storing the entire trie.

### memory model

the trie uses single-threaded ownership with `Box<Node>` for recursive structures. this avoids reference counting overhead while maintaining rust's safety guarantees.

## dependencies

- **tiny-keccak** (2.0): keccak256 hashing
- **rlp** (0.5): ethereum recursive length prefix encoding
- **hex** (0.4): hexadecimal display formatting
- **thiserror** (1.0): ergonomic error handling
- **clap** (4.5): command-line argument parsing

## performance characteristics

- **insertion**: o(log n) where n is trie size
- **retrieval**: o(log n) for successful lookups
- **proof generation**: o(log n) path traversal
- **proof verification**: o(log n) hash computations
- **space**: o(n) nodes with path compression

actual performance depends on key distribution. keys with long shared prefixes benefit from extension node compression.

## limitations

this is an educational implementation prioritizing correctness and code clarity over performance. production use cases should consider:

- no persistent storage backend
- no node caching or memoization
- no parallel proof verification
- no state pruning mechanisms
- single-threaded only

## references

- [ethereum yellow paper](https://ethereum.github.io/yellowpaper/paper.pdf) - appendix d: modified merkle patricia trie
- [ethereum wiki: patricia tree](https://eth.wiki/fundamentals/patricia-tree)
- [rlp specification](https://ethereum.org/en/developers/docs/data-structures-and-encoding/rlp/)

## license

see [LICENSE](LICENSE) file for details.

---

made with love by [bit2swaz](https://x.com/bit2swaz)
