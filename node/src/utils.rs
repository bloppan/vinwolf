use std::fs::File;
use std::io::Read;
use crate::constants::{*};
use crate::jam_types::{RawState, Block, GlobalState};
use crate::blockchain::state::{get_global_state, state_transition_function};
use crate::utils::codec::{ReadError, Decode, BytesReader};
use crate::utils::trie::merkle_state;
use crate::{blockchain::state::set_global_state};

use std::fs;
use std::path::{Path, PathBuf};
use tokio::io::{AsyncWriteExt};
use std::error::Error;

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

pub async fn read_file_to_buffer(path: &str, buf: &mut Vec<u8>) -> Result<(), Box<dyn Error>> {
    let mut file = tokio::fs::File::create(path).await?; 
    file.write_all(&buf).await?; 
    Ok(())
}

pub fn read_bin_file(path: &Path) -> Result<Vec<u8>, ()> {
    
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(e) => { log::error!("Failed to open file {}: {}", path.display(), e); return Err(()) },
    };

    let mut test_content = Vec::new();

    match file.read_to_end(&mut test_content) {
        Ok(_) => { return Ok(test_content) },
        Err(e) => { log::error!("Failed to read file {}: {}", path.display(), e); return Err(()) },
    }
}

pub fn read_filenames_in_dir(dir: &str) -> Result<Vec<PathBuf>, ()> {

    let path = Path::new(dir);

    let entries = match fs::read_dir(path) {
        Ok(res) => { res },
        Err(_) => { log::error!("Failed to read directory {:?}", path); return Err(()); },
    };

    let mut files = Vec::new();

    for entry in entries.filter_map(Result::ok) {
        let entry_path = entry.path();

        if entry_path.is_file() && entry_path.extension().map(|e| e == "bin").unwrap_or(false) {
            log::info!("New file found: {:?}", entry_path);
            files.push(entry_path);
        }
    }

    Ok(files)
}

pub fn decode_trace_bin_file(file_content: &[u8]) -> Result<(RawState, Block, RawState), ReadError> {

    let mut reader = BytesReader::new(&file_content);
    let pre_state = RawState::decode(&mut reader)?;
    let block = Block::decode(&mut reader)?;
    let post_state = RawState::decode(&mut reader)?;

    return Ok((pre_state, block, post_state));
}

pub fn import_block(path: &Path) -> Result<(), ()> {
    
    let test_content = read_bin_file(path)?;
    let (pre_state, block, post_state) = match decode_trace_bin_file(&test_content) {
        Ok(result) => result,
        Err(_) => { log::error!("Failed to decode {:?}", path); return Err(()) },
    };

    let state = GlobalState::default();
    let expected_state = GlobalState::default();
    
    assert_eq!(pre_state.state_root.clone(), merkle_state(&state.serialize().map, 0).unwrap());
    assert_eq!(post_state.state_root.clone(), merkle_state(&expected_state.serialize().map, 0).unwrap());

    set_global_state(state.clone());

    let error = state_transition_function(&block);
    
    if error.is_err() {
        println!("****************************************************** Error: {:?}", error);
        return Err(());
    }

    let result_state = get_global_state().lock().unwrap().clone();
    
    /*println!("post_sta state_root: {:x?}", post_state.state_root);
    println!("expected state_root: {:x?}", merkle_state(&expected_state.serialize().map, 0).unwrap());
    println!("result   state_root: {:x?}", merkle_state(&result_state.serialize().map, 0).unwrap());*/
    
    assert_eq!(post_state.state_root, merkle_state(&result_state.serialize().map, 0).unwrap());

    Ok(())
}