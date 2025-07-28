pub mod common;
pub mod shuffle;
pub mod trie;
pub mod bandersnatch;
pub mod serialization;

#[macro_export] macro_rules! print_hash {
    ($hash:expr) => {{
        let hash_str = $hash.iter().map(|byte| format!("{:02x}", byte)).collect::<String>();
        let truncated_hash = format!("{}...{}", &hash_str[..4], &hash_str[hash_str.len()-4..]);
        truncated_hash
    }};
}

#[macro_export] macro_rules! print_hash_len {
    ($hash:expr, $first:expr, $last:expr) => {{
        let hash_str = $hash.iter().map(|byte| format!("{:02x}", byte)).collect::<String>();
        let first_part = &hash_str[..$first * 2]; 
        let truncated_hash = format!("{}", first_part);
        truncated_hash
    }};
}

#[macro_export] macro_rules! print_hash_start {
    ($hash:expr) => {{
        let hash_str = $hash.iter().map(|byte| format!("{:02x}", byte)).collect::<String>();
        let truncated_hash = format!("{}", &hash_str[..4]);
        truncated_hash
    }};
}


#[macro_export] macro_rules! print_hash_end {
    ($hash:expr) => {{
        let hash_str = $hash.iter().map(|byte| format!("{:02x}", byte)).collect::<String>();
        let truncated_hash = format!("{}", &hash_str[hash_str.len()-4..]);
        truncated_hash
    }};
}