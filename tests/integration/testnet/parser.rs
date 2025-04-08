use serde::Deserialize;
use std::collections::HashMap;
use std::convert::TryInto;
use serde_json;
use std::io::Read;
use std::path::PathBuf;
use hex;
use crate::integration::w3f::codec::{TestBody, encode_decode_test};


use vinwolf::types::{
    Account, AccumulatedHistory, AuthPools, AuthQueues, AvailabilityAssignments, BlockHistory, DisputesRecords, EntropyPool, 
    GlobalState, Privileges, ReadyQueue, Safrole, ServiceId, Statistics, TimeSlot, ValidatorsData, Gas
};

use vinwolf::utils::codec::{Decode, BytesReader};

use super::{read_test, TestnetState, TestData, ParsedTransitionFile};

#[derive(Debug, Deserialize)]
pub struct ParsedServiceAccount {
    pub s: ServiceId,
    pub c: Vec<u8>, 
    pub b: u64,
    pub g: Gas,
    pub m: Gas,
    pub l: u64,
    pub i: u32,
    pub clen: u32,
}

pub fn parse_service_account(input: &str) -> ParsedServiceAccount {

    let mut map = HashMap::new();

    for part in input.split(' ') {
        if part.contains("clen") || part.contains("s=") {
            let mut iter = part.split('|');
            if let (Some(key), Some(value)) = (iter.next(), iter.next()) {
                //println!("--- key = {}, value = {}", key, value);
                let mut key_iter = key.split('=');
                if let (Some(key), Some(value)) = (key_iter.next(), key_iter.next()) {
                    //println!("** key = {}", key);
                    //println!("** value = {}", value);
                    map.insert(key.trim(), value.trim());
                }
                let mut iter2 = value.split('=');
                if let (Some(key), Some(value)) = (iter2.next(), iter2.next()) {
                    //println!("** key = {}", key);
                    //println!("** value = {}", value);
                    map.insert(key.trim(), value.trim());
                }
                map.insert(key.trim(), value.trim());
            }
            continue;
        }
        let mut iter = part.split('=');
        if let (Some(key), Some(value)) = (iter.next(), iter.next()) {
            //println!("--- key = {}, value = {}", key, value);
            map.insert(key.trim(), value.trim());
        }
    }

    ParsedServiceAccount {
        s: map.get("s").unwrap_or(&"0").parse::<u32>().unwrap(),
        c: hex::decode(map.get("c").unwrap().trim_start_matches("0x")).unwrap(),
        b: map.get("b").unwrap_or(&"0").parse::<u64>().unwrap(),
        g: map.get("g").unwrap_or(&"0").parse::<Gas>().unwrap(),
        m: map.get("m").unwrap_or(&"0").parse::<Gas>().unwrap(),
        l: map.get("l").unwrap_or(&"0").parse::<u64>().unwrap(),
        i: map.get("i").unwrap_or(&"0").parse::<u32>().unwrap(),
        clen: map.get("clen").unwrap_or(&"0").parse::<u32>().unwrap(),
    }

}

#[derive(Debug, Deserialize)]
pub struct ParsedAccountPreimage {
    pub s: ServiceId,
    pub h: Vec<u8>, 
    pub plen: u32,
}

pub fn parse_account_preimage(input: &str) -> ParsedAccountPreimage {

    let mut map = HashMap::new();

    for part in input.split('|') {
        if part.contains("plen") {
            let mut iter = part.split('=');
            if let (Some(key), Some(value)) = (iter.next(), iter.next()) {
                //println!("--- key = {}, value = {}", key, value);
                map.insert(key.trim(), value.trim());
            }
            continue;
        }
        let mut iter = part.split('=');
        if let (Some(key), Some(value)) = (iter.next(), iter.next()) {
            //println!("---* key = {}, value = {}", key, value);
            map.insert(key.trim(), value.trim());
        }
    }

    ParsedAccountPreimage {
        s: map.get("s").unwrap_or(&"0").parse::<u32>().unwrap(),
        h: hex::decode(map.get("h").unwrap().trim_start_matches("0x")).unwrap(),
        plen: map.get("plen").unwrap_or(&"0").parse::<usize>().unwrap() as u32,
    }

}

#[derive(Debug, Deserialize)]
pub struct ParsedAccountStorage {
    pub s: ServiceId,
    pub h: Vec<u8>,
}

