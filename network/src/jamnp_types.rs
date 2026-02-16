use jam_types::{*};

pub type StreamKind = u8;

#[derive(Debug, Clone, PartialEq)]
pub struct LastFinalizedBlock {
    pub header_hash: OpaqueHash,
    pub slot: TimeSlot,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Leaf {
    pub header_hash: OpaqueHash,
    pub slot: TimeSlot,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Handshake {
    pub last_finalized_block: LastFinalizedBlock,
    pub leafs: Vec<Leaf>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Announcement {
    pub header: Header,
    pub last_finalized_block: LastFinalizedBlock,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportedBlocks {
    pub last_finalized_block: LastFinalizedBlock,
    pub leafs: Vec<Leaf>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TicketDistributed {
    pub epoch: TimeSlot,
    pub ticket: Ticket,
} 

#[derive(Debug)]
pub enum NetworkError {
    Decode(ReadError),
    Stream(StreamError),
    Connection(ConnectionError),
}

#[derive(Debug)]
pub enum StreamError {
    Read(String),
    Write(String),
}

#[derive(Debug)]
pub enum ConnectionError {
    Open(String),
    Connect(String),
}

impl From<ReadError> for NetworkError {
    fn from(e: ReadError) -> Self {
        NetworkError::Decode(e)
    }
}

impl From<quinn::ConnectionError> for NetworkError {
    fn from(e: quinn::ConnectionError) -> Self {
        NetworkError::Connection(ConnectionError::Open(e.to_string()))
    }
}

impl From<quinn::ConnectError> for NetworkError {
    fn from(e: quinn::ConnectError) -> Self {
        NetworkError::Connection(ConnectionError::Connect(e.to_string()))
    }
}

impl From<quinn::ReadExactError> for NetworkError {
    fn from(e: quinn::ReadExactError) -> Self {
        NetworkError::Stream(StreamError::Read(e.to_string()))
    }
}

impl From<quinn::WriteError> for NetworkError {
    fn from(e: quinn::WriteError) -> Self {
        NetworkError::Stream(StreamError::Write(e.to_string()))
    }
}

impl From<quinn::ClosedStream> for NetworkError {
    fn from(e: quinn::ClosedStream) -> Self {
        NetworkError::Stream(StreamError::Write(e.to_string()))
    }
}

impl Default for ImportedBlocks {
    fn default() -> Self {
        ImportedBlocks { 
            last_finalized_block: LastFinalizedBlock::default(), 
            leafs: Vec::new(), 
        }
    }
}

impl Default for LastFinalizedBlock {
    fn default() -> Self {
        LastFinalizedBlock { 
            header_hash: OpaqueHash::default(), 
            slot: TimeSlot::default() 
        }
    }
}

impl Default for Leaf {
    fn default() -> Self {
        Leaf {
            header_hash: OpaqueHash::default(),
            slot: TimeSlot::default(),
        }
    }
}

impl Default for Handshake {
    fn default() -> Self {
        Handshake { 
            last_finalized_block: LastFinalizedBlock::default(), 
            leafs: Vec::new() 
        }
    }
}

impl Default for Announcement {
    fn default() -> Self {
        Announcement { 
            header: Header::default(), 
            last_finalized_block: LastFinalizedBlock::default() 
        }
    }
}