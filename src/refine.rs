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

pub fn decode_refine_ctx(refine_blob: &Vec<u8>) -> RefineContext {

    let mut anchor = [0u8; 32];
    anchor.copy_from_slice(&refine_blob[0..32]);
    let mut state_root = [0u8; 32];
    state_root.copy_from_slice(&refine_blob[32..64]);
    let mut beefy_root = [0u8; 32];
    beefy_root.copy_from_slice(&refine_blob[64..96]);
    let mut lookup_anchor = [0u8; 32];
    lookup_anchor.copy_from_slice(&refine_blob[96..128]);
    let mut lookup_anchor_slot: u32 = decode_trivial(&refine_blob[128..132].to_vec()) as u32;
    let prerequisite = if refine_blob.len() >= 164 {
        let mut prereq = [0u8; 32];
        prereq.copy_from_slice(&refine_blob[132..164]);
        Some(prereq)
    } else {
        None
    };

    /*println!("anchor: {:02x?}", anchor);
    println!("state_root: {:02x?}", state_root);
    println!("beefy_root: {:02x?}", beefy_root);
    println!("lookup_anchor: {:02x?}", lookup_anchor);
    println!("lookup_anchor_slot: {:02x?}", lookup_anchor_slot);
    println!("prerequisite: {:?}", prerequisite);*/

    RefineContext {
        anchor,
        state_root,
        beefy_root,
        lookup_anchor,
        lookup_anchor_slot,
        prerequisite
    }
}

pub fn encode_refine_ctx(refine_ctx: &RefineContext) -> Vec<u8> {

    let mut refine_blob: Vec<u8> = vec![];

    refine_blob.extend_from_slice(&refine_ctx.anchor);
    refine_blob.extend_from_slice(&refine_ctx.state_root);
    refine_blob.extend_from_slice(&refine_ctx.beefy_root);
    refine_blob.extend_from_slice(&refine_ctx.lookup_anchor);
    refine_blob.extend_from_slice(&encode_trivial(refine_ctx.lookup_anchor_slot as usize, 4));

    if let Some(prereq) = &refine_ctx.prerequisite {
        refine_blob.extend_from_slice(prereq);
    } else {
        refine_blob.extend_from_slice(&encode_general(0));
    }

    return refine_blob;
}