use std::array::from_fn;

use crate::types::{ValidatorsData, ValidatorData, BandersnatchPublic, Ed25519Public, BlsPublic, Metadata, Safrole};
use crate::blockchain::state::{get_validators, get_disputes};
use crate::utils::common::set_offenders_null;

impl Default for ValidatorsData {
    fn default() -> Self {
        ValidatorsData {
            0: Box::new(from_fn(|_| ValidatorData {
                bandersnatch: [0u8; std::mem::size_of::<BandersnatchPublic>()],
                ed25519: [0u8; std::mem::size_of::<Ed25519Public>()],
                bls: [0u8; std::mem::size_of::<BlsPublic>()],
                metadata: [0u8; std::mem::size_of::<Metadata>()],
            }))
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ValidatorSet {
    Previous,
    Current,
    Next,
}

pub fn key_rotation(
    safrole_state: &mut Safrole, 
    curr_validators: &mut ValidatorsData, 
    prev_validators: &mut ValidatorsData,
) { 
    *prev_validators = curr_validators.clone();
    *curr_validators = safrole_state.pending_validators.clone(); 
    // In addition to the active set of validator keys "curr_validators" and staging set "next_validators", internal to the Safrole state 
    // we retain a pending set "pending_validators". The active set is the set of keys identifying the nodes which are currently privileged 
    // to author blocks and carry out the validation processes, whereas the pending set "pending_validators", which is reset to "next_validators" 
    // at the beginning of each epoch, is the set of keys which will be active in the next epoch and which determine the Bandersnatch ring root 
    // which authorizes tickets into the sealing-key contest for the next epoch.
    safrole_state.pending_validators = get_validators(ValidatorSet::Next);   
    // The posterior queued validator key set "pending_validators" is defined such that incoming keys belonging to the offenders 
    // are replaced with a null key containing only zeroes.
    set_offenders_null(&mut safrole_state.pending_validators, &get_disputes().offenders);
}
