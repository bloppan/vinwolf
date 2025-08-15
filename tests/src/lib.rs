use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use jam_types::ProcessError;

mod test_codec;
mod test_types;
mod codec;
//mod accumulate;
mod authorization;
mod assurances;
mod disputes;
mod recent_history;
//mod preimages;
mod pvm;
//mod reports;
mod safrole;
mod shuffle;
mod trie;
mod statistics;
mod traces;

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

#[allow(dead_code)]
trait FromProcessError {
    fn from_process_error(error: ProcessError) -> Self;
}
