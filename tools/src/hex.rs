#![forbid(unsafe_code)]

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FromHexError {
    InvalidHexCharacter { c: char, index: usize },
    OddLength,
    InvalidStringLength,
}

#[inline]
fn to_hex_lower(byte: u8) -> (u8, u8) {
    const LUT: &[u8; 16] = b"0123456789abcdef";
    let hi = LUT[(byte >> 4) as usize];
    let lo = LUT[(byte & 0x0f) as usize];
    (hi, lo)
}

#[inline]
fn to_hex_upper(byte: u8) -> (u8, u8) {
    const LUT: &[u8; 16] = b"0123456789ABCDEF";
    let hi = LUT[(byte >> 4) as usize];
    let lo = LUT[(byte & 0x0f) as usize];
    (hi, lo)
}

pub fn encode<T: AsRef<[u8]>>(data: T) -> String {
    let bytes = data.as_ref();
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        let (hi, lo) = to_hex_lower(b);
        s.push(hi as char);
        s.push(lo as char);
    }
    s
}

pub fn encode_upper<T: AsRef<[u8]>>(data: T) -> String {
    let bytes = data.as_ref();
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        let (hi, lo) = to_hex_upper(b);
        s.push(hi as char);
        s.push(lo as char);
    }
    s
}

#[inline]
fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

pub fn decode<T: AsRef<[u8]>>(s: T) -> Result<Vec<u8>, FromHexError> {
    let s = s.as_ref();
    if s.len() % 2 != 0 {
        return Err(FromHexError::OddLength);
    }
    let mut out = Vec::with_capacity(s.len() / 2);
    let mut i = 0usize;
    while i < s.len() {
        let h = s[i];
        let l = s[i + 1];
        let hv = hex_val(h).ok_or_else(|| FromHexError::InvalidHexCharacter { c: h as char, index: i })?;
        let lv = hex_val(l).ok_or_else(|| FromHexError::InvalidHexCharacter { c: l as char, index: i + 1 })?;
        out.push((hv << 4) | lv);
        i += 2;
    }
    Ok(out)
}

pub fn decode_to_slice<T: AsRef<[u8]>>(s: T, out: &mut [u8]) -> Result<(), FromHexError> {
    let s = s.as_ref();
    if s.len() % 2 != 0 {
        return Err(FromHexError::OddLength);
    }
    if out.len() != s.len() / 2 {
        return Err(FromHexError::InvalidStringLength);
    }
    let mut i = 0usize;
    let mut j = 0usize;
    while i < s.len() {
        let h = s[i];
        let l = s[i + 1];
        let hv = hex_val(h).ok_or_else(|| FromHexError::InvalidHexCharacter { c: h as char, index: i })?;
        let lv = hex_val(l).ok_or_else(|| FromHexError::InvalidHexCharacter { c: l as char, index: i + 1 })?;
        out[j] = (hv << 4) | lv;
        i += 2;
        j += 1;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn roundtrip_lower() {
        let data = [0u8, 1u8, 2u8, 3u8, 4u8];
        println!("data: {}", encode(&data));
        let encoded = encode(&data);
        println!("decoded: {:?}", decode(&encoded));
        let data = b"\x00\x01\xfe\xffhello";
        let h = encode(data);
        assert_eq!(h, "0001fe666f68656c6c6f".replace("fe", "fe"));
        let d = decode(&h).unwrap();
        assert_eq!(d, data);
    }
    #[test]
    fn roundtrip_upper() {
        let data = b"\x00\xab\xcd\xef";
        let h = encode_upper(data);
        assert_eq!(h, "00ABCDEF");
        let d = decode(&h).unwrap();
        assert_eq!(d, data);
    }
    #[test]
    fn errors() {
        assert!(matches!(decode("a"), Err(FromHexError::OddLength)));
        assert!(matches!(decode("xz"), Err(FromHexError::InvalidHexCharacter{..})));
        let mut buf = [0u8; 1];
        assert!(matches!(decode_to_slice("0001", &mut buf), Err(FromHexError::InvalidStringLength)));
    }
}
