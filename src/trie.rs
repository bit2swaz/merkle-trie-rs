use tiny_keccak::{Hasher, Keccak};

use crate::nibbles::Nibbles;
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

    pub fn insert(&mut self, key: &[u8], value: &[u8]) {
        let nibbles = Nibbles::from_raw(key, false);
        let nibbles_vec = nibbles.as_slice().to_vec();
        self.root = Box::new(Self::insert_at(*self.root.clone(), &nibbles_vec, value.to_vec()));
    }

    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        let nibbles = Nibbles::from_raw(key, false);
        let nibbles_vec = nibbles.as_slice().to_vec();
        Self::get_at(&self.root, &nibbles_vec)
    }

    pub fn get_proof(&self, key: &[u8]) -> Vec<Vec<u8>> {
        let nibbles = Nibbles::from_raw(key, false);
        let nibbles_vec = nibbles.as_slice().to_vec();
        let mut proof = Vec::new();
        Self::get_proof_at(&self.root, &nibbles_vec, &mut proof);
        proof
    }


    pub fn verify_proof(
        root_hash: &[u8; 32],
        key: &[u8],
        proof: &[Vec<u8>],
    ) -> Option<Vec<u8>> {
        if proof.is_empty() {
            return None;
        }

        let nibbles = Nibbles::from_raw(key, false);
        let nibbles_vec = nibbles.as_slice().to_vec();

        let first_item = &proof[0];
        let computed_hash = Self::compute_hash(first_item);
        if computed_hash != *root_hash {
            return None;
        }

        Self::verify_proof_recursive(&nibbles_vec, proof, 0)
    }

    pub fn print_tree(&self) {
        println!("trie structure:");
        println!("root hash: {}", hex::encode(self.root_hash()));
        Self::print_node(&self.root, 0, "");
    }

    fn insert_at(node: Node, nibbles: &[u8], value: Vec<u8>) -> Node {
        match node {
            Node::Null => {
                Node::Leaf {
                    key: nibbles.to_vec(),
                    value,
                }
            }
            Node::Leaf {
                key: leaf_key,
                value: leaf_value,
            } => {
                let common_len = Self::common_prefix_len(&leaf_key, nibbles);

                if common_len == leaf_key.len() && common_len == nibbles.len() {
                    Node::Leaf {
                        key: leaf_key,
                        value,
                    }
                } else if common_len == 0 {
                    let mut children: [Box<Node>; 16] = [
                        Box::new(Node::Null), Box::new(Node::Null), Box::new(Node::Null), Box::new(Node::Null),
                        Box::new(Node::Null), Box::new(Node::Null), Box::new(Node::Null), Box::new(Node::Null),
                        Box::new(Node::Null), Box::new(Node::Null), Box::new(Node::Null), Box::new(Node::Null),
                        Box::new(Node::Null), Box::new(Node::Null), Box::new(Node::Null), Box::new(Node::Null),
                    ];

                    if leaf_key.is_empty() {
                        children[nibbles[0] as usize] = Box::new(Node::Leaf {
                            key: nibbles[1..].to_vec(),
                            value,
                        });
                        return Node::Branch {
                            children,
                            value: Some(leaf_value),
                        };
                    } else {
                        children[leaf_key[0] as usize] = Box::new(Node::Leaf {
                            key: leaf_key[1..].to_vec(),
                            value: leaf_value,
                        });
                    }

                    if nibbles.is_empty() {
                        return Node::Branch {
                            children,
                            value: Some(value),
                        };
                    } else {
                        children[nibbles[0] as usize] = Box::new(Self::insert_at(
                            *children[nibbles[0] as usize].clone(),
                            &nibbles[1..],
                            value,
                        ));
                    }

                    Node::Branch {
                        children,
                        value: None,
                    }
                } else {
                    let shared = nibbles[..common_len].to_vec();
                    let leaf_remainder = &leaf_key[common_len..];
                    let nibbles_remainder = &nibbles[common_len..];

                    let mut children: [Box<Node>; 16] = [
                        Box::new(Node::Null), Box::new(Node::Null), Box::new(Node::Null), Box::new(Node::Null),
                        Box::new(Node::Null), Box::new(Node::Null), Box::new(Node::Null), Box::new(Node::Null),
                        Box::new(Node::Null), Box::new(Node::Null), Box::new(Node::Null), Box::new(Node::Null),
                        Box::new(Node::Null), Box::new(Node::Null), Box::new(Node::Null), Box::new(Node::Null),
                    ];

                    if leaf_remainder.is_empty() {
                        if nibbles_remainder.is_empty() {
                            return Node::Leaf {
                                key: shared,
                                value,
                            };
                        }
                        children[nibbles_remainder[0] as usize] = Box::new(Node::Leaf {
                            key: nibbles_remainder[1..].to_vec(),
                            value,
                        });
                        let branch = Node::Branch {
                            children,
                            value: Some(leaf_value),
                        };
                        
                        if common_len > 0 {
                            return Node::Extension {
                                prefix: shared,
                                next: Box::new(branch),
                            };
                        }
                        return branch;
                    } else {
                        children[leaf_remainder[0] as usize] = Box::new(Node::Leaf {
                            key: leaf_remainder[1..].to_vec(),
                            value: leaf_value,
                        });
                    }

                    if nibbles_remainder.is_empty() {
                        let branch = Node::Branch {
                            children,
                            value: Some(value),
                        };
                        
                        if common_len > 0 {
                            return Node::Extension {
                                prefix: shared,
                                next: Box::new(branch),
                            };
                        }
                        return branch;
                    } else {
                        children[nibbles_remainder[0] as usize] = Box::new(Node::Leaf {
                            key: nibbles_remainder[1..].to_vec(),
                            value,
                        });
                    }

                    let branch = Node::Branch {
                        children,
                        value: None,
                    };

                    if common_len > 0 {
                        Node::Extension {
                            prefix: shared,
                            next: Box::new(branch),
                        }
                    } else {
                        branch
                    }
                }
            }
            Node::Extension { prefix, next } => {
                let common_len = Self::common_prefix_len(&prefix, nibbles);

                if common_len == prefix.len() {
                    let remaining = &nibbles[common_len..];
                    Node::Extension {
                        prefix,
                        next: Box::new(Self::insert_at(*next, remaining, value)),
                    }
                } else {
                    let shared = prefix[..common_len].to_vec();
                    let ext_remainder = &prefix[common_len..];
                    let nibbles_remainder = &nibbles[common_len..];

                    let mut children: [Box<Node>; 16] = [
                        Box::new(Node::Null), Box::new(Node::Null), Box::new(Node::Null), Box::new(Node::Null),
                        Box::new(Node::Null), Box::new(Node::Null), Box::new(Node::Null), Box::new(Node::Null),
                        Box::new(Node::Null), Box::new(Node::Null), Box::new(Node::Null), Box::new(Node::Null),
                        Box::new(Node::Null), Box::new(Node::Null), Box::new(Node::Null), Box::new(Node::Null),
                    ];

                    if ext_remainder.len() == 1 {
                        children[ext_remainder[0] as usize] = next;
                    } else {
                        children[ext_remainder[0] as usize] = Box::new(Node::Extension {
                            prefix: ext_remainder[1..].to_vec(),
                            next,
                        });
                    }

                    if nibbles_remainder.is_empty() {
                        let branch = Node::Branch {
                            children,
                            value: Some(value),
                        };
                        
                        if common_len > 0 {
                            return Node::Extension {
                                prefix: shared,
                                next: Box::new(branch),
                            };
                        }
                        return branch;
                    } else {
                        children[nibbles_remainder[0] as usize] = Box::new(Node::Leaf {
                            key: nibbles_remainder[1..].to_vec(),
                            value,
                        });
                    }

                    let branch = Node::Branch {
                        children,
                        value: None,
                    };

                    if common_len > 0 {
                        Node::Extension {
                            prefix: shared,
                            next: Box::new(branch),
                        }
                    } else {
                        branch
                    }
                }
            }
            Node::Branch { mut children, value: branch_value } => {
                if nibbles.is_empty() {
                    Node::Branch {
                        children,
                        value: Some(value),
                    }
                } else {
                    let idx = nibbles[0] as usize;
                    children[idx] = Box::new(Self::insert_at(
                        *children[idx].clone(),
                        &nibbles[1..],
                        value,
                    ));
                    Node::Branch {
                        children,
                        value: branch_value,
                    }
                }
            }
        }
    }

    fn get_at(node: &Node, nibbles: &[u8]) -> Option<Vec<u8>> {
        match node {
            Node::Null => None,
            Node::Leaf { key, value } => {
                if key == nibbles {
                    Some(value.clone())
                } else {
                    None
                }
            }
            Node::Extension { prefix, next } => {
                if nibbles.len() < prefix.len() {
                    return None;
                }
                if &nibbles[..prefix.len()] == prefix.as_slice() {
                    Self::get_at(next, &nibbles[prefix.len()..])
                } else {
                    None
                }
            }
            Node::Branch { children, value } => {
                if nibbles.is_empty() {
                    value.clone()
                } else {
                    let idx = nibbles[0] as usize;
                    Self::get_at(&children[idx], &nibbles[1..])
                }
            }
        }
    }

    fn get_proof_at(node: &Node, nibbles: &[u8], proof: &mut Vec<Vec<u8>>) {
        let encoded = rlp::encode(node);
        proof.push(encoded.to_vec());

        match node {
            Node::Null => {
            }
            Node::Leaf { key: _, value: _ } => {
            }
            Node::Extension { prefix, next } => {
                if nibbles.len() >= prefix.len() && &nibbles[..prefix.len()] == prefix.as_slice() {
                    Self::get_proof_at(next, &nibbles[prefix.len()..], proof);
                }
            }
            Node::Branch { children, value: _ } => {
                if !nibbles.is_empty() {
                    let idx = nibbles[0] as usize;
                    Self::get_proof_at(&children[idx], &nibbles[1..], proof);
                }
            }
        }
    }

    fn verify_proof_recursive(
        nibbles: &[u8],
        proof: &[Vec<u8>],
        proof_index: usize,
    ) -> Option<Vec<u8>> {
        if proof_index >= proof.len() {
            return None;
        }

        let current_item = &proof[proof_index];
        
        let node: Node = match rlp::decode(current_item) {
            Ok(n) => n,
            Err(_) => return None,
        };

        match node {
            Node::Null => None,
            Node::Leaf { key, value } => {
                if key == nibbles {
                    Some(value)
                } else {
                    None
                }
            }
            Node::Extension { prefix, next: _ } => {
                if nibbles.len() < prefix.len() || &nibbles[..prefix.len()] != prefix.as_slice() {
                    return None;
                }

                if proof_index + 1 >= proof.len() {
                    return None;
                }

                Self::verify_proof_recursive(&nibbles[prefix.len()..], proof, proof_index + 1)
            }
            Node::Branch { children: _, value } => {
                if nibbles.is_empty() {
                    value
                } else {
                    if proof_index + 1 >= proof.len() {
                        return None;
                    }

                    Self::verify_proof_recursive(&nibbles[1..], proof, proof_index + 1)
                }
            }
        }
    }

    fn compute_hash(data: &[u8]) -> [u8; 32] {
        let mut hasher = Keccak::v256();
        let mut output = [0u8; 32];
        hasher.update(data);
        hasher.finalize(&mut output);
        output
    }

    fn common_prefix_len(a: &[u8], b: &[u8]) -> usize {
        let mut len = 0;
        let min_len = a.len().min(b.len());
        for i in 0..min_len {
            if a[i] == b[i] {
                len += 1;
            } else {
                break;
            }
        }
        len
    }

    fn print_node(node: &Node, depth: usize, prefix: &str) {
        let indent = "  ".repeat(depth);
        
        match node {
            Node::Null => {
                println!("{}{}[null]", indent, prefix);
            }
            Node::Leaf { key, value } => {
                println!(
                    "{}{}[leaf] path: {:?}, value: {:?}",
                    indent,
                    prefix,
                    Self::format_nibbles(key),
                    String::from_utf8_lossy(value)
                );
            }
            Node::Extension { prefix: ext_prefix, next } => {
                println!(
                    "{}{}[extension] prefix: {:?}",
                    indent,
                    prefix,
                    Self::format_nibbles(ext_prefix)
                );
                Self::print_node(next, depth + 1, "└─ ");
            }
            Node::Branch { children, value } => {
                if let Some(v) = value {
                    println!(
                        "{}{}[branch] value: {:?}",
                        indent,
                        prefix,
                        String::from_utf8_lossy(v)
                    );
                } else {
                    println!("{}{}[branch]", indent, prefix);
                }
                
                for (i, child) in children.iter().enumerate() {
                    if !matches!(**child, Node::Null) {
                        let child_prefix = format!("[{:x}] ", i);
                        Self::print_node(child, depth + 1, &child_prefix);
                    }
                }
            }
        }
    }

    fn format_nibbles(nibbles: &[u8]) -> String {
        nibbles
            .iter()
            .map(|n| format!("{:x}", n))
            .collect::<Vec<_>>()
            .join("")
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

    #[test]
    fn test_insert_single_key() {
        let mut trie = EthTrie::new();
        let initial_hash = trie.root_hash();
        
        trie.insert(b"test", b"value");
        let after_insert_hash = trie.root_hash();
        
        assert_ne!(initial_hash, after_insert_hash, "root hash must change after insert");
    }

    #[test]
    fn test_insert_and_get_single_key() {
        let mut trie = EthTrie::new();
        
        trie.insert(b"test", b"value");
        let result = trie.get(b"test");
        
        assert_eq!(result, Some(b"value".to_vec()));
    }

    #[test]
    fn test_get_nonexistent_key() {
        let mut trie = EthTrie::new();
        
        trie.insert(b"test", b"value");
        let result = trie.get(b"other");
        
        assert_eq!(result, None);
    }

    #[test]
    fn test_insert_dog_then_do() {
        let mut trie = EthTrie::new();
        
        trie.insert(b"dog", b"puppy");
        let hash_after_dog = trie.root_hash();
        println!("hash after inserting 'dog': {}", hex::encode(hash_after_dog));
        
        trie.insert(b"do", b"verb");
        let hash_after_do = trie.root_hash();
        println!("hash after inserting 'do': {}", hex::encode(hash_after_do));
        
        assert_ne!(hash_after_dog, hash_after_do, "root hash must change after inserting 'do'");
        
        assert_eq!(trie.get(b"dog"), Some(b"puppy".to_vec()));
        assert_eq!(trie.get(b"do"), Some(b"verb".to_vec()));
    }

    #[test]
    fn test_update_existing_key() {
        let mut trie = EthTrie::new();
        
        trie.insert(b"key", b"value1");
        assert_eq!(trie.get(b"key"), Some(b"value1".to_vec()));
        
        trie.insert(b"key", b"value2");
        assert_eq!(trie.get(b"key"), Some(b"value2".to_vec()));
    }

    #[test]
    fn test_insert_with_shared_prefix() {
        let mut trie = EthTrie::new();
        
        trie.insert(b"test", b"value1");
        trie.insert(b"testing", b"value2");
        trie.insert(b"tea", b"value3");
        
        assert_eq!(trie.get(b"test"), Some(b"value1".to_vec()));
        assert_eq!(trie.get(b"testing"), Some(b"value2".to_vec()));
        assert_eq!(trie.get(b"tea"), Some(b"value3".to_vec()));
        assert_eq!(trie.get(b"te"), None);
    }

    #[test]
    fn test_insert_multiple_keys() {
        let mut trie = EthTrie::new();
        
        let keys = vec![b"a", b"b", b"c", b"d", b"e"];
        let values = vec![b"1", b"2", b"3", b"4", b"5"];
        
        for (key, value) in keys.iter().zip(values.iter()) {
            trie.insert(*key, *value);
        }
        
        for (key, value) in keys.iter().zip(values.iter()) {
            assert_eq!(trie.get(*key), Some((*value).to_vec()));
        }
    }

    #[test]
    fn test_fuzz_insert_and_get_100_keys() {
        use std::collections::HashMap;
        let mut trie = EthTrie::new();
        let mut expected = HashMap::new();
        
        for i in 0..100 {
            let key = format!("key_{}", i);
            let value = format!("value_{}", i);
            
            trie.insert(key.as_bytes(), value.as_bytes());
            expected.insert(key, value);
        }
        
        for (key, value) in expected.iter() {
            let result = trie.get(key.as_bytes());
            assert_eq!(
                result,
                Some(value.as_bytes().to_vec()),
                "failed to retrieve key: {}",
                key
            );
        }
    }

    #[test]
    fn test_insert_empty_key() {
        let mut trie = EthTrie::new();
        
        trie.insert(b"", b"empty_key_value");
        trie.insert(b"a", b"value_a");
        
        assert_eq!(trie.get(b""), Some(b"empty_key_value".to_vec()));
        assert_eq!(trie.get(b"a"), Some(b"value_a".to_vec()));
    }

    #[test]
    fn test_complex_branching() {
        let mut trie = EthTrie::new();
        
        trie.insert(b"do", b"verb");
        trie.insert(b"dog", b"puppy");
        trie.insert(b"doge", b"coin");
        trie.insert(b"horse", b"stallion");
        
        assert_eq!(trie.get(b"do"), Some(b"verb".to_vec()));
        assert_eq!(trie.get(b"dog"), Some(b"puppy".to_vec()));
        assert_eq!(trie.get(b"doge"), Some(b"coin".to_vec()));
        assert_eq!(trie.get(b"horse"), Some(b"stallion".to_vec()));
        assert_eq!(trie.get(b"d"), None);
    }

    #[test]
    fn test_get_proof_single_key() {
        let mut trie = EthTrie::new();
        trie.insert(b"test", b"value");
        
        let proof = trie.get_proof(b"test");
        
        assert!(!proof.is_empty(), "proof should not be empty");
    }

    #[test]
    fn test_get_proof_multiple_keys() {
        let mut trie = EthTrie::new();
        trie.insert(b"do", b"verb");
        trie.insert(b"dog", b"puppy");
        trie.insert(b"doge", b"coin");
        
        let proof_do = trie.get_proof(b"do");
        let proof_dog = trie.get_proof(b"dog");
        let proof_doge = trie.get_proof(b"doge");
        
        assert!(!proof_do.is_empty());
        assert!(!proof_dog.is_empty());
        assert!(!proof_doge.is_empty());
        
        assert!(proof_doge.len() >= proof_do.len());
    }

    #[test]
    fn test_verify_proof_valid() {
        let mut trie = EthTrie::new();
        trie.insert(b"test", b"value");
        
        let root_hash = trie.root_hash();
        let proof = trie.get_proof(b"test");
        
        let result = EthTrie::verify_proof(&root_hash, b"test", &proof);
        
        assert_eq!(result, Some(b"value".to_vec()), "valid proof should return the value");
    }

    #[test]
    fn test_verify_proof_invalid_key() {
        let mut trie = EthTrie::new();
        trie.insert(b"test", b"value");
        
        let root_hash = trie.root_hash();
        let proof = trie.get_proof(b"test");
        
        let result = EthTrie::verify_proof(&root_hash, b"other", &proof);
        
        assert_eq!(result, None, "invalid key should return None");
    }

    #[test]
    fn test_verify_proof_wrong_root_hash() {
        let mut trie = EthTrie::new();
        trie.insert(b"test", b"value");
        
        let proof = trie.get_proof(b"test");
        
        let wrong_hash = [0u8; 32];
        let result = EthTrie::verify_proof(&wrong_hash, b"test", &proof);
        
        assert_eq!(result, None, "wrong root hash should return None");
    }

    #[test]
    fn test_verify_proof_complex_trie() {
        let mut trie = EthTrie::new();
        
        trie.insert(b"do", b"verb");
        trie.insert(b"dog", b"puppy");
        trie.insert(b"doge", b"coin");
        trie.insert(b"horse", b"stallion");
        
        let root_hash = trie.root_hash();
        
        let test_cases: &[(&[u8], &[u8])] = &[
            (b"do", b"verb"),
            (b"dog", b"puppy"),
            (b"doge", b"coin"),
            (b"horse", b"stallion"),
        ];
        
        for (key, expected_value) in test_cases {
            let proof = trie.get_proof(*key);
            let result = EthTrie::verify_proof(&root_hash, *key, &proof);
            
            assert_eq!(
                result,
                Some(expected_value.to_vec()),
                "proof verification failed for key: {:?}",
                String::from_utf8_lossy(*key)
            );
        }
    }

    #[test]
    fn test_proof_for_nonexistent_key() {
        let mut trie = EthTrie::new();
        trie.insert(b"test", b"value");
        
        let root_hash = trie.root_hash();
        let proof = trie.get_proof(b"other");
        
        assert!(!proof.is_empty());
        
        let result = EthTrie::verify_proof(&root_hash, b"other", &proof);
        assert_eq!(result, None, "nonexistent key should return None");
    }

    #[test]
    fn test_empty_proof() {
        let trie = EthTrie::new();
        let root_hash = trie.root_hash();
        
        let result = EthTrie::verify_proof(&root_hash, b"test", &[]);
        assert_eq!(result, None);
    }

    #[test]
    fn test_proof_round_trip() {
        let mut trie = EthTrie::new();
        
        for i in 0..10 {
            let key = format!("key{}", i);
            let value = format!("value{}", i);
            trie.insert(key.as_bytes(), value.as_bytes());
        }
        
        let root_hash = trie.root_hash();
        
        for i in 0..10 {
            let key = format!("key{}", i);
            let expected_value = format!("value{}", i);
            
            let proof = trie.get_proof(key.as_bytes());
            let result = EthTrie::verify_proof(&root_hash, key.as_bytes(), &proof);
            
            assert_eq!(
                result,
                Some(expected_value.as_bytes().to_vec()),
                "round trip failed for key: {}",
                key
            );
        }
    }

    #[test]
    fn test_print_tree() {
        let mut trie = EthTrie::new();
        
        trie.print_tree();
        
        trie.insert(b"test", b"value");
        trie.insert(b"testing", b"another");
        
        trie.print_tree();
    }
}
