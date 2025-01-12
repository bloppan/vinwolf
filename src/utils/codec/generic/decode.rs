
use std::collections::BTreeMap;
use crate::utils::codec::{Encode, Decode, EncodeSize, EncodeLen, DecodeLen, ReadError, BytesReader};

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