use std::alloc::GlobalAlloc;
use std::collections::VecDeque;

use jam_types::{Block, GlobalState, Hash, Header, KeyValue, OpaqueHash, ReadError, TimeSlot};
use state_handler::{get_global_state, get_state_root};
use codec::{Encode, EncodeLen, Decode, DecodeLen, BytesReader};
use utils::common::parse_state_keyvals;
use utils::{trie::merkle_state, log, hex};
use state_handler::set_global_state;
use safrole::{create_ring_set, get_verifiers, set_verifiers};
use utils::bandersnatch::Verifier;
use vinwolf_target::read_all_bins;

use std::io::{Read, Write};
use std::thread;
use std::time::Duration;
use std::collections::HashMap;
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::{LazyLock, Mutex};

use super::BUILD_PROFILE;

pub static VINWOLF_INFO: LazyLock<PeerInfo> = LazyLock::new(|| {
    
    PeerInfo {
        name: "vinwolf-target".as_bytes().to_vec(),
        app_version: Version {
            major: 0,
            minor: 2,
            patch: 10,
        },
        jam_version: Version {
            major: 0,
            minor: 7,
            patch: 0,
        },
    }
});

pub struct State {
    pub header: Header,
    pub state: Vec<KeyValue>,
}

#[derive(Debug)]
pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

pub struct PeerInfo {
    pub name: Vec<u8>,
    pub app_version: Version,
    pub jam_version: Version,
}

pub struct SetState {
    pub header: Header,
    pub state: Vec<KeyValue>,
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

impl Encode for PeerInfo {

    fn encode(&self) -> Vec<u8> {
        
        let mut blob = vec![];

        self.name.encode_len().encode_to(&mut blob);
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
            name: Vec::<u8>::decode_len(reader)?,
            app_version: Version::decode(reader)?, 
            jam_version: Version::decode(reader)?,
        })
    }
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

pub fn connect_to_unix_socket(path: &str) -> Result<(), Box<dyn std::error::Error>> {

    let mut stream = UnixStream::connect(path)?;
    // Write
    stream.write_all(&vec![0, 0])?;
    std::thread::sleep(std::time::Duration::from_millis(2000));
    let mut buffer = [0u8; 1024];
    // Read
    let _n = stream.read(&mut buffer)?;
    Ok(())
}

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

fn simple_fork(state_root: &OpaqueHash) -> (OpaqueHash, GlobalState, VecDeque<Verifier>) {

    let state_record = get_state_record();

    let state = if let Some((pre_state_root, pre_state, verifiers_records)) = state_record.iter().find(|(s_root, _, _)| *state_root == *s_root ) {
        (*pre_state_root, pre_state.clone(), verifiers_records.clone())
    } else {
        state_record.back().unwrap().clone()
    };

    return state;
}

