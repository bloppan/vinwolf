
use crate::{Encode, EncodeSize, EncodeLen};
use std::collections::HashMap;

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

impl Encode for i64 {
    fn encode(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
    fn encode_to(&self, writer: &mut Vec<u8>) {
        writer.extend_from_slice(&self.encode())
    }
}

impl Encode for i128 {
    fn encode(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
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

/*impl Encode for &T {
    fn encode(&self) -> Vec<u8> {
        self.to_vec()        
    }
    fn encode_to(&self, writer: &mut Vec<u8>) {
        writer.extend_from_slice(&self.encode())
    }
}*/

/*impl Encode for Vec<u8> {
    fn encode(&self) -> Vec<u8> {
        self.clone() 
    }
    fn encode_to(&self, writer: &mut Vec<u8>) {
        writer.extend_from_slice(&self.encode())
    }
}
*/
impl<T> Encode for Vec<T> 
where T: Encode
{
    fn encode(&self) -> Vec<u8> {
        let mut blob = Vec::with_capacity(self.len());
        for item in self.iter() {
            item.encode_to(&mut blob);
        }
        blob
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl<T> Encode for Option<T> 
where T: Encode,
{
    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::with_capacity(std::mem::size_of::<Self>());

        match self {
            None => {
                blob.push(0);
            }
            Some(data) => {
                blob.push(1);
                data.encode_to(&mut blob);
            }
        }

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

/*impl<T> Encode for Option<Vec<T>>
where T: Encode + Default,
{
    fn encode(&self) -> Vec<u8> {
        let mut blob = Vec::new();
        self.encode_to(&mut blob);
        return blob;
    }
    fn encode_to(&self, writer: &mut Vec<u8>) {
        writer.extend_from_slice(&self.encode());
    }
}*/

impl<const N: usize> Encode for [u8; N] {
    fn encode(&self) -> Vec<u8> {
        self.to_vec()
    }
    fn encode_to(&self, writer: &mut Vec<u8>) {
        writer.extend_from_slice(&self.encode())
    }
}

impl<const N: usize> Encode for Box<[u8; N]> {
    fn encode(&self) -> Vec<u8> {
        self.to_vec()
    }
    fn encode_to(&self, writer: &mut Vec<u8>) {
        writer.extend_from_slice(&self.encode())
    }
}
// TODO arreglar esto
impl<const N: usize> Encode for Box<[u32; N]> {

    fn encode(&self) -> Vec<u8> {

        let mut encoded = Vec::new();

        for i in 0..N {
            encoded.extend_from_slice(&self[i].encode());
        }

        return encoded;
    }
    fn encode_to(&self, writer: &mut Vec<u8>) {
        writer.extend_from_slice(&self.encode())
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

impl<const N: usize, const M: usize> Encode for Vec<([u8; N], [u8; M])> {

    fn encode(&self) -> Vec<u8> {
        
        let mut encoded = Vec::with_capacity(N * M);
        encode_unsigned(self.len()).encode_to(&mut encoded);

        for item in self.iter() {
            item.0.encode_to(&mut encoded);
            item.1.encode_to(&mut encoded);
        }

        return encoded;
    }

    fn encode_to(&self, writer: &mut Vec<u8>) {
        writer.extend_from_slice(&self.encode());    
    }
}

impl<T, U> Encode for HashMap<T, U> 
where T: Encode + Eq + std::hash::Hash,
      U: Encode
{
    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::new();

        encode_unsigned(self.len()).encode_to(&mut blob);

        for (key, value) in self.iter() {
            key.encode_to(&mut blob);
            value.encode_to(&mut blob);
        }

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl<T> EncodeLen for Vec<T> 
where T: Encode
{
    fn encode_len(&self) -> Vec<u8> {

        let mut blob = Vec::new();

        encode_unsigned(self.len()).encode_to(&mut blob);
        
        for item in self.iter() {
            item.encode_to(&mut blob);
        }

        return blob;
    }
}

impl EncodeSize for u8 {
    fn encode_size(&self, l: usize) -> Vec<u8> {
        encode_integer(*self as usize, l)
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

impl EncodeSize for i128 {
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
    
    if l == 0 {
        return vec![];
    }

    let mut result = Vec::with_capacity(l);
    let mut value = x;

    for _ in 0..l {
        result.push((value & 0xFF) as u8);
        value >>= 8;
    }

    result
}

/*pub fn encode_integer(x: usize, l: usize) -> Vec<u8> {

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
}*/

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


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_encode_integer() {
        assert_eq!(Vec::<u8>::new(), encode_integer(0, 0));       
        assert_eq!(vec![0x00], encode_integer(0, 1));             
        assert_eq!(vec![0xFF], encode_integer(255, 1));           
        assert_eq!(vec![0x00, 0x01], encode_integer(256, 2));     
        assert_eq!(vec![0x3C], encode_integer(1340, 1));          
        assert_eq!(vec![0x3C, 0x05], encode_integer(1340, 2));    
        assert_eq!(vec![0x34, 0x12], encode_integer(4660, 2));    
        assert_eq!(vec![0xFF, 0xFF], encode_integer(65535, 2));   
        assert_eq!(vec![0x00, 0x00, 0x01], encode_integer(65536, 3));
        assert_eq!(vec![0x78, 0x56, 0x34, 0x12], encode_integer(0x12345678, 4)); 
        assert_eq!(vec![0xEF, 0xCD, 0xAB, 0x89], encode_integer(0x89ABCDEF, 4)); 
        assert_eq!(vec![0xFF, 0xFF, 0xFF, 0xFF], encode_integer(0xFFFFFFFF, 4)); 
        assert_eq!(vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE], encode_integer(0xEE_DD_CC_BB_AA, 5)); 
        assert_eq!(vec![0x78, 0x56, 0x34, 0x12, 0x00, 0x00], encode_integer(0x12345678, 6)); 
        assert_eq!(vec![0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE], encode_integer(0xDEBC9A78563412, 7)); 
        assert_eq!(vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF], encode_integer(0xEFCDAB8967452301, 8));
    }

    #[test]
    fn test_encode_unsigned() {
        assert_eq!(vec![0x81, 0x0E], encode_unsigned(270));
    }
}