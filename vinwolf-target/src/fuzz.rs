use std::collections::VecDeque;

use jam_types::{Block, GlobalState, KeyValue, OpaqueHash};
use state_handler::{get_global_state, get_state_root};
use codec::{Encode, EncodeLen, Decode, DecodeLen, BytesReader};
use utils::common::parse_state_keyvals;
use utils::{trie::merkle_state, log, hex};
use state_handler::set_global_state;
use safrole::{create_ring_set, get_verifiers, set_verifiers};
use utils::bandersnatch::Verifier;
use vinwolf_target::read_all_bins;

use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::{LazyLock, Mutex};

use super::fuzz_types::*;

use super::BUILD_PROFILE;

pub static VINWOLF_INFO: LazyLock<PeerInfo> = LazyLock::new(|| {
    
    PeerInfo {
        app_name: "vinwolf-target".as_bytes().to_vec(),
        app_version: Version {
            major: 0,
            minor: 2,
            patch: 11,
        },
        jam_version: Version {
            major: 0,
            minor: 7,
            patch: 0,
        },
        fuzz_features: 2,
        fuzz_version: 1,
    }
});

static STATE_RECORD: LazyLock<Mutex<VecDeque<(OpaqueHash, GlobalState, VecDeque<Verifier>)>>> = LazyLock::new(|| { Mutex::new(VecDeque::new())});

fn update_state_record(pre_state_root: &OpaqueHash, post_state_root: &OpaqueHash, state: GlobalState, verifiers_record: VecDeque<Verifier>) {

    let mut state_record = STATE_RECORD.lock().unwrap().clone();

    if state_record[0].0 == *pre_state_root {
        // We are in a fork process
        return;
    } 

    state_record.push_back((*post_state_root, state.clone(), verifiers_record));
    state_record.pop_front();
    
    set_state_record(state_record);
}

fn set_state_record(state_record: VecDeque<(OpaqueHash, GlobalState, VecDeque<Verifier>)>) {
    *STATE_RECORD.lock().unwrap() = state_record;
}

fn get_state_record() -> VecDeque<(OpaqueHash, GlobalState, VecDeque<Verifier>)> {
    STATE_RECORD.lock().unwrap().clone()
}

fn print_vinwolf_target_info(socket_path: &str) -> String {
    
    let vinwolf_info = &*VINWOLF_INFO;

    format!("Running {} mode {:?} version: {}.{}.{} GP version: {}.{}.{} listening on {}", 
                BUILD_PROFILE,
                match String::from_utf8(vinwolf_info.app_name.clone()) {
                Ok(name) => name,  
                Err(_) => "Invalid UTF-8".to_string(),  
                }, 
                vinwolf_info.app_version.major, 
                vinwolf_info.app_version.minor, 
                vinwolf_info.app_version.patch,
                vinwolf_info.jam_version.major, 
                vinwolf_info.jam_version.minor, 
                vinwolf_info.jam_version.patch,
                socket_path)
}

pub fn run_unix_server(socket_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    
    let vinwolf_info = &*VINWOLF_INFO;
    let listener = UnixListener::bind(socket_path)?;

    println!("{:?}", print_vinwolf_target_info(socket_path));
    log::info!("{:?}", print_vinwolf_target_info(socket_path));

    listen_socket(&listener)?;

    Ok(())
}

fn listen_socket(listener: &UnixListener) -> Result<(), Box<dyn std::error::Error>> {

    loop {
        //println!("waiting for connection");
        match listener.accept() {

            Ok((mut socket, _)) => {

               read_socket(&mut socket);
            }
            Err(error) => {
                println!("Error accepting connection");
                log::error!("Accepting connection: {}", error);
                return Err(Box::new(error));
            }
        }
    }
}

fn print_peer_info(peer: &PeerInfo, rol: &str) {

    log::info!(
        "{} info: {:?} version: {}.{}.{} GP version: {}.{}.{} features: {} fuzz-proto version: {}",
        rol,
        match String::from_utf8(peer.app_name.clone()) {
            Ok(name) => name,  
            Err(_) => "Invalid UTF-8".to_string(),  
        }, 
        peer.app_version.major, 
        peer.app_version.minor, 
        peer.app_version.patch,
        peer.jam_version.major, 
        peer.jam_version.minor, 
        peer.jam_version.patch,
        peer.fuzz_features,
        peer.fuzz_version
    );
}

