use std::collections::BTreeMap;

pub trait Encode {
    fn encode(&self) -> Vec<u8>;
}

impl Encode for usize {
    fn encode(&self) -> Vec<u8> {
        let x = *self;
    
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
        result.extend_from_slice(&encode_trivial(x, l as usize));
        result
    }
}

impl Encode for u8 {
    fn encode(&self) -> Vec<u8> {
        (*self as usize).encode()
    }
}

impl Encode for u16 {
    fn encode(&self) -> Vec<u8> {
        (*self as usize).encode()
    }
}

impl Encode for u32 {
    fn encode(&self) -> Vec<u8> {
        (*self as usize).encode()
    }
}

impl Encode for i32 {
    fn encode(&self) -> Vec<u8> {
        (*self as usize).encode()
    }
}

impl Encode for u64 {
    fn encode(&self) -> Vec<u8> {
        (*self as usize).encode()
    }
}

impl Encode for i64 {
    fn encode(&self) -> Vec<u8> {
        (*self as usize).encode()
    }
}

impl Encode for &[u8] {
    fn encode(&self) -> Vec<u8> {
        self.to_vec()        
    }
}

impl Encode for Vec<u8> {
    fn encode(&self) -> Vec<u8> {
        self.as_slice().encode()  
    }
}

pub trait EncodeLen {
    fn encode_len(&self) -> Vec<u8>;
}

impl<T: Encode> EncodeLen for &[T] {
    fn encode_len(&self) -> Vec<u8> {
        if self.is_empty() {
            return vec![0];
        }

        let mut seq = Vec::new();
        seq.push(self.len() as u8); 

        for item in *self {
            seq.extend_from_slice(&item.encode());
        }

        seq
    }
}

