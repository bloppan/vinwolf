use sp_core::blake2_256;

use crate::types::{
    AvailabilityAssignments, DisputesExtrinsic, DisputesRecords, Hash, DisputesErrorCode, OutputDataDisputes, Verdict
};
use crate::constants::{EPOCH_LENGTH, ONE_THIRD_VALIDATORS, VALIDATORS_SUPER_MAJORITY};
use crate::blockchain::state::{get_time, get_validators};
use crate::blockchain::state::validators::ValidatorSet;
use crate::blockchain::state::ProcessError;
use crate::utils::common::{VerifySignature, is_sorted_and_unique, has_duplicates};
use crate::utils::codec::Encode;

impl DisputesExtrinsic {
    // TODO - Implement process for verdicts, culprits and faults
    pub fn process(
        &self, 
        disputes_state: &mut DisputesRecords,
        availability_state: &mut AvailabilityAssignments,
    ) -> Result<OutputDataDisputes, ProcessError> {
    
    // The disputes extrinsic may contain one or more verdicts v as a compilation of judgments coming from 
    // exactly two-thirds plus one of either the active validator set or the previous epoch's validator set, 
    // i.e. the Ed25519 keys of κ or λ. 
    if self.is_empty() {
        return Ok(OutputDataDisputes { offenders_mark: Vec::new() });
    }

    // The disputes extrinsic may contain one or more verdicts
    if self.verdicts.len() < 1{
        return Err(ProcessError::DisputesError(DisputesErrorCode::NoVerdictsFound));
    }

    // Verify culprits ed25519 signatures
    for culprit in &self.culprits {
        let mut message = Vec::from(b"jam_guarantee");
        culprit.target.encode_to(&mut message);
        if !culprit.signature.verify_signature(&message, &culprit.key) {
            return Err(ProcessError::DisputesError(DisputesErrorCode::BadSignature));
        }
    }

    // Verdicts must be ordered by report hash.
    let verdict_targets: Vec<_> = self.verdicts.iter().map(|v| v.target).collect();
    if !is_sorted_and_unique(&verdict_targets) {
        return Err(ProcessError::DisputesError(DisputesErrorCode::VerdictsNotSortedUnique));
    }

    // Additionally, disputes extrinsic may contain proofs of the misbehavior of one or more validators, either 
    // by guaranteeing a work-report found to be invalid (culprits), or by signing a judgment found to be contradiction 
    // to a work-report's validity (faults). Both are considered a kind of offense
    
    // Culprits must be ordered by Ed25519 keys.
    let culprit_keys: Vec<_> = self.culprits.iter().map(|c| c.key).collect();
    if !is_sorted_and_unique(&culprit_keys) {
        return Err(ProcessError::DisputesError(DisputesErrorCode::CulpritsNotSortedUnique));
    }
    
    // Faults must be ordered by Ed25519 keys.
    let faults_keys: Vec<_> = self.faults.iter().map(|f| f.key).collect();
    if !is_sorted_and_unique(&faults_keys) {
        return Err(ProcessError::DisputesError(DisputesErrorCode::FaultsNotSortedUnique));
    }

    let mut disputes_records = Vec::new();
    let mut already_reported = Vec::new();
    let mut bad_set = Vec::new();
    let mut good_set = Vec::new();
    
    bad_set.extend_from_slice(&disputes_state.bad);
    good_set.extend_from_slice(&disputes_state.good);

    disputes_records.extend_from_slice(&disputes_state.good);
    disputes_records.extend_from_slice(&disputes_state.bad);
    disputes_records.extend_from_slice(&disputes_state.wonky);
    already_reported.extend_from_slice(&disputes_state.offenders);

    // There may be no duplicate report hashes within the extrinsic, nor amongst any past reported hashes
    let mut offenders_reported = Vec::with_capacity(disputes_records.len() + verdict_targets.len());
    offenders_reported.extend_from_slice(&disputes_records);
    offenders_reported.extend_from_slice(&verdict_targets);
    // Check if there are offenders already judged
    if has_duplicates(&offenders_reported) {
        return Err(ProcessError::DisputesError(DisputesErrorCode::AlreadyJudged));
    }
    // The judgments of all verdicts must be ordered by validator index and there may be no duplicates
    for verdict in &self.verdicts {
        if !is_sorted_and_unique(&verdict.votes.iter().map(|vote| vote.index).collect::<Vec<_>>()) {
            return Err(ProcessError::DisputesError(DisputesErrorCode::JudgementsNotSortedUnique));
        }
    }

    let mut new_offenders = Vec::with_capacity(culprit_keys.len() + faults_keys.len());
    new_offenders.extend_from_slice(&culprit_keys);  
    new_offenders.extend_from_slice(&faults_keys);
    // In the disputes extrinsic can not be offenders already reported
    already_reported.extend_from_slice(&new_offenders);
    if has_duplicates(&already_reported) {
        return Err(ProcessError::DisputesError(DisputesErrorCode::OffenderAlreadyReported));
    }   

    // We define vote_count to derive from the sequence of verdicts introduced in the block's extrinsic, containing only 
    // the report hash and the sum of positive judgments.
    let vote_count: Vec<(Hash, usize)> = vote_count(&self.verdicts);
    
    // We first save in this auxiliar records the new offenders
    let mut new_records = DisputesRecords::default();

    for (target, count) in vote_count {
        // We require this total to be either exactly two-thirds-plus-one, zero or one-third of the validator set 
        // indicating, respectively, that the report is good, that it's bad, or that it's wonky.
        match count {
            VALIDATORS_SUPER_MAJORITY => {
                    // Any verdict containing solely valid judgments implies the same report having at least one valid
                    // entry in the faults sequence
                    if self.faults.len() < 1 {
                        return Err(ProcessError::DisputesError(DisputesErrorCode::NotEnoughFaults));
                    }
                    new_records.good.push(target);
            },
            0 => {
                    // Any verdict containing solely invalid judgments implies the same report having at least two 
                    // valid entries in the culprits sequence
                    if self.culprits.len() < 2 {
                        return Err(ProcessError::DisputesError(DisputesErrorCode::NotEnoughCulprits));
                    }
                    new_records.bad.push(target);
            },
            ONE_THIRD_VALIDATORS => new_records.wonky.push(target),
            _ => { return Err(ProcessError::DisputesError(DisputesErrorCode::BadVoteSplit)); }
        }
    }

    let epoch = get_time() as usize / EPOCH_LENGTH;
    let verdict_ages = self.verdicts.iter().map(|v| v.age).collect::<Vec<_>>();

    // Verdicts comes from either the active validator set or the previous epoch's validator set
    if verdict_ages.len() > 1 {
        if !verdict_ages.iter().all(|x| *x == verdict_ages[0]) {
            return Err(ProcessError::DisputesError(DisputesErrorCode::AgesNotEqual));
        }
    }

    let validator_set = if epoch == verdict_ages[0] as usize {
        get_validators(ValidatorSet::Current)
    } else {
        get_validators(ValidatorSet::Previous)
    };

    // Offender signatures must be similarly valid and reference work-reports with judgemets and may not report
    // keys which are already in the punish-set

    bad_set.extend_from_slice(&new_records.bad);

    for culprit in &self.culprits {
        let is_in_bad = bad_set.contains(&culprit.target);
        if !is_in_bad {
            return Err(ProcessError::DisputesError(DisputesErrorCode::CulpritsVerdictNotBad));
        }
        if !already_reported.contains(&culprit.key) {
            if !validator_set.0.iter().any(|v| v.ed25519 == culprit.key) {
                return Err(ProcessError::DisputesError(DisputesErrorCode::CulpritKeyNotFound));
            }
        }
    }
    
    good_set.extend_from_slice(&new_records.good);

    for fault in &self.faults {
        let is_in_bad = bad_set.contains(&fault.target);
        let is_in_good = good_set.contains(&fault.target);
    
        if (is_in_bad && !is_in_good && !fault.vote) || (is_in_good && !is_in_bad && fault.vote) {
            return Err(ProcessError::DisputesError(DisputesErrorCode::FaultVerdictWrong));
        }

        if !already_reported.contains(&fault.key) {
            if !validator_set.0.iter().any(|v| v.ed25519 == fault.key) {
                return Err(ProcessError::DisputesError(DisputesErrorCode::FaultKeyNotFound));
            }
        }
    }

    // Verify Ed25519 signatures
    let jam_valid = Vec::from(b"jam_valid");
    let jam_invalid = Vec::from(b"jam_invalid");
    
    // Verify verdict ed25519 signatures
    for verdict in &self.verdicts {
        
        if epoch - verdict.age as usize > 1 {
            return Err(ProcessError::DisputesError(DisputesErrorCode::BadJudgementAge));
        }

        for vote in &verdict.votes {
            let mut message = Vec::new();
            if vote.vote == true {
                jam_valid.encode_to(&mut message);
            } else {
                jam_invalid.encode_to(&mut message);
            }
            verdict.target.encode_to(&mut message);

            if !vote.signature.verify_signature(&message, &validator_set.0[vote.index as usize].ed25519) {
                return Err(ProcessError::DisputesError(DisputesErrorCode::BadSignature));
            }
        }
    }
    // Verify fault ed25519 signatures
    for fault in &self.faults {
        let mut message = Vec::new();
        if fault.vote == true {
            jam_valid.encode_to(&mut message);
        } else {
            jam_invalid.encode_to(&mut message);
        }
        fault.target.encode_to(&mut message);

        if !fault.signature.verify_signature(&message, &fault.key) {
            return Err(ProcessError::DisputesError(DisputesErrorCode::BadSignature));
        }
    }
    
    // If the state was initialized, then we save the auxiliar records in the state
    disputes_state.good.extend_from_slice(&new_records.good);
    disputes_state.bad.extend_from_slice(&new_records.bad);
    disputes_state.wonky.extend_from_slice(&new_records.wonky);
    let mut offenders = new_offenders.clone();
    offenders.sort();
    disputes_state.offenders.extend_from_slice(&offenders);
    
    // We clear any work-reports which we judged as uncertain or invalid from their core:
    for assignment in availability_state.0.iter_mut() {
        if let Some(availability_assignment) = assignment {
            // Calculate target hash
            let target_hash = blake2_256(&availability_assignment.report.encode());
            // Check if the hash is contained in bad or wonky sets
            if disputes_state.bad.contains(&target_hash)
                || disputes_state.wonky.contains(&target_hash)
            {
                *assignment = None; // Set to None
            }
        }
    }
    
    Ok(OutputDataDisputes { offenders_mark: new_offenders })
    }
}

