use std::collections::BTreeMap;

pub mod work_report;

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

pub trait Encode {
    fn encode(&self) -> Vec<u8>;
    fn encode_to(&self, writer: &mut Vec<u8>);
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


pub trait EncodeSize {
    fn encode_size(&self, l: usize) -> Vec<u8>;
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

pub trait EncodeLen {
    fn encode_len(&self) -> Vec<u8>;
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

pub fn seq_to_number(v: &Vec<u8>) -> u32 {

    let mut result = 0;
    let size = v.len();
    //println!("vec = {:?}, size = {size}", v);
    for i in 0..size {
        result |= (v[(size - i - 1) as usize] as u32) << (size - i - 1) * 8; 
    }
    result
}

pub fn decode_to_bits(v: &[u8]) -> Vec<bool> {

    let mut bools = Vec::new();
    for byte in v {
        for i in 0..8 { 
            let bit = (byte >> i) & 1; 
            bools.push(bit == 1); // Convert bit (0 o 1) to boolean
        }
    }

    return bools;
}

pub fn decode_from_le(data: &mut BytesReader, l: usize) -> Result<usize, ReadError> {
    let mut array = [0u8; std::mem::size_of::<usize>()];
    let len = std::cmp::min(l, std::mem::size_of::<usize>());
    let bytes = data.read_bytes(len)?;
    array[..len].copy_from_slice(bytes);  
    Ok(usize::from_le_bytes(array))
}

pub fn decode_unsigned(data: &mut BytesReader) -> Result<usize, ReadError> {

    let first_byte = data.read_byte()?;
    let l = first_byte.leading_ones() as usize;

    if l == 0 { 
        return Ok(first_byte as usize); 
    }

    let result = decode_from_le(data, l)?;

    if l == 8 {
        return Ok(result);
    }

    let mask = (1 << (7 - l)) - 1;

    return Ok(result | ((first_byte & mask) as usize) << (8 * l));
}
/*
pub fn decompact_len(v: &[u8]) -> Vec<usize> {

    if v.is_empty() {
        return vec![0];
    }

    let length = v[0] as usize;
    let mut result = Vec::with_capacity(length);
    let mut k = 1;

    for _ in 0..length {
        let l = v[k].leading_ones() as usize;
        let member = &v[k..=k + l]; 
        result.push(decode_unsigned(member));
        k += l + 1;
    }

    result
}*/


pub trait Decode: Sized {
    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError>;
}

impl Decode for u8 {
    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        let bytes = reader.read_bytes(1)?;
        Ok(bytes[0])
    }
}

impl Decode for u16 {
    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        let bytes = reader.read_bytes(2)?;
        let mut array = [0u8; 2];
        array.copy_from_slice(bytes);
        Ok(u16::from_le_bytes(array))
    }
}

impl Decode for u32 {
    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        let bytes = reader.read_bytes(4)?;
        let mut array = [0u8; 4];
        array.copy_from_slice(bytes);
        Ok(u32::from_le_bytes(array))
    }
}

impl Decode for i32 {
    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        let bytes = reader.read_bytes(4)?;
        let mut array = [0u8; 4];
        array.copy_from_slice(bytes);
        Ok(i32::from_le_bytes(array))
    }
}

impl Decode for u64 {
    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        let bytes = reader.read_bytes(8)?;
        let mut array = [0u8; 8];
        array.copy_from_slice(bytes);
        Ok(u64::from_le_bytes(array))
    }
}

impl Decode for usize {
    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        let mut array = [0u8; std::mem::size_of::<usize>()]; 
        let bytes = reader.read_bytes(std::mem::size_of::<usize>())?; 
        array.copy_from_slice(bytes); 
        Ok(usize::from_le_bytes(array)) 
    }
}

impl Decode for i64 {
    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        let bytes = reader.read_bytes(8)?;
        let mut array = [0u8; 8];
        array.copy_from_slice(bytes);
        Ok(i64::from_le_bytes(array))
    }
}

impl<const N: usize> Decode for [u8; N] {
    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        let bytes = reader.read_bytes(N)?;
        let mut array = [0u8; N];
        array.copy_from_slice(bytes);
        Ok(array)
    }
}

impl<const N: usize, const M: usize> Decode for [[u8; N]; M] {

    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {

        let mut array = [[0u8; N]; M];

        for sub_array in array.iter_mut() {
            let bytes = reader.read_bytes(N)?;
            sub_array.copy_from_slice(bytes);
        }

        Ok(array)
    }
}


