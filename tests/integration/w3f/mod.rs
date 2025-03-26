use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

extern crate vinwolf;

use vinwolf::types::ProcessError;

mod safrole;
mod seals;
pub mod codec;
mod trie;
mod erasure;
mod history;
mod disputes;
mod shuffle;
mod reports;
mod assurances;
mod authorization;
mod statistics;
mod preimages;
mod accumulate;
mod pvm;
mod host_function;

//mod jamtestnet;

pub fn read_test_file(filename: &str) -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(filename);
    let mut file = match File::open(&path) {
        Ok(file) => file,
        Err(e) => panic!("Failed to open file '{}': {}", path.display(), e),
    };
    let mut test_content = Vec::new();
    let _ = file.read_to_end(&mut test_content);
    test_content
}

trait FromProcessError {
    fn from_process_error(error: ProcessError) -> Self;
}