// The disputes extrinsic may contain one or more verdicts v as a compilation of judgments coming from exactly 
// two-thirds plus one of either the active validator set or the previous epoch’s validator set, i.e. the Ed25519 
// keys of κ or λ. Additionally, it may contain proofs of the misbehavior of one or more validators, either by 
// guaranteeing a work-report found to be invalid (culprits), or by signing a judgment found to be contradiction 
// to a work-report’s validity (faults). Both are considered a kind of offense.

impl DisputesExtrinsic {

    pub fn is_empty(&self) -> bool {
        self.verdicts.is_empty()
        && self.culprits.is_empty()
        && self.faults.is_empty()
    }
}

fn vote_count(verdicts: &[Verdict]) -> Vec<(Hash, usize)> {

    let mut vote_count: Vec<(Hash, usize)> = Vec::with_capacity(verdicts.len());
    
    for verdict in verdicts {
        let hash = verdict.target.clone();
        let mut count = 0;
        for vote in verdict.votes.iter() {
            if vote.vote == true {
                count += 1;
            }
        }
        vote_count.push((hash, count));
    }

    return vote_count;
}

#[cfg(test)]
mod test {

    use sp_core::{ed25519, Pair};
    use crate::types::ValidatorIndex;
    
    fn bad_hash_order<const N: usize>(data: &Vec<[u8; N]>) -> bool {

        if data.len() <= 1 {
            return false;
        }
    
        'next_vector: for i in 0..data.len() - 1 {
            let current = data[i];
            let next = data[i + 1];
            let mut equals = 0;
    
            for j in 0..N {
                 if current[j] > next[j] {
                    return true; // Bad order
                } else if current[j] < next[j] {
                    continue 'next_vector;
                } else {
                    equals += 1;
                }
            }
    
            if equals == N {
                return true; // There are duplicates
            }
        }
    
