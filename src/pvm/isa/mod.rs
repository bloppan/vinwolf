pub mod no_arg;
pub mod one_imm;
pub mod one_offset;
pub mod one_reg_one_imm;
pub mod one_reg_one_imm_one_offset;
pub mod one_reg_two_imm;
pub mod three_reg;
pub mod two_imm;
pub mod two_reg_one_imm;
pub mod two_reg_one_offset;
pub mod two_reg_two_imm;
pub mod two_reg;


fn skip(i: &u64, k: &[bool]) -> u64 {
    let mut j = i + 1;
    //println!("k = {:?}", k);
    while j < k.len() as u64 && k[j as usize] == false {
        j += 1;
    }
    //println!("j = {}", j-1);
    std::cmp::min(24, j - 1)
}


pub fn extend_sign(v: &[u8], n: usize) -> u64 {

    if ![1, 2, 3, 4, 8].contains(&n) {
        return 0;
    }

    let mut buffer = [0u8; 8];
    buffer[..n].copy_from_slice(&v[..n]);

    let x = u64::from_le_bytes(buffer);
    let sign_bit = (x >> ((8 * n) - 1)) & 1;

    if sign_bit == 1 {
        x | (!0u64 << (8 * n - 1))
    } else {
        x
    }
}

/*fn basic_block_seq(pc: usize, k: &Vec<bool>) -> u32 {
    return 1 + skip(pc, k) as u32;
}*/

#[cfg(test)]
mod test { 
    use super::*;

    #[test]
    fn test_extend_sign() {
        let test_cases = vec![
            (vec![0x01], 1, 1u64),
            (vec![0xFF], 1, 0xFFFFFFFFFFFFFFFFu64),
            (vec![0x40], 1, 0x40),
            (vec![0x80], 1, 0xFFFFFFFFFFFFFF80),
            (vec![0x01, 0x00], 2, 1u64),
            (vec![0x80, 0xFF], 2, 0xFFFFFFFFFFFFFF80),
            (vec![0xFF, 0xFF], 2, 0xFFFFFFFFFFFFFFFFu64),
            (vec![0x00, 0x80], 2, 0xFFFFFFFFFFFF8000u64),
            (vec![0x01, 0x00, 0x00], 3, 1u64),
            (vec![0xFF, 0xFF, 0xFF], 3, 0xFFFFFFFFFFFFFFFFu64),
            (vec![0x00, 0x00, 0x80], 3, 0xFFFFFFFFFF800000u64),
            (vec![0xD4, 0xFE, 0xFF], 3, 0xFFFFFFFFFFFFFED4u64),
            (vec![0x01, 0x00, 0x00, 0x00], 4, 1u64),
            (vec![0xFF, 0xFF, 0xFF, 0xFF], 4, 0xFFFFFFFFFFFFFFFFu64),
            (vec![0x00, 0x00, 0x00, 0x80], 4, 0xFFFFFFFF80000000u64),
            (vec![0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], 8, 1u64),
            (vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF], 8, 0xFFFFFFFFFFFFFFFFu64),
            (vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80], 8, 0x8000000000000000u64),
        ];
    
        for (input, size, expected) in test_cases {
            let result = extend_sign(&input, size);
            assert_eq!(expected, result, "Fail input {:?} size {}", input, size);
        }
    }
}