use std::fs::File;
use std::io::{Read};
use std::path::PathBuf;

extern crate vinwolf;

mod safrole;
mod pvm;
mod codec;
mod trie;
mod erasure;
mod history;

pub fn read_test_file(filename: &str) -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(filename);
    let mut file = File::open(&path).expect("Failed to open file");
    let mut test_content = Vec::new();
    file.read_to_end(&mut test_content).expect("Failed to read file");
    return test_content;
}