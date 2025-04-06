// Preimages are static data which is presently being requested to be available for workloads to be able to 
// fetch on demand. Prior to accumulation, we must first integrate all preimages provided in the lookup extrinsic. 

use crate::types::{ProcessError, PreimagesExtrinsic, PreimagesErrorCode};

use std::collections::HashSet;

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


impl PreimagesExtrinsic {

    pub fn process(&self) -> Result<(), ProcessError> {

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
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::types::Preimage;

    #[test]
    fn test_preimages_extrinsic_process() {
        let preimages = vec![
            Preimage { requester: 0, blob: vec![1, 2, 3] },
            Preimage { requester: 1, blob: vec![4, 5, 6] },
            Preimage { requester: 2, blob: vec![7, 8, 9] },
        ];
        let preimages_extrinsic = PreimagesExtrinsic { preimages };
        assert_eq!(preimages_extrinsic.process(), Ok(()));

        let preimages = vec![
            Preimage { requester: 0, blob: vec![1, 2, 3] },
            Preimage { requester: 1, blob: vec![4, 5, 6] },
            Preimage { requester: 0, blob: vec![7, 8, 9] },
        ];
        let preimages_extrinsic = PreimagesExtrinsic { preimages };
        assert_eq!(preimages_extrinsic.process(), Err(ProcessError::PreimagesError(PreimagesErrorCode::PreimagesNotSortedOrUnique)));
    }

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