use crate::{FromLeBytes, Decode, DecodeLen, DecodeSize, ReadError, BytesReader};
use std::collections::HashMap;

pub fn decode_unsigned(data: &mut BytesReader) -> Result<usize, ReadError> {

    let first_byte = data.read_byte()?;
    let l = first_byte.leading_ones() as usize;

    if l == 0 { 
        return Ok(first_byte as usize); 
    }

    let result = decode_integer(data, l)?;

    if l == 8 {
        return Ok(result);
    }

    let mask = (1 << (7 - l)) - 1;

    return Ok(result | ((first_byte & mask) as usize) << (8 * l));
}

pub fn decode_integer(data: &mut BytesReader, l: usize) -> Result<usize, ReadError> {
    let mut array = [0u8; std::mem::size_of::<usize>()];
    let len = std::cmp::min(l, std::mem::size_of::<usize>());
    let bytes = data.read_bytes(len)?;
    array[..len].copy_from_slice(bytes);  
    Ok(usize::from_le_bytes(array))
}

// TODO revisar esta funcion
pub fn decode<T: FromLeBytes>(bytes: &[u8], n: usize) -> T {
    let size = std::mem::size_of::<T>();
    let mut buffer = [0u8; 16]; // up to 128 bits

    let len = n.min(bytes.len()).min(size);
    buffer[..len].copy_from_slice(&bytes[..len]);

    T::from_le_bytes(&buffer[..size])
}

pub fn decode_to_bits(bytes: &mut BytesReader, n: usize) -> Result<Vec<bool>, ReadError> {
    let mut bools = Vec::with_capacity(n * 8); 

    for _ in 0..n {
        let byte = bytes.read_byte()?;
        for i in 0..8 {
            bools.push(byte & (1 << i) != 0); 
        }
    }

    Ok(bools)
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



/*impl<const N: usize, const M: usize> Decode for [[u8; N]; M] {

    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {

        let mut array = [[0u8; N]; M];

        for sub_array in array.iter_mut() {
            let bytes = reader.read_bytes(N)?;
            sub_array.copy_from_slice(bytes);
        }

        Ok(array)
    }
}*/


impl Decode for Vec<u32> {
    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        let len = decode_unsigned(reader)?;
        let mut result = Vec::with_capacity(len);
        for _ in 0..len {
            result.push(u32::decode(reader)?);
        }
        Ok(result)
    }
}

/*impl<const N: usize> Decode for [u8; N] {
    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        let bytes = reader.read_bytes(N)?;
        let mut array = [0u8; N];
        array.copy_from_slice(bytes);
        Ok(array)
    }
}*/

impl<T, const N: usize> Decode for [T; N]
where
    T: Decode + Default + Copy,
{
    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        let mut array: [T; N] = [T::default(); N];
        for i in 0..N {
            array[i] = T::decode(reader)?;
        }
        Ok(array)
    }
}

// TODO: arreglar esto
impl<const N: usize> Decode for Box<[u32; N]>
{
    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {

        let mut array: Box<[u32; N]> = Box::new(std::array::from_fn(|_| u32::default()));

        for i in 0..N {
            array[i] = u32::decode(reader)?;
        }

        Ok(array)
    }
}


use std::convert::TryInto;

impl<T, const N: usize, const M: usize> Decode for Box<[Box<[T; N]>; M]>
where
    T: Decode + Default + Copy,
{
    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        let mut items = Vec::with_capacity(M);
        for _ in 0..M {
            let mut inner = [T::default(); N];
            for i in 0..N {
                inner[i] = T::decode(reader)?;
            }
            items.push(Box::new(inner));
        }

        let array: [Box<[T; N]>; M] = items.try_into().map_err(|_| ReadError::NotEnoughData)?;
        Ok(Box::new(array))
    }
}



impl<const N: usize, const M: usize> Decode for Vec<([u8; N], [u8; M])> {

    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        
        let mut result = Vec::with_capacity(N * M);
        let len = decode_unsigned(reader)?;
        
        for _ in 0..len {
            result.push((<[u8; N]>::decode(reader)?, <[u8; M]>::decode(reader)?));
        }

        Ok( result )
    }
}

impl<T> Decode for Option<T> 
where T: Decode
{
    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        let option = blob.read_byte()?;
        
        match option {
            0 => Ok(None),
            1 => {
                let data = T::decode(blob)?;
                Ok(Some(data))
            }
            _ => Err(ReadError::InvalidData),
        }
    }
}

impl<T, U> Decode for HashMap<T, U> 
where T: Decode + Eq + std::hash::Hash, 
      U: Decode
{
    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {

        let len = decode_unsigned(reader)?;
        
        let mut result = HashMap::with_capacity(len);
        
        for _ in 0..len {
            let key = T::decode(reader)?;
            let value = U::decode(reader)?;
            result.insert(key, value);
        }
        
        Ok(result)
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
        let mut result = vec![];

        for _ in 0..len {
            result.push(T::decode(reader)?);
        }

        Ok(result)
    }
}