pub fn run_unix_server(socket_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    
    let vinwolf_info = &*VINWOLF_INFO;

    let listener = UnixListener::bind(socket_path)?;
    println!("Running {} mode {:?} version: {}.{}.{} protocol version: {}.{}.{} listening on {}", 
                BUILD_PROFILE,
                match String::from_utf8(vinwolf_info.name.clone()) {
                Ok(name) => name,  
                Err(_) => "Invalid UTF-8".to_string(),  
                }, 
                vinwolf_info.app_version.major, 
                vinwolf_info.app_version.minor, 
                vinwolf_info.app_version.patch,
                vinwolf_info.jam_version.major, 
                vinwolf_info.jam_version.minor, 
                vinwolf_info.jam_version.patch,
                socket_path);
    log::info!(
                "Running {} mode {:?} version: {}.{}.{} protocol version: {}.{}.{} listening on {}", 
                BUILD_PROFILE,
                match String::from_utf8(vinwolf_info.name.clone()) {
                Ok(name) => name,  
                Err(_) => "Invalid UTF-8".to_string(),  
                }, 
                vinwolf_info.app_version.major, 
                vinwolf_info.app_version.minor, 
                vinwolf_info.app_version.patch,
                vinwolf_info.jam_version.major, 
                vinwolf_info.jam_version.minor, 
                vinwolf_info.jam_version.patch,
                socket_path);

    loop {
        //println!("waiting for connection");
        match listener.accept() {

            Ok((mut socket, _)) => {

                log::info!("New incomming connection accepted...");

                loop {

                    let mut buffer = vec![0u8; 1024];
                    match socket.read(&mut buffer) {
                        
                        Ok(0) => {
                            std::thread::sleep(std::time::Duration::from_millis(200));
                            //println!("Connection closed");
                            break;
                        }
                        Ok(n) => {
                            //println!("Reading msg......");
                            let mut bytes_read = n;

                            while bytes_read < std::mem::size_of::<u32>() {
                                //println!("reading len");
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
                                //println!("reading msg");
                                match socket.read(&mut buffer[bytes_read..]) {
                                    Ok(n) => bytes_read += n ,
                                    Err(e) => { break; }
                                }; 
                            }

                            let mut reader = BytesReader::new(&buffer);

                            let msg_len_d = match u32::decode(&mut reader) {
                                Ok(len) => {
                                    log::info!("decoded msg len: {:?}", len);
                                    len
                                },
                                Err(error) => {
                                    log::error!("Failed to decode msg len: {:?}", error);
                                    return Err(Box::new(ReadError::InvalidData));
                                },
                            };

                            let msg_type: Message = match reader.read_byte() {
                                Ok(m_type) => {
                                    log::info!("decoded msg type: {:?}", m_type);
                                    m_type.into()
                                },
                                Err(error) => {
                                    log::error!("Failed to decode msg type: {:?}", error);
                                    return Err(Box::new(ReadError::InvalidData));
                                },
                            };
                            
                            log::info!("New message from fuzzer with length: {:?}, total bytes read: {:?}", msg_len, bytes_read);

                            match msg_type {
                                Message::PeerInfo => { 

                                    let fuzzer_info = match PeerInfo::decode(&mut reader) {
                                        Ok(fuzzer_info) => fuzzer_info,
                                        Err(error) => {
                                            log::error!("Failed to decode the peer info: {:?}", error);
                                            return Err(Box::new(ReadError::InvalidData));
                                        }
                                    };

                                    log::info!(
                                        "Fuzzer info: {:?} version: {}.{}.{} protocol version: {}.{}.{}",
                                        match String::from_utf8(fuzzer_info.name.clone()) {
                                            Ok(name) => name,  
                                            Err(_) => "Invalid UTF-8".to_string(),  
                                        }, 
                                        fuzzer_info.app_version.major, 
                                        fuzzer_info.app_version.minor, 
                                        fuzzer_info.app_version.patch,
                                        fuzzer_info.jam_version.major, 
                                        fuzzer_info.jam_version.minor, 
                                        fuzzer_info.jam_version.patch
                                    );

                                   log::info!(
                                        "Target info: {:?} version: {}.{}.{} protocol version: {}.{}.{}", 
                                        match String::from_utf8(vinwolf_info.name.clone()) {
                                            Ok(name) => name,  
                                            Err(_) => "Invalid UTF-8".to_string(),  
                                        }, 
                                        vinwolf_info.app_version.major, 
                                        vinwolf_info.app_version.minor, 
                                        vinwolf_info.app_version.patch,
                                        vinwolf_info.jam_version.major, 
                                        vinwolf_info.jam_version.minor, 
                                        vinwolf_info.jam_version.patch
                                    );

                                    let peer_info_len = vinwolf_info.name.len() + 7 + 1; // OJO con esto

                                    let msg = [
                                        (peer_info_len as u32).encode(), 
                                        vec![msg_type as u8],
                                        vinwolf_info.encode(),
                                    ].concat();
                                    
                                    match socket.write_all(&msg) {
                                        Ok(_) => {},
                                        Err(e) => { break; }
                                    }; 
                                },
                                Message::SetState => {
                                    //println!("SetState frame received");
                                    log::info!("SetState frame received");
                                    
                                    let header = match Header::decode(&mut reader) {
                                        Ok(header) => header,
                                        Err(error) => {
                                            log::error!("Failed to decode Header: {:?}", error);
                                            let state_root = get_state_root().lock().unwrap().clone();
                                            match socket.write_all(&construct_fuzz_msg(Message::StateRoot, &state_root)) {
                                                Ok(_) => {},
                                                Err(e) => { break; }
                                            }; 
                                            log::info!("SetState - same state root {}", hex::encode(state_root));
                                            continue;
                                        },
                                    };

                                    let keyvals = match Vec::<KeyValue>::decode_len(&mut reader) {
                                        Ok(keyvals) => keyvals,
                                        Err(error) => {
                                            log::error!("Failed to decode the state key-values: {:?}", error);
                                            let state_root = get_state_root().lock().unwrap().clone();
                                            match socket.write_all(&construct_fuzz_msg(Message::StateRoot, &state_root)) {
                                                Ok(_) => {},
                                                Err(e) => { break; }
                                            };
                                            log::info!("SetState - same state root {}", hex::encode(state_root));
                                            continue;
                                        },
                                    };

                                    let mut global_state = GlobalState::default();
                                    log::info!("Parse keyvals");
                                    match parse_state_keyvals(&keyvals, &mut global_state) {
                                        Ok(_) => { },
                                        Err(error) => {
                                            log::error!("Failed to decode state: {:?}", error);
                                            let state_root = get_state_root().lock().unwrap().clone();
                                            match socket.write_all(&construct_fuzz_msg(Message::StateRoot, &state_root)) {
                                                Ok(_) => {},
                                                Err(e) => { break; }
                                            };
                                            log::info!("SetState - same state root {}", hex::encode(state_root));
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

                                    match socket.write_all(&construct_fuzz_msg(Message::StateRoot, &state_root)) {
                                        Ok(_) => {},
                                        Err(e) => { break; }
                                    };
                                    log::info!("SetState - state root {}", hex::encode(state_root));
                                },
                                Message::ImportBlock => {

                                    log::info!("ImportBlock frame received");

                                    let block = match Block::decode(&mut reader) {
                                        Ok(block) => block,
                                        Err(error) => {
                                            log::error!("Failed to decode block: {:?}", error);
                                            let state_root = get_state_root().lock().unwrap().clone();
                                            match socket.write_all(&construct_fuzz_msg(Message::StateRoot, &state_root)) {
                                                Ok(_) => {},
                                                Err(e) => { break; }
                                            };
                                            log::info!("SetState - same state root {}", hex::encode(state_root));
                                            continue;
                                        },
                                    };

                                    let header_hash = sp_core::blake2_256(&(block.header.encode()));
                                    log::info!("Header hash: 0x{}", hex::encode(header_hash));
                                    
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
                                        },
                                        Err(error) => {
                                            //println!("Refused block {}", utils::print_hash!(header_hash));
                                            log::error!("Bad block: {:?}", error);
                                        },
                                    }
                                    let state_root = get_state_root().lock().unwrap().clone();
                                    log::info!("SetState - state root {}", hex::encode(state_root));
                                    match socket.write_all(&construct_fuzz_msg(Message::StateRoot, &state_root)) {
                                        Ok(_) => {},
                                        Err(e) => { break; }
                                    };
                                },
                                Message::GetState => {

                                    log::info!("GetState frame received");
                                    let header_hash = match OpaqueHash::decode(&mut reader) {
                                        Ok(header_hash) => header_hash,
                                        Err(error) => {
                                            log::error!("Failed to decode the header hash: {:?}", error);
                                            return Err(Box::new(error));
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
                                    let state_frame = construct_fuzz_msg(Message::State, &serialized_state);
                                    match socket.write_all(&state_frame) {
                                        Ok(_) => {},
                                        Err(e) => { break; }
                                    };
                                },
                                _ => {
                                    log::error!("Message type not supported: {:?}", msg_type);
                                    println!("Message type not supported {:?}", msg_type);
                                    break;
                                 },
                            };
                        }
                        Err(error) => {
                            println!("Error reading fuzzer data");
                            log::error!("Error reading fuzzer data: {}", error);
                            break;
                        }
                    }
                }
            }
            Err(error) => {
                println!("Error accepting connection");
                log::error!("Accepting connection: {}", error);
                return Err(Box::new(error));
            }
        }
    }
}

fn construct_fuzz_msg(msg_type: Message, msg: &[u8]) -> Vec<u8> {
    let mut buffer: Vec<u8> = vec![];
    buffer.extend_from_slice(&[(msg.len() as u32 + 1).encode(), vec![msg_type as u8], msg.encode()].concat());
    let _len = buffer.len();
    return buffer;
}

pub fn run_fuzzer(path: &str) -> Result<(), Box<dyn std::error::Error>> {

    let vinwolf_info = &*VINWOLF_INFO;

    let mut socket = UnixStream::connect(path)?;

    let peer_info_len = vinwolf_info.name.len() + 7 + 1; // OJO con esto !!!

    let msg = [
        (peer_info_len as u32).encode(), 
        vec![Message::PeerInfo as u8],
        vinwolf_info.encode(),
    ].concat();

    socket.write_all(&msg)?;

    std::thread::sleep(std::time::Duration::from_millis(500));
    let mut buffer = [0u8; 1024000];
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
            let set_state = SetState {
                header: block.header.clone(),
                state: pre_keyvals,
            };
            log::info!("Set state");
            let set_state_msg = [vec![Message::SetState as u8], set_state.encode()].concat();
            let msg = [(set_state_msg.len() as u32).encode(), set_state_msg].concat();
            socket.write_all(&msg).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(500));
            let bytes_read = socket.read(&mut buffer).unwrap();
            let state_root_received: OpaqueHash = buffer[5..bytes_read].try_into().unwrap();
            log::info!("state_root received: {}", hex::encode(&state_root_received));
            assert_eq!(pre_state_root, state_root_received);
        }

        // Export block
        log::info!("Exporting block {}", utils::print_hash!(&(sp_core::blake2_256(&block.header.encode()))));
        let import_block_msg = [vec![Message::ImportBlock as u8], block.encode()].concat();
        let msg = [(import_block_msg.len() as u32).encode(), import_block_msg].concat();
        socket.write_all(&msg).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(500));
        let bytes_read = socket.read(&mut buffer).unwrap();
        let state_root_received: OpaqueHash = buffer[5..bytes_read].try_into().unwrap();
        log::info!("state_root received: {}", hex::encode(&state_root_received));
        assert_eq!(post_state_root, state_root_received);
        log::info!("OK - The state root received matches");
    }
}