        return false; // Correct order
    }
    
    fn bad_index_order(index: &[ValidatorIndex]) -> bool {
    
        for i in 0..index.len() {
            if index[i] == i as u16 {
                continue;
            }
            return true;
        }
        return false;
    }
    
    #[test]
    fn verify_ed25519_signature() {
        let jam_invalid = Vec::from(b"jam_invalid");
        let signature: [u8; 64] = [
            0x64, 0x7c, 0x04, 0x63, 0x0e, 0x91, 0x1a, 0x43, 0x2f, 0x99, 0xe6, 0xc1, 0x10, 0x8b, 0xcf, 0x4c,
            0x06, 0x49, 0x67, 0x54, 0x03, 0x3b, 0x77, 0xc5, 0xeb, 0x82, 0x71, 0xa5, 0xd0, 0x6a, 0x85, 0xe1,
            0x88, 0x4d, 0xb0, 0xfb, 0x97, 0x7e, 0x23, 0x2e, 0x41, 0x66, 0x43, 0xcc, 0xfa, 0xf4, 0xf3, 0x34,
            0xe9, 0x9f, 0x3b, 0x8d, 0x9c, 0xdf, 0xc6, 0x5a, 0x8e, 0x4e, 0xcb, 0xc9, 0xdb, 0x28, 0x40, 0x05,
        ];
        let target: [u8; 32] = [
            0x0e, 0x57, 0x51, 0xc0, 0x26, 0xe5, 0x43, 0xb2, 0xe8, 0xab, 0x2e, 0xb0, 0x60, 0x99, 0xda, 0xa1,
            0xd1, 0xe5, 0xdf, 0x47, 0x77, 0x8f, 0x77, 0x87, 0xfa, 0xab, 0x45, 0xcd, 0xf1, 0x2f, 0xe3, 0xa8,
        ];
        let public_key: [u8; 32] = [
            0x3b, 0x6a, 0x27, 0xbc, 0xce, 0xb6, 0xa4, 0x2d, 0x62, 0xa3, 0xa8, 0xd0, 0x2a, 0x6f, 0x0d, 0x73,
            0x65, 0x32, 0x15, 0x77, 0x1d, 0xe2, 0x43, 0xa6, 0x3a, 0xc0, 0x48, 0xa1, 0x8b, 0x59, 0xda, 0x29,
        ];
    
        // Build the message
        let mut message = Vec::new();
        message.extend_from_slice(&jam_invalid);
        message.extend_from_slice(&target);
    
        // Convert to ed25519 types
        let signature = ed25519::Signature::from_raw(signature);
        let public_key = ed25519::Public::from_raw(public_key);
    
        let mut signature_result: bool = false;
        // Verificar la firma
        if ed25519::Pair::verify(&signature, &message, &public_key) {
            signature_result = true;
        }

        assert_eq!(true, signature_result);
    }

    #[test]
    fn sorted_arrays() {

        let vector_hashes = vec![
            [0x11, 0xda, 0x6d, 0x1f, 0x76, 0x1d, 0xdf, 0x9b, 0xdb, 0x4c, 0x9d, 0x6e, 0x53, 0x03, 0xeb, 0xd4, 0x1f, 0x61, 0x85, 0x8d, 0x0a, 0x56, 0x47, 0xa1, 0xa7, 0xbf, 0xe0, 0x89, 0xbf, 0x92, 0x1b, 0xe9],
            [0x22, 0x35, 0x1e, 0x22, 0x10, 0x5a, 0x19, 0xaa, 0xbb, 0x42, 0x58, 0x91, 0x62, 0xad, 0x7f, 0x1e, 0xa0, 0xdf, 0x1c, 0x25, 0xce, 0xbf, 0x0e, 0x4a, 0x9f, 0xcd, 0x26, 0x13, 0x01, 0x27, 0x48, 0x62],
            [0x3b, 0x6a, 0x27, 0xbc, 0xce, 0xb6, 0xa4, 0x2d, 0x62, 0xa3, 0xa8, 0xd0, 0x2a, 0x6f, 0x0d, 0x73, 0x65, 0x32, 0x15, 0x77, 0x1d, 0xe2, 0x43, 0xa6, 0x3a, 0xc0, 0x48, 0xa1, 0x8b, 0x59, 0xda, 0x29],
            [0x7b, 0x0a, 0xa1, 0x73, 0x5e, 0x5b, 0xa5, 0x8d, 0x32, 0x36, 0x31, 0x6c, 0x67, 0x1f, 0xe4, 0xf0, 0x0e, 0xd3, 0x66, 0xee, 0x72, 0x41, 0x7c, 0x9e, 0xd0, 0x2a, 0x53, 0xa8, 0x01, 0x9e, 0x85, 0xb8]
        ];
        assert_eq!(false, bad_hash_order(&vector_hashes)); // Ordered and no duplicates

        let vector_hashes2 = vec![
            [0x11, 0xda, 0x6d, 0x1f, 0x76, 0x1d, 0xdf, 0x9b, 0xdb, 0x4c, 0x9d, 0x6e, 0x53, 0x03, 0xeb, 0xd4, 0x1f, 0x61, 0x85, 0x8d, 0x0a, 0x56, 0x47, 0xa1, 0xa7, 0xbf, 0xe0, 0x89, 0xbf, 0x92, 0x1b, 0xe9],
            [0x22, 0x35, 0x1e, 0x22, 0x10, 0x5a, 0x19, 0xaa, 0xbb, 0x42, 0x58, 0x91, 0x62, 0xad, 0x7f, 0x1e, 0xa0, 0xdf, 0x1c, 0x25, 0xce, 0xbf, 0x0e, 0x4a, 0x9f, 0xcd, 0x26, 0x13, 0x01, 0x27, 0x48, 0x62],
            [0x22, 0x35, 0x1d, 0x73, 0x5e, 0x5b, 0xa5, 0x8d, 0x32, 0x36, 0x31, 0x6c, 0x67, 0x1f, 0xe4, 0xf0, 0x0e, 0xd3, 0x66, 0xee, 0x72, 0x41, 0x7c, 0x9e, 0xd0, 0x2a, 0x53, 0xa8, 0x01, 0x9e, 0x85, 0xb8],
            [0x3b, 0x6a, 0x27, 0xbc, 0xce, 0xb6, 0xa4, 0x2d, 0x62, 0xa3, 0xa8, 0xd0, 0x2a, 0x6f, 0x0d, 0x73, 0x65, 0x32, 0x15, 0x77, 0x1d, 0xe2, 0x43, 0xa6, 0x3a, 0xc0, 0x48, 0xa1, 0x8b, 0x59, 0xda, 0x29]
        ];
        assert_eq!(true, bad_hash_order(&vector_hashes2)); // Bad order

        let vector_hashes2 = vec![
            [0x11, 0xda, 0x6d, 0x1f, 0x76, 0x1d, 0xdf, 0x9b, 0xdb, 0x4c, 0x9d, 0x6e, 0x53, 0x03, 0xeb, 0xd4, 0x1f, 0x61, 0x85, 0x8d, 0x0a, 0x56, 0x47, 0xa1, 0xa7, 0xbf, 0xe0, 0x89, 0xbf, 0x92, 0x1b, 0xe9],
            [0x22, 0x35, 0x1e, 0x22, 0x10, 0x5a, 0x19, 0xaa, 0xbb, 0x42, 0x58, 0x91, 0x62, 0xad, 0x7f, 0x1e, 0xa0, 0xdf, 0x1c, 0x25, 0xce, 0xbf, 0x0e, 0x4a, 0x9f, 0xcd, 0x26, 0x13, 0x01, 0x27, 0x48, 0x62],
            [0x22, 0x35, 0x1e, 0x22, 0x10, 0x5a, 0x19, 0xaa, 0xbb, 0x42, 0x58, 0x91, 0x62, 0xad, 0x7f, 0x1e, 0xa0, 0xdf, 0x1c, 0x25, 0xce, 0xbf, 0x0e, 0x4a, 0x9f, 0xcd, 0x26, 0x13, 0x01, 0x27, 0x48, 0x61],
            [0x3b, 0x6a, 0x27, 0xbc, 0xce, 0xb6, 0xa4, 0x2d, 0x62, 0xa3, 0xa8, 0xd0, 0x2a, 0x6f, 0x0d, 0x73, 0x65, 0x32, 0x15, 0x77, 0x1d, 0xe2, 0x43, 0xa6, 0x3a, 0xc0, 0x48, 0xa1, 0x8b, 0x59, 0xda, 0x29]
        ];
        assert_eq!(true, bad_hash_order(&vector_hashes2)); // Bad order

        let vector_hashes4 = vec![
            [0x11, 0xda, 0x6d, 0x1f, 0x76, 0x1d, 0xdf, 0x9b, 0xdb, 0x4c, 0x9d, 0x6e, 0x53, 0x03, 0xeb, 0xd4, 0x1f, 0x61, 0x85, 0x8d, 0x0a, 0x56, 0x47, 0xa1, 0xa7, 0xbf, 0xe0, 0x89, 0xbf, 0x92, 0x1b, 0xe9],
            [0x22, 0x35, 0x1e, 0x22, 0x10, 0x5a, 0x19, 0xaa, 0xbb, 0x42, 0x58, 0x91, 0x62, 0xad, 0x7f, 0x1e, 0xa0, 0xdf, 0x1c, 0x25, 0xce, 0xbf, 0x0e, 0x4a, 0x9f, 0xcd, 0x26, 0x13, 0x01, 0x27, 0x48, 0x62],
            [0x3b, 0x6a, 0x27, 0xbc, 0xce, 0xb6, 0xa4, 0x2d, 0x62, 0xa3, 0xa8, 0xd0, 0x2a, 0x6f, 0x0d, 0x73, 0x65, 0x32, 0x15, 0x77, 0x1d, 0xe2, 0x43, 0xa6, 0x3a, 0xc0, 0x48, 0xa1, 0x8b, 0x59, 0xda, 0x29],
            [0x3b, 0x6a, 0x27, 0xbc, 0xce, 0xb6, 0xa4, 0x2d, 0x62, 0xa3, 0xa8, 0xd0, 0x2a, 0x6f, 0x0d, 0x73, 0x65, 0x32, 0x15, 0x77, 0x1d, 0xe2, 0x43, 0xa6, 0x3a, 0xc0, 0x48, 0xa1, 0x8b, 0x59, 0xda, 0x29]
        ];
        assert_eq!(true, bad_hash_order(&vector_hashes4)); // Duplicates
    }

    #[test]
    fn sorted_index() {
        
        let mut index = vec![0, 1, 2, 3];
        assert_eq!(false, bad_index_order(&index));

        index = vec![0, 1];
        assert_eq!(false, bad_index_order(&index));
    
        index = vec![0];
        assert_eq!(false, bad_index_order(&index));

        index = vec![1];
        assert_eq!(true, bad_index_order(&index));

        index = vec![0, 1, 2, 4, 5];
        assert_eq!(true, bad_index_order(&index));
    }
}


