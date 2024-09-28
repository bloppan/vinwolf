use serde::Deserialize;
use crate::codec::*;

#[allow(non_camel_case_types)]
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct RefineContext {
    pub anchor: [u8; 32],
    pub state_root: [u8; 32],
    pub beefy_root: [u8; 32],
    pub lookup_anchor: [u8; 32],
    pub lookup_anchor_slot: u32,
    pub prerequisite: Option<[u8; 32]>,
}

impl RefineContext {
    pub fn decode(refine_blob: &[u8]) -> Result<Self, ReadError> {
        let mut blob = SliceReader::new(refine_blob);
        let anchor = blob.read_32bytes()?;
        let state_root = blob.read_32bytes()?;
        let beefy_root = blob.read_32bytes()?;
        let lookup_anchor = blob.read_32bytes()?;
        let lookup_anchor_slot = blob.read_u32()?;
        let prerequisite = if blob.read_next_byte()? != 0 {
            Some(blob.read_32bytes()?)
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
            refine_blob.extend_from_slice(&(0u8).encode());
        }
    
        Ok(refine_blob)
    }

    pub fn len(&self) -> usize {
        let base_size = 32 * 4 + 4; 
        let prerequisite_size = if self.prerequisite.is_some() { 33 } else { 1 };
        base_size + prerequisite_size
    }
}

