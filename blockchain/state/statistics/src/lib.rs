/*
    The Jam chain does not explicitly issue rewards —we leave this as a job to be done by the staking subsystem (in Polkadot’s case 
    envisioned as a system parachain hosted without fees— in the current imagining of a public Jam network). However, much as with 
    validator punishment information, it is important for the Jam chain to facilitate the arrival of information on validator activity 
    in to the staking subsystem so that it may be acted upon.

    Such performance information cannot directly cover all aspects of validator activity; whereas block production, guarantor reports 
    and availability assurance can easily be tracked on-chain, Grandpa, Beefy and auditing activity cannot. In the latter case, this is 
    instead tracked with validator voting activity: validators vote on their impression of each other’s efforts and a median may be 
    accepted as the truth for any given validator. With an assumption of 50% honest validators, this gives an adequate means of oraclizing 
    this information.

    The validator statistics are made on a per-epoch basis and we retain one record of completed statistics together with one record which 
    serves as an accumulator for the present epoch. Both are tracked in π, which is thus a sequence of two elements, with the first being 
    the accumulator and the second the previous epoch’s statistics. For each epoch we track a performance record for each validator.
*/

use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::collections::{HashMap, HashSet};

use jam_types::{
    Block, CoresStatistics, Ed25519Public, Gas, ServiceId, ServicesStatistics, SeviceActivityRecord, Statistics, ValidatorStatistics, ValidatorsData, WorkReport
};
use utils::common::VerifySignature;
use codec::Encode;
use constants::node::{CORES_COUNT, EPOCH_LENGTH, SEGMENT_SIZE, VALIDATORS_COUNT};

static ACC_STATS: Lazy<Mutex<HashMap<ServiceId, (Gas, u32)>>> = Lazy::new(|| {
    Mutex::new(HashMap::default())
});

static XFER_STATS: Lazy<Mutex<HashMap<ServiceId, (u32, Gas)>>> = Lazy::new(|| {
    Mutex::new(HashMap::default())
});

pub fn set_acc_stats(acc_stats: HashMap<ServiceId, (Gas, u32)>) {
    *ACC_STATS.lock().unwrap() = acc_stats;   
}

pub fn get_acc_stats() -> HashMap<ServiceId, (Gas, u32)> {
    ACC_STATS.lock().unwrap().clone()
}

pub fn set_xfer_stats(xfer_stats: HashMap<ServiceId, (u32, Gas)>) {
    *XFER_STATS.lock().unwrap() = xfer_stats;   
}

pub fn get_xfer_stats() -> HashMap<ServiceId, (u32, Gas)> {
    XFER_STATS.lock().unwrap().clone()
}

