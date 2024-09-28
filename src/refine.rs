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

    pub fn decode(refine_blob: &[u8]) -> Self {
        let anchor = refine_blob[0..32].try_into().expect("slice with incorrect length");
        let state_root = refine_blob[32..64].try_into().expect("slice with incorrect length");
        let beefy_root = refine_blob[64..96].try_into().expect("slice with incorrect length");
        let lookup_anchor = refine_blob[96..128].try_into().expect("slice with incorrect length");
        let lookup_anchor_slot = decode_trivial(&refine_blob[128..132]) as u32;
        let prerequisite = if refine_blob[132] != 0 {
            Some(refine_blob[132..164].try_into().expect("slice with incorrect length"))
        } else {
            None
        };

        RefineContext {
            anchor,
            state_root,
            beefy_root,
            lookup_anchor,
            lookup_anchor_slot,
            prerequisite,
        }
    }

    pub fn encode(&self) -> Vec<u8> {

        let mut refine_blob: Vec<u8> = Vec::with_capacity(164);  // 132 fixed bytes + 32 optional (prerequisite)
    
        refine_blob.extend_from_slice(&self.anchor);
        refine_blob.extend_from_slice(&self.state_root);
        refine_blob.extend_from_slice(&self.beefy_root);
        refine_blob.extend_from_slice(&self.lookup_anchor);
        refine_blob.extend_from_slice(&encode_trivial(self.lookup_anchor_slot as usize, 4));
    
        if let Some(prereq) = &self.prerequisite {
            refine_blob.extend_from_slice(prereq);
        } else {
            refine_blob.extend_from_slice(&(0u8).encode());
        }
    
        refine_blob
    }
}

