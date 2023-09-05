use crate::{
    de::{Deserialize, DeserializeError},
    lib::*,
    merkleization::{MerkleizationError, Merkleized, Node},
    ser::{Serialize, SerializeError},
    Serializable, SimpleSerialize,
};
use ruint::Uint;

// impl U256 {
//     pub fn new() -> Self {
//         Self(BigUint::default())
//     }

//     pub fn zero() -> Self {
//         Self::default()
//     }

//     pub fn try_from_bytes_le(bytes: &[u8]) -> Result<Self, DeserializeError> {
//         Self::deserialize(bytes)
//     }

//     pub fn from_bytes_le(bytes: [u8; 32]) -> Self {
//         Self::deserialize(&bytes).unwrap()
//     }

//     pub fn to_bytes_le(&self) -> Vec<u8> {
//         let mut bytes = self.0.to_bytes_le();
//         bytes.resize(Self::size_hint(), 0u8);
//         bytes
//     }

//     pub fn from_hex(data: &str) -> Option<Self> {
//         let data = data.strip_prefix("0x").unwrap_or(data);
//         BigUint::parse_bytes(data.as_bytes(), 16).map(Self)
//     }
// }

impl<const BITS: usize, const LIMBS: usize> Serializable for Uint<BITS, LIMBS> {
    fn is_variable_size() -> bool {
        false
    }

    fn size_hint() -> usize {
        LIMBS * 8
    }
}

impl<const BITS: usize, const LIMBS: usize> Serialize for Uint<BITS, LIMBS> {
    fn serialize(&self, buffer: &mut Vec<u8>) -> Result<usize, SerializeError> {
        buffer.extend_from_slice(&self.to_le_bytes_vec());
        Ok(Self::size_hint())
    }
}

impl<const BITS: usize, const LIMBS: usize> Deserialize for Uint<BITS, LIMBS> {
    fn deserialize(encoding: &[u8]) -> Result<Self, DeserializeError> {
        let byte_size = Self::size_hint();
        if encoding.len() < byte_size {
            return Err(DeserializeError::ExpectedFurtherInput {
                provided: encoding.len(),
                expected: byte_size,
            });
        }
        if encoding.len() > byte_size {
            return Err(DeserializeError::AdditionalInput {
                provided: encoding.len(),
                expected: byte_size,
            });
        }

        // SAFETY: index is safe because encoding.len() == byte_size; qed
        Ok(Self::try_from_le_slice(&encoding[..byte_size]).unwrap())
    }
}

impl<const BITS: usize, const LIMBS: usize> Merkleized for Uint<BITS, LIMBS> {
    fn hash_tree_root(&mut self) -> Result<Node, MerkleizationError> {
        let data: Vec<u8> = self.to_le_bytes_vec();
        let node = Node::try_from(data.as_ref()).expect("is right size");
        Ok(node)
    }

    fn is_composite_type() -> bool {
        false
    }
}

impl<const BITS: usize, const LIMBS: usize> SimpleSerialize for Uint<BITS, LIMBS> {}

// #[cfg(feature = "serde")]
// impl serde::Serialize for U256 {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         let output = format!("{}", self.0);
//         serializer.collect_str(&output)
//     }
// }

// #[cfg(feature = "serde")]
// impl<'de> serde::Deserialize<'de> for U256 {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: serde::Deserializer<'de>,
//     {
//         let s = <String>::deserialize(deserializer)?;
//         let value = s.parse::<BigUint>().map_err(serde::de::Error::custom)?;
//         Ok(Self(value))
//     }
// }

#[cfg(test)]
mod tests {
    use ruint::aliases::U256;

    use super::*;
    use crate::serialize;

    #[test]
    fn encode_ruints() {
        let tests = vec![(u8::default(), [0u8]), (2u8, [2u8]), (u8::MAX, [u8::MAX])];
        for (value, expected) in tests {
            let result = serialize(&value).expect("can encode");
            assert_eq!(result, expected);
        }
        let tests =
            vec![(2u16, [2u8, 0u8]), (1337u16, [57u8, 5u8]), (u16::MAX, [u8::MAX, u8::MAX])];
        for (value, expected) in tests {
            let result = serialize(&value).expect("can encode");
            assert_eq!(result, expected);
        }
        let tests = vec![
            (2u32, [2u8, 0u8, 0u8, 0u8]),
            (1337u32, [57u8, 5u8, 0u8, 0u8]),
            (u32::MAX, [u8::MAX, u8::MAX, u8::MAX, u8::MAX]),
        ];
        for (value, expected) in tests {
            let result = serialize(&value).expect("can encode");
            assert_eq!(result, expected);
        }
        let tests = vec![
            (2u64, [2u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8]),
            (1337u64, [57u8, 5u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8]),
            (u64::MAX, [u8::MAX; 8]),
        ];
        for (value, expected) in tests {
            let result = serialize(&value).expect("can encode");
            assert_eq!(result, expected);
        }
        let tests = vec![
            (
                2u128,
                [2u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8],
            ),
            (
                1337u128,
                [57u8, 5u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8],
            ),
            (u128::MAX, [u8::MAX; 16]),
        ];
        for (value, expected) in tests {
            let result = serialize(&value).expect("can encode");
            assert_eq!(result, expected);
        }
        let tests = vec![
            (U256::from_le_bytes([2u8; 32]), [2u8; 32]),
            (U256::from_le_bytes([u8::MAX; 32]), [u8::MAX; 32]),
        ];
        for (value, expected) in tests {
            let result = serialize(&value).expect("can encode");
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn decode_ruints() {
        let tests = vec![(u8::default(), [0u8]), (2u8, [2u8]), (u8::MAX, [u8::MAX])];
        for (expected, bytes) in tests {
            let result = u8::deserialize(&bytes).expect("can encode");
            assert_eq!(result, expected);
        }
        let tests =
            vec![(2u16, [2u8, 0u8]), (1337u16, [57u8, 5u8]), (u16::MAX, [u8::MAX, u8::MAX])];
        for (expected, bytes) in tests {
            let result = u16::deserialize(&bytes).expect("can encode");
            assert_eq!(result, expected);
        }
        let tests = vec![
            (2u32, [2u8, 0u8, 0u8, 0u8]),
            (1337u32, [57u8, 5u8, 0u8, 0u8]),
            (u32::MAX, [u8::MAX, u8::MAX, u8::MAX, u8::MAX]),
        ];
        for (expected, bytes) in tests {
            let result = u32::deserialize(&bytes).expect("can encode");
            assert_eq!(result, expected);
        }
        let tests = vec![
            (2u64, [2u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8]),
            (1337u64, [57u8, 5u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8]),
            (u64::MAX, [u8::MAX; 8]),
        ];
        for (expected, bytes) in tests {
            let result = u64::deserialize(&bytes).expect("can encode");
            assert_eq!(result, expected);
        }
        let tests = vec![
            (
                2u128,
                [2u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8],
            ),
            (
                1337u128,
                [57u8, 5u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8],
            ),
            (u128::MAX, [u8::MAX; 16]),
        ];
        for (expected, bytes) in tests {
            let result = u128::deserialize(&bytes).expect("can encode");
            assert_eq!(result, expected);
        }
        let tests = vec![
            (U256::from_le_bytes([2u8; 32]), [2u8; 32]),
            (U256::from_le_bytes([u8::MAX; 32]), [u8::MAX; 32]),
        ];
        for (expected, bytes) in tests {
            let result = U256::deserialize(&bytes).expect("can encode");
            assert_eq!(result, expected);
        }
    }
}