pub fn process(
    statistics: &mut Statistics,
    curr_validators: &ValidatorsData,
    block: &Block,
    reporters: &[Ed25519Public],
    new_available_wr: &[WorkReport],
) {
    
    log::debug!("Process statistics");

    let tau = state_handler::time::get();
    let post_tau = block.header.unsigned.slot;
    let author_index = &block.header.unsigned.author_index;

    if post_tau / EPOCH_LENGTH as u32 != tau / EPOCH_LENGTH as u32 {
        // We are in a new epoch
        // Update the last record with the current one
        statistics.prev = statistics.curr.clone();

        // Reset the current record
        statistics.curr = ValidatorStatistics::default();
    }
    // The number of blocks produced by the validator
    statistics.curr.records[*author_index as usize].blocks += 1;
    // The number of tickets introduced by the validator
    statistics.curr.records[*author_index as usize].tickets += block.extrinsic.tickets.len() as u32;
    
    for preimage in block.extrinsic.preimages.iter() {
        // The number of preimages introduced by the validator
        statistics.curr.records[*author_index as usize].preimages += 1;
        // The total number of octets across all preimages introduced by the validator
        statistics.curr.records[*author_index as usize].preimages_size = statistics.curr.records[*author_index as usize].preimages_size.saturating_add(preimage.blob.len() as u32);
    }

    let mut services: HashSet<ServiceId> = HashSet::new();
    // The core and service activity statistics are tracked only on a per-block basis unlike the validator statistics
    // which are tracked over the whole epoch.
    statistics.cores = CoresStatistics::default();
    statistics.services = ServicesStatistics::default();

    for validator_index in 0..VALIDATORS_COUNT {
        if reporters.contains(&curr_validators.list[validator_index].ed25519) {
            statistics.curr.records[validator_index as usize].guarantees += 1;
        }
    }

    for guarantee in &block.extrinsic.guarantees {

        statistics.cores.records[guarantee.report.core_index as usize].imports += guarantee.report.results.iter().map(|result| result.refine_load.imports).sum::<u16>();
        statistics.cores.records[guarantee.report.core_index as usize].extrinsic_count += guarantee.report.results.iter().map(|result| result.refine_load.extrinsic_count).sum::<u16>();
        statistics.cores.records[guarantee.report.core_index as usize].extrinsic_size += guarantee.report.results.iter().map(|result| result.refine_load.extrinsic_size).sum::<u32>();
        statistics.cores.records[guarantee.report.core_index as usize].exports += guarantee.report.results.iter().map(|result| result.refine_load.exports).sum::<u16>();
        statistics.cores.records[guarantee.report.core_index as usize].gas_used += guarantee.report.results.iter().map(|result| result.refine_load.gas_used).sum::<u64>();
        statistics.cores.records[guarantee.report.core_index as usize].bundle_size += guarantee.report.package_spec.length;

        /*for result in guarantee.report.results.iter() {
            if statistics.services.records.get(&result.service).is_none() {
                statistics.services.records.insert(result.service, SeviceActivityRecord::default());
            }
            statistics.services.records.get_mut(&result.service).unwrap().refinement_count += 1;
        }*/
        services.extend(guarantee.report.results.iter().map(|result| result.service));
    }
    
    services.extend(block.extrinsic.preimages.iter().map(|preimage| preimage.requester));
    services.extend(get_acc_stats().iter().map(|(service, _)| *service));
    services.extend(get_xfer_stats().iter().map(|(service, _)| *service));
    
    for service in services.iter() {

        statistics.services.records.insert(*service, SeviceActivityRecord::default());

        for guarantee in &block.extrinsic.guarantees {
            for result in guarantee.report.results.iter() {
                if result.service == *service {
                    statistics.services.records.get_mut(service).unwrap().imports += result.refine_load.imports as u32;
                    statistics.services.records.get_mut(service).unwrap().extrinsic_count += result.refine_load.extrinsic_count as u32;
                    statistics.services.records.get_mut(service).unwrap().extrinsic_size += result.refine_load.extrinsic_size as u32;
                    statistics.services.records.get_mut(service).unwrap().exports += result.refine_load.exports as u32;
                    statistics.services.records.get_mut(service).unwrap().refinement_count += 1;
                    statistics.services.records.get_mut(service).unwrap().refinement_gas_used += result.refine_load.gas_used;
                }
            }
        }

        for preimage in &block.extrinsic.preimages {
            if preimage.requester == *service {
                statistics.services.records.get_mut(service).unwrap().provided_count += 1;
                statistics.services.records.get_mut(service).unwrap().provided_size += preimage.blob.len() as u32;
            }
        }

        if let Some((acc_gas, acc_count)) = get_acc_stats().get(service) {
            statistics.services.records.get_mut(service).unwrap().accumulate_gas_used += *acc_gas as u64; // TODO fix this
            statistics.services.records.get_mut(service).unwrap().accumulate_count += *acc_count;
        }

        if let Some((xfer_count, xfer_gas)) = get_xfer_stats().get(service) {
            statistics.services.records.get_mut(service).unwrap().on_transfers_count += *xfer_count;
            statistics.services.records.get_mut(service).unwrap().on_transfers_gas_used += *xfer_gas as u64; // TODO fix this
        }
    }

    // The number of availability assurances made by the validator
    for assurance in block.extrinsic.assurances.iter() {
        statistics.curr.records[assurance.validator_index as usize].assurances += 1;
        for core_index in 0..CORES_COUNT {
            if assurance.bitfield[core_index / 8] & (1 << core_index % 8) != 0 {
                statistics.cores.records[core_index as usize].popularity += 1;
            }
        }
    }

    for new_wr in new_available_wr.iter() {
        statistics.cores.records[new_wr.core_index as usize].da_load += new_wr.package_spec.length + SEGMENT_SIZE as u32 * (new_wr.package_spec.exports_count * (65/64)) as u32;
    }

}
