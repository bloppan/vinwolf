use std::collections::BTreeMap;

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
/* 
    Equation 274
    First case: _y = (x mod 2^(8l)) 
    Second case: _y = x 
*/
pub fn serialize_number(x: u64) -> Vec<u8> {
    
    let mut l = 0;
    let mut _y: u64; 

    // Determine l value
    while l < 8 && (1 << (7 * (l + 1))) <= x {
        l += 1;
    }
    // Calculate prefix
    let (prefix, _y) = if ((1 << (7 * l)) <= x) && (1 << (7 * (l + 1)) > x) {
        ((256u16 - (1u16 << (8 - l)) + (x >> (8 * l)) as u16) as u8, x % (1 << (8 * l)))
    } else {
        (255, x)
    };

    // Number serialization
    let mut octet;
    let mut serialized: Vec<u8> = Vec::new();
    serialized.push(prefix);
    for i in 0..l {
        octet = ((_y >> (8 * i)) & 0xFF) as u8;
        serialized.push(octet);
    }

    serialized
}

pub fn serialize_string(s: String) -> Vec<u8> {

    s.into_bytes()
}

pub fn sequence_encoder(v: &Vec<u64>) -> Vec<u8> {

    let mut u: Vec<u8> = Vec::new();
    for i in 0..v.len() {
        u.extend(serialize_number(v[i]));
    }
    return u;
}

pub fn serialize_bits(sequence: Vec<u8>) -> Vec<bool> {
    let mut bools = Vec::new();
    for byte in sequence {
        for i in 0..8 { 
            let bit = (byte >> i) & 1; 
            bools.push(bit == 1); // Convert bit (0 o 1) to boolean
        }
    }
    bools
}

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
/**
*****************************************************************************************************************************************
**/
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

pub fn test_bits_codec() {

    let bits = vec![17, 6];
    let serialized = serialize_bits(bits);
    
    println!("Secuencia serializada: {:?}", serialized);

}

pub fn test_sequencer_codec() {

    let sequence = vec![150000, 18446744073709551610u64, 15446744073709551610u64];
    let res_sequence: Vec<u8>;
    res_sequence = sequence_encoder(&sequence);
    println!("Numeros {:?}", sequence);
    println!("Serializacion: {:?}", res_sequence);

}

pub fn test_integer_codec() {

        // Ejemplo de uso
        let number = 18446744073709551610u64;
        let serialized = serialize_number(number);
        // Mostrar el resultado
        println!("Número: {}", number);
        println!("Serialización: {:?}", serialized);
        let serialized = trivial_serialize(number, 4);
        // Mostrar el resultado
        println!("Número: {}", number);
        println!("Serialización: {:?}", serialized);
        let number = 150000u64;    
        let serialized = serialize_number(number);
        // Mostrar el resultado
        println!("Número: {}", number);
        println!("Serialización: {:?}", serialized);
        let serialized = trivial_serialize(number, 4);
        // Mostrar el resultado
        println!("Número: {}", number);
        println!("Serialización: {:?}", serialized);
        let number = 1000u64;
        let serialized = serialize_number(number);
        // Mostrar el resultado
        println!("Número: {}", number);
        println!("Serialización: {:?}", serialized);
        let serialized = trivial_serialize(number, 4);
        // Mostrar el resultado
        println!("Número: {}", number);
        println!("Serialización: {:?}", serialized);
}
