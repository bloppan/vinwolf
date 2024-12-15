use sp_core::blake2_256;
use std::convert::TryInto;

use crate::types::Hash;
use crate::utils::codec::EncodeSize;

// The Fisher-Yates shuffle function is defined formally as:
fn fisher_yattes_shuffle<T: Clone>(s: &[T], r: &[u32]) -> Vec<T> {

    let len = s.len();

    if len == 0 {
        return Vec::new();
    }

    let index = r[0] as usize % len;
    let mut result = Vec::new();

    result.extend_from_slice(&s[index..index + 1]);
    
    let mut new_s = Vec::from(s);
    new_s.swap_remove(index); // Remove index element and swap it for the last one

    result.extend_from_slice(&fisher_yattes_shuffle(&new_s, &r[1..]));

    return result;
}

// Since it is often useful to shuffle a sequence based on some random seed in the form of a hash, we provide a secondary
// form of the shuffle function F which accepts a 32-byte hash instead of the numeric sequence
pub fn shuffle<T: Clone>(s: &[T], hash: &Hash) -> Vec<T> {

    fisher_yattes_shuffle(s, &sequencer(&hash, s.len()))    
}

// We define the numeric-sequence-from-hash function, thus:
fn sequencer(entropy: &Hash, len: usize) -> Vec<u32> {

    let mut sequence: Vec<u32> = Vec::new();

    for i in 0..len {

        let encoded_4 = (i / 8).encode_size(4);

        let mut payload = Vec::from(entropy);
        payload.extend_from_slice(&encoded_4);

        let hash = blake2_256(&payload);

        let start = (4 * i) % 32;
        let item = u32::from_le_bytes(hash[start..start + 4].try_into().unwrap());

        sequence.push(item);
    }

    return sequence;
}
