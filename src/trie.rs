use sp_core::blake2_256;
use sp_core::keccak_256;
use crate::types::{Hash};
use crate::codec::{Encode, EncodeSize};
use crate::codec::history::{Mmr, MmrPeak};

// State Merklization involves transforming the serialized mapping into a cryptographic commitment. 
// We define this commitment as the root of the binary Patricia Merkle Trie with a format optimized 
// for modern compute hardware, primarily by optimizing sizes to fit succinctly into typical memory
// layouts and reducing the need for unpredictable branching.

// Hash function used us blake 256
fn hash(data: &[u8]) -> Hash {
    return blake2_256(data);
}

fn bit(k: &[u8], i: usize) -> bool {
    (k[i >> 3] & (1 << (7 - (i & 7)))) != 0
}

#[derive(Debug, PartialEq)]
pub enum MerkleError {
    LengthMismatch(usize),
    OutOfRange,
}

fn branch(l: &[u8], r: &[u8]) -> Result<[u8; 64], MerkleError> {
    // Assert keys have 32 bytes
    if l.len() != 32 || r.len() != 32 {
        return Err(MerkleError::LengthMismatch(64));
    }
    
    let mut branch_encoded = Vec::with_capacity(64);
    // Each node is either a branch or a leaf. The first bit discriminate between these two types.
    let head = l[0] & 0x7f;  
    head.encode_to(&mut branch_encoded);
    // Use the last 255 bits of the 0-bit (left) sub-trie identity
    l[1..].encode_to(&mut branch_encoded);
    // Use the full 256 bits of the 1-bit (right) sub-trie identity
    r.encode_to(&mut branch_encoded);
    let mut branch_hash = [0u8; 64];
    branch_hash.copy_from_slice(&branch_encoded);
    return Ok(branch_hash);
}

fn leaf(k: &[u8], v: &[u8]) -> [u8; 64] {
    let mut encoded = Vec::with_capacity(64);
    // Leaf nodes are further subdivided into embedded-value leaves and regular leaves
    // The second bit of the node discriminates between these.
    if v.len() <= 32 {
        // In the case of embedded-value leaf, the remaining 6 bits of the first byte are used 
        // to store the embedded value size
        let v_len_1 = v.len().encode_size(1).pop().unwrap() & 0x3F;
        let head = (0b10000000 | v_len_1) as u8;
        head.encode_to(&mut encoded);
        // The following 31 bytes are dedicated to the first 31 bytes of the key
        k[..31].encode_to(&mut encoded);
        // The last 32 bytes are defined as the value
        v.encode_to(&mut encoded);
        // Fill with zeroes if its length is less than 32 bytes
        vec![0; 32 - v_len_1 as usize].encode_to(&mut encoded);
    } else {
        // In the case of a regular leaf, the remaining 6 bits of the first byte are zeroed
        let head = 0b11000000 as u8;
        head.encode_to(&mut encoded);
        // The following 31 bytes store the first 31 bytes of the key
        k[..31].encode_to(&mut encoded);
        // The last 32 bytes store the hash of the value 
        hash(v).encode_to(&mut encoded);
    }
    let mut leaf_hash = [0u8; 64];
    leaf_hash.copy_from_slice(&encoded);
    leaf_hash
}

pub fn merkle(kvs: &[(Vec<u8>, Vec<u8>)], i: usize) -> Result<Hash, MerkleError> {
    // Empty (sub-)tries are identified as the zero hash
    if kvs.is_empty() {
        return Ok([0u8; 32]);
    }
    // Generate a leaf if there only is one element
    if kvs.len() == 1 {
        let (k, v) = &kvs[0];
        return Ok(hash(&leaf(k, v)));
    }
    // Right and left vectors
    let mut l = Vec::new(); 
    let mut r = Vec::new(); 
    
    for (k, v) in kvs {
        // Determine if kv has to be on right or left
        if bit(k, i) {
            r.push((k.clone(), v.clone()));
        } else {
            l.push((k.clone(), v.clone()));
        }
    }
    // Recursive calls to calculate letf and right 
    let left_hash = merkle(&l, i + 1)?;
    let right_hash = merkle(&r, i + 1)?;

    let branch_value = branch(&left_hash, &right_hash)?;
    // Blake 256 of branch value
    Ok(hash(&branch_value))
}