pub trait DecodeLen: Sized {
    fn decode_len(reader: &mut BytesReader) -> Result<Self, ReadError>;
}

impl<T> DecodeLen for Vec<T>
where
    T: Decode + Default,
{
    fn decode_len(reader: &mut BytesReader) -> Result<Vec<T>, ReadError> {

        if reader.data.is_empty() {
            return Ok(vec![T::default()]);
        }

        let len = decode_unsigned(reader)?;  
        let mut result = Vec::with_capacity(len);

        for _ in 0..len {
            result.push(T::decode(reader)?);
        }

        Ok(result)
    }
}

pub struct BytesReader<'a> {
    position: usize,
    data: &'a [u8],
}

impl<'a> BytesReader<'a> {

    pub fn new(data: &'a [u8]) -> Self {
        BytesReader { data, position: 0 }
    }

    pub fn read_bytes(&mut self, length: usize) -> Result<&[u8], ReadError> {

        if self.position + length > self.data.len() {
            return Err(ReadError::NotEnoughData);
        }

        let bytes = &self.data[self.position..self.position + length];
        self.position += length;

        Ok(bytes)
    }

    pub fn read_byte(&mut self) -> Result<u8, ReadError> {

        if self.position + 1 > self.data.len() {
            return Err(ReadError::NotEnoughData);
        }

        let byte = self.data[self.position] as u8;
        self.position += 1;
        
        Ok(byte)
    }

    pub fn get_position(&self) -> usize {
        self.position
    }
}

// Definición de `ReadError`
#[derive(Debug)]
pub enum ReadError {
    NotEnoughData,
    InvalidData,
    ConversionError,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    macro_rules! reader {
        ($($byte:expr),*) => {
            {
                let data: &[u8] = &[$($byte),*];
                BytesReader::new(data)
            }
        };
    }
    