impl DecodeSize for Vec<u8> {
    fn decode_size(reader: &mut BytesReader, l: usize) -> Result<usize, ReadError> {
        let bytes = reader.read_bytes(l)?;
        Ok(decode_integer(&mut BytesReader::new(bytes), l)?)
    }
}

impl<const N: usize> DecodeSize for [u8; N] {
    fn decode_size(reader: &mut BytesReader, l: usize) -> Result<usize, ReadError> {
        let bytes = reader.read_bytes(l)?;
        Ok(decode_integer(&mut BytesReader::new(bytes), l)?)
    }
}

#[cfg(test)]
mod test { 

    use super::*;

    #[test]
    fn decode_unsigned_test() {

        let array: [u8; 2] = [0x87, 0xE0];
        let mut reader = BytesReader::new(&array);
        let value = decode_unsigned(&mut reader).unwrap();
        println!("value: {value}");

        
    }
    #[test]
    fn test_decode_size_array() {

        let array: [u8; 32] = (1..33).map(|x| x as u8).collect::<Vec<u8>>().try_into().unwrap();
        let mut reader = BytesReader::new(&array);
        assert_eq!(0x04030201, <[u8; 32]>::decode_size(&mut reader, 4).unwrap());
    }

    #[test]
    fn test_decode_integer() {
        let test_cases = vec![
            (vec![0x01], 1, 1usize),
            (vec![0xFF], 1, 0xFFusize),
            (vec![0x40], 1, 0x40usize),
            (vec![0x80], 1, 0x80usize),
            (vec![0x01, 0x00], 2, 1usize),
            (vec![0x80, 0xFF], 2, 0xFF80usize),
            (vec![0xFF, 0xFF], 2, 0xFFFFusize),
            (vec![0x00, 0x80], 2, 0x8000usize),
            (vec![0x01, 0x00, 0x00], 3, 1usize),
            (vec![0xFF, 0xFF, 0xFF], 3, 0xFFFFFFusize),
            (vec![0x00, 0x00, 0x80], 3, 0x800000usize),
            (vec![0xD4, 0xFE, 0xFF], 3, 0xFFFED4usize),
            (vec![0x01, 0x00, 0x00, 0x00], 4, 1usize),
            (vec![0xFF, 0xFF, 0xFF, 0xFF], 4, 0xFFFFFFFFusize),
            (vec![0x00, 0x00, 0x00, 0x80], 4, 0x80000000usize),
            (vec![0x01, 0x00, 0x00, 0x00, 0x00], 5, 1usize),
            (vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF], 5, 0xFFFFFFFFFFusize),
            (vec![0x00, 0x00, 0x00, 0x00, 0x80], 5, 0x8000000000usize),
            (vec![0x01, 0x00, 0x00, 0x00, 0x00, 0x00], 6, 1usize),
            (vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF], 6, 0xFFFFFFFFFFFFusize),
            (vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x80], 6, 0x800000000000usize),
            (vec![0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], 7, 1usize),
            (vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF], 7, 0xFFFFFFFFFFFFFFusize),
            (vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80], 7, 0x80000000000000usize),
            (vec![0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], 8, 1usize),
            (vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF], 8, 0xFFFFFFFFFFFFFFFFusize),
            (vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80], 8, 0x8000000000000000usize),
        
        ];
        for (input, size, expected) in test_cases {
            let result = decode_integer(&mut BytesReader::new(&input), size).unwrap();
            assert_eq!(expected, result);
        }
    }


    #[test]
    fn test_decode_size() {
        let test_cases = vec![
            (vec![0x01], 1, 1usize),
            (vec![0xFF], 1, 0xFFusize),
            (vec![0x40], 1, 0x40usize),
            (vec![0x80], 1, 0x80usize),
            (vec![0x01, 0x00], 2, 1usize),
            (vec![0x80, 0xFF], 2, 0xFF80usize),
            (vec![0xFF, 0xFF], 2, 0xFFFFusize),
            (vec![0x00, 0x80], 2, 0x8000usize),
            (vec![0x01, 0x00, 0x00], 3, 1usize),
            (vec![0xFF, 0xFF, 0xFF], 3, 0xFFFFFFusize),
            (vec![0x00, 0x00, 0x80], 3, 0x800000usize),
            (vec![0xD4, 0xFE, 0xFF], 3, 0xFFFED4usize),
            (vec![0x01, 0x00, 0x00, 0x00], 4, 1usize),
            (vec![0xFF, 0xFF, 0xFF, 0xFF], 4, 0xFFFFFFFFusize),
            (vec![0x00, 0x00, 0x00, 0x80], 4, 0x80000000usize),
        ];
        for (input, size, expected) in test_cases {
            let result = Vec::<u8>::decode_size(&mut super::BytesReader::new(&input), size).unwrap();
            assert_eq!(expected, result);
        }
    }

    #[test]
    fn decode_test() {
        let bytes = [0x01, 0x02, 0x03, 0x04];
        let value: u32 = decode::<u32>(&bytes, 4);
        assert_eq!(value, 0x04030201);

        let byte = [0x01];
        let value: u8 = decode::<u8>(&byte, 1);
        assert_eq!(value, 0x01);

    }

}