use crate::types::{OpaqueHash, TimeSlot, RefineContext};
use crate::utils::codec::{BytesReader, Decode, DecodeLen, Encode, EncodeLen, EncodeSize, ReadError};

impl Encode for RefineContext {

    fn encode(&self) -> Vec<u8> {

        let mut refine_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<RefineContext>());  
        
        self.anchor.encode_to(&mut refine_blob);
        self.state_root.encode_to(&mut refine_blob);
        self.beefy_root.encode_to(&mut refine_blob);
        self.lookup_anchor.encode_to(&mut refine_blob);
        self.lookup_anchor_slot.encode_size(4).encode_to(&mut refine_blob);
        self.prerequisites.encode_len().encode_to(&mut refine_blob);
   
        return refine_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for RefineContext {

    fn decode(refine_blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(RefineContext {
            anchor: OpaqueHash::decode(refine_blob)?,
            state_root: OpaqueHash::decode(refine_blob)?,
            beefy_root: OpaqueHash::decode(refine_blob)?,
            lookup_anchor: OpaqueHash::decode(refine_blob)?,
            lookup_anchor_slot: TimeSlot::decode(refine_blob)?,
            prerequisites: Vec::<OpaqueHash>::decode_len(refine_blob)?,
        })
    }
}

