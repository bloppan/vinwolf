
use crate::types::{OpaqueHash, OutputPreimages, PreimagesExtrinsic, ServiceAccounts, Account, TimeSlot, PreimageData, Balance};
use crate::constants::{MIN_BALANCE, MIN_BALANCE_PER_ITEM, MIN_BALANCE_PER_OCTET};
use crate::blockchain::state::ProcessError;
use crate::utils::codec::{BytesReader, ReadError};
use crate::utils::codec::generic::decode_unsigned;
use crate::utils::serialization::construct_lookup_key;

pub fn process(
    services: &mut ServiceAccounts, 
    post_tau: &TimeSlot, 
    preimages_extrinsic: &PreimagesExtrinsic
) -> Result<OutputPreimages, ProcessError> {

    if preimages_extrinsic.preimages.len() == 0 {
        return Ok(OutputPreimages::Ok());
    }

    preimages_extrinsic.process(services, post_tau)?;

    Ok(OutputPreimages::Ok())
}

impl Account {
    pub fn get_footprint_and_threshold(&self) -> (u32, u64, Balance) {

        let items: u32 = 2 * self.lookup.len() as u32 + self.storage.len() as u32;

        let mut octets: u64 = 0;
        for (lookup_key, _timeslot) in self.lookup.iter() {
            let length = u32::from_le_bytes([lookup_key[1], lookup_key[3], lookup_key[5], lookup_key[7]]);
            octets += 81 + length as u64;
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
        if let Some(timeslot_record) = account.lookup.get(&construct_lookup_key(hash, length)) {
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

#[cfg(test)]
mod tests {

    use super::*;
    use crate::types::{PreimagesErrorCode, Preimage};

    #[test]
    fn test_preimages_extrinsic_process() {
        let mut services = ServiceAccounts::default();
        let slot = TimeSlot::default();

        let preimages = vec![
            Preimage { requester: 0, blob: vec![1, 2, 3] },
            Preimage { requester: 1, blob: vec![4, 5, 6] },
            Preimage { requester: 2, blob: vec![7, 8, 9] },
        ];
        let preimages_extrinsic = PreimagesExtrinsic { preimages };
        assert_eq!(preimages_extrinsic.process(&mut services, &slot), Err(ProcessError::PreimagesError(PreimagesErrorCode::RequesterNotFound)));

        let preimages = vec![
            Preimage { requester: 0, blob: vec![1, 2, 3] },
            Preimage { requester: 1, blob: vec![4, 5, 6] },
            Preimage { requester: 0, blob: vec![7, 8, 9] },
        ];
    
        let preimages_extrinsic = PreimagesExtrinsic { preimages };
        assert_eq!(preimages_extrinsic.process(&mut services, &slot), Err(ProcessError::PreimagesError(PreimagesErrorCode::PreimagesNotSortedOrUnique)));
    }

}