    #[test]
    fn test_encode_size() {
        assert_eq!(vec![0xDF, 0x63], 25567u32.encode_size(2));
        assert_eq!(vec![252, 255, 255, 255], (-4_i32).encode_size(4));
        assert_eq!(vec![0, 255, 255, 255], (-256_i32).encode_size(4));
        assert_eq!(vec![255, 254], (-257_i32).encode_size(2));
        assert_eq!(vec![0x1F, 0xB2, 0x01], 111135u32.encode_size(3));
        assert_eq!(Vec::<u8>::new(), 56323u32.encode_size(0));
        assert_eq!(vec![0x21, 0xFF, 0xFF, 0xFF], 0xFFFFFF21u32.encode_size(4));
        assert_eq!(vec![0xDF, 0x63, 0x00, 0x00], 25567u64.encode_size(4));
        assert_eq!(vec![0x21, 0xFF, 0xFF, 0xFF], 0xFFFFFF21u64.encode_size(4));
        assert_eq!(vec![0x21, 0, 0, 0, 0, 0, 0, 0, 0, 0], 0x21u64.encode_size(10));
        assert_eq!(vec![0x21, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF], 0xFFFFFFFFFFFFFF21u64.encode_size(10));
        assert_eq!(vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF], 0xFFFFFFFFFFFFFFFFu64.encode_size(8));
        assert_eq!(vec![0xEF, 0x0C, 0x28, 0x31, 0x0A, 0x0D, 0xC9, 0x10], 1209512312351231215u64.encode_size(8));
        assert_eq!(vec![0x3C, 0xE2, 0x01, 0, 0, 0, 0, 0, 0], 123452u64.encode_size(9));
        assert_eq!(vec![0x00], 256u64.encode_size(1));
    }

    #[test]
    fn test_encode_len() {
        assert_eq!(vec![0], (&[] as &[u8]).encode_len());
        assert_eq!(vec![1, 0], (&[0u8][..]).encode_len());
        assert_eq!(vec![1, 0x2C, 0x1], (&[300u16][..]).encode_len());
        assert_eq!(vec![4, 0x2C, 0x1, 18, 0, 127, 0, 128, 0], (&[300u16, 18u16, 127u16, 128u16][..]).encode_len());
        assert_eq!(vec![2, 0xF0, 0x49, 0x02, 0, 0xFF, 0xFF, 0x1F, 0], (&[150000u32, 2097151u32][..]).encode_len());
        assert_eq!(vec![2, 0x1, 0, 0, 0, 0, 0, 0, 0x80, 0, 0, 0, 0, 8, 0, 0, 0], (&[9223372036854775809u64, 0x800000000u64][..]).encode_len());
        assert_eq!(vec![2, 1, 2, 3, 4], (&[vec![1,2], vec![3,4]][..]).encode_len());
    }

    #[test]
    fn test_encode_from_bits() {
        assert_eq!(Vec::<u8>::new(), encode_from_bits(&[]));
        assert_eq!(vec![0], encode_from_bits(&[false]));
        assert_eq!(vec![1], encode_from_bits(&[true]));
        assert_eq!(vec![0x55], encode_from_bits(&[true, false, true, false, true, false, true, false]));
        assert_eq!(vec![0x55, 1], encode_from_bits(&[true, false, true, false, true, false, true, false, true]));
    }

    /*#[test]
    fn test_decode_from_le() {
        assert_eq!(0, decode_from_le(&mut reader![], 1).unwrap());
        assert_eq!(0, decode_from_le(&mut reader![0], 1).unwrap());
        assert_eq!(27, decode_from_le(&mut reader![27], 1).unwrap());
        assert_eq!(150000, decode_from_le(&mut reader![0xF0, 0x49, 0x02], 3).unwrap());
        assert_eq!(9223372036854775809, decode_from_le(&mut reader![1, 0, 0, 0, 0, 0, 0, 0x80], 8).unwrap());
    }*/

    #[test]
    fn test_decode_len() {
        assert_eq!(vec![0], Vec::<u8>::decode_len(&mut reader![]).unwrap());
        assert_eq!(vec![0], Vec::<u8>::decode_len(&mut reader![1, 0]).unwrap());
        assert_eq!(vec![1], Vec::<u8>::decode_len(&mut reader![1, 1]).unwrap());
        assert_eq!(vec![1, 2], Vec::<u8>::decode_len(&mut reader![2, 1, 2]).unwrap());
        assert_eq!(vec![300, 300], Vec::<u16>::decode_len(&mut reader![2, 0x2C, 0x01, 0x2C, 0x01]).unwrap());
        assert_eq!(vec![300, 127], Vec::<u16>::decode_len(&mut reader![2, 0x2C, 0x01, 0x7F, 0x00]).unwrap());
        assert_eq!(vec![300, 150000, 1], Vec::<u32>::decode_len(&mut reader![3, 0x2C, 0x01, 0x00, 0x00, 0xF0, 0x49, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00]).unwrap());
        assert_eq!(vec![9223372036854775809], Vec::<u64>::decode_len(&mut reader![1, 1, 0, 0, 0, 0, 0, 0, 0x80]).unwrap());
        assert_eq!(vec![vec![1, 2], vec![3, 4]], Vec::<[u8; 2]>::decode_len(&mut reader![2, 1, 2, 3, 4]).unwrap());
        assert_eq!(vec![-1, -255], Vec::<i32>::decode_len(&mut reader![2, 0xFF, 0xFF, 0xFF, 0xFF, 0x01, 0xFF, 0xFF, 0xFF]).unwrap());
    }

    #[test]
    fn test_decode_to_bits() {
        assert_eq!(Vec::<bool>::new(), decode_to_bits(&[]));
        assert_eq!(vec![false, false, false, false, false, false, false, false], decode_to_bits(&[0]));
        assert_eq!(vec![true, false, false, false, false, false, false, false], decode_to_bits(&[1]));
        assert_eq!(vec![true, true, false, true, true, false, false, false,
                        true, true, true, true, true, true, true, true], decode_to_bits(&[27, 255]));
        assert_eq!(vec![true, false, true, false, true, false, true, false,
                        false, false, false, false, false, false, false, false,
                        false, false, false, false, true, true, true, true], decode_to_bits(&[0x55, 0, 0xF0]));
    }
    
    #[test]
    fn test_encode_unsigned() {
        assert_eq!(vec![0], encode_unsigned(0));
        assert_eq!(vec![100], encode_unsigned(100));
        assert_eq!(vec![127], encode_unsigned(127));
        assert_eq!(vec![128, 128], encode_unsigned(128));
        assert_eq!(vec![128, 194], encode_unsigned(194));
        assert_eq!(vec![128, 255], encode_unsigned(255));
        assert_eq!(vec![129, 0], encode_unsigned(256));
        assert_eq!(vec![129, 44], encode_unsigned(300));
        assert_eq!(vec![131, 255], encode_unsigned(1023));
        assert_eq!(vec![191, 255], encode_unsigned(0x3FFF));
        assert_eq!(vec![192, 0, 64], encode_unsigned(0x4000));
        assert_eq!(vec![191, 255], encode_unsigned(16383));
        assert_eq!(vec![192, 255, 255], encode_unsigned(65535));
        assert_eq!(vec![193, 0, 0], encode_unsigned(65536));
        assert_eq!(vec![222, 0x60, 0x79], encode_unsigned(1997152));
        assert_eq!(vec![194, 240, 73], encode_unsigned(150000));
        assert_eq!(vec![224, 10, 0, 32], encode_unsigned(2097162));
        assert_eq!(vec![239, 255, 255, 255], encode_unsigned(0xFFFFFFF));
        assert_eq!(vec![240, 0, 0, 0, 16], encode_unsigned(0x10000000));
        assert_eq!(vec![247, 255, 255, 255, 255], encode_unsigned(0x7FFFFFFFF));
        assert_eq!(vec![248, 0, 0, 0, 0, 8], encode_unsigned(0x800000000));
        assert_eq!(vec![251, 255, 255, 255, 255, 255], encode_unsigned(0x3FFFFFFFFFF));
        assert_eq!(vec![252, 0, 0, 0, 0, 0, 4], encode_unsigned(0x40000000000));
        assert_eq!(vec![253, 255, 255, 255, 255, 255, 255], encode_unsigned(0x1FFFFFFFFFFFF));
        assert_eq!(vec![254, 0, 0, 0, 0, 0, 0, 2], encode_unsigned(0x2000000000000));
        assert_eq!(vec![255, 254, 255, 255, 255, 255, 255, 255, 255], encode_unsigned(0xFFFFFFFFFFFFFFFE)); 
        assert_eq!(vec![255, 255, 255, 255, 255, 255, 255, 255, 127], encode_unsigned(9223372036854775807));
        assert_eq!(vec![255, 0, 0, 0, 0, 0, 0, 0, 128], encode_unsigned(9223372036854775808));
        assert_eq!(vec![255, 1, 0, 0, 0, 0, 0, 0, 128], encode_unsigned(9223372036854775809));
    }

    #[test]
    fn test_decode_unsigned() {
        assert_eq!(0, decode_unsigned(&mut reader![0]).unwrap());
        assert_eq!(100, decode_unsigned(&mut reader![100]).unwrap());
        assert_eq!(300, decode_unsigned(&mut reader![129, 44]).unwrap());
        assert_eq!(150000, decode_unsigned(&mut reader![194, 240, 73]).unwrap());
        assert_eq!(2097151, decode_unsigned(&mut reader![223, 255, 255]).unwrap());
        assert_eq!(0x800000000, decode_unsigned(&mut reader![248, 0, 0, 0, 0, 8]).unwrap());
        assert_eq!(0x2000000000000, decode_unsigned(&mut reader![254, 0, 0, 0, 0, 0, 0, 2]).unwrap());
        assert_eq!(9223372036854775807, decode_unsigned(&mut reader![255, 255, 255, 255, 255, 255, 255, 255, 127]).unwrap());
        assert_eq!(9223372036854775808, decode_unsigned(&mut reader![255, 0, 0, 0, 0, 0, 0, 0, 128]).unwrap());
        assert_eq!(9223372036854775809, decode_unsigned(&mut reader![255, 1, 0, 0, 0, 0, 0, 0, 128]).unwrap());
    }

    /*#[test]
    fn test_compact_len() {
        assert_eq!(vec![0], (&[] as &[u8]).compact_len());
        assert_eq!(vec![1, 0], (&[0u8][..]).compact_len());
        assert_eq!(vec![1, 129, 44], (&[300u16][..]).compact_len());
        assert_eq!(vec![4, 129, 44, 18, 127, 128, 128], (&[300u16, 18u16, 127u16, 128u16][..]).compact_len());
        assert_eq!(vec![2, 194, 240, 73, 223, 255, 255], (&[150000u32, 2097151u32][..]).compact_len());
        assert_eq!(vec![2, 255, 1, 0, 0, 0, 0, 0, 0, 128, 248, 0, 0, 0, 0, 8][..], (&[9223372036854775809u64, 0x800000000u64][..]).compact_len());
    }*/
}