pub fn parse_account_storage(input: &str) -> ParsedAccountStorage {
    let mut map = HashMap::new();
    for part in input.split(' ') {
        if part.contains("s=") {
            let mut service = part.split('|')
                                             .next()
                                             .and_then(|s_part| s_part.strip_prefix("s="))
                                             .unwrap_or("0");
            map.insert("s", service);
            continue;
        }
        if part.contains("k=") {
            let mut iter = part.split('=');
            if let (Some(key), Some(value)) = (iter.next(), iter.next()) {
                map.insert(key.trim(), value.trim());
            }
        }
    }

    ParsedAccountStorage {
        s: map.get("s").unwrap_or(&"0").parse::<u32>().unwrap(),
        h: hex::decode(map.get("k").unwrap().trim_start_matches("0x")).unwrap(),
    }
}

#[derive(Debug, Deserialize)]
pub struct ParsedAccountLookup {
    pub s: ServiceId,
    pub h: Vec<u8>, 
    pub l: u32,
    pub t: Vec<TimeSlot>, 
    pub tlen: u8,
}

pub fn parse_account_lookup(input: &str) -> ParsedAccountLookup {

    let mut map = HashMap::new();

    for part in input.split(' ') {
        if part.contains("tlen") {
            let mut iter = part.split('=');
            if let (Some(key), Some(value)) = (iter.next(), iter.next()) {
                //println!("--- key = {}, value = {}", key, value);
                map.insert(key.trim(), value.trim());
            }
            continue;
        }
        let mut iter = part.split('|');
        if let (Some(key), Some(value)) = (iter.next(), iter.next()) {
            //println!("--- key = {}, value = {}", key, value);
            let mut key_iter = key.split('=');
            if let (Some(key), Some(value)) = (key_iter.next(), key_iter.next()) {
                //println!("** value = {}", value);
                map.insert(key.trim(), value.trim());
            }
            let mut iter2 = value.split('=');
            if let (Some(key), Some(value)) = (iter2.next(), iter2.next()) {
                //println!("** value = {}", value);
                map.insert(key.trim(), value.trim());
            }
            map.insert(key.trim(), value.trim());
        }
        //println!("part: {}", part);
    }

    let tlen_value = map.get("tlen").unwrap_or(&"0").parse::<usize>().unwrap();

    let t_string = map.get("t").unwrap_or(&"[]").trim(); 
    let t_cleaned = t_string.trim_start_matches('[').trim_end_matches(']'); 

    let t_all: Vec<u32> = t_cleaned
        .split(',')
        .filter_map(|n| n.trim().parse::<u32>().ok())
        .collect();

    let t = t_all.into_iter().take(tlen_value).collect();

    ParsedAccountLookup {
        s: map.get("s").unwrap_or(&"0").parse::<u32>().unwrap(),
        h: hex::decode(map.get("h").unwrap().trim_start_matches("0x")).unwrap(),
        l: map.get("l").unwrap_or(&"0").parse::<u32>().unwrap(),
        t, 
        tlen: tlen_value as u8,
    }

}

pub fn deserialize_state_transition_file(dir: &str, filename: &str) -> Result<ParsedTransitionFile, Box<dyn std::error::Error>> {
    
    //let filename = format!("tests/jamtestnet/data/{}/state_transitions/{}", dir, filename);
    //let filename = format!("tests/javajam-trace/stf/state_transitions/{}", filename);
    let filename = format!("{}/{}", dir, filename);
    //println!("filename: {}", filename);
    //let state_content = read_test(&format!("tests/jamtestnet/data/fallback/state_transitions/{}", filename));
    let mut file = std::fs::File::open(&filename).expect("Failed to open JSON file");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Failed to read JSON file");
    let testcase: TestData = serde_json::from_str(&contents).expect("Failed to deserialize JSON");
    let pre_state_root = hex::decode(&testcase.pre_state.state_root[2..])?.try_into().unwrap();
    let post_state_root = hex::decode(&testcase.post_state.state_root[2..])?.try_into().unwrap();
    let pre_state = read_state_transition(&testcase.pre_state)?;
    let post_state = read_state_transition(&testcase.post_state)?;
    /*let pre_state = get_serialized_state(&testcase.pre_state)?;
    let post_state = get_serialized_state(&testcase.post_state)?;*/
    Ok(ParsedTransitionFile {
        pre_state_root,
        post_state_root,
        pre_state,
        post_state,
    })
}

