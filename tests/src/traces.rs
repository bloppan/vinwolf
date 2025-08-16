#[cfg(test)]
mod tests {
    
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::collections::HashSet;

    use jam_types::{Block, RawState, GlobalState, Header, KeyValue};
    use state_handler::{get_global_state, set_global_state, set_state_root};
    use state_controller::stf;
    use codec::{Decode, DecodeLen, BytesReader};
    use utils::{common::parse_state_keyvals, serialization};
    use utils::trie::merkle_state;

    #[test]
    fn run_traces_tests() {

        use dotenv::dotenv;
        dotenv().ok();
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).init();

        //let base = Path::new("/home/bernar/workspace/jamtestnet/0.6.7/");
        let base = Path::new("/home/bernar/workspace/jam-stuff/fuzz-reports/archive/0.6.7");
        //let base = Path::new("/home/bernar/workspace/vinwolf/tests/jamtestvectors/traces"); fuzz-reports/jamzig/1755185281
        //let base = Path::new("/home/bernar/workspace/jam-stuff/fuzz-reports/boka/0.6.7"); 
        
        let skip: HashSet<String> = ["1754982087"]
            .iter()
            .map(|s| s.to_string())
            .collect();

        let base = Path::new("/home/bernar/workspace/jam-stuff/fuzz-reports/archive/0.6.7/1755083543");
        //process_all_dirs(base, &skip).unwrap();
        process_all_bins(&base).unwrap();
    }

    fn process_all_dirs(base_dir: &Path, skip_dirs: &HashSet<String>) -> std::io::Result<()> {
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
        }

        Ok(())
    }

    fn process_all_bins(dir_path: &Path) -> std::io::Result<()> {
        let mut bin_files: Vec<(u32, PathBuf)> = fs::read_dir(dir_path)?
            .filter_map(|f| {
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

        for (slot, bin_path) in bin_files {
            let test_content = utils::common::read_bin_file(&bin_path)
                .unwrap_or_else(|e| {
                    log::error!("Error reading {:?}: {:?}", bin_path, e);
                    panic!("");
                });

            println!(
                "Processed {:?} (slot {}) with {} bytes",
                bin_path,
                slot,
                test_content.len()
            );
            log::info!("Process test file {:?}", bin_path);
            process_trace_test(&test_content);
            log::info!("{:?} processed successfully", bin_path);
        }

        Ok(())
    }

    fn process_trace_test(test_content: &[u8]) {

        let mut reader = BytesReader::new(&test_content);
        let pre_state = RawState::decode(&mut reader).expect("Error decoding pre state");
        let block = Block::decode(&mut reader).expect("Error decoding the block");
        let post_state = RawState::decode(&mut reader).expect("Error decoding post state");

        let mut state = GlobalState::default();
        let mut expected_state = GlobalState::default();

        log::info!("test len: {:?} reader pos: {:?}", test_content.len(), reader.get_position());
        parse_state_keyvals(&pre_state.keyvals, &mut state).expect("Error decoding pre state keyvals");
        assert_eq!(pre_state.state_root.clone(), merkle_state(&serialization::serialize(&state).map, 0));
        log::info!("**********************");
        parse_state_keyvals(&post_state.keyvals, &mut expected_state).expect("Error decoding post state keyvals");
        assert_eq!(post_state.state_root.clone(), merkle_state(&serialization::serialize(&expected_state).map, 0));

        set_global_state(state.clone());
        set_state_root(pre_state.state_root.clone());
        
        match stf(&block) {
            Ok(_) => { },
            Err(e) => { log::error!("{:?}", e) },
        };

        let result_state = get_global_state().lock().unwrap();
        
        assert_eq_state(&expected_state, &result_state);

        /*log::info!("post_sta state_root: 0x{}", hex::encode(post_state.state_root));
        log::info!("expected state_root: 0x{}", hex::encode(merkle_state(&serialization::serialize(&expected_state).map, 0)));
        log::info!("result   state_root: 0x{}", hex::encode(merkle_state(&serialization::serialize(&result_state).map, 0)));*/
        
        assert_eq!(post_state.state_root, merkle_state(&serialization::serialize(&result_state).map, 0));
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
                        log::error!("Key storage not found : {:x?}", *item.0);
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
        assert_eq!(expected_state.statistics.services, result_state.statistics.services);
    }
}


    /* JamDuna */
    //pub const TEST_DIR: &str = "/home/bernar/workspace/jam-stuff/fuzz-reports/jamduna/fixed/jam-duna-target-v0.5-0.6.7_gp-0.6.7"; /* OK */
    //pub const TEST_DIR: &str = "/home/bernar/workspace/jam-stuff/fuzz-reports/jamduna/fixed/jam-duna-target-v0.7-0.6.7_gp-0.6.7/1754982087";  // ******** accounts not match 5-6 
    //pub const TEST_DIR: &str = "/home/bernar/workspace/jam-stuff/fuzz-reports/jamduna/jam-duna-target-v0.8-0.6.7_gp-0.6.7/fixed/1754982630";  // /* OK */ 
    //pub const TEST_DIR: &str = "/home/bernar/workspace/jam-stuff/fuzz-reports/jamduna/jam-duna-target-v0.8-0.6.7_gp-0.6.7/1755105426"; /* OK */
    /* Jamixir */
    //pub const TEST_DIR: &str = "/home/bernar/workspace/jam-stuff/fuzz-reports/jamixir/fixed/1754983524/traces"; /* OK */
    //pub const TEST_DIR: &str = "/home/bernar/workspace/jam-stuff/fuzz-reports/jamixir/1755106159/traces"; /* OK */
    /* Jamzig */
    //pub const TEST_DIR: &str = "/home/bernar/workspace/jam-stuff/fuzz-reports/jamzig/jamzig-target-0.1.0_gp-0.6.7/fixed/1754988078"; // /* OK */
    //pub const TEST_DIR: &str = "/home/bernar/workspace/jam-stuff/fuzz-reports/jamzig/jamzig-target-0.1.0_gp-0.6.7/1755081941";  // /* OK */
    /* Jamzilla */
    //pub const TEST_DIR: &str = "/home/bernar/workspace/jam-stuff/fuzz-reports/jamzilla/jam-node-0.1.0_gp-0.6.7/fixed/1754984893"; /* OK */
    //pub const TEST_DIR: &str = "/home/bernar/workspace/jam-stuff/fuzz-reports/jamzilla/jam-node-0.1.0_gp-0.6.7/1755082451"; // /* OK */
    /* JavaJam */
    //pub const TEST_DIR: &str = "/home/bernar/workspace/jam-stuff/fuzz-reports/javajam/javajam-0.6.7_gp-0.6.7/1754582958"; // ***** panic (maybe bless hostcall) 3-4
    //pub const TEST_DIR: &str = "/home/bernar/workspace/jam-stuff/fuzz-reports/javajam/javajam-0.6.7_gp-0.6.7/1754725568"; // ****** accounts not match 3-4 
    //pub const TEST_DIR: &str = "/home/bernar/workspace/jam-stuff/fuzz-reports/javajam/javajam-0.6.7_gp-0.6.7/1754754058/traces"; // ****** accounts not match 1-4
    //pub const TEST_DIR: &str = "/home/bernar/workspace/jam-stuff/fuzz-reports/javajam/javajam-0.6.7_gp-0.6.7/1754990132"; // /* OK */
    /* SpaceJam */
    //pub const TEST_DIR: &str = "/home/bernar/workspace/jam-stuff/fuzz-reports/spacejam/1755083543"; // 1-2 /* OK */