use sp_core::blake2_256;
use once_cell::sync::Lazy;
use std::{collections::HashMap, sync::Mutex};

use crate::types::{Account, OpaqueHash, OutputPreimages, PreimagesErrorCode, PreimagesExtrinsic, ServiceAccounts, Services, TimeSlot};
use crate::blockchain::state::ProcessError;

static SERVICES_STATE: Lazy<Mutex<Services>> = Lazy::new(|| Mutex::new(Services{0: Vec::new()}));

pub fn set_services_state(post_state: &Services) {
    let mut state = SERVICES_STATE.lock().unwrap();
    *state = post_state.clone();
}

pub fn get_services_state() -> Services {
    let state = SERVICES_STATE.lock().unwrap(); 
    return state.clone();
}

pub fn process_services(
    services: &mut ServiceAccounts, 
    post_tau: &TimeSlot, 
    preimages_extrinsic: &PreimagesExtrinsic
) -> Result<OutputPreimages, ProcessError> {

    if preimages_extrinsic.preimages.len() == 0 {
        return Ok(OutputPreimages::Ok());
    }

    preimages_extrinsic.process()?;

    for preimage in preimages_extrinsic.preimages.iter() {
        let hash = blake2_256(&preimage.blob);
        let length = preimage.blob.len() as u32;
        if services.service_accounts.contains_key(&preimage.requester) {
            let account = services.service_accounts.get_mut(&preimage.requester).unwrap();
            if account.preimages.contains_key(&hash) {
                return Err(ProcessError::PreimagesError(PreimagesErrorCode::PreimageUnneeded));
            }
            if let Some(timeslots) = account.lookup.get(&(hash, length)) {
                if timeslots.len() > 0 {
                    return Err(ProcessError::PreimagesError(PreimagesErrorCode::PreimageUnneeded));
                }
            } else {
                return Err(ProcessError::PreimagesError(PreimagesErrorCode::PreimageUnneeded));
            }
            account.preimages.insert(hash, preimage.blob.clone());
            let timeslot_values = vec![post_tau.clone()];
            account.lookup.insert((hash, length), timeslot_values);
        } else {
            return Err(ProcessError::PreimagesError(PreimagesErrorCode::RequesterNotFound));
        }
    }

    Ok(OutputPreimages::Ok())
}


