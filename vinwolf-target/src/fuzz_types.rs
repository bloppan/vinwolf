use std::{collections::VecDeque, str::Utf8Error};

use jam_types::*;

pub type Features = u32;

#[derive(Debug)]
pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

pub struct PeerInfo {
    pub fuzz_version: u8,
    pub fuzz_features: Features,
    pub jam_version: Version,
    pub app_version: Version,
    pub app_name: Vec<u8>,
}

pub type State = Vec<KeyValue>;

pub struct AncestryItem {
    pub slot: TimeSlot,
    pub header_hash: OpaqueHash,
}

pub type Ancestry = VecDeque<AncestryItem>;

pub struct Initialize {
    pub header: Header,
    pub keyvals: State,
    pub ancestry: Ancestry,
}

pub type GetState = HeaderHash;

pub type GetExports = HeaderHash;

pub type Error = Utf8Error;



#[derive(Debug)]
pub enum Message {
    PeerInfo = 0,
    Initialize = 1,
    StateRoot = 2,
    ImportBlock = 3,
    GetState = 4,
    State = 5,
    Error = 255,
    Unknown,
}

impl From<u8> for Message {
    fn from(value: u8) -> Self {
        match value {
            0 => Message::PeerInfo,
            1 => Message::Initialize,
            2 => Message::StateRoot,
            3 => Message::ImportBlock,
            4 => Message::GetState,
            5 => Message::State,
            255 => Message::Error,
            _ => panic!("Unknown message type: {:?}", value),
        }
    }
}