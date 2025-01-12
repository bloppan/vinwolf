
use std::collections::BTreeMap;
use crate::utils::codec::{Encode, Decode, EncodeSize, EncodeLen, DecodeLen, ReadError};

pub fn encode_unsigned(x: usize) -> Vec<u8> {

    if x == 0 {
        return vec![0];
    }

    let bit_length = 64 - x.leading_zeros();
    let mut l = ((bit_length - 1) / 7) as u32;

    if l > 8 {
        l = 8;
    }

    if l == 0 && x < 128 {
        return vec![x as u8];
    }

    let prefix = if l < 8 {
        (256u64 - (1u64 << (8 - l)) + ((x >> (8 * l)) as u64)) as u8
    } else {
        255u8
    };

    let mut result = vec![prefix];
    result.extend_from_slice(&x.encode_size(l as usize));
    result
}

impl Encode for u64 {
    fn encode(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
    fn encode_to(&self, writer: &mut Vec<u8>) {
        writer.extend_from_slice(&self.encode())
    }
}

impl Encode for bool {
    fn encode(&self) -> Vec<u8> {
        vec![if *self { 1u8 } else { 0u8 }]
    }
    fn encode_to(&self, writer: &mut Vec<u8>) {
        writer.push(self.encode()[0])
    }
}

impl Encode for u8 {
    fn encode(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
    fn encode_to(&self, writer: &mut Vec<u8>) {
        writer.push(self.encode()[0])
    }
}

impl Encode for u16 {
    fn encode(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
    fn encode_to(&self, writer: &mut Vec<u8>) {
        writer.extend_from_slice(&self.encode())
    }
}

impl Encode for u32 {
    fn encode(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
    fn encode_to(&self, writer: &mut Vec<u8>) {
        writer.extend_from_slice(&self.encode())
    }
}

impl Encode for i32 {
    fn encode(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
    fn encode_to(&self, writer: &mut Vec<u8>) {
        writer.extend_from_slice(&self.encode())
    }
}

impl Encode for usize {
    fn encode(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
    fn encode_to(&self, writer: &mut Vec<u8>) {
        writer.extend_from_slice(&self.encode())
    }
}

impl Encode for [u8] {
    fn encode(&self) -> Vec<u8> {
        self.to_vec()        
    }
    fn encode_to(&self, writer: &mut Vec<u8>) {
        writer.extend_from_slice(&self.encode())
    }
}

impl Encode for &[u8] {
    fn encode(&self) -> Vec<u8> {
        self.to_vec()        
    }
    fn encode_to(&self, writer: &mut Vec<u8>) {
        writer.extend_from_slice(&self.encode())
    }
}

impl Encode for Vec<u8> {
    fn encode(&self) -> Vec<u8> {
        self.clone() 
    }
    fn encode_to(&self, writer: &mut Vec<u8>) {
        writer.extend_from_slice(&self.encode())
    }
}

impl<const N: usize> Encode for [u8; N] {
    fn encode(&self) -> Vec<u8> {
        self.to_vec()
    }
    fn encode_to(&self, writer: &mut Vec<u8>) {
        writer.extend_from_slice(&self.encode())
    }
}

impl<const N: usize> Encode for Vec<[u8; N]> {
    fn encode(&self) -> Vec<u8> {
        let mut encoded = Vec::new();
        for array in self {
            encoded.extend_from_slice(&array.encode());
        }
        encoded
    }
    
    fn encode_to(&self, writer: &mut Vec<u8>) {
        for array in self {
            writer.extend_from_slice(&array.encode());
        }
    }
}

impl<const N: usize, const M: usize> Encode for [[u8; N]; M] {

    fn encode(&self) -> Vec<u8> {

        let mut encoded = Vec::with_capacity(N * M);

        for array in self {
            encoded.extend_from_slice(&array.encode());
        }

        return encoded;
    }
    
    fn encode_to(&self, writer: &mut Vec<u8>) {

        for array in self {
            writer.extend_from_slice(&array.encode());
        }
    }
}


impl<const N: usize, const M: usize> Encode for Vec<[[u8; N]; M]> {

    fn encode(&self) -> Vec<u8> {

        let mut encoded = Vec::with_capacity(N * M);

        for array in self {
            for inner_array in array {
                encoded.extend_from_slice(&inner_array.encode());
            }
        }

        return encoded;
    }
    
    fn encode_to(&self, writer: &mut Vec<u8>) {

        for array in self {
            for inner_array in array {
                writer.extend_from_slice(&inner_array.encode());
            }
        }
    }
}

impl EncodeSize for u16 {
    fn encode_size(&self, l: usize) -> Vec<u8> {
        encode_integer(*self as usize, l)
    }
}

impl EncodeSize for u32 {
    fn encode_size(&self, l: usize) -> Vec<u8> {
        encode_integer(*self as usize, l)
    }
}

impl EncodeSize for u64 {
    fn encode_size(&self, l: usize) -> Vec<u8> {
        encode_integer(*self as usize, l)
    }
}

impl EncodeSize for i32 {
    fn encode_size(&self, l: usize) -> Vec<u8> {
        encode_integer(*self as usize, l)
    }
}

impl EncodeSize for i64 {
    fn encode_size(&self, l: usize) -> Vec<u8> {
        encode_integer(*self as usize, l)
    }
}

impl EncodeSize for usize {
    fn encode_size(&self, l: usize) -> Vec<u8> {
        encode_integer(*self as usize, l)
    }
}

impl<const N: usize> EncodeSize for [u8; N] {

    fn encode_size(&self, l: usize) -> Vec<u8> {

        let mut res: Vec<u8> = Vec::with_capacity(l);
        
        if l > N {
            return res; 
        }

        res.extend_from_slice(&self[..l]); 
        res
    }
}

pub fn encode_integer(x: usize, l: usize) -> Vec<u8> {

    let mut sequence: Vec<u8> = Vec::new();
    if l == 0 { return sequence } 

    let max_octets = std::mem::size_of::<usize>();
    for i in 0..std::cmp::min(l, max_octets) {
        sequence.push(((x >> (8 * i)) & 0xFF) as u8);
    }

    if l > max_octets {
        let sign_bit = (x / (1 << (8 * max_octets - 1))) as usize;
        for _j in max_octets..l { // Extend sign bit
            sequence.push((sign_bit * 255) as u8);
        }
    }
    
    return sequence;
}

impl<T: Encode> EncodeLen for &[T] {

    fn encode_len(&self) -> Vec<u8> {
        
        if self.is_empty() {
            return vec![0];
        }

        let mut encoded = Vec::with_capacity(self.len());
        encode_unsigned(self.len()).encode_to(&mut encoded);

        for item in *self {
            item.encode_to(&mut encoded);
        }

        return encoded;
    }
}

pub fn encode_from_bits(v: &[bool]) -> Vec<u8> {
    v.chunks(8)
        .map(|chunk| {
            chunk.iter()
                .enumerate()
                .fold(0u8, |acc, (i, &bit)| acc | ((bit as u8) << i))
        })
        .collect()
}
