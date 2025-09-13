use std::collections::VecDeque;

use jam_types::*;
use super::fuzz_types::*;
use codec::{Encode, Decode, EncodeLen, DecodeLen, BytesReader};
use codec::generic_codec::{encode_unsigned, decode_unsigned};

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

impl Encode for Message {

    fn encode(&self) -> Vec<u8> {
        
        let mut blob = vec![];

        match self {

            Message::PeerInfo => blob.push(0),
            Message::Initialize => blob.push(1),
            Message::StateRoot => blob.push(2),
            Message::ImportBlock => blob.push(3),
            Message::GetState => blob.push(4),
            Message::State => blob.push(5),
            Message::Error => blob.push(255),
            _ => { println!("Unknown message type: {:?}", self); },
        };

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for Message {

    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        
        let msg_type = reader.read_byte()?;

        let msg = match msg_type {

            0 => Message::PeerInfo,
            1 => Message::Initialize,
            2 => Message::StateRoot,
            3 => Message::ImportBlock,
            4 => Message::GetState,
            5 => Message::State,
            255 => Message::Error,
            _ => Message::Unknown,
        };

        return Ok(msg);
    }
}

impl Encode for PeerInfo {

    fn encode(&self) -> Vec<u8> {
        
        let mut blob = vec![];

        self.fuzz_version.encode_to(&mut blob);
        self.fuzz_features.encode_to(&mut blob);
        self.jam_version.encode_to(&mut blob);
        self.app_version.encode_to(&mut blob);
        self.app_name.encode_len().encode_to(&mut blob);

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for PeerInfo {

    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        
        Ok(PeerInfo { 
            fuzz_version: u8::decode(reader)?,
            fuzz_features: Features::decode(reader)?,
            jam_version: Version::decode(reader)?,
            app_version: Version::decode(reader)?, 
            app_name: Vec::<u8>::decode_len(reader)?,
        })
    }
}

impl Encode for AncestryItem {

    fn encode(&self) -> Vec<u8> {
        
        let mut blob = vec![];

        self.slot.encode_to(&mut blob);
        self.header_hash.encode_to(&mut blob);

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for AncestryItem {

    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        
        Ok(AncestryItem { 
            slot: TimeSlot::decode(reader)?, 
            header_hash: OpaqueHash::decode(reader)? 
        })
    }
}

impl Encode for Initialize {

    fn encode(&self) -> Vec<u8> {
        
        let mut blob = vec![];

        self.header.encode_to(&mut blob);
        self.keyvals.encode_len().encode_to(&mut blob);

        encode_unsigned(self.ancestry.len()).encode_to(&mut blob);
        for item in self.ancestry.iter() {
            item.encode_to(&mut blob);
        }

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for Initialize {

    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        
        Ok(Initialize { 
            header: Header::decode(reader)?, 
            keyvals: Vec::<KeyValue>::decode_len(reader)?, 
            ancestry: {
                let mut ancestry_vec = VecDeque::new();
                let ancestry_len = decode_unsigned(reader)?;
                for _ in 0..ancestry_len {  
                    let ancestry = AncestryItem::decode(reader)?;
                    ancestry_vec.push_back(ancestry);
                }
                ancestry_vec
            } 
        })
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use std::sync::LazyLock;
    use std::path::PathBuf;
    use utils::log;
    
    static FUZZER_TESTS_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
        let crate_root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let workspace_root = PathBuf::from(crate_root)
            .parent()
            .unwrap()
            .to_path_buf();

        workspace_root.join("external/jam-conformance/fuzz-proto/examples/v1")
    });

    #[test]
    fn all_fuzzer_test() {

        log::Builder::from_env(log::Env::default().default_filter_or("debug"))
        .with_dotenv(true)
        .init();

        let dir = std::fs::read_dir(&*FUZZER_TESTS_PATH).unwrap();
        let bin_files: Vec<PathBuf> = dir.filter_map(|f| {
            let f = f.ok()?.path();
            if f.extension()? == "bin" {
                return Some(f);
            }
            None
        })
        .collect();
        
        for file in bin_files.iter() {
            log::info!("file: {:?}", file);
            let test_content = utils::common::read_bin_file(file).unwrap();
            let mut reader = BytesReader::new(&test_content);
            let msg_type: Message = reader.read_byte().unwrap().into();

            match msg_type {

                Message::PeerInfo => {
                    let peer_info_decoded = PeerInfo::decode(&mut reader).unwrap();
                    let peer_info_encoded = [Message::PeerInfo.encode(), peer_info_decoded.encode()].concat();
                    assert_eq!(test_content, peer_info_encoded);
                },
                Message::Initialize => {
                    let initialize_decoded = Initialize::decode(&mut reader).unwrap();
                    let initialize_encoded = [Message::Initialize.encode(), initialize_decoded.encode()].concat();
                    assert_eq!(test_content, initialize_encoded);
                },
                Message::StateRoot => {
                    let state_root_decoded = StateRoot::decode(&mut reader).unwrap();
                    let state_root_encoded = [Message::StateRoot.encode(), state_root_decoded.encode()].concat();
                    assert_eq!(test_content, state_root_encoded);
                },
                Message::ImportBlock => {
                    let block_decoded = Block::decode(&mut reader).unwrap();
                    let block_encoded = [Message::ImportBlock.encode(), block_decoded.encode()].concat();
                    assert_eq!(test_content, block_encoded);
                },
                Message::GetState => {
                    let get_state_decoded = GetState::decode(&mut reader).unwrap();
                    let get_state_encoded = [Message::GetState.encode(), get_state_decoded.encode()].concat();
                    assert_eq!(test_content, get_state_encoded);
                },
                Message::State => {
                    let state_decoded = State::decode_len(&mut reader).unwrap();
                    let state_encoded = [Message::State.encode(), state_decoded.encode_len()].concat();
                    //assert_eq!(test_content, state_encoded);
                },
                Message::Error => {
                    let error_decoded = Vec::<u8>::decode_len(&mut reader).unwrap();
                    let error_encoded = [Message::Error.encode(), error_decoded.encode_len()].concat();
                    assert_eq!(test_content, error_encoded);
                    log::error!("{:?}", String::from_utf8(error_decoded).unwrap())
                },
                _ => {
                    log::error!("Message type {:?} not supported", msg_type);
                },
            };
        }
    }
}
