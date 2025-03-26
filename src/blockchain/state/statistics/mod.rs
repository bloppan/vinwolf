/*
    The Jam chain does not explicitly issue rewards —we leave this as a job to be done by the staking 
    subsystem (in Polkadot’s case envisioned as a system parachain hosted without fees— in the current 
    imagining of a public Jam network). However, much as with validator punishment information, it is 
    important for the Jam chain to facilitate the arrival of information on validator activity in to the 
    staking subsystem so that it may be acted upon.

    Such performance information cannot directly cover all aspects of validator activity; whereas block 
    production, guarantor reports and availability assurance can easily be tracked on-chain, Grandpa, Beefy 
    and auditing activity cannot. In the latter case, this is instead tracked with validator voting activity: 
    validators vote on their impression of each other’s efforts and a median may be accepted as the truth for 
    any given validator. With an assumption of 50% honest validators, this gives an adequate means of oraclizing 
    this information.

    The validator statistics are made on a per-epoch basis and we retain one record of completed statistics together
    with one record which serves as an accumulator for the present epoch. Both are tracked in π, which is thus a
    sequence of two elements, with the first being the accumulator and the second the previous epoch’s statistics.

    For each epoch we track a performance record for each validator.
*/
use std::default::Default;

use crate::types::{Statistics, ValidatorIndex, TimeSlot, Extrinsic, ActivityRecords};
use crate::constants::EPOCH_LENGTH;
use super::get_time;

pub fn process_statistics(
    statistics: &mut Statistics,
    post_tau: &TimeSlot,
    author_index: &ValidatorIndex,
    extrinsic: &Extrinsic,
) {

    let tau = get_time();

    if post_tau / EPOCH_LENGTH as u32 != tau / EPOCH_LENGTH as u32 {
        // We are in a new epoch
        // Update the last record with the current one
        statistics.prev = statistics.curr.clone();

        // Reset the current record
        statistics.curr = ActivityRecords::default();
    }
    // The number of blocks produced by the validator
    statistics.curr.records[*author_index as usize].blocks += 1;
    // The number of tickets introduced by the validator
    statistics.curr.records[*author_index as usize].tickets += extrinsic.tickets.tickets.len() as u32;
    
    for preimage in extrinsic.preimages.preimages.iter() {
        // The number of preimages introduced by the validator
        statistics.curr.records[*author_index as usize].preimages += 1;
        // The total number of octets across all preimages introduced by the validator
        statistics.curr.records[*author_index as usize].preimages_size += preimage.blob.len() as u32;
    }

    // The number of reports guaranteed by the 
    for guarantee in extrinsic.guarantees.report_guarantee.iter() {
        for signature in guarantee.signatures.iter() {
            statistics.curr.records[signature.validator_index as usize].guarantees += 1;
        }
    }

    // The number of availability assurances made by the validator
    for assurance in extrinsic.assurances.assurances.iter() {
        statistics.curr.records[assurance.validator_index as usize].assurances += 1;
    }
}