fn read_socket(socket: &mut UnixStream) {

    let vinwolf_info = &*VINWOLF_INFO;

    log::info!("New incomming connection accepted...");

    loop {

        let mut buffer = vec![0u8; 1024];

        match socket.read(&mut buffer) {
            
            Ok(0) => {
                std::thread::sleep(std::time::Duration::from_millis(200));
            }
            Ok(n) => {

                let mut bytes_read = n;

                while bytes_read < std::mem::size_of::<u32>() {
                    match socket.read(&mut buffer[bytes_read..]) {
                        Ok(n) => bytes_read += n ,
                        Err(e) => { break; }
                    }; 
                }

                let msg_len = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);

                if msg_len > buffer.len() as u32 {
                    buffer.resize((msg_len + 4) as usize, 0);
                }

                while bytes_read < 4 + msg_len as usize {
                    match socket.read(&mut buffer[bytes_read..]) {
                        Ok(n) => bytes_read += n ,
                        Err(e) => { break; }
                    }; 
                }

                let mut reader = BytesReader::new(&buffer);

                let msg_len_d = match u32::decode(&mut reader) {
                    Ok(len) => { len },
                    Err(error) => {
                        log::error!("Failed to decode msg len: {:?}", error);
                        continue;
                    },
                };

                let msg_type: Message = match reader.read_byte() {
                    Ok(m_type) => { m_type.into() },
                    Err(error) => {
                        log::error!("Failed to decode msg type: {:?}", error);
                        continue;
                    },
                };

                log::info!("New message from fuzzer with length: {:?}, total bytes read: {:?}", msg_len, bytes_read);

                match msg_type {
                    Message::PeerInfo => { 

                        let fuzzer_info = match PeerInfo::decode(&mut reader) {
                            Ok(fuzzer_info) => fuzzer_info,
                            Err(error) => {
                                log::error!("Failed to decode the peer info: {:?}", error);
                                continue;
                            }
                        };

                        print_peer_info(&fuzzer_info, "Fuzzer");
                        print_peer_info(&vinwolf_info, "Target");

                        if send_to_peer(&fuzz_msg(Message::PeerInfo, &vinwolf_info.encode()), socket).is_err() {
                            break;
                        }
                    },
                    Message::Initialize => {

                        log::info!("Initialize frame received");
                        
                        let initialize = match Initialize::decode(&mut reader) {
                            Ok(initialize) => initialize,
                            Err(error) => {
                                log::error!("Failed to decode initialize frame: {:?}", error);
                                continue;
                            }
                        };

                        let mut global_state = GlobalState::default();
                        log::debug!("Parse keyvals");
                        match parse_state_keyvals(&initialize.keyvals, &mut global_state) {
                            Ok(_) => { },
                            Err(error) => {
                                log::error!("Failed to parse the state keyvals: {:?}", error);
                                continue;
                            },
                        }

                        let state_root = merkle_state(&utils::serialization::serialize(&global_state).map, 0);
                        state_handler::set_state_root(state_root.clone());
                        set_global_state(global_state.clone());

                        set_verifiers(VecDeque::new());
                        let mut verifiers = VecDeque::new();
                        let pending_validators = state_handler::get_global_state().lock().unwrap().safrole.pending_validators.clone();
                        let curr_validators = state_handler::get_global_state().lock().unwrap().curr_validators.clone();
                        verifiers.push_back(Verifier::new(create_ring_set(&curr_validators)));
                        verifiers.push_back(Verifier::new(create_ring_set(&pending_validators)));
                        set_verifiers(verifiers.clone());
                        block::header::set_parent_header(OpaqueHash::default());

                        //set_global_state(global_state.clone());
                        let mut state_record = VecDeque::new();
                        state_record.push_back((OpaqueHash::default(), GlobalState::default(), VecDeque::new()));
                        state_record.push_back((state_root, global_state.clone(), verifiers));
                        set_state_record(state_record);

                        log::info!("SetState - state root {}", hex::encode(state_root));
                        if send_to_peer(&fuzz_msg(Message::StateRoot, &state_root), socket).is_err() {
                            break;
                        }
                    },
                    Message::ImportBlock => {

                        log::info!("ImportBlock frame received");

                        let block = match Block::decode(&mut reader) {
                            Ok(block) => block,
                            Err(error) => {
                                log::error!("Failed to decode block: {:?}", error);
                                if send_to_peer(&fuzz_msg(Message::Error, &format!("Failed to decode block: {:?}", error).as_bytes().to_vec().encode_len()), socket).is_err() {
                                    break;
                                }
                                continue;
                            },
                        };
                        
                        let (pre_state_root, pre_state, verifiers) = simple_fork(&block.header.unsigned.parent_state_root);

                        set_verifiers(verifiers);
                        set_global_state(pre_state.clone());
                        state_handler::set_state_root(pre_state_root.clone());

                        match state_controller::stf(&block) {
                            Ok(_) => {
                                //println!("Block {} processed successfully", utils::print_hash!(header_hash));
                                log::info!("Block proccessed successfully");
                                let post_state_root = get_state_root().lock().unwrap().clone();
                                update_state_record(&block.header.unsigned.parent_state_root, &post_state_root, get_global_state().lock().unwrap().clone(), get_verifiers());
                                log::info!("SetState - state root {}", hex::encode(post_state_root));
                                if send_to_peer(&fuzz_msg(Message::StateRoot, &post_state_root), socket).is_err() {
                                    break;
                                }
                            },
                            Err(error) => {
                                //println!("Refused block {}", utils::print_hash!(header_hash));
                                log::error!("Block execution failure: {:?}", error);
                                if send_to_peer(&fuzz_msg(Message::Error, &format!("Block execution failure: {:?}", error).as_bytes().to_vec().encode_len()), socket).is_err() {
                                    break;
                                }
                            },
                        }                        
                    },
                    Message::GetState => {

                        log::info!("GetState frame received");
                        let header_hash = match OpaqueHash::decode(&mut reader) {
                            Ok(header_hash) => header_hash,
                            Err(error) => {
                                log::error!("Failed to decode the header hash: {:?}", error);
                                continue;
                            }
                        };

                        log::info!("Header hash received: 0x{}", hex::encode(header_hash));
                        let global_state = get_global_state().lock().unwrap().clone();
                        let raw_state = utils::serialization::serialize(&global_state);

                        let mut keyvalues: Vec<KeyValue> = vec![];

                        for entry in raw_state.map.iter() {
                            let keyvalue = KeyValue {
                                key: *entry.0,
                                value: entry.1.clone(),
                            };
                            keyvalues.push(keyvalue);
                        }

                        let serialized_state = keyvalues.encode_len();

                        if send_to_peer(&fuzz_msg(Message::State, &serialized_state), socket).is_err() {
                            break;
                        }
                    },
                    _ => {
                            log::error!("Message type not supported: {:?}", msg_type);
                            send_to_peer(&fuzz_msg(Message::Error, &format!("Message type not supported: {:?}", msg_type).as_bytes().to_vec().encode_len()), socket).unwrap();
                            break;
                        },
                };
            }
            Err(error) => {
                //println!("Error reading fuzzer data");
                log::error!("Error reading fuzzer data: {}", error);
                break;
            }
        }
    }
}

