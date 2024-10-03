use serde::Deserialize;
use crate::codec::*;

use crate::types::*;

#[allow(non_camel_case_types)]
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct RefineContext {
    pub anchor: OpaqueHash,
    pub state_root: OpaqueHash,
    pub beefy_root: OpaqueHash,
    pub lookup_anchor: OpaqueHash,
    pub lookup_anchor_slot: TimeSlot,
    pub prerequisite: Option<OpaqueHash>,
}

impl RefineContext {
    pub fn decode(blob: &mut SliceReader) -> Result<Self, ReadError> {
        let anchor = blob.read::<OpaqueHash>()?;
        let state_root = blob.read::<OpaqueHash>()?;
        let beefy_root = blob.read::<OpaqueHash>()?;
        let lookup_anchor = blob.read::<OpaqueHash>()?;
        let lookup_anchor_slot = blob.read::<TimeSlot>()?;
        let prerequisite = if blob.read::<u8>()? != 0 {
            Some(blob.read::<OpaqueHash>()?)
        } else {
            None
        };
        Ok(RefineContext {
            anchor,
            state_root,
            beefy_root,
            lookup_anchor,
            lookup_anchor_slot,
            prerequisite,
        })
    }
    
    const MIN_SIZE: usize = 133;

    pub fn encode(&self) -> Result<Vec<u8>, ReadError> {

        if self.len() < Self::MIN_SIZE {
            return Err(ReadError::NotEnoughData);
        }

        let mut refine_blob: Vec<u8> = Vec::with_capacity(Self::MIN_SIZE);  // 133 fixed bytes 
        refine_blob.extend_from_slice(&self.anchor);
        refine_blob.extend_from_slice(&self.state_root);
        refine_blob.extend_from_slice(&self.beefy_root);
        refine_blob.extend_from_slice(&self.lookup_anchor);
        refine_blob.extend_from_slice(&self.lookup_anchor_slot.to_le_bytes());

        if let Some(prereq) = &self.prerequisite {
            refine_blob.extend_from_slice(prereq);
        } else {
            refine_blob.push(0u8);
        }
    
        Ok(refine_blob)
    }

    pub fn len(&self) -> usize {
        let base_size = 32 * 4 + 4; 
        let prerequisite_size = if self.prerequisite.is_some() { 32 + 1 } else { 1 };
        base_size + prerequisite_size
    }
}

