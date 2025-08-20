use jam_types::{Block, OpaqueHash, KeyValue, Header, GlobalState, ReadError};
use state_handler::{get_global_state, get_state_root};
use codec::{Encode, EncodeLen, Decode, DecodeLen, BytesReader};
use utils::common::parse_state_keyvals;
use utils::trie::merkle_state;
use state_handler::set_global_state;

use tokio::net::{UnixListener, UnixStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt}; 
use bytes::BytesMut;
use once_cell::sync::Lazy;

use super::BUILD_PROFILE;

pub static VINWOLF_INFO: Lazy<PeerInfo> = Lazy::new(|| {
    
    PeerInfo {
        name: "vinwolf-target".as_bytes().to_vec(),
        app_version: Version {
            major: 0,
            minor: 1,
            patch: 2,
        },
        jam_version: Version {
            major: 0,
            minor: 6,
            patch: 7,
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

pub async fn connect_to_unix_socket(path: &str) -> Result<(), Box<dyn std::error::Error>> {

    let mut stream = UnixStream::connect(path).await?;
    // Write
    stream.write_all(&vec![0, 0]).await?;
    tokio::time::sleep(std::time::Duration::from_millis(2000)).await;
    let mut buffer = [0u8; 1024];
    // Read
    let _n = stream.read(&mut buffer).await?;
    Ok(())
}

pub async fn run_unix_server(socket_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    
    let vinwolf_info = &*VINWOLF_INFO;

    let listener = UnixListener::bind(socket_path)?;
    
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
        match listener.accept().await {

            Ok((mut socket, _)) => {

                log::info!("New incomming connection accepted...");

                loop {
                    let mut buffer = BytesMut::with_capacity(1024);

                    let read_result = socket.read_buf(&mut buffer).await;

                    match read_result {
                        
                        Ok(0) => {
                            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                            break;
                        }
                        Ok(n) => {

                            let mut bytes_read = n;

                            while bytes_read < std::mem::size_of::<u32>() {

                                bytes_read += socket.read_buf(&mut buffer).await?;
                            }

                            let msg_len = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);

                            while bytes_read < 4 + msg_len as usize {

                                bytes_read += socket.read_buf(&mut buffer).await?;
 
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
                                    
                                    socket.write_all(&msg).await?;                                   
                                },
                                Message::SetState => {
                                    log::info!("SetState frame received");
                                    
                                    let _header = match Header::decode(&mut reader) {
                                        Ok(header) => header,
                                        Err(error) => {
                                            log::error!("Failed to decode Header: {:?}", error);
                                            let state_root = get_state_root().lock().unwrap().clone();
                                            socket.write_all(&construct_fuzz_msg(Message::StateRoot, &state_root)).await?;
                                            log::info!("SetState - same state root {}", hex::encode(state_root));
                                            continue;
                                        },
                                    };

                                    let keyvals = match Vec::<KeyValue>::decode_len(&mut reader) {
                                        Ok(keyvals) => keyvals,
                                        Err(error) => {
                                            log::error!("Failed to decode the state key-values: {:?}", error);
                                            let state_root = get_state_root().lock().unwrap().clone();
                                            socket.write_all(&construct_fuzz_msg(Message::StateRoot, &state_root)).await?;
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
                                            socket.write_all(&construct_fuzz_msg(Message::StateRoot, &state_root)).await?;
                                            log::info!("SetState - same state root {}", hex::encode(state_root));
                                            continue;
                                        },
                                    }

                                    set_global_state(global_state.clone());
                                    let state_root = merkle_state(&utils::serialization::serialize(&global_state).map, 0);
                                    state_handler::set_state_root(state_root.clone());
                                    socket.write_all(&construct_fuzz_msg(Message::StateRoot, &state_root)).await?;
                                    log::info!("SetState - state root {}", hex::encode(state_root));
                                },
                                Message::ImportBlock => {

                                    log::info!("ImportBlock frame received");

                                    let block = match Block::decode(&mut reader) {
                                        Ok(block) => block,
                                        Err(error) => {
                                            log::error!("Failed to decode block: {:?}", error);
                                            let state_root = get_state_root().lock().unwrap().clone();
                                            socket.write_all(&construct_fuzz_msg(Message::StateRoot, &state_root)).await?;
                                            log::info!("SetState - same state root {}", hex::encode(state_root));
                                            continue;
                                        },
                                    };

                                    let header_hash = sp_core::blake2_256(&(block.header.encode()));
                                    log::info!("Header hash: 0x{}", hex::encode(header_hash));
                                    
                                    match state_controller::stf(&block) {
                                        Ok(_) => {
                                            //log::info!("Block proccessed successfully");
                                        },
                                        Err(error) => {
                                            log::error!("Bad block: {:?}", error);
                                        },
                                    }
                                    let state_root = get_state_root().lock().unwrap().clone();
                                    log::info!("SetState - state root {}", hex::encode(state_root));
                                    socket.write_all(&construct_fuzz_msg(Message::StateRoot, &state_root)).await?;
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
                                    socket.write_all(&state_frame).await?;
                                },
                                _ => {
                                    log::error!("Message type not supported: {:?}", msg_type);
                                    return Err(Box::new(ReadError::InvalidData));
                                 },
                            };
                        }
                        Err(error) => {
                            log::error!("Error reading fuzzer data: {}", error);
                            return Err(Box::new(error));
                        }
                    }
                }
            }
            Err(error) => {
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

pub async fn run_fuzzer(path: &str) -> Result<(), Box<dyn std::error::Error>> {

    let vinwolf_info = &*VINWOLF_INFO;

    let mut socket = UnixStream::connect(path).await?;

    let peer_info_len = vinwolf_info.name.len() + 7 + 1; // OJO con esto !!!

    let msg = [
        (peer_info_len as u32).encode(), 
        vec![Message::PeerInfo as u8],
        vinwolf_info.encode(),
    ].concat();

    socket.write_all(&msg).await?;

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    let mut buffer = [0u8; 1024];
    // Read
    let n = socket.read(&mut buffer).await?;
    let test_content = utils::common::read_bin_file(std::path::Path::new("/home/bernar/workspace/jam-stuff/fuzz-reports/0.6.7/traces/1755530300/00000004.bin")).unwrap();
    let mut reader = BytesReader::new(&test_content);
    let pre_state_root = OpaqueHash::decode(&mut reader).unwrap();
    let pre_keyvals = Vec::<KeyValue>::decode_len(&mut reader).unwrap();
    let old_block = Block::decode(&mut reader).unwrap();
    let post_state_root = OpaqueHash::decode(&mut reader).unwrap();
    let post_keyvals = Vec::<KeyValue>::decode_len(&mut reader).unwrap();  

    let test_content = utils::common::read_bin_file(std::path::Path::new("/home/bernar/workspace/jam-stuff/fuzz-reports/0.6.7/traces/1755530300/00000005.bin")).unwrap();
    let mut reader = BytesReader::new(&test_content);

    let pre_state_root = OpaqueHash::decode(&mut reader).unwrap();
    let pre_keyvals = Vec::<KeyValue>::decode_len(&mut reader).unwrap();
    let block = Block::decode(&mut reader).unwrap();
    let post_state_root = OpaqueHash::decode(&mut reader).unwrap();
    let post_keyvals = Vec::<KeyValue>::decode_len(&mut reader).unwrap();    

    let set_state = SetState {
        header: old_block.header.clone(),
        state: pre_keyvals,
    };

    let set_state_msg = [vec![2], set_state.encode()].concat();
    let msg = [(set_state_msg.len() as u32).encode(), set_state_msg].concat();

    socket.write_all(&msg).await?;
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    let import_block_msg = [vec![1], block.encode()].concat();
    let msg = [(import_block_msg.len() as u32).encode(), import_block_msg].concat();

    socket.write_all(&msg).await?;


    tokio::time::sleep(std::time::Duration::from_millis(50000)).await;
    Ok(())
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

    use dotenv::dotenv;
    dotenv().ok();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).init();
    
    let test_content = utils::common::read_bin_file(std::path::Path::new("/home/bernar/workspace/jam-stuff/fuzz-proto/examples/2_set_state.bin")).unwrap();
    let mut reader = BytesReader::new(&test_content);

    let header = Header::decode(&mut reader).unwrap();
    let keyvals = Vec::<KeyValue>::decode_len(&mut reader).unwrap();
}