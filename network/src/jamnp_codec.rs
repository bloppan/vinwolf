
use codec::{*};
use jam_types::{*};
use crate::jamnp_types::{LastBlock, Leaf, Handshake, Announcement};

impl Encode for LastBlock {

    fn encode(&self) -> Vec<u8> {
        
        let mut blob = Vec::with_capacity(std::mem::size_of::<OpaqueHash>() + std::mem::size_of::<TimeSlot>());

        self.header_hash.encode_to(&mut blob);
        self.slot.encode_to(&mut blob);

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for LastBlock {

    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(LastBlock{
            header_hash: OpaqueHash::decode(reader)?,
            slot: TimeSlot::decode(reader)?,
        })
    }
}

impl Encode for Leaf {

    fn encode(&self) -> Vec<u8> {
        
        let mut blob = Vec::with_capacity(std::mem::size_of::<OpaqueHash>() + std::mem::size_of::<TimeSlot>());

        self.header_hash.encode_to(&mut blob);
        self.slot.encode_to(&mut blob);

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for Leaf {

    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(Leaf{
            header_hash: OpaqueHash::decode(reader)?,
            slot: TimeSlot::decode(reader)?,
        })
    }
}

impl Encode for Handshake {

    fn encode(&self) -> Vec<u8> {
        
        let mut blob = Vec::with_capacity(std::mem::size_of::<Handshake>());

        self.last_block.encode_to(&mut blob);
        self.leafs.encode_len().encode_to(&mut blob);

        return blob;
    }
    
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for Handshake {

    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(Handshake{
            last_block: LastBlock::decode(reader)?,
            leafs: Vec::<Leaf>::decode_len(reader)?,
        })
    }
}

impl Encode for Announcement {

    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::with_capacity(std::mem::size_of::<Announcement>());

        self.header.encode_to(&mut blob);
        self.last_block.encode_to(&mut blob);

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for Announcement {
    
    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(Announcement{
            header: Header::decode(reader)?,
            last_block: LastBlock::decode(reader)?,
        })
    }
}


