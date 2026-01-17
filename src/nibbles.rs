#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nibbles {
    data: Vec<u8>,
}

impl Nibbles {
    pub fn new(data: Vec<u8>) -> Self {
        Nibbles { data }
    }

    pub fn from_raw(data: &[u8], _is_leaf: bool) -> Self {
        let mut nibbles = Vec::with_capacity(data.len() * 2);
        for &byte in data {
            nibbles.push(byte >> 4);
            nibbles.push(byte & 0x0F);
        }
        Nibbles { data: nibbles }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<u8> {
        self.data.get(index).copied()
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    pub fn get_common_prefix_length(a: &Nibbles, b: &Nibbles) -> usize {
        let mut count = 0;
        let min_len = a.len().min(b.len());
        
        for i in 0..min_len {
            if a.data[i] == b.data[i] {
                count += 1;
            } else {
                break;
            }
        }
        
        count
    }
}

pub fn encode_compact(nibbles: &[u8], is_leaf: bool) -> Vec<u8> {
    let len = nibbles.len();
    let is_odd = len % 2 == 1;
    
    let mut result = Vec::new();
    
    if is_odd {
        let flag = if is_leaf { 0x3 } else { 0x1 };
        let first_byte = (flag << 4) | nibbles[0];
        result.push(first_byte);
        
        for i in (1..len).step_by(2) {
            result.push((nibbles[i] << 4) | nibbles[i + 1]);
        }
    } else {
        let flag = if is_leaf { 0x20 } else { 0x00 };
        result.push(flag);
        
        for i in (0..len).step_by(2) {
            result.push((nibbles[i] << 4) | nibbles[i + 1]);
        }
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_raw_single_byte() {
        let nibbles = Nibbles::from_raw(&[0xAB], true);
        assert_eq!(nibbles.data, vec![10, 11]);
        assert_eq!(nibbles.len(), 2);
    }

    #[test]
    fn test_from_raw_multiple_bytes() {
        let nibbles = Nibbles::from_raw(&[0x12, 0x34], true);
        assert_eq!(nibbles.data, vec![1, 2, 3, 4]);
        assert_eq!(nibbles.len(), 4);
    }

    #[test]
    fn test_from_raw_zero_byte() {
        let nibbles = Nibbles::from_raw(&[0x00], false);
        assert_eq!(nibbles.data, vec![0, 0]);
    }

    #[test]
    fn test_from_raw_max_byte() {
        let nibbles = Nibbles::from_raw(&[0xFF], true);
        assert_eq!(nibbles.data, vec![15, 15]);
    }

    #[test]
    fn test_from_raw_empty() {
        let nibbles = Nibbles::from_raw(&[], true);
        assert_eq!(nibbles.data, Vec::<u8>::new());
        assert_eq!(nibbles.len(), 0);
        assert!(nibbles.is_empty());
    }

    #[test]
    fn test_from_raw_mixed_values() {
        let nibbles = Nibbles::from_raw(&[0xA0, 0x0F, 0x5A], false);
        assert_eq!(nibbles.data, vec![10, 0, 0, 15, 5, 10]);
    }

    #[test]
    fn test_get_common_prefix_length_identical() {
        let n1 = Nibbles::from_raw(&[0xAB, 0xCD], true);
        let n2 = Nibbles::from_raw(&[0xAB, 0xCD], true);
        assert_eq!(Nibbles::get_common_prefix_length(&n1, &n2), 4);
    }

    #[test]
    fn test_get_common_prefix_length_no_match() {
        let n1 = Nibbles::from_raw(&[0xAB], true);
        let n2 = Nibbles::from_raw(&[0xCD], true);
        assert_eq!(Nibbles::get_common_prefix_length(&n1, &n2), 0);
    }

    #[test]
    fn test_get_common_prefix_length_partial_match() {
        let n1 = Nibbles::from_raw(&[0xAB], true);
        let n2 = Nibbles::from_raw(&[0xAD], true);
        assert_eq!(Nibbles::get_common_prefix_length(&n1, &n2), 1);
    }

    #[test]
    fn test_get_common_prefix_length_different_lengths() {
        let n1 = Nibbles::from_raw(&[0xAB, 0xCD], true);
        let n2 = Nibbles::from_raw(&[0xAB], true);
        assert_eq!(Nibbles::get_common_prefix_length(&n1, &n2), 2);
    }

    #[test]
    fn test_get_common_prefix_length_empty() {
        let n1 = Nibbles::from_raw(&[], true);
        let n2 = Nibbles::from_raw(&[0xAB], true);
        assert_eq!(Nibbles::get_common_prefix_length(&n1, &n2), 0);
    }

    #[test]
    fn test_get_common_prefix_length_both_empty() {
        let n1 = Nibbles::from_raw(&[], true);
        let n2 = Nibbles::from_raw(&[], true);
        assert_eq!(Nibbles::get_common_prefix_length(&n1, &n2), 0);
    }

    #[test]
    fn test_get_common_prefix_length_longer_match() {
        let n1 = Nibbles::from_raw(&[0x12, 0x34, 0x56], true);
        let n2 = Nibbles::from_raw(&[0x12, 0x34, 0x78], true);
        assert_eq!(Nibbles::get_common_prefix_length(&n1, &n2), 4);
    }

    #[test]
    fn test_nibble_values_in_range() {
        let data = vec![0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
                       0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
        let nibbles = Nibbles::from_raw(&data, true);
        
        for &nibble in &nibbles.data {
            assert!(nibble <= 15, "nibble value {} exceeds maximum of 15", nibble);
        }
    }

    #[test]
    fn test_get_method() {
        let nibbles = Nibbles::from_raw(&[0xAB], true);
        assert_eq!(nibbles.get(0), Some(10));
        assert_eq!(nibbles.get(1), Some(11));
        assert_eq!(nibbles.get(2), None);
    }

    #[test]
    fn test_as_slice() {
        let nibbles = Nibbles::from_raw(&[0x12], true);
        let slice = nibbles.as_slice();
        assert_eq!(slice, &[1, 2]);
    }

    #[test]
    fn test_byte_splitting_comprehensive() {
        let test_cases = vec![
            (0x00, vec![0, 0]),
            (0x01, vec![0, 1]),
            (0x10, vec![1, 0]),
            (0x0F, vec![0, 15]),
            (0xF0, vec![15, 0]),
            (0x5A, vec![5, 10]),
            (0xA5, vec![10, 5]),
            (0xFF, vec![15, 15]),
        ];

        for (byte_val, expected) in test_cases {
            let nibbles = Nibbles::from_raw(&[byte_val], true);
            assert_eq!(
                nibbles.data, expected,
                "failed for byte value 0x{:02X}",
                byte_val
            );
        }
    }
    #[test]
    fn test_encode_compact_leaf_odd_length() {
        let nibbles = [0xA, 0xB, 0xC];
        let result = encode_compact(&nibbles, true);
        assert_eq!(result, vec![0x3A, 0xBC]);
    }

    #[test]
    fn test_encode_compact_leaf_even_length() {
        let nibbles = [0xA, 0xB];
        let result = encode_compact(&nibbles, true);
        assert_eq!(result, vec![0x20, 0xAB]);
    }

    #[test]
    fn test_encode_compact_extension_even_length() {
        let nibbles = [0xA, 0xB];
        let result = encode_compact(&nibbles, false);
        assert_eq!(result, vec![0x00, 0xAB]);
    }

    #[test]
    fn test_encode_compact_extension_odd_length() {
        let nibbles = [0xA, 0xB, 0xC];
        let result = encode_compact(&nibbles, false);
        assert_eq!(result, vec![0x1A, 0xBC]);
    }

    #[test]
    fn test_encode_compact_leaf_empty() {
        let nibbles: [u8; 0] = [];
        let result = encode_compact(&nibbles, true);
        assert_eq!(result, vec![0x20]);
    }

    #[test]
    fn test_encode_compact_extension_empty() {
        let nibbles: [u8; 0] = [];
        let result = encode_compact(&nibbles, false);
        assert_eq!(result, vec![0x00]);
    }

    #[test]
    fn test_encode_compact_leaf_single_nibble() {
        let nibbles = [0x5];
        let result = encode_compact(&nibbles, true);
        assert_eq!(result, vec![0x35]);
    }

    #[test]
    fn test_encode_compact_extension_single_nibble() {
        let nibbles = [0x5];
        let result = encode_compact(&nibbles, false);
        assert_eq!(result, vec![0x15]);
    }

    #[test]
    fn test_encode_compact_leaf_four_nibbles() {
        let nibbles = [0x1, 0x2, 0x3, 0x4];
        let result = encode_compact(&nibbles, true);
        assert_eq!(result, vec![0x20, 0x12, 0x34]);
    }

    #[test]
    fn test_encode_compact_extension_four_nibbles() {
        let nibbles = [0x1, 0x2, 0x3, 0x4];
        let result = encode_compact(&nibbles, false);
        assert_eq!(result, vec![0x00, 0x12, 0x34]);
    }

    #[test]
    fn test_encode_compact_leaf_five_nibbles() {
        let nibbles = [0x1, 0x2, 0x3, 0x4, 0x5];
        let result = encode_compact(&nibbles, true);
        assert_eq!(result, vec![0x31, 0x23, 0x45]);
    }

    #[test]
    fn test_encode_compact_extension_five_nibbles() {
        let nibbles = [0x1, 0x2, 0x3, 0x4, 0x5];
        let result = encode_compact(&nibbles, false);
        assert_eq!(result, vec![0x11, 0x23, 0x45]);
    }

    #[test]
    fn test_encode_compact_with_zeros() {
        let nibbles = [0x0, 0x0, 0xF, 0xF];
        let result = encode_compact(&nibbles, true);
        assert_eq!(result, vec![0x20, 0x00, 0xFF]);
    }

    #[test]
    fn test_encode_compact_all_max_nibbles() {
        let nibbles = [0xF, 0xF, 0xF];
        let result = encode_compact(&nibbles, true);
        assert_eq!(result, vec![0x3F, 0xFF]);
    }

    #[test]
    fn test_encode_compact_alternating_pattern() {
        let nibbles = [0xA, 0x5, 0xA, 0x5];
        let result = encode_compact(&nibbles, false);
        assert_eq!(result, vec![0x00, 0xA5, 0xA5]);
    }
}
