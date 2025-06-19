/*
    Preimages are static data which is presently being requested to be available for workloads to be able to 
    fetch on demand. Prior to accumulation, we must first integrate all preimages provided in the lookup extrinsic. 
 */

use std::collections::HashSet;
use crate::types::{TimeSlot, ServiceAccounts, ProcessError, PreimagesExtrinsic, PreimagesErrorCode, StateKeyType};
use crate::utils::serialization::{StateKeyTrait, construct_lookup_key};

impl PreimagesExtrinsic {

    pub fn process(
        &self, 
        services: 
        &mut ServiceAccounts, 
        post_tau: &TimeSlot) 
    -> Result<(), ProcessError> {

        // The lookup extrinsic is a sequence of pairs of service indices and data. These pairs must be ordered and 
        // without duplicates.
        let pairs = self.preimages.iter().map(|preimage| (preimage.requester, preimage.blob.clone())).collect::<Vec<_>>();
        if has_duplicates(&pairs) {
            return Err(ProcessError::PreimagesError(PreimagesErrorCode::PreimagesNotSortedOrUnique));
        }
        let pairs = pairs.iter().map(|(requester, blob)| (*requester, blob.as_slice())).collect::<Vec<_>>();
        //println!("pairs: {:x?}", pairs);
        if !is_sorted_preimages(&pairs) {
            return Err(ProcessError::PreimagesError(PreimagesErrorCode::PreimagesNotSortedOrUnique));
        }

        for preimage in self.preimages.iter() {
            let hash = sp_core::blake2_256(&preimage.blob);
            let length = preimage.blob.len() as u32;
            let lookup_key = StateKeyType::Account(preimage.requester, construct_lookup_key(&hash, length).to_vec()).construct();
            if services.contains_key(&preimage.requester) {
                let account = services.get_mut(&preimage.requester).unwrap();
                if account.preimages.contains_key(&hash) {
                    return Err(ProcessError::PreimagesError(PreimagesErrorCode::PreimageUnneeded));
                }
                if let Some(timeslots) = account.lookup.get(&lookup_key) {
                    if timeslots.len() > 0 {
                        return Err(ProcessError::PreimagesError(PreimagesErrorCode::PreimageUnneeded));
                    }
                } else {
                    return Err(ProcessError::PreimagesError(PreimagesErrorCode::PreimageUnneeded));
                }
                account.preimages.insert(hash, preimage.blob.clone());
                let timeslot_values = vec![post_tau.clone()];
                account.lookup.insert(lookup_key, timeslot_values);
            } else {
                return Err(ProcessError::PreimagesError(PreimagesErrorCode::RequesterNotFound));
            }
        }

        Ok(())
    }
}

fn has_duplicates<T: Eq + std::hash::Hash, U: Eq + std::hash::Hash>(tuples: &[(T, U)]) -> bool {
    let mut seen = HashSet::new();
    for tuple in tuples {
        if !seen.insert(tuple) {
            return true; 
        }
    }
    false
}

fn is_sorted_preimages(preimages: &[(u32, &[u8])]) -> bool {
    preimages.windows(2).all(|w| {
        let (req1, blob1) = w[0];
        let (req2, blob2) = w[1];

        if req1 < req2 {
            return true; 
        } else if req1 > req2 {
            return false; 
        }

        blob1 <= blob2
    })
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_has_duplicates() {
        let tuples = vec![(1, 2), (3, 4), (1, 2)];
        assert_eq!(has_duplicates(&tuples), true);

        let tuples = vec![(1, 2), (3, 4), (5, 6)];
        assert_eq!(has_duplicates(&tuples), false);
    }

    #[test]
    fn test_is_sorted_preimages() {
        let preimages = vec![(4, &[1u8, 2, 3][..]), (2, &[4, 5, 6]), (3, &[7, 8, 9])];
        assert_eq!(is_sorted_preimages(&preimages), false);

        let preimages = vec![(1, &[1u8, 2, 3][..]), (2, &[4, 5, 6]), (3, &[7, 8, 9])];
        assert_eq!(is_sorted_preimages(&preimages), true);

        let preimages = vec![(1, &[1, 2, 3][..]), (3, &[4, 5, 6]), (2, &[7, 8, 9])];
        assert_eq!(is_sorted_preimages(&preimages), false);

        let preimages = vec![(1, &[3][..]), (3, &[1, 5, 6]), (5, &[7, 8, 9])];
        assert_eq!(is_sorted_preimages(&preimages), true);
    }

}