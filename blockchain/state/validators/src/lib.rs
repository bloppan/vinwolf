use constants::node::VALIDATORS_COUNT;
use jam_types::{Ed25519Public, Safrole, ValidatorSet, ValidatorsData, ValidatorData};
use utils::common::set_offenders_null;

pub fn key_rotation(safrole_state: &mut Safrole, 
                    curr_validators: &mut ValidatorsData, 
                    prev_validators: &mut ValidatorsData, 
                    offenders: &[Ed25519Public]
) { 
    log::info!("Key rotation");
    *prev_validators = curr_validators.clone();
    *curr_validators = safrole_state.pending_validators.clone(); 
    // In addition to the active set of validator keys "curr_validators" and staging set "next_validators", internal to the Safrole state 
    // we retain a pending set "pending_validators". The active set is the set of keys identifying the nodes which are currently privileged 
    // to author blocks and carry out the validation processes, whereas the pending set "pending_validators", which is reset to "next_validators" 
    // at the beginning of each epoch, is the set of keys which will be active in the next epoch and which determine the Bandersnatch ring root 
    // which authorizes tickets into the sealing-key contest for the next epoch.
    safrole_state.pending_validators = state_handler::validators::get(ValidatorSet::Next);   
    // The posterior queued validator key set "pending_validators" is defined such that incoming keys belonging to the offenders 
    // are replaced with a null key containing only zeroes.
    set_offenders_null(&mut safrole_state.pending_validators, offenders); 
}

pub fn extract_keys<T: Clone, F: Fn(&ValidatorData) -> T>(validators: &ValidatorsData, selector: F) -> Box<[T; VALIDATORS_COUNT]> {
    Box::new(std::array::from_fn(|i| selector(&validators.list[i]).clone()))
}
