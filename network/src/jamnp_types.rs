use jam_types::{*};

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