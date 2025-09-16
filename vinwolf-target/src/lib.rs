use safrole::{get_verifiers, set_verifiers, create_ring_set};
use jam_types::{Block, GlobalState, OpaqueHash, RawState, ReadError};
use codec::{Decode, BytesReader};
use std::collections::VecDeque;
use std::path::{PathBuf, Path};
use std::{fs, collections::HashSet};
use utils::{common::parse_state_keyvals, serialization, trie::merkle_state, log, hex};
use utils::bandersnatch::Verifier;

pub fn parse_trace_file(test_content: &[u8]) -> Result<(GlobalState, Block, GlobalState), ReadError>{

    let mut reader = BytesReader::new(&test_content);
    let pre_state = RawState::decode(&mut reader).expect("Error decoding pre state");
    let block = Block::decode(&mut reader).expect("Error decoding the block");
    let post_state = RawState::decode(&mut reader).expect("Error decoding post state");

    let mut state = GlobalState::default();
    let mut expected_state = GlobalState::default();

    parse_state_keyvals(&pre_state.keyvals, &mut state).expect("Error decoding pre state keyvals");
    assert_eq!(pre_state.state_root.clone(), merkle_state(&serialization::serialize(&state).map, 0));
    parse_state_keyvals(&post_state.keyvals, &mut expected_state).expect("Error decoding post state keyvals");
    assert_eq!(post_state.state_root.clone(), merkle_state(&serialization::serialize(&expected_state).map, 0));

    return Ok((state, block, expected_state));
}

pub fn parse_genesis_file(test_content: &[u8]) -> Result<(Block, GlobalState), ReadError>{

    let mut reader = BytesReader::new(&test_content);
    let block = Block::decode(&mut reader).expect("Error decoding the block");
    let first_state = RawState::decode(&mut reader).expect("Error decoding post state");
    let mut state = GlobalState::default();
    parse_state_keyvals(&first_state.keyvals, &mut state).expect("Error decoding post state keyvals");
    assert_eq!(first_state.state_root.clone(), merkle_state(&serialization::serialize(&state).map, 0));

    return Ok((block, state));
}

pub fn process_trace(path: &Path) {
    
    let test_content = utils::common::read_bin_file(&path).expect("Error reading the trace bin file");
    let (pre_state, block, post_state) = parse_trace_file(&test_content).unwrap();

    state_handler::set_global_state(pre_state.clone());
    state_handler::set_state_root(utils::trie::merkle_state(&utils::serialization::serialize(&pre_state).map, 0));
    block::header::set_parent_header(OpaqueHash::default()); // Dont verify the parent header hash

    if get_verifiers().len() == 0 {
        let mut verifiers = VecDeque::new();
        let pending_validators = state_handler::get_global_state().lock().unwrap().safrole.pending_validators.clone();
        let curr_validators = state_handler::get_global_state().lock().unwrap().curr_validators.clone();
        verifiers.push_back(Verifier::new(create_ring_set(&curr_validators)));
        verifiers.push_back(Verifier::new(create_ring_set(&pending_validators)));
        set_verifiers(verifiers);
    }
    
    match state_controller::stf(&block) {
        Ok(_) => { println!("Block {:?} processed successfully", path); },
        Err(e) => { println!("Refused block: {:?}", e) },
    };

    let result_state = state_handler::get_global_state().lock().unwrap().clone();        
    let state_root_result = utils::trie::merkle_state(&utils::serialization::serialize(&result_state).map, 0);
    let state_root_expected = utils::trie::merkle_state(&utils::serialization::serialize(&post_state).map, 0);

    assert_eq_state(&post_state, &result_state);
    assert_eq!(state_root_expected, state_root_result);
    
    log::info!("Trace {:?} processed successfully", path);
}

pub fn read_all_bins(dir_path: &Path) -> Vec<(u32, PathBuf)> {

    let dir= match std::fs::read_dir(dir_path) {
        Ok(bin_files) => bin_files,
        Err(error) => panic!("Dir {:?} could not be open: {:?}", dir_path, error),
    };

    let mut bin_files: Vec<(u32, PathBuf)> = dir.filter_map(|f| {
        let f = f.ok()?.path();
        if f.extension()? == "bin" {
            if let Some(stem) = f.file_stem()?.to_str() {
                if let Ok(num) = stem.parse::<u32>() {
                    return Some((num, f));
                }
            }
        }
        None
    })
    .collect();

    bin_files.sort_by_key(|(num, _)| *num);

    return bin_files;
}

pub fn process_all_bins(dir_path: &Path) -> std::io::Result<()> {

    let bin_files = read_all_bins(dir_path);

    for (_slot, bin_path) in bin_files {
        
        process_trace(&bin_path);
        log::info!("{:?} processed successfully", bin_path);
    }

    Ok(())
}

