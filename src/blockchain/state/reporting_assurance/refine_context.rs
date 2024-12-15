use crate::types::{OpaqueHash, TimeSlot};
use crate::utils::codec::{BytesReader, Encode, EncodeSize, Decode, ReadError};
use crate::utils::codec::{encode_unsigned, decode_unsigned};

// A refinement context, denoted by the set X, describes the context of the chain at the point 
// that the report’s corresponding work-package was evaluated. It identifies two historical blocks, 
// the anchor, header hash a along with its associated posterior state-root s and posterior Beefy root b; 
// and the lookupanchor, header hash l and of timeslot t. Finally, it identifies the hash of an optional 
// prerequisite work-package p.




#[derive(Debug, Clone, PartialEq)]
pub struct RefineContext {
    pub anchor: OpaqueHash,
    pub state_root: OpaqueHash,
    pub beefy_root: OpaqueHash,
    pub lookup_anchor: OpaqueHash,
    pub lookup_anchor_slot: TimeSlot,
    pub prerequisites: Vec<OpaqueHash>,
}

impl Encode for RefineContext {

    fn encode(&self) -> Vec<u8> {

        let mut refine_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<RefineContext>());  
        
        self.anchor.encode_to(&mut refine_blob);
        self.state_root.encode_to(&mut refine_blob);
        self.beefy_root.encode_to(&mut refine_blob);
        self.lookup_anchor.encode_to(&mut refine_blob);
        self.lookup_anchor_slot.encode_size(4).encode_to(&mut refine_blob);

        encode_unsigned(self.prerequisites.len()).encode_to(&mut refine_blob);
        for prerequisite in &self.prerequisites {
            prerequisite.encode_to(&mut refine_blob);
        }
   
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
            prerequisites: {
                let len = decode_unsigned(refine_blob)?;
                let mut prereqs_vec = Vec::with_capacity(len);
                for _ in 0..len {
                    prereqs_vec.push(OpaqueHash::decode(refine_blob)?);
                }
                prereqs_vec
            },
        })
    }
}