fn node<T>(v: &[u8], hash_fn: fn(&[u8]) -> Hash) -> T
where
    T: From<Vec<u8>> + From<Hash>,
{
    let len = v.len();

    if len == 0 {
        return T::from([0u8; 32]);
    } else if len == 1 {
        return T::from(v.to_vec());
    } else {
        let prefix  = Vec::from("node");
        let left    = node::<Vec<u8>>(&v[..(len / 2)], hash_fn); 
        let right   = node::<Vec<u8>>(&v[(len / 2)..], hash_fn); 
        
        let mut combined = prefix.clone();
        combined.extend_from_slice(&left);
        combined.extend_from_slice(&right);

        return T::from(hash_fn(&combined));
    }
}

pub fn merkle_b(v: &[u8], hash_fn: fn(&[u8]) -> Hash) -> Hash {

    if v.len() == 1 {
        return hash_fn(&[v[0]]);
    }

    let mut hash = [0u8; 32];
    hash.copy_from_slice(&node::<Vec<u8>>(v, hash_fn));
    return hash;
}

pub fn append(r: &Mmr, l: Hash, hash_fn: fn(&[u8]) -> Hash) -> Mmr {

    P(r, l, 0, hash_fn)
}

fn P(r: &Mmr, l: Hash, n: usize, hash_fn: fn(&[u8]) -> Hash) -> Mmr {

    let mut result: Mmr = Mmr{peaks: Vec::new()};

    if n >= r.peaks.len() {
        result.peaks.extend_from_slice(&r.peaks);
        result.peaks.push(Some(l));
    } else if n < r.peaks.len() && r.peaks[n].is_none() {
        result.peaks.extend_from_slice(&R(r, n, Some(l)).peaks);
    } else {
        let mut combined = Vec::new();
        if let Some(ref rn) = r.peaks[n] {
            combined.extend_from_slice(rn);
        }
        combined.extend_from_slice(&l);
        result.peaks.extend_from_slice(&P(&R(r, n, None), hash_fn(&combined), n + 1, hash_fn).peaks);
    }

    return result;
}

fn R(s: &Mmr, i: usize, v: MmrPeak) -> Mmr {

    let mut result = s.clone();
    result.peaks[i] = v;

    return result;
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_R_func() {
        let hash_1: Hash = [1u8; 32];
        let hash_2: Hash = [2u8; 32];
        let s = Mmr { peaks: vec![Some(hash_1), Some(hash_2)] };
        let n = 1;
        let l: Hash = [3u8; 32];
        let expected = Mmr { peaks: vec![Some(hash_1), Some(l)] };
        assert_eq!(expected.peaks, R(&s, n, Some(l)).peaks);
    }

    #[test]
    fn test_P_func() {
        let hash_1: Hash = [1u8; 32];
        let hash_2: Hash = [2u8; 32];
        let r = Mmr { peaks: vec![Some(hash_1), Some(hash_2), None] };
        let l: Hash = [3u8; 32];

        let result_1 = P(&r, l, 3, keccak_256);
        let expected_1 = Mmr { peaks: vec![Some(hash_1), Some(hash_2), None, Some(l)] };
        assert_eq!(expected_1.peaks, result_1.peaks);

        let result_2 = P(&r, l, 2, keccak_256);
        let expected_2 = Mmr { peaks: vec![Some(hash_1), Some(hash_2), Some(l)] };
        assert_eq!(expected_2.peaks, result_2.peaks);
    }
}