fn read_state_transition(testcase_state: &TestnetState) -> Result<GlobalState, Box<dyn std::error::Error>> {

    let mut global_state = GlobalState::default();

    for keyval in testcase_state.keyvals.iter() {
        //let key = hex::decode(&keyval.0[2..])?;
        let value = hex::decode(&keyval.1[2..])?;

        if keyval.2.len() >= 2 && keyval.2.len() <= 3 {
            let key_type = &keyval.2[1..];
            match key_type {
                "1" => {
                    global_state.auth_pools = AuthPools::decode(&mut BytesReader::new(&value)).expect("Error decoding AuthPools");
                    "AuthPools"
                },
                "2" => {
                    global_state.auth_queues = AuthQueues::decode(&mut BytesReader::new(&value)).expect("Error decoding AuthQueues");
                    "AuthQueues"
                },
                "3" => {
                    global_state.recent_history = BlockHistory::decode(&mut BytesReader::new(&value)).expect("Error decoding BlockHistory");
                    "BlockHistory"
                }
                "4" => {
                    global_state.safrole = Safrole::decode(&mut BytesReader::new(&value)).expect("Error decoding Safrole");
                    "Safrole"
                }
                "5" => {
                    global_state.disputes = DisputesRecords::decode(&mut BytesReader::new(&value)).expect("Error decoding DisputesRecords");
                    "DisputesRecords"
                }
                "6" => {
                    global_state.entropy = EntropyPool::decode(&mut BytesReader::new(&value)).expect("Error decoding EntropyPool");
                    "EntropyPool"
                }
                "7" => {
                    global_state.next_validators = ValidatorsData::decode(&mut BytesReader::new(&value)).expect("Error decoding ValidatorsData");
                    "ValidatorsData"
                }
                "8" => {
                    global_state.curr_validators = ValidatorsData::decode(&mut BytesReader::new(&value)).expect("Error decoding ValidatorsData");
                    "ValidatorsData"
                }
                "9" => {
                    global_state.prev_validators = ValidatorsData::decode(&mut BytesReader::new(&value)).expect("Error decoding ValidatorsData");
                    "ValidatorsData"
                }
                "10" => {
                    global_state.availability = AvailabilityAssignments::decode(&mut BytesReader::new(&value)).expect("Error decoding AvailabilityAssignments");
                    "AvailabilityAssignments"
                }
                "11" => {
                    global_state.time = TimeSlot::decode(&mut BytesReader::new(&value)).expect("Error decoding TimeSlot");
                    "TimeSlot"
                }
                "12" => {
                    global_state.privileges = Privileges::decode(&mut BytesReader::new(&value)).expect("Error decoding Privileges");
                    "Privileges"
                }
                "13" => {
                    global_state.statistics = Statistics::decode(&mut BytesReader::new(&value)).expect("Error decoding Statistics");
                    "Statistics"
                }
                "14" => {
                    global_state.ready_queue = ReadyQueue::decode(&mut BytesReader::new(&value)).expect("Error decoding ReadyQueue");
                    "ReadyQueue"
                }
                "15" => {
                    global_state.accumulation_history = AccumulatedHistory::decode(&mut BytesReader::new(&value)).expect("Error decoding AccumulatedHistory");
                    "AccumulatedHistory"
                }
                _ => {
                    println!("Unknown key type: {}", key_type);
                    "Unknown"
                }
            };
            //println!("key_type: {}", key_type);
        } else if keyval.2 == "account_lookup" {
            let parsed_account_lookup = parse_account_lookup(&keyval.3);
            let service = parsed_account_lookup.s;
            if global_state.service_accounts.service_accounts.get(&parsed_account_lookup.s).is_none() {
                let account = Account::default();
                global_state.service_accounts.service_accounts.insert(service, account);
            }
            let mut hash = [0u8; 32];
            hash.copy_from_slice(&parsed_account_lookup.h);
            global_state.service_accounts.service_accounts.get_mut(&service).unwrap().lookup.insert((hash, parsed_account_lookup.l), parsed_account_lookup.t.clone());

            //println!("key_type: Account lookup: {:?}", parsed_account_lookup);
        } else if keyval.2 == "service_account" {
            let parsed_service_account = parse_service_account(&keyval.3);
            let service = parsed_service_account.s;
            if global_state.service_accounts.service_accounts.get(&parsed_service_account.s).is_none() {
                let account = Account::default();
                global_state.service_accounts.service_accounts.insert(service, account);
            }
            
            let service = global_state.service_accounts.service_accounts.get_mut(&service).unwrap();
            service.balance = parsed_service_account.b;
            let mut code_hash = [0u8; 32];
            code_hash.copy_from_slice(&parsed_service_account.c);
            service.code_hash = code_hash;
            service.gas = parsed_service_account.g;
            service.min_gas = parsed_service_account.m;
            service.bytes = parsed_service_account.l;
            service.items = parsed_service_account.i;

            //println!("key_type: Service account: {:?}", parsed_service_account);
        } else if keyval.2 == "account_preimage" {
            let parsed_account_preimage = parse_account_preimage(&keyval.3);
            let service = parsed_account_preimage.s;
            if global_state.service_accounts.service_accounts.get(&parsed_account_preimage.s).is_none() {
                let account = Account::default();
                global_state.service_accounts.service_accounts.insert(service, account);
            }
            let mut hash = [0u8; 32];
            hash.copy_from_slice(&parsed_account_preimage.h);
            let blob = hex::decode(&keyval.1[2..]).unwrap();
            global_state.service_accounts.service_accounts.get_mut(&service).unwrap().preimages.insert(hash, blob);

           // println!("key_type: Account preimage: {:?}", parsed_account_preimage);
        } else if keyval.2 == "account_storage" {
            let parsed_account_storage = parse_account_storage(&keyval.3);
            let service = parsed_account_storage.s;
            if global_state.service_accounts.service_accounts.get(&parsed_account_storage.s).is_none() {
                let account = Account::default();
                global_state.service_accounts.service_accounts.insert(service, account);
            }
            let mut hash = [0u8; 32];
            hash.copy_from_slice(&parsed_account_storage.h);
            let blob = hex::decode(&keyval.1[2..]).unwrap();
            global_state.service_accounts.service_accounts.get_mut(&service).unwrap().storage.insert(hash, blob);
        } else {
            println!("Unknown key type: {}", keyval.2);
        }
    }

    Ok(global_state)
}

