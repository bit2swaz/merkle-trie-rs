use rlp::{Encodable, RlpStream};
use tiny_keccak::{Hasher, Keccak};

use crate::nibbles::encode_compact;

#[derive(Clone, Debug)]
pub enum Node {
    Null,
    Leaf {
        key: Vec<u8>,
        value: Vec<u8>,
    },
    Extension {
        prefix: Vec<u8>,
        next: Box<Node>,
    },
    Branch {
        children: [Box<Node>; 16],
        value: Option<Vec<u8>>,
    },
}

impl Node {
    fn hash_or_raw(node: &Node) -> Vec<u8> {
        let encoded = rlp::encode(node);
        
        if encoded.len() < 32 {
            encoded.to_vec()
        } else {
            let mut hasher = Keccak::v256();
            let mut output = [0u8; 32];
            hasher.update(&encoded);
            hasher.finalize(&mut output);
            output.to_vec()
        }
    }
}

impl Encodable for Node {
    fn rlp_append(&self, s: &mut RlpStream) {
        match self {
            Node::Null => {
                s.append_empty_data();
            }
            Node::Leaf { key, value } => {
                s.begin_list(2);
                let encoded_path = encode_compact(key, true);
                s.append(&encoded_path);
                s.append(value);
            }
            Node::Extension { prefix, next } => {
                s.begin_list(2);
                let encoded_path = encode_compact(prefix, false);
                s.append(&encoded_path);
                let next_data = Node::hash_or_raw(next);
                s.append(&next_data);
            }
            Node::Branch { children, value } => {
                s.begin_list(17);
                
                for child in children.iter() {
                    let child_data = Node::hash_or_raw(child);
                    s.append(&child_data);
                }
                
                match value {
                    Some(v) => s.append(v),
                    None => s.append_empty_data(),
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_node_encoding() {
        let node = Node::Null;
        let encoded = rlp::encode(&node);
        assert_eq!(encoded.to_vec(), vec![0x80]);
    }

    #[test]
    fn test_leaf_node_encoding() {
        let node = Node::Leaf {
            key: vec![0xA, 0xB, 0xC],
            value: vec![0x01, 0x02, 0x03],
        };
        let encoded = rlp::encode(&node);
        assert!(encoded.len() > 0);
        assert!(encoded[0] >= 0xc0);
    }

    #[test]
    fn test_extension_node_encoding() {
        let next_node = Node::Null;
        let node = Node::Extension {
            prefix: vec![0x1, 0x2],
            next: Box::new(next_node),
        };
        let encoded = rlp::encode(&node);
        assert!(encoded.len() > 0);
        assert!(encoded[0] >= 0xc0);
    }

    #[test]
    fn test_branch_node_encoding() {
        let children: [Box<Node>; 16] = [
            Box::new(Node::Null), Box::new(Node::Null), Box::new(Node::Null), Box::new(Node::Null),
            Box::new(Node::Null), Box::new(Node::Null), Box::new(Node::Null), Box::new(Node::Null),
            Box::new(Node::Null), Box::new(Node::Null), Box::new(Node::Null), Box::new(Node::Null),
            Box::new(Node::Null), Box::new(Node::Null), Box::new(Node::Null), Box::new(Node::Null),
        ];
        let node = Node::Branch {
            children,
            value: None,
        };
        let encoded = rlp::encode(&node);
        assert!(encoded.len() > 0);
    }

    #[test]
    fn test_branch_node_with_value() {
        let children: [Box<Node>; 16] = [
            Box::new(Node::Null), Box::new(Node::Null), Box::new(Node::Null), Box::new(Node::Null),
            Box::new(Node::Null), Box::new(Node::Null), Box::new(Node::Null), Box::new(Node::Null),
            Box::new(Node::Null), Box::new(Node::Null), Box::new(Node::Null), Box::new(Node::Null),
            Box::new(Node::Null), Box::new(Node::Null), Box::new(Node::Null), Box::new(Node::Null),
        ];
        let node = Node::Branch {
            children,
            value: Some(vec![0xAA, 0xBB]),
        };
        let encoded = rlp::encode(&node);
        assert!(encoded.len() > 0);
    }

    #[test]
    fn test_hash_or_raw_small_node() {
        let node = Node::Null;
        let result = Node::hash_or_raw(&node);
        assert_eq!(result, vec![0x80]);
        assert!(result.len() < 32);
    }

    #[test]
    fn test_hash_or_raw_large_node() {
        let children: [Box<Node>; 16] = [
            Box::new(Node::Leaf { key: vec![0x1, 0x2, 0x3, 0x4], value: vec![0xAA; 10] }),
            Box::new(Node::Null), Box::new(Node::Null), Box::new(Node::Null),
            Box::new(Node::Null), Box::new(Node::Null), Box::new(Node::Null), Box::new(Node::Null),
            Box::new(Node::Null), Box::new(Node::Null), Box::new(Node::Null), Box::new(Node::Null),
            Box::new(Node::Null), Box::new(Node::Null), Box::new(Node::Null), Box::new(Node::Null),
        ];
        let node = Node::Branch {
            children,
            value: Some(vec![0xFF; 20]),
        };
        
        let result = Node::hash_or_raw(&node);
        assert_eq!(result.len(), 32);
    }
}
