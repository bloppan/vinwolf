use std::fs::File;
use std::io::Read;
use crate::constants::{*};
use crate::jam_types::{
    RawState, Block, AuthPools, AuthQueues, BlockHistory, Safrole, DisputesRecords, EntropyPool, ValidatorsData, AvailabilityAssignments,
    Privileges, Statistics, ReadyQueue, AccumulatedHistory, OpaqueHash, Gas, ServiceId, Account, KeyValue, Header
};
use crate::blockchain::state::{get_global_state, get_state_root, state_transition_function};
use crate::utils::codec::{ReadError, Encode, EncodeLen, Decode, DecodeLen, BytesReader};
use crate::utils::codec::generic::{decode, encode_unsigned};
use crate::utils::trie::merkle_state;
use crate::{blockchain::state::set_global_state, jam_types::{GlobalState, TimeSlot}};

use bitvec::vec;
use tokio::net::UnixListener;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt}; 
use std::fs;
use std::path::{Path, PathBuf};

pub struct State {
    pub header: Header,
    pub state: Vec<KeyValue>,
}

struct Version {
    major: u8,
    minor: u8,
    patch: u8,
}

impl Encode for Version {

    fn encode(&self) -> Vec<u8> {

        let mut blob = vec![];

        self.major.encode_to(&mut blob);
        self.minor.encode_to(&mut blob);
        self.patch.encode_to(&mut blob);

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for Version {
    
    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(Version { 
            major: u8::decode(reader)?, 
            minor: u8::decode(reader)?,
            patch: u8::decode(reader)?,  
        })
    }
}

struct PeerInfo {
    name: String,
    app_version: Version,
    jam_version: Version,
}

impl Encode for PeerInfo {

    fn encode(&self) -> Vec<u8> {
        
        let mut blob = vec![];

        self.name.clone().into_bytes().encode_to(&mut blob);
        self.app_version.encode_to(&mut blob);
        self.jam_version.encode_to(&mut blob);

        return blob;
    }

    fn encode_to(&self, writer: &mut Vec<u8>) {
        writer.extend_from_slice(&self.encode());
    }
}

impl Decode for PeerInfo {

    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        
        Ok(PeerInfo { 
            name: {
                let name_len = reader.read_byte()?;
                let name = reader.read_bytes(name_len as usize)?;
                String::from_utf8_lossy(&name).to_string()
            }, 
            app_version: Version::decode(reader)?, 
            jam_version: Version::decode(reader)?,
        })
    }
}

struct SetState {
    header: Header,
    state: Vec<KeyValue>,
}

impl Encode for SetState {

    fn encode(&self) -> Vec<u8> {
        
        let mut blob = vec![];

        self.header.encode_to(&mut blob);
        self.state.encode_len().encode_to(&mut blob);

        return blob;
    }

    fn encode_to(&self, writer: &mut Vec<u8>) {
        writer.extend_from_slice(&self.encode());
    }
}

impl Decode for SetState {

    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        
        Ok(SetState { 
            header: Header::decode(reader)?,
            state: Vec::<KeyValue>::decode_len(reader)?, 
        })
    }
}

#[derive(Debug)]
enum Message {
    PeerInfo = 0,
    ImportBlock = 1,
    SetState = 2,
    GetState = 3,
    State = 4,
    StateRoot = 5,
}

impl From<u8> for Message {
    fn from(value: u8) -> Self {
        match value {
            0 => Message::PeerInfo,
            1 => Message::ImportBlock,
            2 => Message::SetState,
            3 => Message::GetState,
            4 => Message::State,
            5 => Message::StateRoot,
            _ => panic!("Unknown message type"),
        }
    }
}