fn send_to_peer(msg: &[u8], socket: &mut UnixStream) -> Result<(), Box<dyn std::error::Error>> {
    match socket.write_all(&msg) {
        Ok(_) => { return Ok(()) },
        Err(error) => { return Err(Box::new(error)); },
    }
}

fn fuzz_msg(msg_type: Message, msg: &[u8]) -> Vec<u8> {
    [(msg.len() as u32 + 1).encode(), msg_type.encode(), msg.encode()].concat()
}

fn simple_fork(state_root: &OpaqueHash) -> (OpaqueHash, GlobalState, VecDeque<Verifier>) {

    let state_record = get_state_record();

    let state = if let Some((pre_state_root, pre_state, verifiers_records)) = state_record.iter().find(|(s_root, _, _)| *state_root == *s_root ) {
        (*pre_state_root, pre_state.clone(), verifiers_records.clone())
    } else {
        state_record.back().unwrap().clone()
    };

    return state;
}

pub fn run_fuzzer(path: &str) -> Result<(), Box<dyn std::error::Error>> {

    let vinwolf_info = &*VINWOLF_INFO;

    let mut socket = UnixStream::connect(path)?;

    let peer_info_msg = [Message::PeerInfo.encode(), vinwolf_info.encode()].concat();
    let msg = [(peer_info_msg.len() as u32).encode(), peer_info_msg.encode()].concat();
    socket.write_all(&msg)?;

    std::thread::sleep(std::time::Duration::from_millis(500));
    let mut buffer = vec![0u8; 1024000];
    let n = socket.read(&mut buffer)?;


    let path = std::path::Path::new("/home/bernar/workspace/jam-conformance/fuzz-reports/0.7.0/traces/");

    for entry in std::fs::read_dir(path).unwrap() {

        let dir_entry = entry.unwrap();
        let dir_path = dir_entry.path();

        if !dir_path.is_dir() {
            continue;
        }

        log::info!("Fuzzing dir: {:?}", dir_path);
        println!("Fuzzing dir: {:?}", dir_path);
        fuzz_dir(&mut socket, &dir_path);
    }

    Ok(())
}