pub fn process_all_dirs(base_dir: &Path, skip_dirs: &HashSet<String>) -> std::io::Result<Vec<PathBuf>> {

    let mut dirs: Vec<PathBuf> = Vec::new();

    for entry in fs::read_dir(base_dir)? {
        let dir_entry = entry?;
        let path = dir_entry.path();

        if !path.is_dir() {
            continue;
        }

        // Nombre del directorio
        let dir_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name.to_string(),
            None => continue,
        };

        // Saltar si está en la lista de exclusión
        if skip_dirs.contains(&dir_name) {
            println!("Skip directory {}", dir_name);
            continue;
        }

        // Procesar el directorio
        process_all_bins(&path)?;
        println!("");
        dirs.push(path);

        // Clean up the verifiers
        set_verifiers(VecDeque::new());
        // Clean up the stored parent header
        block::header::set_parent_header(OpaqueHash::default());
        // Clean up the ancestors
        storage::set_ancestors(storage::AncestorsInfo::default());
    }

    Ok(dirs)
}

fn assert_eq_state(expected_state: &GlobalState, result_state: &GlobalState) {
    assert_eq!(expected_state.time, result_state.time);
    assert_eq!(expected_state.safrole.epoch_root, result_state.safrole.epoch_root);
    assert_eq!(expected_state.safrole.pending_validators, result_state.safrole.pending_validators);
    assert_eq!(expected_state.safrole.seal, result_state.safrole.seal);
    assert_eq!(expected_state.safrole.ticket_accumulator, result_state.safrole.ticket_accumulator);
    assert_eq!(expected_state.safrole, result_state.safrole);
    assert_eq!(expected_state.entropy, result_state.entropy);
    assert_eq!(expected_state.curr_validators, result_state.curr_validators);
    assert_eq!(expected_state.prev_validators, result_state.prev_validators);
    assert_eq!(expected_state.disputes.offenders, result_state.disputes.offenders);
    assert_eq!(expected_state.availability, result_state.availability);
    assert_eq!(expected_state.ready_queue, result_state.ready_queue);
    assert_eq!(expected_state.accumulation_history, result_state.accumulation_history);
    assert_eq!(expected_state.privileges, result_state.privileges);
    assert_eq!(expected_state.next_validators, result_state.next_validators);
    assert_eq!(expected_state.auth_queues, result_state.auth_queues);
    assert_eq!(expected_state.recent_history, result_state.recent_history);
    assert_eq!(expected_state.recent_acc_outputs, result_state.recent_acc_outputs);

    /*for service_account in expected_state.service_accounts.iter() {
        if let Some(account) = result_state.service_accounts.get(&service_account.0) {
        log::info!("- Expected for service: {:?}", *service_account.0);
        for item in service_account.1.storage.iter() { 
            log::info!("key: {}", hex::encode(item.0));
            if item.1.len() > 31 {
                log::info!("val: {}...", hex::encode(&item.1[..31]));
            } else {
                log::info!("val: {} | len: {:?}", hex::encode(item.1), item.1.len());
            }
            if let Some(result_item) = account.storage.get(item.0) {
                log::info!("key: {} result", hex::encode(item.0));
                if result_item.len() > 31 {
                    log::info!("val: {}... result", hex::encode(&result_item[..31]));
                } else {
                    log::info!("val: {} | len: {:?} result", hex::encode(result_item), result_item.len());
                }
            } else {
                log::error!("key: {} not found in result storage", hex::encode(item.0));
            }
        }
        } else {
            log::error!("!! Service account not found in result state: {:?}", service_account.0);
        }
    }*/

    for service_account in expected_state.service_accounts.iter() {
        if let Some(account) = result_state.service_accounts.get(&service_account.0) {
            log::debug!("checking service: {:?}", service_account.0);
            assert_eq!(service_account.1.code_hash, account.code_hash);
            assert_eq!(service_account.1.balance, account.balance);
            assert_eq!(service_account.1.acc_min_gas, account.acc_min_gas);
            assert_eq!(service_account.1.xfer_min_gas, account.xfer_min_gas);
            assert_eq!(service_account.1.gratis_storage_offset, account.gratis_storage_offset);
            assert_eq!(service_account.1.created_at, account.created_at);
            assert_eq!(service_account.1.last_acc, account.last_acc);
            assert_eq!(service_account.1.parent_service, account.parent_service);
            assert_eq!(service_account.1.items, account.items);
            assert_eq!(service_account.1.octets, account.octets);

            for item in service_account.1.storage.iter() {
                if let Some(value) = account.storage.get(item.0) {
                    if item.1 != value {
                        log::debug!("key: {}", hex::encode(&item.0));
                        log::debug!("expected value: {} != result value: {}", hex::encode(item.1), hex::encode(value));
                        assert_eq!(item.1, value);
                    }
                } else {
                    log::error!("Service: {:?}. Key storage not found : {:x?}", *service_account.0, *item.0);
                }
            }
            assert_eq!(service_account.1.storage, account.storage);
        } else {
            log::error!("Service account not found in state: {:?}", service_account.0);
        }
    }
    assert_eq!(expected_state.service_accounts, result_state.service_accounts);
    assert_eq!(expected_state.auth_pools, result_state.auth_pools);
    assert_eq!(expected_state.statistics.curr, result_state.statistics.curr);
    assert_eq!(expected_state.statistics.prev, result_state.statistics.prev);
    assert_eq!(expected_state.statistics.cores, result_state.statistics.cores);
    for expected_id_record in expected_state.statistics.services.records.iter() {
        if let Some(result_record) = result_state.statistics.services.records.get(&expected_id_record.0) {
            assert_eq!(expected_id_record.1, result_record);
        } else {
            panic!("Service {:?} not found in statistics", expected_id_record.0);
        }
    }
}