pub fn encode_trivial(x: usize, l: usize) -> Vec<u8> {

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

pub fn decode(v: &[u8]) -> usize {

    let l = v[0].leading_ones();

    if l == 0 { 
        return v[0] as usize; 
    }

    let result = decode_trivial(&v[1..]);

    if l == 8 {
        return result;
    }

    let mask = (1 << (7 - l)) - 1;

    return result | ((v[0] & mask) as usize) << (8 * l);
}

pub fn decode_trivial(v: &[u8]) -> usize {

    let mut result: usize = 0;
    let max_octets = std::mem::size_of::<usize>();
    for i in 0..std::cmp::min(v.len(), max_octets) {
        result |= (v[i as usize] as usize) << (8 * i);
    }

    return result;
}

pub enum Type {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I64(i64),
}

pub fn decode_len(v: &[u8]) -> Vec<usize> {

    if v.is_empty() {
        return vec![0];
    }

    let length = v[0] as usize;
    let mut result = Vec::with_capacity(length);
    let mut k = 1;

    for _ in 0..length {
        let l = v[k].leading_ones() as usize;
        let member = &v[k..=k + l]; 
        result.push(decode(member));
        k += l + 1;
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

use std::convert::TryInto;

pub struct SliceReader<'a> {
    data: &'a [u8],
    position: usize,
}

#[derive(Debug)]
pub enum ReadError {
    NotEnoughData,
}

impl<'a> SliceReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        SliceReader { data, position: 0 }
    }

    fn read_bytes(&mut self, length: usize) -> Result<&'a [u8], ReadError> {
        let start = self.position;
        let end = start + length;
        if end > self.data.len() {
            return Err(ReadError::NotEnoughData);
        }
        self.position = end;
        Ok(&self.data[start..end])
    }
    
    pub fn current_slice(&self) -> &'a [u8] {
        &self.data[self.position..]
    }

    pub fn get_pos(&self) -> usize {
        return self.position;
    }

    pub fn inc_pos(&mut self, pos: usize) -> Result<(), ReadError> {
        let end = self.data.len();
        if pos + self.position > end {
            return Err(ReadError::NotEnoughData);
        }
        self.position += pos;
        Ok(())
    }

    pub fn read_next_byte(&mut self) -> Result<u8, ReadError> {
        if self.position >= self.data.len() {
            return Err(ReadError::NotEnoughData);
        }
        let byte = self.data[self.position];
        self.position += 1; // Avanzar la posición
        Ok(byte)
    }

    pub fn read_u16(&mut self) -> Result<u16, ReadError> {
        let bytes = self.read_bytes(2)?;
        let mut array = [0u8; 2];
        array.copy_from_slice(bytes);
        Ok(u16::from_le_bytes(array))
    }

    pub fn read_u32(&mut self) -> Result<u32, ReadError> {
        let bytes = self.read_bytes(4)?;
        let mut array = [0u8; 4];
        array.copy_from_slice(bytes);
        Ok(u32::from_le_bytes(array))
    }

    pub fn read_u64(&mut self) -> Result<u64, ReadError> {
        let bytes = self.read_bytes(8)?;
        let mut array = [0u8; 8];
        array.copy_from_slice(bytes);
        Ok(u64::from_le_bytes(array))
    }

    pub fn read_i64(&mut self) -> Result<i64, ReadError> {
        let bytes = self.read_bytes(8)?;
        let mut array = [0u8; 8];
        array.copy_from_slice(bytes);
        Ok(i64::from_le_bytes(array))
    }

    pub fn read_32bytes(&mut self) -> Result<[u8; 32], ReadError> {
        let bytes = self.read_bytes(32)?; 
        let mut array = [0u8; 32]; 
        array.copy_from_slice(bytes); 
        Ok(array) 
    }

    pub fn read_64bytes(&mut self) -> Result<[u8; 64], ReadError> {
        let bytes = self.read_bytes(64)?; 
        let mut array = [0u8; 64]; 
        array.copy_from_slice(bytes); 
        Ok(array) 
    }

    pub fn read_96bytes(&mut self) -> Result<[u8; 96], ReadError> {
        let bytes = self.read_bytes(96)?; 
        let mut array = [0u8; 96]; 
        array.copy_from_slice(bytes); 
        Ok(array) 
    }

    pub fn read_784bytes(&mut self) -> Result<[u8; 784], ReadError> {
        let bytes = self.read_bytes(784)?; 
        let mut array = [0u8; 784]; 
        array.copy_from_slice(bytes); 
        Ok(array) 
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_encode_trivial() {
        assert_eq!(vec![0xDF, 0x63, 0x00, 0x00], encode_trivial(25567, 4));
        assert_eq!(vec![0x21, 0xFF, 0xFF, 0xFF], encode_trivial(0xFFFFFF21, 4));
        assert_eq!(vec![0x21, 0, 0, 0, 0, 0, 0, 0, 0, 0,], encode_trivial(0x21, 10));
        assert_eq!(vec![0x21, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF], encode_trivial(0xFFFFFFFFFFFFFF21, 10));
        assert_eq!(vec![0x1F, 0xB2, 0x01], encode_trivial(111135, 3));
        assert_eq!(vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF], encode_trivial(0xFFFFFFFFFFFFFFFF, 8));
        assert_eq!(vec![0xEF, 0x0C, 0x28, 0x31, 0x0A, 0x0D, 0xC9, 0x10], encode_trivial(1209512312351231215, 8));
        assert_eq!(Vec::<u8>::new(), encode_trivial(56323, 0));
        assert_eq!(vec![0x3C, 0xE2, 0x01, 0, 0, 0, 0, 0, 0], encode_trivial(123452, 9));
        assert_eq!(vec![0x00], encode_trivial(256, 1));
    }

    #[test]
    fn test_encode() {
        assert_eq!(vec![0], (0u8).encode());
        assert_eq!(vec![100], (100u8).encode());
        assert_eq!(vec![127], (127u8).encode());
        assert_eq!(vec![128, 128], (128u8).encode());
        assert_eq!(vec![128, 194], (194u8).encode());
        assert_eq!(vec![128, 255], (255u8).encode());
        assert_eq!(vec![129, 0], (256u16).encode());
        assert_eq!(vec![129, 44], (300u16).encode());
        assert_eq!(vec![131, 255], (1023u16).encode());
        assert_eq!(vec![191, 255], (0x3FFFu16).encode());
        assert_eq!(vec![192, 0, 64], (0x4000u16).encode());
        assert_eq!(vec![191, 255], (16383u16).encode());
        assert_eq!(vec![192, 255, 255], (65535u16).encode());
        assert_eq!(vec![193, 0, 0], (65536u32).encode());
        assert_eq!(vec![222, 0x60, 0x79], (1997152u32).encode());
        assert_eq!(vec![194, 240, 73], (150000u32).encode());
        assert_eq!(vec![224, 10, 0, 32], (2097162u32).encode());
        assert_eq!(vec![239, 255, 255, 255], (0xFFFFFFFu32).encode());
        assert_eq!(vec![240, 0, 0, 0, 16], (0x10000000u32).encode());
        assert_eq!(vec![247, 255, 255, 255, 255], (0x7FFFFFFFFu64).encode());
        assert_eq!(vec![248, 0, 0, 0, 0, 8], (0x800000000u64).encode());
        assert_eq!(vec![251, 255, 255, 255, 255, 255], (0x3FFFFFFFFFFu64).encode());
        assert_eq!(vec![252, 0, 0, 0, 0, 0, 4], (0x40000000000u64).encode());
        assert_eq!(vec![253, 255, 255, 255, 255, 255, 255], (0x1FFFFFFFFFFFFu64).encode());
        assert_eq!(vec![254, 0, 0, 0, 0, 0, 0, 2], (0x2000000000000u64).encode());
        assert_eq!(vec![255, 253, 255, 255, 255, 255, 255, 255, 255], (-3).encode()); 
        assert_eq!(vec![255, 254, 255, 255, 255, 255, 255, 255, 255], (0xFFFFFFFFFFFFFFFEusize).encode()); // -2 
        assert_eq!(vec![255, 255, 255, 255, 255, 255, 255, 255, 127], (9223372036854775807u64).encode());
        assert_eq!(vec![255, 0, 0, 0, 0, 0, 0, 0, 128], (9223372036854775808u64).encode());
        assert_eq!(vec![255, 1, 0, 0, 0, 0, 0, 0, 128], (9223372036854775809u64).encode());
    }

    #[test]
    fn test_encode_len() {
        assert_eq!(vec![0], (&[] as &[u8]).encode_len());
        assert_eq!(vec![1, 0], (&[0u8][..]).encode_len());
        assert_eq!(vec![1, 129, 44], (&[300u16][..]).encode_len());
        assert_eq!(vec![4, 129, 44, 18, 127, 128, 128], (&[300u16, 18u16, 127u16, 128u16][..]).encode_len());
        assert_eq!(vec![2, 194, 240, 73, 223, 255, 255], (&[150000u32, 2097151u32][..]).encode_len());
        assert_eq!(vec![2, 255, 1, 0, 0, 0, 0, 0, 0, 128, 248, 0, 0, 0, 0, 8][..], (&[9223372036854775809u64, 0x800000000u64][..]).encode_len());
    }

    #[test]
    fn test_encode_from_bits() {
        assert_eq!(Vec::<u8>::new(), encode_from_bits(&[]));
        assert_eq!(vec![0], encode_from_bits(&[false]));
        assert_eq!(vec![1], encode_from_bits(&[true]));
        assert_eq!(vec![0x55], encode_from_bits(&[true, false, true, false, true, false, true, false]));
        assert_eq!(vec![0x55, 1], encode_from_bits(&[true, false, true, false, true, false, true, false, true]));
    }

    #[test]
    fn test_decode_trivial() {
        assert_eq!(0, decode_trivial(&[]));
        assert_eq!(0, decode_trivial(&[0]));
        assert_eq!(27, decode_trivial(&[27]));
        assert_eq!(150000, decode_trivial(&[0xF0, 0x49, 0x02]));
        assert_eq!(9223372036854775809, decode_trivial(&[1, 0, 0, 0, 0, 0, 0, 0x80]));
    } 

    #[test]
    fn test_decode() {
        assert_eq!(0, decode(&[0]));
        assert_eq!(100, decode(&[100]));
        assert_eq!(300, decode(&[129, 44]));
        assert_eq!(150000, decode(&[194, 240, 73]));
        assert_eq!(2097151, decode(&[223, 255, 255]));
        assert_eq!(0x800000000, decode(&[248, 0, 0, 0, 0, 8]));
        assert_eq!(0x2000000000000, decode(&[254, 0, 0, 0, 0, 0, 0, 2]));
        assert_eq!(9223372036854775807, decode(&[255, 255, 255, 255, 255, 255, 255, 255, 127]));
        assert_eq!(9223372036854775808, decode(&[255, 0, 0, 0, 0, 0, 0, 0, 128]));      
        assert_eq!(9223372036854775809, decode(&[255, 1, 0, 0, 0, 0, 0, 0, 128]));     
    }

    #[test]
    fn test_decode_len() {
        assert_eq!(vec![0], decode_len(&[]));
        assert_eq!(vec![0], decode_len(&[1, 0]));
        assert_eq!(vec![1], decode_len(&[1, 1]));
        assert_eq!(vec![1, 2], decode_len(&[2, 1, 2]));
        assert_eq!(vec![300, 300], decode_len(&[2, 129, 44, 129, 44]));
        assert_eq!(vec![300, 127], decode_len(&[2, 129, 44, 127]));
        assert_eq!(vec![300, 127], decode_len(&[2, 129, 44, 127, 22, 38, 189, 1]));
        assert_eq!(vec![9223372036854775809, 65535], decode_len(&[2, 255, 1, 0, 0, 0, 0, 0, 0, 128, 192, 255, 255]));
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
