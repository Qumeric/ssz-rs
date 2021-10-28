use crate::de::{deserialize_homogeneous_composite, Deserialize, DeserializeError};
use crate::merkleization::{
    merkleize, pack, Context, MerkleizationError, Merkleized, Root, BYTES_PER_CHUNK,
};
use crate::ser::{serialize_composite, Serialize, SerializeError};
use crate::{SimpleSerialize, Sized};
use std::convert::TryInto;
use std::fmt;
use std::iter::FromIterator;
use std::ops::{Deref, DerefMut};

/// A homogenous collection of a fixed number of values.
/// NOTE: a `Vector` of length `0` is illegal.
#[derive(PartialEq, Eq, Clone)]
pub struct Vector<T: SimpleSerialize, const N: usize>([T; N]);

impl<T, const N: usize> fmt::Debug for Vector<T, N>
where
    T: SimpleSerialize + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Vector<{}>{:?}", N, self.0)
    }
}

impl<T, const N: usize> Default for Vector<T, N>
where
    T: SimpleSerialize + Default + Clone,
{
    fn default() -> Self {
        let inner = vec![T::default(); N];
        let inner = inner.try_into().unwrap_or_else(|_| {
            // NOTE: using the error from `try_into` demands that `T` also implement
            // `fmt::Debug` which bubbles up into all instances of `Vector`.
            // Just ignore the error with a simple panic for now...
            panic!("could not construct inner array type from default vector")
        });
        Self(inner)
    }
}

impl<T, const N: usize> Deref for Vector<T, N>
where
    T: SimpleSerialize,
{
    type Target = [T; N];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, const N: usize> DerefMut for Vector<T, N>
where
    T: SimpleSerialize,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T, const N: usize> Sized for Vector<T, N>
where
    T: SimpleSerialize,
{
    fn is_variable_size() -> bool {
        T::is_variable_size()
    }

    fn size_hint() -> usize {
        T::size_hint() * N
    }
}

impl<T, const N: usize> Serialize for Vector<T, N>
where
    T: SimpleSerialize,
{
    fn serialize(&self, buffer: &mut Vec<u8>) -> Result<usize, SerializeError> {
        if N == 0 {
            return Err(SerializeError::IllegalType { bound: N });
        }
        serialize_composite(&self.0, buffer)
    }
}

impl<T, const N: usize> Deserialize for Vector<T, N>
where
    T: SimpleSerialize,
{
    fn deserialize(encoding: &[u8]) -> Result<Self, DeserializeError> {
        if N == 0 {
            return Err(DeserializeError::IllegalType { bound: N });
        }
        if !T::is_variable_size() {
            let expected_length = N * T::size_hint();
            if encoding.len() < expected_length {
                return Err(DeserializeError::InputTooShort);
            }
            if encoding.len() > expected_length {
                return Err(DeserializeError::ExtraInput);
            }
        }
        let elements = deserialize_homogeneous_composite(encoding)?;
        elements
            .try_into()
            .map(Vector)
            .map_err(|_| DeserializeError::InputTooShort)
    }
}

impl<T, const N: usize> SimpleSerialize for Vector<T, N> where T: SimpleSerialize + Copy {}

impl<T, const N: usize> Merkleized for Vector<T, N>
where
    T: SimpleSerialize,
{
    fn hash_tree_root(&self, context: &Context) -> Result<Root, MerkleizationError> {
        if T::is_composite_type() {
            let mut chunks = vec![0u8; self.len() * BYTES_PER_CHUNK];
            for (i, elem) in self.iter().enumerate() {
                let chunk = elem.hash_tree_root(context)?;
                let range = i * BYTES_PER_CHUNK..(i + 1) * BYTES_PER_CHUNK;
                chunks[range].copy_from_slice(&chunk);
            }
            Ok(merkleize(&chunks, None, context)?)
        } else {
            let chunks = pack(&self.0)?;
            Ok(merkleize(&chunks, None, context)?)
        }
    }
}

impl<T, const N: usize> FromIterator<T> for Vector<T, N>
where
    T: SimpleSerialize + Default + Clone,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let inner = iter.into_iter().take(N).collect::<Vec<_>>();
        let inner = inner.try_into().unwrap_or_else(|_| {
            // NOTE: using the error from `try_into` demands that `T` also implement
            // `fmt::Debug` which bubbles up into all instances of `Vector`.
            // Just ignore the error with a simple panic for now...
            panic!("could not construct inner array type from default vector")
        });
        Self(inner)
    }
}