/**
*****************************************************************************************************************************************
**/
/* Eq 272 */
pub fn trivial_serialize(x: u64, l: u8) -> Vec<u8> {

    let mut octet;
    let mut serialized: Vec<u8> = Vec::new();

    if l == 0 || l > 8 { return serialized } 

    for i in 0..l {
        octet = ((x >> (8 * i)) & 0xFF) as u8;
        serialized.push(octet);
    }
    serialized
}

pub fn serialize_string(s: String) -> Vec<u8> {

    s.into_bytes()
}

/*pub fn sequence_encoder(v: &Vec<u64>) -> Vec<u8> {

    let mut u: Vec<u8> = Vec::new();
    for i in 0..v.len() {
        u.extend(serialize_number(v[i]));
    }
    return u;
}*/

pub fn serialize_dict(dict: &BTreeMap<String, u8>) -> Vec<u8> {
    let mut serialized_pairs = Vec::new();

    // Serializar cada par clave/valor
    for (key, value) in dict.iter() {
        let mut serialized_pair = Vec::new();
        // Serializar la clave
        serialized_pair.extend(key.bytes());
        // Serializar el valor
        serialized_pair.push(*value);
        serialized_pairs.push(serialized_pair);
    }
    // Crear la secuencia final con el discriminador de longitud
    let mut result = vec![serialized_pairs.len() as u8];
    for pair in serialized_pairs {
        result.extend(pair);
    }

    result
}

