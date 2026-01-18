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
}