fn fuzz_dir(socket: &mut UnixStream, dir_path: &std::path::Path) {

    let bin_files = read_all_bins(dir_path);

    for trace in bin_files.iter().enumerate() {
        
        let mut buffer = [0u8; 1024000];

        let test_content = std::fs::read(&trace.1.1).unwrap();
        let mut reader = BytesReader::new(&test_content);
        let pre_state_root = OpaqueHash::decode(&mut reader).unwrap();
        let pre_keyvals = Vec::<KeyValue>::decode_len(&mut reader).unwrap();
        let block = Block::decode(&mut reader).unwrap();
        let post_state_root = OpaqueHash::decode(&mut reader).unwrap();
        let post_keyvals = Vec::<KeyValue>::decode_len(&mut reader).unwrap();

        if trace.0 == 0 {
            // Set state
            let set_state = Initialize {
                header: block.header.clone(),
                keyvals: pre_keyvals,
                ancestry: VecDeque::new(),
            };
            println!("Set state");
            let set_state_msg = [vec![Message::Initialize as u8], set_state.encode()].concat();
            let msg = [(set_state_msg.len() as u32).encode(), set_state_msg].concat();
            socket.write_all(&msg).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(500));
            let bytes_read = socket.read(&mut buffer).unwrap();
            let state_root_received: OpaqueHash = buffer[5..bytes_read].try_into().unwrap();
            println!("state_root received: {}", hex::encode(&state_root_received));
            assert_eq!(pre_state_root, state_root_received);
        }

        // Export block
        println!("Exporting block {}", utils::print_hash!(&(sp_core::blake2_256(&block.header.encode()))));
        let import_block_msg = [vec![Message::ImportBlock as u8], block.encode()].concat();
        let msg = [(import_block_msg.len() as u32).encode(), import_block_msg].concat();
        socket.write_all(&msg).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(500));
        let bytes_read = socket.read(&mut buffer).unwrap();

        if buffer[4] == Message::Error as u8 {
            let mut reader = BytesReader::new(&buffer[5..bytes_read]);
            let msg = Vec::<u8>::decode_len(&mut reader).unwrap();
            println!("MSG ERROR FROM TARGET: {:?}", String::from_utf8(msg).unwrap());
        } else {
            let state_root_received: OpaqueHash = buffer[5..bytes_read].try_into().unwrap();
            println!("state_root received: {}", hex::encode(&state_root_received));
            assert_eq!(post_state_root, state_root_received);
            println!("OK - The state root received matches");
        }
    }
}

