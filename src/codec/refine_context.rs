use crate::codec::{BytesReader, Encode, EncodeSize, Decode, ReadError};
use crate::types::{OpaqueHash, TimeSlot};

// A refinement context, denoted by the set X, describes the context of the chain at the point 
// that the report’s corresponding work-package was evaluated. It identifies two historical blocks, 
// the anchor, header hash a along with its associated posterior state-root s and posterior Beefy root b; 
// and the lookupanchor, header hash l and of timeslot t. Finally, it identifies the hash of an optional 
// prerequisite work-package p.

pub struct RefineContext {
    pub anchor: OpaqueHash,
    pub state_root: OpaqueHash,
    pub beefy_root: OpaqueHash,
    pub lookup_anchor: OpaqueHash,
    pub lookup_anchor_slot: TimeSlot,
    pub prerequisite: Option<OpaqueHash>,
}

impl Decode for RefineContext {

    fn decode(refine_blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(RefineContext {
            anchor: OpaqueHash::decode(refine_blob)?,
            state_root: OpaqueHash::decode(refine_blob)?,
            beefy_root: OpaqueHash::decode(refine_blob)?,
            lookup_anchor: OpaqueHash::decode(refine_blob)?,
            lookup_anchor_slot: TimeSlot::decode(refine_blob)?,
            prerequisite: if u8::decode(refine_blob)? != 0 {
                Some(OpaqueHash::decode(refine_blob)?)
            } else {
                None
            },
        })
    }
}

impl Encode for RefineContext {

    fn encode(&self) -> Vec<u8> {

        let mut refine_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<RefineContext>());  
        
        self.anchor.encode_to(&mut refine_blob);
        self.state_root.encode_to(&mut refine_blob);
        self.beefy_root.encode_to(&mut refine_blob);
        self.lookup_anchor.encode_to(&mut refine_blob);
        self.lookup_anchor_slot.encode_size(4).encode_to(&mut refine_blob);

        if let Some(prereq) = &self.prerequisite {
            prereq.encode_to(&mut refine_blob);
        } else {
            0_u8.encode_to(&mut refine_blob);
        }
    
        return refine_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