pub async fn run_unix_server(socket_path: &str) -> Result<(), io::Error>  {
    
    let vinwolf_info = PeerInfo {
        name: "vinwolf-target".to_string(),
        app_version: Version {
            major: 0,
            minor: 1,
            patch: 0,
        },
        jam_version: Version {
            major: 0,
            minor: 6,
            patch: 6,
        },
    };
    // Intentar enlazar el servidor al socket Unix
    let listener = UnixListener::bind(socket_path)?;

    println!("Server listening on {}", socket_path);

    loop {
        match listener.accept().await {

            Ok((mut socket, _)) => {

                log::info!("New incomming connection accepted...");

                loop {
                    let mut buffer = vec![0; 1024000];
                    match socket.read(&mut buffer).await {
                        
                        Ok(0) => {
                            // Si el cliente cierra la conexión
                            //println!("no bytes received");
                            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                            break;
                        }
                        Ok(n) => {
                            // Mostrar lo que se recibe del cliente
                            let mut reader = BytesReader::new(&buffer);

                            let msg_len = u32::decode(&mut reader).unwrap();
                            let msg_type: Message = reader.read_byte().unwrap().into();
                            log::info!("New message. Total bytes received: {:?} bytes. Message length: {:?} bytes", n, msg_len);
                            match msg_type {
                                Message::PeerInfo => { 
                                    let peer_info = PeerInfo::decode(&mut reader).unwrap();
                                    log::info!(
                                        "Fuzzer info: {:?} version: {}.{}.{} protocol version: {}.{}.{}",
                                        peer_info.name,
                                        peer_info.app_version.major, peer_info.app_version.minor, peer_info.app_version.patch,
                                        peer_info.jam_version.major, peer_info.jam_version.minor, peer_info.jam_version.patch
                                    );
                                    
                                    /*let msg = [(
                                                        vinwolf_info.encode().len() as u32).encode(),
                                                        encode_unsigned(vinwolf_info.name.len()),
                                                        vinwolf_info.encode()].concat();*/

                                    //socket.write(&msg).await?;
                                    socket.write_all(&buffer[..n]).await?;
                                    //println!("msg: {:x?}", msg);
                                    log::info!("Target info: {:?} version: {}.{}.{} protocol version: {}.{}.{}", 
                                                vinwolf_info.name,
                                                vinwolf_info.app_version.major, vinwolf_info.app_version.minor, vinwolf_info.app_version.patch,
                                                vinwolf_info.jam_version.major, vinwolf_info.jam_version.minor, vinwolf_info.jam_version.patch, 
                                    );
                                    log::info!("(Actually the target info that I'm sending is 'jamzig-fuzzer version: 0.1.0 protocol version: 0.6.6', in order to have the fuzzer accepting the handshake)");
                                },
                                Message::SetState => {
                                    log::info!("SetState frame received");
                                    let header = Header::decode(&mut reader).unwrap();
                                    //log::info!("SET STATE - Header: {:?}", header);
                                    let serialized_state = Vec::<KeyValue>::decode_len(&mut reader).unwrap();
                                    log::debug!("{:?} key-values received", serialized_state.len());
                                    let mut global_state = GlobalState::default();
                                    set_state(serialized_state, &mut global_state);
                                    set_global_state(global_state.clone());
                                    let state_root = merkle_state(&global_state.serialize().map, 0).unwrap();
                                    crate::blockchain::state::set_state_root(state_root.clone());
                                    socket.write_all(&construct_fuzz_msg(Message::StateRoot, &state_root)).await?;
                                    log::info!("SetState - send to fuzzer the state root setted: 0x{}", hex::encode(state_root));
                                },
                                Message::ImportBlock => {
                                    log::info!("ImportBlock frame received");
                                    let block = Block::decode(&mut reader).unwrap();
                                    let header_hash = sp_core::blake2_256(&(block.header.encode()));
                                    log::info!("Header hash: 0x{}", hex::encode(header_hash));
                                    log::trace!("Block received: {:x?}", block);
                                    match crate::blockchain::state::state_transition_function(&block) {
                                        Ok(_) => {
                                            log::info!("Block proccessed successfully");
                                        },
                                        Err(_) => {
                                            log::error!("Bad block");
                                        },
                                    }
                                    let state_root = get_state_root().lock().unwrap().clone();
                                    log::info!("Send to fuzzer the state root result: 0x{}", hex::encode(state_root));
                                    socket.write_all(&construct_fuzz_msg(Message::StateRoot, &state_root)).await?;
                                },
                                Message::GetState => {
                                    log::info!("GetState frame received");
                                    let hash = OpaqueHash::decode(&mut reader).unwrap();
                                    log::info!("Header hash received: 0x{}", hex::encode(hash));
                                    let raw_state = get_global_state().lock().unwrap().clone().serialize();
                                    let serialized_state = get_global_state().lock().unwrap().clone().serialize().map.encode();
                                    //log::info!("len raw_state: {:?}", raw_state.map.len());
                                    let msg = [encode_unsigned(raw_state.map.len()), serialized_state].concat();
                                    log::info!("Send serialized state");
                                    let key_values = get_global_state().lock().unwrap().clone().serialize();
                                    for item in key_values.map.iter() {
                                        log::debug!("key: {} val: {}", hex::encode(item.0), hex::encode(item.1));
                                    }
                                    //socket.write_all(&construct_fuzz_msg(Message::State, &serialized_state)).await?;
                                    socket.write_all(&construct_fuzz_msg(Message::State, &msg)).await?;
                                },
                                _ => {
                                    log::info!("Message type not supported: {:?}", msg_type);
                                 },
                            };

                            // Opcional: responder al cliente
                            /*if let Err(e) = socket.write_all(&buffer[..n]).await {
                                eprintln!("Error al enviar datos al cliente: {}", e);
                                break;
                            }*/
                        }
                        Err(e) => {
                            eprintln!("Error reading fuzzer data: {}", e);
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error al aceptar conexión: {}", e);
            }
        }
    }
}

fn construct_fuzz_msg(msg_type: Message, msg: &[u8]) -> Vec<u8> {
    let mut buffer: Vec<u8> = vec![];
    let msg_len = (msg.len() + 1) as u32;
    buffer.extend_from_slice(&[msg_len.encode(), vec![msg_type as u8], msg.encode()].concat());
    return buffer;
}


#[derive(Debug, Clone, PartialEq)]
pub struct TestCase {
    pub pre_state: RawState,
    pub block: Block,
    pub post_state: RawState,
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

pub fn read_files_in_directory(dir: &str) -> Result<Vec<PathBuf>, ()> {

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

pub fn decode_test_bin_file(file_content: &[u8]) -> Result<(RawState, Block, RawState), ReadError> {

    let mut reader = BytesReader::new(&file_content);
    let pre_state = RawState::decode(&mut reader)?;
    let block = Block::decode(&mut reader)?;
    let post_state = RawState::decode(&mut reader)?;

    return Ok((pre_state, block, post_state));
}

pub fn import_state_block(path: &Path) -> Result<(), ()> {
    
    let test_content = read_bin_file(path)?;
    let (pre_state, block, post_state) = match decode_test_bin_file(&test_content) {
        Ok(result) => result,
        Err(_) => { log::error!("Failed to decode {:?}", path); return Err(()) },
    };

    let mut state = GlobalState::default();
    let mut expected_state = GlobalState::default();

    set_raw_state(pre_state.clone(), &mut state);
    set_raw_state(post_state.clone(), &mut expected_state);

    assert_eq!(pre_state.state_root.clone(), merkle_state(&state.serialize().map, 0).unwrap());
    assert_eq!(post_state.state_root.clone(), merkle_state(&expected_state.serialize().map, 0).unwrap());

    set_global_state(state.clone());

    let error = state_transition_function(&block);
    
    if error.is_err() {
        println!("****************************************************** Error: {:?}", error);
        return Err(());
    }

    let result_state = get_global_state().lock().unwrap().clone();
    
    assert_eq_state(&expected_state, &result_state);

    /*println!("post_sta state_root: {:x?}", post_state.state_root);
    println!("expected state_root: {:x?}", merkle_state(&expected_state.serialize().map, 0).unwrap());
    println!("result   state_root: {:x?}", merkle_state(&result_state.serialize().map, 0).unwrap());*/
    
    assert_eq!(post_state.state_root, merkle_state(&result_state.serialize().map, 0).unwrap());

    Ok(())
}

pub fn run_traces_tests(file: &PathBuf) -> Result<(), ()> {

    let test_content = read_bin_file(file)?;
    let (pre_state, block, post_state) = match decode_test_bin_file(&test_content) {
        Ok(result) => result,
        Err(_) => { log::error!("File {:?} failed to decode", file); return Err(()) },
    };

    let mut state = GlobalState::default();
    let mut expected_state = GlobalState::default();

    set_raw_state(pre_state.clone(), &mut state);
    set_raw_state(post_state.clone(), &mut expected_state);

    assert_eq!(pre_state.state_root.clone(), merkle_state(&state.serialize().map, 0).unwrap());
    assert_eq!(post_state.state_root.clone(), merkle_state(&expected_state.serialize().map, 0).unwrap());

    set_global_state(state.clone());

    let error = state_transition_function(&block);
    
    if error.is_err() {
        println!("****************************************************** Error: {:?}", error);
        return Err(());
    }

    let result_state = get_global_state().lock().unwrap().clone();
    
    assert_eq_state(&expected_state, &result_state);

    println!("post_sta state_root: {:x?}", post_state.state_root);
    println!("expected state_root: {:x?}", merkle_state(&expected_state.serialize().map, 0).unwrap());
    println!("result   state_root: {:x?}", merkle_state(&result_state.serialize().map, 0).unwrap());
    
    assert_eq!(post_state.state_root, merkle_state(&result_state.serialize().map, 0).unwrap());

    Ok(())
}

pub fn set_state(raw_state: Vec<KeyValue>, state: &mut GlobalState) {

        for keyval in raw_state.iter() {
            
            if is_simple_key(keyval) {

                let mut reader = BytesReader::new(&keyval.value);
                let key = keyval.key[0] & 0xFF;

                match key {
                    AUTH_POOLS => {
                        state.auth_pools = AuthPools::decode(&mut reader).expect("Error decoding AuthPools");
                    },
                    AUTH_QUEUE => {
                        state.auth_queues = AuthQueues::decode(&mut reader).expect("Error decoding AuthQueues");
                    },
                    RECENT_HISTORY => {
                        state.recent_history = BlockHistory::decode(&mut reader).expect("Error decoding BlockHistory");
                    },
                    SAFROLE => {
                        state.safrole = Safrole::decode(&mut reader).expect("Error decoding Safrole");
                    },
                    DISPUTES => {
                        state.disputes = DisputesRecords::decode(&mut reader).expect("Error decoding Disputes");
                    },
                    ENTROPY => {
                        state.entropy = EntropyPool::decode(&mut reader).expect("Error decoding Entropy");
                    },
                    NEXT_VALIDATORS => {
                        state.next_validators = ValidatorsData::decode(&mut reader).expect("Error decoding NextValidators");
                    },
                    CURR_VALIDATORS => {
                        state.curr_validators = ValidatorsData::decode(&mut reader).expect("Error decoding CurrValidators");
                    },
                    PREV_VALIDATORS => {
                        state.prev_validators = ValidatorsData::decode(&mut reader).expect("Error decoding PrevValidators");
                    },
                    AVAILABILITY => {
                        state.availability = AvailabilityAssignments::decode(&mut reader).expect("Error decoding Availability");
                    },
                    TIME => {
                        state.time = TimeSlot::decode(&mut reader).expect("Error decoding Time");
                    },
                    PRIVILEGES => {
                        state.privileges = Privileges::decode(&mut reader).expect("Error decoding Privileges");
                    },
                    STATISTICS => {
                        state.statistics = Statistics::decode(&mut reader).expect("Error decoding Statistics");
                    },
                    READY_QUEUE => {
                        state.ready_queue = ReadyQueue::decode(&mut reader).expect("Error decoding ReadyQueue");
                    },
                    ACCUMULATION_HISTORY => {
                        state.accumulation_history = AccumulatedHistory::decode(&mut reader).expect("Error decoding AccumulationHistory");
                    },
                    _ => {
                        panic!("Key {:?} not supported", key);
                    },
                }

            } else if is_service_info_key(keyval) {

                let mut service_reader = BytesReader::new(&keyval.key[1..]);
                let service_id = ServiceId::decode(&mut service_reader).expect("Error decoding service id");

                if state.service_accounts.get(&service_id).is_none() {
                    let account = Account::default();
                    state.service_accounts.insert(service_id, account);
                }
                let mut account_reader = BytesReader::new(&keyval.value);
                let mut account = state.service_accounts.get(&service_id).unwrap().clone();
                account.code_hash = OpaqueHash::decode(&mut account_reader).expect("Error decoding code_hash");
                account.balance = Gas::decode(&mut account_reader).expect("Error decoding balance") as u64;
                account.acc_min_gas = Gas::decode(&mut account_reader).expect("Error decoding acc_min_gas");
                account.xfer_min_gas = Gas::decode(&mut account_reader).expect("Error decoding xfer_min_gas");
                // TODO bytes and items
                state.service_accounts.insert(service_id, account);
            } else {
                let service_id_vec = vec![keyval.key[0], keyval.key[2], keyval.key[4], keyval.key[6]];
                let service_id = decode::<ServiceId>(&service_id_vec, std::mem::size_of::<ServiceId>());
                let mut key_hash = vec![keyval.key[1], keyval.key[3], keyval.key[5]];
                key_hash.extend_from_slice(&keyval.key[7..]);

                // Preimage
                if is_preimage_key(keyval) { 
                    
                    if state.service_accounts.get(&service_id).is_none() {
                        state.service_accounts.insert(service_id, Account::default());
                    }
                    //let hash = sp_core::blake2_256(&keyval.value);
                    state.service_accounts.get_mut(&service_id).unwrap().preimages.insert(keyval.key, keyval.value.clone());
                    /*println!("preimage key: {:x?}", hash);
                    println!("preimage len: {:?}", keyval.value.len());
                    println!("----------------------------------------------------------------------");*/

                // Storage
                } else if is_storage_key(keyval) {
                    
                    if state.service_accounts.get(&service_id).is_none() {
                        state.service_accounts.insert(service_id, Account::default());
                    }

                    let mut storage_key  = [0u8; 31];
                    storage_key.copy_from_slice(&keyval.key);
                    //println!("insert to service: {:?} storage key: {:x?}", service_id, storage_key);
                    //println!("insert value: {:x?}", keyval.value);
                    state.service_accounts.get_mut(&service_id).unwrap().storage.insert(storage_key, keyval.value.clone());
                    /*println!("storage key: {:x?}", storage_key);
                    println!("storage val: {:x?}", keyval.value);
                    println!("----------------------------------------------------------------------");*/

                // Lookup
                } else {
                    let service_id_vec = vec![keyval.key[0], keyval.key[2], keyval.key[4], keyval.key[6]];
                    let service_id = decode::<ServiceId>(&service_id_vec, std::mem::size_of::<ServiceId>());
                    
                    let mut timeslots_reader = BytesReader::new(&keyval.value);
                    let timeslots = Vec::<u32>::decode_len(&mut timeslots_reader).expect("Error decoding timeslots");
                    
                    if state.service_accounts.get(&service_id).is_none() {
                        state.service_accounts.insert(service_id, Account::default());
                    }
                    
                    let account = state.service_accounts.get_mut(&service_id).unwrap();
                    account.lookup.insert(keyval.key, timeslots.clone());
                }
            }
        }
}

pub fn set_raw_state(raw_state: RawState, state: &mut GlobalState) {

        for keyval in raw_state.keyvals.iter() {
            
            if is_simple_key(keyval) {

                let mut reader = BytesReader::new(&keyval.value);
                let key = keyval.key[0] & 0xFF;

                match key {
                    AUTH_POOLS => {
                        state.auth_pools = AuthPools::decode(&mut reader).expect("Error decoding AuthPools");
                    },
                    AUTH_QUEUE => {
                        state.auth_queues = AuthQueues::decode(&mut reader).expect("Error decoding AuthQueues");
                    },
                    RECENT_HISTORY => {
                        state.recent_history = BlockHistory::decode(&mut reader).expect("Error decoding BlockHistory");
                    },
                    SAFROLE => {
                        state.safrole = Safrole::decode(&mut reader).expect("Error decoding Safrole");
                    },
                    DISPUTES => {
                        state.disputes = DisputesRecords::decode(&mut reader).expect("Error decoding Disputes");
                    },
                    ENTROPY => {
                        state.entropy = EntropyPool::decode(&mut reader).expect("Error decoding Entropy");
                    },
                    NEXT_VALIDATORS => {
                        state.next_validators = ValidatorsData::decode(&mut reader).expect("Error decoding NextValidators");
                    },
                    CURR_VALIDATORS => {
                        state.curr_validators = ValidatorsData::decode(&mut reader).expect("Error decoding CurrValidators");
                    },
                    PREV_VALIDATORS => {
                        state.prev_validators = ValidatorsData::decode(&mut reader).expect("Error decoding PrevValidators");
                    },
                    AVAILABILITY => {
                        state.availability = AvailabilityAssignments::decode(&mut reader).expect("Error decoding Availability");
                    },
                    TIME => {
                        state.time = TimeSlot::decode(&mut reader).expect("Error decoding Time");
                    },
                    PRIVILEGES => {
                        state.privileges = Privileges::decode(&mut reader).expect("Error decoding Privileges");
                    },
                    STATISTICS => {
                        state.statistics = Statistics::decode(&mut reader).expect("Error decoding Statistics");
                    },
                    READY_QUEUE => {
                        state.ready_queue = ReadyQueue::decode(&mut reader).expect("Error decoding ReadyQueue");
                    },
                    ACCUMULATION_HISTORY => {
                        state.accumulation_history = AccumulatedHistory::decode(&mut reader).expect("Error decoding AccumulationHistory");
                    },
                    _ => {
                        panic!("Key {:?} not supported", key);
                    },
                }

            } else if is_service_info_key(keyval) {

                let mut service_reader = BytesReader::new(&keyval.key[1..]);
                let service_id = ServiceId::decode(&mut service_reader).expect("Error decoding service id");

                if state.service_accounts.get(&service_id).is_none() {
                    let account = Account::default();
                    state.service_accounts.insert(service_id, account);
                }
                let mut account_reader = BytesReader::new(&keyval.value);
                let mut account = state.service_accounts.get(&service_id).unwrap().clone();
                account.code_hash = OpaqueHash::decode(&mut account_reader).expect("Error decoding code_hash");
                account.balance = Gas::decode(&mut account_reader).expect("Error decoding balance") as u64;
                account.acc_min_gas = Gas::decode(&mut account_reader).expect("Error decoding acc_min_gas");
                account.xfer_min_gas = Gas::decode(&mut account_reader).expect("Error decoding xfer_min_gas");
                // TODO bytes and items
                state.service_accounts.insert(service_id, account);
            } else {
                let service_id_vec = vec![keyval.key[0], keyval.key[2], keyval.key[4], keyval.key[6]];
                let service_id = decode::<ServiceId>(&service_id_vec, std::mem::size_of::<ServiceId>());
                let mut key_hash = vec![keyval.key[1], keyval.key[3], keyval.key[5]];
                key_hash.extend_from_slice(&keyval.key[7..]);

                // Preimage
                if is_preimage_key(keyval) { 
                    
                    if state.service_accounts.get(&service_id).is_none() {
                        state.service_accounts.insert(service_id, Account::default());
                    }
                    //let hash = sp_core::blake2_256(&keyval.value);
                    state.service_accounts.get_mut(&service_id).unwrap().preimages.insert(keyval.key, keyval.value.clone());
                    /*println!("preimage key: {:x?}", hash);
                    println!("preimage len: {:?}", keyval.value.len());
                    println!("----------------------------------------------------------------------");*/

                // Storage
                } else if is_storage_key(keyval) {
                    
                    if state.service_accounts.get(&service_id).is_none() {
                        state.service_accounts.insert(service_id, Account::default());
                    }

                    let mut storage_key  = [0u8; 31];
                    storage_key.copy_from_slice(&keyval.key);
                    //println!("insert to service: {:?} storage key: {:x?}", service_id, storage_key);
                    //println!("insert value: {:x?}", keyval.value);
                    state.service_accounts.get_mut(&service_id).unwrap().storage.insert(storage_key, keyval.value.clone());
                    /*println!("storage key: {:x?}", storage_key);
                    println!("storage val: {:x?}", keyval.value);
                    println!("----------------------------------------------------------------------");*/

                // Lookup
                } else {
                    let service_id_vec = vec![keyval.key[0], keyval.key[2], keyval.key[4], keyval.key[6]];
                    let service_id = decode::<ServiceId>(&service_id_vec, std::mem::size_of::<ServiceId>());
                    
                    let mut timeslots_reader = BytesReader::new(&keyval.value);
                    let timeslots = Vec::<u32>::decode_len(&mut timeslots_reader).expect("Error decoding timeslots");
                    
                    if state.service_accounts.get(&service_id).is_none() {
                        state.service_accounts.insert(service_id, Account::default());
                    }
                    
                    let account = state.service_accounts.get_mut(&service_id).unwrap();
                    account.lookup.insert(keyval.key, timeslots.clone());
                }
            }
        }
    }

pub fn assert_eq_state(expected_state: &GlobalState, result_state: &GlobalState) {
        assert_eq!(expected_state.time, result_state.time);
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
        assert_eq!(expected_state.recent_history.blocks, result_state.recent_history.blocks);           
        //assert_eq!(expected_state.service_accounts, result_state.service_accounts);
        for service_account in expected_state.service_accounts.iter() {
            if let Some(account) = result_state.service_accounts.get(&service_account.0) {
                //assert_eq!(service_account.1, account);
                //println!("TESTING service {:?}", service_account.0);
                //println!("Account: {:x?}", account);
                let (_items, _octets, _threshold) = account.get_footprint_and_threshold();
                for item in service_account.1.storage.iter() {
                    if let Some(value) = account.storage.get(item.0) {
                        assert_eq!(item.1, value);
                        //println!("storage Key {:x?} ", item.0);
                    } else {
                        panic!("Key storage not found : {:x?}", *item.0);
                    }
                }

                assert_eq!(service_account.1.storage, account.storage);
                //println!("items: {items}, octets: {octets}");
                /*println!("Lookup expected");
                for item in expected_state.service_accounts.get(&service_account.0).unwrap().lookup.iter() {
                    println!("{:x?} | {:?}", item.0, item.1);
                }
                println!("Lookup result");
                for item in account.lookup.iter() {
                    println!("{:x?} | {:?}", item.0, item.1);
                }
                println!("Lookup pre_state");
                for item in test_state.service_accounts.get(&service_account.0).unwrap().lookup.iter() {
                    println!("{:x?} | {:?}", item.0, item.1);
                }

                assert_eq!(service_account.1.lookup, account.lookup);*/
                assert_eq!(service_account.1.lookup, account.lookup);
                assert_eq!(service_account.1.preimages, account.preimages);
                assert_eq!(service_account.1.code_hash, account.code_hash);
                assert_eq!(service_account.1.balance, account.balance);
                assert_eq!(service_account.1.acc_min_gas, account.acc_min_gas);
                assert_eq!(service_account.1.xfer_min_gas, account.xfer_min_gas);

            } else {
                panic!("Service account not found in state: {:?}", service_account.0);
            }
        }
        assert_eq!(expected_state.service_accounts, result_state.service_accounts);
        assert_eq!(expected_state.auth_pools, result_state.auth_pools);
        assert_eq!(expected_state.statistics.curr, result_state.statistics.curr);
        assert_eq!(expected_state.statistics.prev, result_state.statistics.prev);
        assert_eq!(expected_state.statistics.cores, result_state.statistics.cores);
        assert_eq!(expected_state.statistics.services, result_state.statistics.services);
    }

    fn is_simple_key(keyval: &KeyValue) -> bool {

        keyval.key[0] <= 0x0F && keyval.key[1..].iter().all(|&b| b == 0)
    }

    fn is_service_info_key(keyval: &KeyValue) -> bool {

        keyval.key[0] == 0xFF && keyval.key[1..].iter().all(|&b| b == 0)
    }

    fn is_storage_key(keyval: &KeyValue) -> bool {

        keyval.key[1] == 0xFF && keyval.key[3] == 0xFF && keyval.key[5] == 0xFF && keyval.key[7] == 0xFF
    }

    fn is_preimage_key(keyval: &KeyValue) -> bool {

        keyval.key[1] == 0xFE && keyval.key[3] == 0xFF && keyval.key[5] == 0xFF && keyval.key[7] == 0xFF
    }

    /*fn is_lookup_key(keyval: &KeyValue) -> bool {
        
        !is_simple_key(keyval) && !is_service_info_key(keyval) && !is_storage_key(keyval) && !is_preimage_key(keyval)
    }*/