impl<T, const N: usize> IntoIterator for Vector<T, N>
where
    T: SimpleSerialize,
{
    type Item = T;
    type IntoIter = std::array::IntoIter<T, N>;

    fn into_iter(self) -> Self::IntoIter {
        std::array::IntoIter::new(self.0)
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a Vector<T, N>
where
    T: SimpleSerialize,
{
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a mut Vector<T, N>
where
    T: SimpleSerialize,
{
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::list::List;
    use crate::serialize;

    const COUNT: usize = 32;

    #[test]
    fn encode_vector() {
        let mut value: Vector<u16, COUNT> = Vector::default();
        for elem in &mut value {
            *elem = 33u16;
        }
        value[0] = 34u16;
        assert_eq!(value[0], 34u16);
        value[0] = 33u16;
        let encoding = serialize(&value).expect("can encode");
        let expected = [
            33u8, 0u8, 33u8, 0u8, 33u8, 0u8, 33u8, 0u8, 33u8, 0u8, 33u8, 0u8, 33u8, 0u8, 33u8, 0u8,
            33u8, 0u8, 33u8, 0u8, 33u8, 0u8, 33u8, 0u8, 33u8, 0u8, 33u8, 0u8, 33u8, 0u8, 33u8, 0u8,
            33u8, 0u8, 33u8, 0u8, 33u8, 0u8, 33u8, 0u8, 33u8, 0u8, 33u8, 0u8, 33u8, 0u8, 33u8, 0u8,
            33u8, 0u8, 33u8, 0u8, 33u8, 0u8, 33u8, 0u8, 33u8, 0u8, 33u8, 0u8, 33u8, 0u8, 33u8, 0u8,
        ];
        assert_eq!(encoding, expected);
    }

    #[test]
    fn decode_vector() {
        let bytes = vec![
            0u8, 1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 0u8, 1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 0u8,
            1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 0u8, 1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8,
        ];
        let result = Vector::<u8, COUNT>::deserialize(&bytes).expect("can deserialize");
        let expected: Vector<u8, COUNT> = Vector(bytes.try_into().expect("test data"));
        assert_eq!(result, expected);
    }

    #[test]
    fn decode_variable_vector() {
        const COUNT: usize = 4;
        let mut inner: Vec<List<u8, 1>> = (0..4)
            .map(|i| std::array::IntoIter::new([i as u8]).collect())
            .collect();
        let permutation = &mut inner[3];
        let _ = permutation.pop().expect("test data correct");
        let input: Vector<List<u8, 1>, COUNT> =
            Vector(inner.try_into().expect("test data correct"));
        let mut buffer = vec![];
        let _ = input.serialize(&mut buffer).expect("can serialize");
        let expected = vec![16, 0, 0, 0, 17, 0, 0, 0, 18, 0, 0, 0, 19, 0, 0, 0, 0, 1, 2];
        assert_eq!(buffer, expected);
    }

    #[test]
    fn roundtrip_vector() {
        let bytes = vec![
            0u8, 1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 0u8, 1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 0u8,
            1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 0u8, 1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8,
        ];
        let input: Vector<u8, COUNT> = Vector(bytes.try_into().expect("test data"));
        let mut buffer = vec![];
        let _ = input.serialize(&mut buffer).expect("can serialize");
        let recovered = Vector::<u8, COUNT>::deserialize(&buffer).expect("can decode");
        assert_eq!(input, recovered);
    }

    #[test]
    fn roundtrip_variable_vector() {
        const COUNT: usize = 4;
        let mut inner: Vec<List<u8, 1>> = (0..4)
            .map(|i| std::array::IntoIter::new([i as u8]).collect())
            .collect();
        let permutation = &mut inner[3];
        let _ = permutation.pop().expect("test data correct");
        let input: Vector<List<u8, 1>, COUNT> =
            Vector(inner.try_into().expect("test data correct"));
        let mut buffer = vec![];
        let _ = input.serialize(&mut buffer).expect("can serialize");
        let recovered = Vector::<List<u8, 1>, COUNT>::deserialize(&buffer).expect("can decode");
        assert_eq!(input, recovered);
    }
}