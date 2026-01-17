use tiny_keccak::{Hasher, Keccak};

use crate::node::Node;

pub struct EthTrie {
    root: Box<Node>,
}

impl EthTrie {
    pub fn new() -> Self {
        EthTrie {
            root: Box::new(Node::Null),
        }
    }

    pub fn root_hash(&self) -> [u8; 32] {
        let encoded = rlp::encode(&*self.root);
        let mut hasher = Keccak::v256();
        let mut output = [0u8; 32];
        hasher.update(&encoded);
        hasher.finalize(&mut output);
        output
    }
}

impl Default for EthTrie {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_trie_root_hash() {
        let trie = EthTrie::new();
        let hash = trie.root_hash();
        
        let expected: [u8; 32] = [
            0x56, 0xe8, 0x1f, 0x17, 0x1b, 0xcc, 0x55, 0xa6,
            0xff, 0x83, 0x45, 0xe6, 0x92, 0xc0, 0xf8, 0x6e,
            0x5b, 0x48, 0xe0, 0x1b, 0x99, 0x6c, 0xad, 0xc0,
            0x01, 0x62, 0x2f, 0xb5, 0xe3, 0x63, 0xb4, 0x21,
        ];
        
        assert_eq!(hash, expected, 
            "empty trie hash should be 0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421");
    }

    #[test]
    fn test_empty_trie_hash_hex() {
        let trie = EthTrie::new();
        let hash = trie.root_hash();
        let hash_hex = hex::encode(hash);
        
        assert_eq!(
            hash_hex,
            "56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421"
        );
    }

    #[test]
    fn test_new_trie_is_empty() {
        let trie = EthTrie::new();
        match *trie.root {
            Node::Null => (),
            _ => panic!("new trie should have a Null root"),
        }
    }
}