pub fn serialize_dict_with_domain(dict: &BTreeMap<String, u8>, domain: Vec<&str>) -> Vec<u8> {
    let mut result = Vec::new();

    for key in domain {
        if let Some(&value) = dict.get(key) {
            // Clave presente en el diccionario
            result.push(0x01); // Indicador de presencia
            result.push(value); // Valor codificado
        } else {
            // Clave no presente en el diccionario
            result.push(0x00); // Indicador de ausencia
        }
    }

    result
}

/* Eq 291 */
pub fn serial_state_index(i: u8) -> [u8; 32] {

    let mut state = [0u8; 32];
    state[0] = i;
    state
}
/* Eq 291 */
pub fn serial_state_serv(i: u8, s: u32) -> [u8; 32] {

    let mut state = [0u8; 32];
    state[0] = i;
    
    let v: Vec<u8> = trivial_serialize(s as u64, 4);
    for n in 0..4 {
        state[1 + n] = v[n];
    }
    state
}
/* Eq 291 */
pub fn serial_s_and_h(s: u32, h: &str) -> [u8; 32] {

    let s_bytes = s.to_le_bytes(); 
    let h_bytes: Vec<u8> = h.bytes().collect(); 
    let mut state = [0u8; 32];

    if h.len() > 28 { return state }

    for i in 0..4 {
        state[2 * i] = s_bytes[i];
        state[2 * i + 1] = h_bytes[i];
    }

    for i in 4..h.len() {
        state[i + 4] = h_bytes[i];
    }

    state
}

pub fn test_serial_s_and_h() {
    let s = 1000;
    let h = "ABCDEFGHaaaaaadddddddaaab";
    let serialized = serial_s_and_h(s, h);
    println!("Serialized: {:?}", serialized);
}

pub fn test_serial_state_serv() {

    println!("Serial state integer of i=10, s=1000: {:?}", serial_state_serv(10, 1000));
}

pub fn test_serial_state_index() {
    let octet = 27;
    println!("Number = {}", octet);
    println!("Serial state octet {:?}", serial_state_index(octet));
}

pub fn test_dict_domain_codec() {

    // Crear el diccionario de ejemplo
    let mut dict = BTreeMap::new();
    dict.insert("A".to_string(), 10); // A -> 0x0A
    dict.insert("B".to_string(), 20); // B -> 0x14

    // Definir el dominio
    let domain = vec!["A", "B", "C"];

    // Serializar el diccionario con el dominio
    let serialized = serialize_dict_with_domain(&dict, domain);

    // Mostrar el resultado
    println!("Diccionario serializado: {:?}", serialized);
}


pub fn test_dict_codec() {

    // Crear el diccionario de ejemplo
    let mut dict = BTreeMap::new();
    dict.insert("C".to_string(), 3);
    dict.insert("A".to_string(), 1);
    dict.insert("B".to_string(), 2);

    // Serializar el diccionario
    let serialized = serialize_dict(&dict);

    // Mostrar el resultado
    println!("Diccionario serializado: {:?}", serialized);

}
/*
pub fn test_bits_codec() {

    let bits = vec![17, 6];
    let serialized = serialize_bits(bits);
    
    println!("Secuencia serializada: {:?}", serialized);

}*/
/*
pub fn test_sequencer_codec() {

    let sequence = vec![150000, 18446744073709551610u64, 15446744073709551610u64];
    let res_sequence: Vec<u8>;
    res_sequence = sequence_encoder(&sequence);
    println!("Numeros {:?}", sequence);
    println!("Serializacion: {:?}", res_sequence);

}*/
