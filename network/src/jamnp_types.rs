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

#[derive(Debug, PartialEq)]
pub enum NetworkError {
    ReadError(ReadError),
    StreamError(StreamError),
    MessageError(MessageError),
    ConnectionError(ConnectionError),
}

#[derive(Debug, PartialEq)]
pub enum MessageError {

}

#[derive(Debug, PartialEq)]
pub enum StreamError {
    OpenStream,
    ReadStream,
    WriteStream,
}

#[derive(Debug, PartialEq)]
pub enum ConnectionError {
    OpenBidirectionalStream,
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