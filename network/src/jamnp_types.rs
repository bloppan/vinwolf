use jam_types::{*};

#[derive(Debug)]
pub struct LastBlock {
    pub header_hash: OpaqueHash,
    pub slot: TimeSlot,
}

#[derive(Debug)]
pub struct Leaf {
    pub header_hash: OpaqueHash,
    pub slot: TimeSlot,
}

#[derive(Debug)]
pub struct Handshake {
    pub last_block: LastBlock,
    pub leafs: Vec<Leaf>,
}

#[derive(Debug)]
pub struct Announcement {
    pub header: Header,
    pub last_block: LastBlock,
}

impl Default for LastBlock {
    fn default() -> Self {
        LastBlock { 
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
            last_block: LastBlock::default(), 
            leafs: Vec::new() 
        }
    }
}

impl Default for Announcement {
    fn default() -> Self {
        Announcement { 
            header: Header::default(), 
            last_block: LastBlock::default() 
        }
    }
}