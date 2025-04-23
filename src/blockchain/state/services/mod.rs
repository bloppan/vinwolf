use sp_core::blake2_256;

use crate::types::{OpaqueHash, OutputPreimages, PreimagesErrorCode, PreimagesExtrinsic, ServiceAccounts, Account, TimeSlot, PreimageData, Balance};
use crate::constants::{MIN_BALANCE, MIN_BALANCE_PER_ITEM, MIN_BALANCE_PER_OCTET};
use crate::blockchain::state::ProcessError;
use crate::utils::codec::{BytesReader, ReadError};
use crate::utils::codec::generic::decode_unsigned;

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

impl Account {
    pub fn get_footprint_and_threshold(&self) -> (u32, u64, Balance) {

        let items: u32 = 2 * self.lookup.len() as u32 + self.storage.len() as u32;

        let mut octets: u64 = 0;
        for ((_hash, length), _timeslot) in self.lookup.iter() {
            octets += 81 + *length as u64;
        }
        for (_hash, storage_data) in self.storage.iter() {
            octets += 32 + storage_data.len() as u64;
        }

        let threshold = MIN_BALANCE + items as Balance * MIN_BALANCE_PER_ITEM + octets as Balance * MIN_BALANCE_PER_OCTET;

        return (items, octets, threshold);
    }   
}

pub fn decode_preimage(preimage: &[u8]) -> Result<PreimageData, ReadError> {
    
    let mut preimage_reader = BytesReader::new(preimage);
    let metadata_len = decode_unsigned(&mut preimage_reader)?;
    let metadata = preimage_reader.read_bytes(metadata_len as usize)?.to_vec();
    let preimage_len = preimage_reader.data.len(); 
    let code = preimage_reader.read_bytes(preimage_len - metadata_len as usize - 1)?.to_vec();

    Ok(PreimageData {
        metadata,
        code,
    })
}

pub fn historical_preimage_lookup(account: &Account, slot: &TimeSlot, hash: &OpaqueHash) -> Option<Vec<u8>> {

    if let Some(preimage) = account.preimages.get(hash) {
        let length = preimage.len() as u32;
        if let Some(timeslot_record) = account.lookup.get(&(*hash, length)) {
            if check_preimage_availability(timeslot_record, slot) {
                return Some(preimage.clone());
            }
        }
    }

    return None;
}

fn check_preimage_availability(timeslot_record: &[TimeSlot], slot: &TimeSlot) -> bool {
 
    if timeslot_record.len() == 0 {
        return false;
    } else if timeslot_record.len() == 1 {
        if timeslot_record[0] <= *slot {
            return true;
        }
    } else if timeslot_record.len() == 2 {
        if timeslot_record[0] <= *slot && *slot < timeslot_record[1] {
            return true;
        }
    } else if timeslot_record.len() == 3 {
        if (timeslot_record[0] <= *slot && *slot < timeslot_record[1]) || (timeslot_record[2] <= *slot) {
            return true;
        }
    }
 
    return false;
}