/*pub fn create_dictionary(state: &GlobalState) -> Vec<(Vec<u8>, Vec<u8>)> {

    //let mut serial_state: Vec<([u8; 32], Vec<u8>)> = Vec::new();
    let mut serial_state: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();

    let c1 = "0x0100000000000000000000000000000000000000000000000000000000000000";
    let c1_bytes = hex::decode(&c1[2..]).expect("Hex decode failed");
    //let c1_array: [u8; 32] = c1_bytes.try_into().expect("Expected 32-byte array");
    serial_state.push((c1_bytes, state.auth_pools.encode()));

    let c2 = "0x0200000000000000000000000000000000000000000000000000000000000000";
    let c2_bytes = hex::decode(&c2[2..]).expect("Hex decode failed");
    //let c2_array: [u8; 32] = c2_bytes.try_into().expect("Expected 32-byte array");
    serial_state.push((c2_bytes, state.auth_queues.encode()));

    let c3 = "0x0300000000000000000000000000000000000000000000000000000000000000";
    let c3_bytes = hex::decode(&c3[2..]).expect("Hex decode failed");
    //let c3_array: [u8; 32] = c3_bytes.try_into().expect("Expected 32-byte array");
    serial_state.push((c3_bytes, state.recent_history.encode()));

    let c4 = "0x0400000000000000000000000000000000000000000000000000000000000000";
    let c4_bytes = hex::decode(&c4[2..]).expect("Hex decode failed");
    //let c4_array: [u8; 32] = c4_bytes.try_into().expect("Expected 32-byte array");
    serial_state.push((c4_bytes, state.safrole.encode()));

    let c5 = "0x0500000000000000000000000000000000000000000000000000000000000000";
    let c5_bytes = hex::decode(&c5[2..]).expect("Hex decode failed");
    //let c5_array: [u8; 32] = c5_bytes.try_into().expect("Expected 32-byte array");
    serial_state.push((c5_bytes, state.disputes.encode()));

    let c6 = "0x0600000000000000000000000000000000000000000000000000000000000000";
    let c6_bytes = hex::decode(&c6[2..]).expect("Hex decode failed");
    //let c6_array: [u8; 32] = c6_bytes.try_into().expect("Expected 32-byte array");
    serial_state.push((c6_bytes, state.entropy.encode()));

    let c7 = "0x0700000000000000000000000000000000000000000000000000000000000000";
    let c7_bytes = hex::decode(&c7[2..]).expect("Hex decode failed");
    //let c7_array: [u8; 32] = c7_bytes.try_into().expect("Expected 32-byte array");
    serial_state.push((c7_bytes, state.next_validators.encode()));

    let c8 = "0x0800000000000000000000000000000000000000000000000000000000000000";
    let c8_bytes = hex::decode(&c8[2..]).expect("Hex decode failed");
    //let c8_array: [u8; 32] = c8_bytes.try_into().expect("Expected 32-byte array");
    serial_state.push((c8_bytes, state.curr_validators.encode()));

    let c9 = "0x0900000000000000000000000000000000000000000000000000000000000000";
    let c9_bytes = hex::decode(&c9[2..]).expect("Hex decode failed");
    //let c9_array: [u8; 32] = c9_bytes.try_into().expect("Expected 32-byte array");
    serial_state.push((c9_bytes, state.prev_validators.encode()));

    let c10 = "0x0a00000000000000000000000000000000000000000000000000000000000000";
    let c10_bytes = hex::decode(&c10[2..]).expect("Hex decode failed");
    //let c10_array: [u8; 32] = c10_bytes.try_into().expect("Expected 32-byte array");
    serial_state.push((c10_bytes, state.availability.encode()));

    let c11 = "0x0b00000000000000000000000000000000000000000000000000000000000000";
    let c11_bytes = hex::decode(&c11[2..]).expect("Hex decode failed");
    //let c11_array: [u8; 32] = c11_bytes.try_into().expect("Expected 32-byte array");
    serial_state.push((c11_bytes, state.time.encode()));

    let c12 = "0x0c00000000000000000000000000000000000000000000000000000000000000";
    let c12_bytes = hex::decode(&c12[2..]).expect("Hex decode failed");
    //let c12_array: [u8; 32] = c12_bytes.try_into().expect("Expected 32-byte array");
    serial_state.push((c12_bytes, state.privileges.encode()));

    let c13 = "0x0d00000000000000000000000000000000000000000000000000000000000000";
    let c13_bytes = hex::decode(&c13[2..]).expect("Hex decode failed");
    //let c13_array: [u8; 32] = c13_bytes.try_into().expect("Expected 32-byte array");
    serial_state.push((c13_bytes, state.statistics.encode()));

    let c14 = "0x0e00000000000000000000000000000000000000000000000000000000000000";
    let c14_bytes = hex::decode(&c14[2..]).expect("Hex decode failed");
   // let c14_array: [u8; 32] = c14_bytes.try_into().expect("Expected 32-byte array");
    serial_state.push((c14_bytes, state.accumulation_history.encode()));

    let c15 = "0x0f00000000000000000000000000000000000000000000000000000000000000";
    let c15_bytes = hex::decode(&c15[2..]).expect("Hex decode failed");
    //let c15_array: [u8; 32] = c15_bytes.try_into().expect("Expected 32-byte array");
    serial_state.push((c15_bytes, state.ready_queue.encode()));

    let account_lookup = "0x00190000000000004948b8c6f5a11274b52216c70df4731eb79bbf85be9073e8";
    let account_lookup_bytes = hex::decode(&account_lookup[2..]).expect("Hex decode failed");
    //let account_lookup_array: [u8; 32] = account_lookup_bytes.try_into().expect("Expected 32-byte array");
    let account_lookup = "0x0100000000";
    let account_lookup_bytes2 = hex::decode(&account_lookup[2..]).expect("Hex decode failed");
    serial_state.push((account_lookup_bytes, account_lookup_bytes2));

    let account_lookup = "0x00ca000300000000f1711dcda721521258f3d67933e79f50133a8bc2bd664740";
    let account_lookup_bytes = hex::decode(&account_lookup[2..]).expect("Hex decode failed");
    //let account_lookup_array2: [u8; 32] = account_lookup_bytes.try_into().expect("Expected 32-byte array");
    let account_lookup = "0x0100000000";
    let account_lookup_bytes2 = hex::decode(&account_lookup[2..]).expect("Hex decode failed");
    serial_state.push((account_lookup_bytes, account_lookup_bytes2));

    let account_preimage = "0x00fe00ff00ff00ff30f2c101674af1da31769e96ce72e81a4a44c89526d7d3ff";
    let account_preimage_bytes = hex::decode(&account_preimage[2..]).expect("Hex decode failed");
    //let account_preimage_array: [u8; 32] = account_preimage_bytes.try_into().expect("Expected 32-byte array");
    let account_preimage = "0x00000000000000000020000a00000000000628023307320015";
    let account_preimage_bytes2 = hex::decode(&account_preimage[2..]).expect("Hex decode failed");
    serial_state.push((account_preimage_bytes, account_preimage_bytes2));

    let account_preimage = "0x00fe00ff00ff00ff87fb6de829abf2bb25a15b82618432c94e82848d9dd204f5";
    let account_preimage_bytes = hex::decode(&account_preimage[2..]).expect("Hex decode failed");
    //let account_preimage_array2: [u8; 32] = account_preimage_bytes.try_into().expect("Expected 32-byte array");
    let account_preimage = "0x0000000000000200002000bb030000040283464001e2017d02b00228ab00000028ae00000028e60251089b0064797c77510791005127ff0090006c7a570a09330a330828735527c0000d330a01330b80284a5527e0000e330a02330b40ff283c5527f0000e330a03330b20ff282e5527f8000e330a04330b10ff28205527fc000e330a05330b08ff2812887afe00330b04ff93ab02ff85aa0701ae8a2b3308c8b70764ab01c8b90c7ccc97880895bbffd4c808520bf28aa903cf9707c88707320032000000003308249577043200951130ff7b10c8007b15c0007b16b80064859555f8510523029577087d783306015a085d848aff0083a8ff8488ff003306025328bf004c84a8e0003306035128c0004084a8f0003306045128e0003484a8f8003306055128f0002884a8fc003306065128f8001c84a8fe003306075128fc001088a8fe00858601976603017b15ac65b90164756468501002d2fe510728821aaa6aa801c856077c78957b018567ffc87a0a5108183305330695a8c05208a20028180133053306281101510a7d7db73305015a075a8477ff008378ff848cff00330502532cbf0049847ce000330503512cc0003d847cf000330504512ce00031847cf800330505512cf00025847cfc00330506512cf80019847cfe00330507512cfc000d3305085327fe00207b1aac5a1b0164b7645864b650100430fe6458646b6476821a28073308330601c88b05c8650bc88607c97a0a95a8c051087d95b7407d7a3309015a0a6b84aaff0083a9ff849bff00330cbf00330902accb5384abe000330cc000330903aacb4584abf000330ce000330904aacb3784abf800330cf000330905aacb2984abfc00330cf800330906aacb1b84abfe00330cfc00330907aacb0d330bfe00330908acba0dac987c649850100695fdc856068068fc330964330a6464570a0964757b1708481114951714330804951908330a040a0395171833098000330850100848330820a107330964951a1864570a0b8217084921b0004921a8004921a0007b179800330820951798008210c8008215c0008216b8009511d00032000000000000330732008d7a84aa07c8a70b510a0e647c0178c895cc01acbcfbc9a903843cf8c8cb0a580c1d8482ff0014090101010101010101ca920c017bbc95bb08acabfb843907520905280ec8a9090178a895aa01ac9afb320021242a825285a49028240a8942a288c894a62449524f8a8828919248244442244442244442248b0a2952925422959444222112222112222112128a2a545593244949242289482292882422894822492149aa24494a1421a94444242222fa5592962449049025495912";
    let account_preimage_bytes2 = hex::decode(&account_preimage[2..]).expect("Hex decode failed");
    serial_state.push((account_preimage_bytes, account_preimage_bytes2));
    
    let account_service = "0xff00000000000000000000000000000000000000000000000000000000000000";
    let account_service_bytes = hex::decode(&account_service[2..]).expect("Hex decode failed");
    //let account_service_array: [u8; 32] = account_service_bytes.try_into().expect("Expected 32-byte array");
    let account_service = "0xbd87fb6de829abf2bb25a15b82618432c94e82848d9dd204f5d775d4b880ae0d00e40b540200000064000000000000006400000000000000340400000000000004000000";
    let account_service_bytes2 = hex::decode(&account_service[2..]).expect("Hex decode failed");
    serial_state.push((account_service_bytes, account_service_bytes2));


    let result = merkle(&serial_state, 0).unwrap();
    println!("Merkle root: {:x?}", result);

    return serial_state;
}*/

