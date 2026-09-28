/*
    Preimages are static data which is presently being requested to be available for workloads to be able to fetch on demand. 
    Prior to accumulation, we must first integrate all preimages provided in the lookup extrinsic.
 
    The lookup extrinsic is VALIDATED against the prior service accounts state (every pair must be solicited but not yet provided) 
    and INTEGRATED after accumulation into the posterior state with the same function used for the `provide` host call provisions. 
    Pairs which are no longer useful because of the effects of accumulation (request forgotten, service ejected, blob already provided) 
    are disregarded without prejudice.
 */

use codec::{BytesReader, DecodeLen, EncodeLen};
use jam_types::*;
use serialization::{construct_lookup_key, construct_preimage_key, StateKeyTrait};
use std::collections::HashSet;
use tools::log;

pub fn providable(services: &ServiceAccounts, service_id: &ServiceId, blob: &[u8]) -> bool {

    let account = match services.get(service_id) {
        Some(account) => account,
        None => return false,
    };
    let hash = sp_core::blake2_256(blob);
    let lookup_key = StateKeyType::Account(*service_id, construct_lookup_key(&hash, blob.len() as u32)).construct();

    match account.storage.get(&lookup_key) {
        Some(timeslots_blob) => match Vec::<TimeSlot>::decode_len(&mut BytesReader::new(timeslots_blob)) {
            Ok(timeslots) => timeslots.is_empty(),
            Err(_) => false,
        },
        None => false,
    }
}

pub fn validate(preimages_extrinsic: &[Preimage], services: &ServiceAccounts) -> Result<(), ImportError> {

    // The lookup extrinsic is a sequence of pairs of service indices and data. These pairs must be ordered and 
    // without duplicates.
    let pairs = preimages_extrinsic.iter().map(|preimage| (preimage.requester, preimage.blob.clone())).collect::<Vec<_>>();
    if has_duplicates(&pairs) {
        log::error!("Preimages has duplicates");
        return Err(ImportError::PreimagesError(PreimagesErrorCode::PreimagesNotSortedOrUnique));
    }
    let pairs = pairs.iter().map(|(requester, blob)| (*requester, blob.as_slice())).collect::<Vec<_>>();
    if !is_sorted_preimages(&pairs) {
        log::error!("Preimages not sorted");
        return Err(ImportError::PreimagesError(PreimagesErrorCode::PreimagesNotSortedOrUnique));
    }

    // The data must have been solicited by a service but not yet provided in the PRIOR state.
    for preimage in preimages_extrinsic {
        if !services.contains_key(&preimage.requester) {
            log::error!("Requester {:?} not found", preimage.requester);
            return Err(ImportError::PreimagesError(PreimagesErrorCode::RequesterNotFound));
        }
        if !providable(services, &preimage.requester, &preimage.blob) {
            log::error!("Preimage unneeded: not solicited or already provided for service {:?}", preimage.requester);
            return Err(ImportError::PreimagesError(PreimagesErrorCode::PreimageUnneeded));
        }
    }

    Ok(())
}

// This is the preimage integration function, which transforms a dictionary of service states and a set of service/blob pairs into a 
// new dictionary of service states. Preimage provisions into services which no longer exist or whose relevant request is dropped are disregarded:
pub fn integrate(services: &mut ServiceAccounts, pairs: &[(ServiceId, Vec<u8>)], slot: &TimeSlot) {

    for (service_id, blob) in pairs.iter() {
        if !providable(services, service_id, blob) {
            log::debug!("Preimage for service {:?} disregarded: no longer providable", service_id);
            continue;
        }
        let hash = sp_core::blake2_256(blob);
        let lookup_key = StateKeyType::Account(*service_id, construct_lookup_key(&hash, blob.len() as u32)).construct();
        let preimage_key = StateKeyType::Account(*service_id, construct_preimage_key(&hash)).construct();
        let account = services.get_mut(service_id).unwrap();
        account.storage.insert(preimage_key, blob.clone());
        account.storage.insert(lookup_key, vec![*slot].encode_len());
    }
}

// Validate and integrate against the same state (this is used in preimages test vectors)
pub fn process(
    preimages_extrinsic: &[Preimage], 
    services: &mut ServiceAccounts, 
    post_tau: &TimeSlot) 
-> Result<(), ImportError> {

    validate(preimages_extrinsic, services)?;
    let pairs = preimages_extrinsic.iter().map(|preimage| (preimage.requester, preimage.blob.clone())).collect::<Vec<_>>();
    integrate(services, &pairs, post_tau);
    Ok(())
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