#[test]
fn test_set_state() {

    let test_content = utils::common::read_bin_file(std::path::Path::new("/home/bernar/workspace/jam-stuff/fuzz-proto/examples/2_set_state.bin")).unwrap();
    let mut reader = BytesReader::new(&test_content);
    let msg_type: Message = reader.read_byte().unwrap().into();

    let header = Header::decode(&mut reader).unwrap();
    let keyvals = Vec::<KeyValue>::decode_len(&mut reader).unwrap();

    let mut global_state = GlobalState::default();

    match parse_state_keyvals(&keyvals, &mut global_state) {
        Ok(_) => { },
        Err(error) => {
            log::error!("Failed to decode state: {:?}", error);
        },
    }

    let raw_state = utils::serialization::serialize(&global_state);

    for keyval in keyvals.iter() {
        assert_eq!(*raw_state.map.get(&keyval.key).unwrap(), keyval.value);
    }

    let result_test_content = [vec![2], header.encode(), keyvals.encode_len()].concat();
    assert_eq!(test_content, result_test_content);
}

#[test]
fn test_get_state() {
    let test_content = utils::common::read_bin_file(std::path::Path::new("/home/bernar/workspace/jam-stuff/fuzz-proto/examples/12_get_state.bin")).unwrap();
    let mut reader = BytesReader::new(&test_content);
    let msg_type = reader.read_byte().unwrap();
    let header_hash = OpaqueHash::decode(&mut reader).unwrap();
    let result_test_content = [vec![msg_type], header_hash.encode()].concat();
    assert_eq!(test_content, result_test_content);
}

#[test]
fn test_state() {
    
    let test_content = utils::common::read_bin_file(std::path::Path::new("/home/bernar/workspace/jam-stuff/fuzz-proto/examples/2_set_state.bin")).unwrap();
    let mut reader = BytesReader::new(&test_content);

    let header = Header::decode(&mut reader).unwrap();
    let keyvals = Vec::<KeyValue>::decode_len(&mut reader).unwrap();
}