/*pub fn get_serialized_state(testcase_state: &TestnetState) -> Result<SerializedState, Box<dyn std::error::Error>> {

    let mut serialized_state = SerializedState::default();

    for keyval in testcase_state.keyvals.iter() {
        let key = hex::decode(&keyval.0[2..])?;
        let value = hex::decode(&keyval.1[2..])?;

        if keyval.2.len() >= 2 && keyval.2.len() <= 3 {
            let key_type = &keyval.2[1..];
            let key_type = match key_type {
                "1" => {
                    let auth_pools = AuthPools::decode(&mut BytesReader::new(&value)).expect("Error decoding AuthPools");
                    serialized_state.map.insert(StateKey::U8(AUTH_POOLS).construct(), auth_pools.encode());
                    "AuthPools"
                },
                "2" => {
                    let auth_queues = AuthQueues::decode(&mut BytesReader::new(&value)).expect("Error decoding AuthQueues");
                    serialized_state.map.insert(StateKey::U8(AUTH_QUEUE).construct(), auth_queues.encode());
                    "AuthQueues"
                },
                "3" => {
                    let recent_history = BlockHistory::decode(&mut BytesReader::new(&value)).expect("Error decoding BlockHistory");
                    serialized_state.map.insert(StateKey::U8(RECENT_HISTORY).construct(), recent_history.encode());
                    "BlockHistory"
                }
                "4" => {
                    let safrole = Safrole::decode(&mut BytesReader::new(&value)).expect("Error decoding Safrole");
                    serialized_state.map.insert(StateKey::U8(SAFROLE).construct(), safrole.encode());
                    "Safrole"
                }
                "5" => {
                    let disputes = DisputesRecords::decode(&mut BytesReader::new(&value)).expect("Error decoding DisputesRecords");
                    serialized_state.map.insert(StateKey::U8(DISPUTES).construct(), disputes.encode());
                    "DisputesRecords"
                }
                "6" => {
                    let entropy = EntropyPool::decode(&mut BytesReader::new(&value)).expect("Error decoding EntropyPool");
                    serialized_state.map.insert(StateKey::U8(ENTROPY).construct(), entropy.encode());
                    "EntropyPool"
                }
                "7" => {
                    let next_validators = ValidatorsData::decode(&mut BytesReader::new(&value)).expect("Error decoding ValidatorsData");
                    serialized_state.map.insert(StateKey::U8(NEXT_VALIDATORS).construct(), next_validators.encode());
                    "ValidatorsData"
                }
                "8" => {
                    let curr_validators = ValidatorsData::decode(&mut BytesReader::new(&value)).expect("Error decoding ValidatorsData");
                    serialized_state.map.insert(StateKey::U8(CURR_VALIDATORS).construct(), curr_validators.encode());
                    "ValidatorsData"
                }
                "9" => {
                    let prev_validators = ValidatorsData::decode(&mut BytesReader::new(&value)).expect("Error decoding ValidatorsData");
                    serialized_state.map.insert(StateKey::U8(PREV_VALIDATORS).construct(), prev_validators.encode());
                    "ValidatorsData"
                }
                "10" => {
                    let availability = AvailabilityAssignments::decode(&mut BytesReader::new(&value)).expect("Error decoding AvailabilityAssignments");
                    serialized_state.map.insert(StateKey::U8(AVAILABILITY).construct(), availability.encode());
                    "AvailabilityAssignments"
                }
                "11" => {
                    let time = TimeSlot::decode(&mut BytesReader::new(&value)).expect("Error decoding TimeSlot");
                    serialized_state.map.insert(StateKey::U8(TIME).construct(), time.encode());
                    "TimeSlot"
                }
                "12" => {
                    let privileges = Privileges::decode(&mut BytesReader::new(&value)).expect("Error decoding Privileges");
                    serialized_state.map.insert(StateKey::U8(PRIVILEGES).construct(), privileges.encode());
                    "Privileges"
                }
                "13" => {
                    let statistics = Statistics::decode(&mut BytesReader::new(&value)).expect("Error decoding Statistics");
                    serialized_state.map.insert(StateKey::U8(STATISTICS).construct(), statistics.encode());
                    "Statistics"
                }
                "14" => {
                    let accumulation_history = AccumulatedHistory::decode(&mut BytesReader::new(&value)).expect("Error decoding AccumulatedHistory");
                    serialized_state.map.insert(StateKey::U8(ACCUMULATION_HISTORY).construct(), accumulation_history.encode());
                    "AccumulatedHistory"
                }
                "15" => {
                    let ready_queue = ReadyQueue::decode(&mut BytesReader::new(&value)).expect("Error decoding ReadyQueue");
                    serialized_state.map.insert(StateKey::U8(READY_QUEUE).construct(), ready_queue.encode());
                    "ReadyQueue"
                }
                _ => "Unknown",
            };
        } else if keyval.2 == "account_lookup" {
            let parsed_account_lookup = parse_account_lookup(&keyval.3);
            let mut hash = [0u8; 32];
            hash.copy_from_slice(&parsed_account_lookup.h);
            
            let key = StateKey::Account(parsed_account_lookup.s, construct_lookup_key(&hash, parsed_account_lookup.l).to_vec()).construct();
            let value = parsed_account_lookup.t.as_slice().encode_len();
            serialized_state.map.insert(key, value);

        } else if keyval.2 == "service_account" {
            let parsed_service_account = parse_service_account(&keyval.3);
            
            let key = StateKey::Service(255, parsed_service_account.s).construct();
            if serialized_state.map.get(&key).is_none() {
                serialized_state.map.insert(key, vec![0]);
            }
            let service_info = ServiceInfo {
                balance: parsed_service_account.b,
                code_hash: parsed_service_account.c.try_into().unwrap(),
                min_item_gas: parsed_service_account.g,
                min_memo_gas: parsed_service_account.m,
                bytes: parsed_service_account.l,
                items: parsed_service_account.i,
            };
            serialized_state.map.insert(key, service_info.encode());

        } else if keyval.2 == "account_preimage" {
            let parsed_account_preimage = parse_account_preimage(&keyval.3);            
            let mut hash = [0u8; 32];
            hash.copy_from_slice(&parsed_account_preimage.h);

            let key = StateKey::Account(parsed_account_preimage.s, construct_preimage_key(&hash).to_vec()).construct();
            let value: Vec<u8> = hex::decode(&keyval.1[2..]).unwrap();
            serialized_state.map.insert(key, value);

        } else {
            println!("Unknown key type");
        }
    }

    //println!("serialized_state: {:x?}", serialized_state.map);

    Ok(serialized_state)
}*/


