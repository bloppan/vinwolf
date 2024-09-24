use std::collections::BTreeMap;

pub fn seq_to_number(v: &Vec<u8>) -> u32 {

    let mut result = 0;
    let size = v.len();
    //println!("vec = {:?}, size = {size}", v);
    for i in 0..size {
        result |= (v[(size - i - 1) as usize] as u32) << (size - i - 1) * 8; 
    }
    result
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

pub fn decode_trivial(v: &Vec<u8>) -> usize {

    let mut result: usize = 0;
    let max_octets = std::mem::size_of::<usize>();
    for i in 0..std::cmp::min(v.len(), max_octets) {
        result |= (v[i as usize] as usize) << (8 * i);
    }

    return result;
}

pub fn encode_general(x: usize) -> Vec<u8> {

    if x == 0 { 
        return vec![0]; 
    }

    let mut l: i32 = 7;
    // Determine l value
    while l >= 0 {
        if 1 << (7 * l) <= x && x < 1 << (7 * (l + 1)) {
            // 2(^8) - 2(^(8 - l)) + (x / 2^(8l)) + concatenate encode_trivial(x, l)
            return [vec![(256u64 - (1u64 << (8 - l)) + (x >> (8 * l)) as u64) as u8], encode_trivial(x, l as usize)].concat();
        }
        l -= 1;
    }

    return [vec![255], encode_trivial(x, 8)].concat();
}

fn calc_l_from_prefix(prefix: u8) -> u32 {

    let mut i = 0x80;
    let mut l = 0;

    while prefix & i > 0 {
        l += 1;
        i >>= 1;
    }

    return l;
}

pub fn decode_general(v: Vec<u8>) -> usize {

    let l = calc_l_from_prefix(v[0]);

    if l == 0 { 
        return v[0] as usize; 
    }

    let result = decode_trivial(&v[1..].to_vec());

    if l == 8 {
        return result;
    }

    let mut mask = 0;
    let mut i = 0;
    while (7 - i) > l {
        mask |= 1 << i;
        i += 1;
    };

    return result | ((v[0] & mask) as usize) << (8 * l);
}

pub fn encode_variable_length(x: &Vec<usize>) -> Vec<u8> {

    if x.is_empty() {
        return vec![0];
    }

    let mut seq: Vec<u8> = Vec::new();
    seq.push(x.len() as u8);
    for i in 0..x.len() {
        seq.extend_from_slice(&encode_general(x[i]));
    }
    
    return seq;
}

pub fn decode_variable_length(v: &Vec<u8>) -> Vec<usize> {

    if v.is_empty() {
        return vec![0];
    }

    let mut result: Vec<usize> = Vec::new();
    let length: usize = v[0] as usize;
    let mut i: usize = 0;
    let mut k: usize = 1;

    while i < length {
        let l = calc_l_from_prefix(v[k]) as usize;
        let mut member: Vec<u8> = vec![v[k]];
        for j in (k + 1)..(k + 1 + l) {
            member.push(v[j]);
        }
        result.extend_from_slice(&[decode_general(member)]);
        k += l + 1;
        i += 1;
    }

    return result;
}

pub fn decode_to_bits(v: &Vec<u8>) -> Vec<bool> {

    let mut bools = Vec::new();
    for byte in v {
        for i in 0..8 { 
            let bit = (byte >> i) & 1; 
            bools.push(bit == 1); // Convert bit (0 o 1) to boolean
        }
    }

    return bools;
}

pub fn encode_from_bits(v: &Vec<bool>) -> Vec<u8> {

    if v.is_empty() {
        return vec![];
    }

    let mut result: Vec<u8> = vec![0; (v.len() + 7) / 8];
    let mut i: usize = 0;
    let mut j: usize = 0;
    
    for bit in v {
        result[i] |= (*bit as u8) << j;
        j += 1;
        if j % 8 == 0 {
            j = 0;
            i += 1;
        }
    }
    
    return result;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_encode_from_bits() {
        assert_eq!(Vec::<u8>::new(), encode_from_bits(&vec![]));
        assert_eq!(vec![0], encode_from_bits(&vec![false]));
        assert_eq!(vec![1], encode_from_bits(&vec![true]));
        assert_eq!(vec![0x55], encode_from_bits(&vec![true, false, true, false, true, false, true, false]));
        assert_eq!(vec![0x55, 1], encode_from_bits(&vec![true, false, true, false, true, false, true, false, true]));
    }

    #[test]
    fn test_decode_to_bits() {
        assert_eq!(Vec::<bool>::new(), decode_to_bits(&Vec::<u8>::new()));
        assert_eq!(vec![false, false, false, false, false, false, false, false], decode_to_bits(&vec![0]));
        assert_eq!(vec![true, false, false, false, false, false, false, false], decode_to_bits(&vec![1]));
        assert_eq!(vec![true, true, false, true, true, false, false, false,
                        true, true, true, true, true, true, true, true], decode_to_bits(&vec![27, 255]));
        assert_eq!(vec![true, false, true, false, true, false, true, false,
                        false, false, false, false, false, false, false, false,
                        false, false, false, false, true, true, true, true], decode_to_bits(&vec![0x55, 0, 0xF0]));
    }

    #[test]
    fn test_decode_variable_length() {
        assert_eq!(vec![0], decode_variable_length(&Vec::<u8>::new()));
        assert_eq!(vec![0], decode_variable_length(&vec![1, 0]));
        assert_eq!(vec![1], decode_variable_length(&vec![1, 1]));
        assert_eq!(vec![1, 2], decode_variable_length(&vec![2, 1, 2]));
        assert_eq!(vec![300, 300], decode_variable_length(&vec![2, 129, 44, 129, 44]));
        assert_eq!(vec![300, 127], decode_variable_length(&vec![2, 129, 44, 127]));
        assert_eq!(vec![9223372036854775809, 65535], decode_variable_length(&vec![2, 255, 1, 0, 0, 0, 0, 0, 0, 128, 192, 255, 255]));
    }

    #[test]
    fn test_decode_trivial() {
        assert_eq!(0, decode_trivial(&vec![]));
        assert_eq!(0, decode_trivial(&vec![0]));
        assert_eq!(27, decode_trivial(&vec![27]));
        assert_eq!(150000, decode_trivial(&vec![0xF0, 0x49, 0x02]));
        assert_eq!(9223372036854775809, decode_trivial(&vec![1, 0, 0, 0, 0, 0, 0, 0x80]));
    }

    #[test]
    fn test_encode_variable_length() {
        assert_eq!(vec![0], encode_variable_length(&vec![]));
        assert_eq!(vec![1, 0], encode_variable_length(&vec![0]));
        assert_eq!(vec![1, 129, 44], encode_variable_length(&vec![300]));
        assert_eq!(vec![4, 129, 44, 18, 127, 128, 128], encode_variable_length(&vec![300, 18, 127, 128]));
        assert_eq!(vec![2, 194, 240, 73, 223, 255, 255], encode_variable_length(&vec![150000, 2097151]));
        assert_eq!(vec![2, 255, 1, 0, 0, 0, 0, 0, 0, 128, 248, 0, 0, 0, 0, 8], encode_variable_length(&vec![9223372036854775809, 0x800000000]));
    }

    #[test]
    fn test_decode_general() {
        assert_eq!(0, decode_general(vec![0]));
        assert_eq!(100, decode_general(vec![100]));
        assert_eq!(300, decode_general(vec![129, 44]));
        assert_eq!(150000, decode_general(vec![194, 240, 73]));
        assert_eq!(2097151, decode_general(vec![223, 255, 255]));
        assert_eq!(0x800000000, decode_general(vec![248, 0, 0, 0, 0, 8]));
        assert_eq!(0x2000000000000, decode_general(vec![254, 0, 0, 0, 0, 0, 0, 2]));
        assert_eq!(9223372036854775807, decode_general(vec![255, 255, 255, 255, 255, 255, 255, 255, 127]));
        assert_eq!(9223372036854775808, decode_general(vec![255, 0, 0, 0, 0, 0, 0, 0, 128]));      
        assert_eq!(9223372036854775809, decode_general(vec![255, 1, 0, 0, 0, 0, 0, 0, 128]));     
    }

    #[test]
    fn test_encode_general() {
        assert_eq!(vec![255, 254, 255, 255, 255, 255, 255, 255, 255], encode_general(0xFFFFFFFFFFFFFFFE)); // -2
        assert_eq!(vec![0], encode_general(0));
        assert_eq!(vec![100], encode_general(100));
        assert_eq!(vec![127], encode_general(127));
        assert_eq!(vec![128, 128], encode_general(128));
        assert_eq!(vec![128, 194], encode_general(194));
        assert_eq!(vec![128, 255], encode_general(255));
        assert_eq!(vec![129, 0], encode_general(256));
        assert_eq!(vec![129, 44], encode_general(300));
        assert_eq!(vec![131, 255], encode_general(1023));
        assert_eq!(vec![191, 255], encode_general(0x3FFF));
        assert_eq!(vec![192, 0, 64], encode_general(0x4000));
        assert_eq!(vec![191, 255], encode_general(16383));
        assert_eq!(vec![192, 255, 255], encode_general(65535));
        assert_eq!(vec![193, 0, 0], encode_general(65536));
        assert_eq!(vec![222, 0x60, 0x79], encode_general(1997152));
        assert_eq!(vec![194, 240, 73], encode_general(150000));
        assert_eq!(vec![224, 10, 0, 32], encode_general(2097162));
        assert_eq!(vec![239, 255, 255, 255], encode_general(0xFFFFFFF));
        assert_eq!(vec![240, 0, 0, 0, 16], encode_general(0x10000000));
        assert_eq!(vec![247, 255, 255, 255, 255], encode_general(0x7FFFFFFFF));
        assert_eq!(vec![248, 0, 0, 0, 0, 8], encode_general(0x800000000));
        assert_eq!(vec![251, 255, 255, 255, 255, 255], encode_general(0x3FFFFFFFFFF));
        assert_eq!(vec![252, 0, 0, 0, 0, 0, 4], encode_general(0x40000000000));
        assert_eq!(vec![253, 255, 255, 255, 255, 255, 255], encode_general(0x1FFFFFFFFFFFF));
        assert_eq!(vec![254, 0, 0, 0, 0, 0, 0, 2], encode_general(0x2000000000000));
        assert_eq!(vec![255, 255, 255, 255, 255, 255, 255, 255, 127], encode_general(9223372036854775807));
        assert_eq!(vec![255, 0, 0, 0, 0, 0, 0, 0, 128], encode_general(9223372036854775808));      
        assert_eq!(vec![255, 1, 0, 0, 0, 0, 0, 0, 128], encode_general(9223372036854775809));      
    }

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