pub fn read_state_snapshot(filename: &str) -> GlobalState {

    let body_state: Vec<TestBody> = vec![TestBody::AuthPools,
                                        TestBody::AuthQueues,
                                        TestBody::BlockHistory,
                                        TestBody::Safrole,
                                        TestBody::DisputesRecords,
                                        TestBody::EntropyPool,
                                        TestBody::ValidatorsData,
                                        TestBody::ValidatorsData,
                                        TestBody::ValidatorsData,
                                        TestBody::AvailabilityAssignments,
                                        TestBody::TimeSlot,
                                        TestBody::Privileges,
                                        TestBody::Statistics,
                                        TestBody::AccumulatedHistory,
                                        TestBody::ReadyQueue,
                                        TestBody::ServiceAccounts];
                                        
    let state_content = read_test(filename);

    let state_content = state_content.unwrap();
    let _ = encode_decode_test(&state_content.clone(), &body_state);
    let mut state_reader = BytesReader::new(&state_content);
    //let state = GlobalStateTest::decode(&mut state_reader).expect("Error decoding GlobalStateTest");

    let mut state = GlobalState::default();

    state.auth_pools = AuthPools::decode(&mut state_reader).expect("Error decoding AuthPools");
    state.auth_queues = AuthQueues::decode(&mut state_reader).expect("Error decoding AuthQueues");
    state.recent_history = BlockHistory::decode(&mut state_reader).expect("Error decoding BlockHistory");
    state.safrole = Safrole::decode(&mut state_reader).expect("Error decoding Safrole");
    state.disputes = DisputesRecords::decode(&mut state_reader).expect("Error decoding DisputesRecords");
    state.entropy = EntropyPool::decode(&mut state_reader).expect("Error decoding EntropyPool");
    state.next_validators = ValidatorsData::decode(&mut state_reader).expect("Error decoding ValidatorsData");
    state.curr_validators = ValidatorsData::decode(&mut state_reader).expect("Error decoding ValidatorsData");
    state.prev_validators = ValidatorsData::decode(&mut state_reader).expect("Error decoding ValidatorsData");
    state.availability = AvailabilityAssignments::decode(&mut state_reader).expect("Error decoding AvailabilityAssignments");
    state.time = TimeSlot::decode(&mut state_reader).expect("Error decoding TimeSlot");
    state.privileges = Privileges::decode(&mut state_reader).expect("Error decoding Privileges");
    state.statistics = Statistics::decode(&mut state_reader).expect("Error decoding Statistics");
    state.accumulation_history = AccumulatedHistory::decode(&mut state_reader).expect("Error decoding AccumulatedHistory");
    state.ready_queue = ReadyQueue::decode(&mut state_reader).expect("Error decoding ReadyQueue");
    //let service_accounts = ServiceAccounts::decode(&mut state_reader).expect("Error decoding ServiceAccounts");

    return state;
}