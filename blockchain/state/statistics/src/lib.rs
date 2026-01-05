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

use constants::node::{CORES_COUNT, EPOCH_LENGTH, SEGMENT_SIZE, VALIDATORS_COUNT};
use jam_types::{*};
use std::collections::{HashMap, HashSet};
use std::sync::{LazyLock, Mutex};
use tools::log;

static ACC_STATS: LazyLock<Mutex<HashMap<ServiceId, (Gas, u32)>>> = LazyLock::new(|| {
    Mutex::new(HashMap::default())
});

pub fn clean_acc_stats() {
    set_acc_stats(HashMap::new());
}

pub fn set_acc_stats(acc_stats: HashMap<ServiceId, (Gas, u32)>) {
    *ACC_STATS.lock().unwrap() = acc_stats;   
}

pub fn get_acc_stats() -> HashMap<ServiceId, (Gas, u32)> {
    ACC_STATS.lock().unwrap().clone()
}

pub fn add_acc_stats(service: ServiceId, gas_reports: (Gas, u32)) {
    let mut acc_stats = get_acc_stats();
    if !acc_stats.contains_key(&service) {
        acc_stats.insert(service, (0, 0));
    }
    let (gas_stored, num_repors_stored) = acc_stats.get(&service).unwrap();
    log::debug!("Add service: {:?} to acc stats with {:?} gas used. Total gas used: {:?}", service, gas_reports.0, gas_reports.0.saturating_add(*gas_stored));
    acc_stats.insert(service, (gas_reports.0.saturating_add(*gas_stored), (*num_repors_stored).saturating_add(gas_reports.1)));
    set_acc_stats(acc_stats);
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
    if let Some(record) = statistics.curr.records.get_mut(*author_index as usize) {
        record.blocks = record.blocks.saturating_add(1);
    }
    // The number of tickets introduced by the validator    
    if let Some(record) = statistics.curr.records.get_mut(*author_index as usize) {
        record.tickets = record.tickets.saturating_add(block.extrinsic.tickets.len() as u32);
    }

    for preimage in block.extrinsic.preimages.iter() {
        // The number of preimages introduced by the validator
        if let Some(record) = statistics.curr.records.get_mut(*author_index as usize) {
            record.preimages = record.preimages.saturating_add(1);
        }
        // The total number of octets across all preimages introduced by the validator
        if let Some(record) = statistics.curr.records.get_mut(*author_index as usize) {
            record.preimages_size = record.preimages_size.saturating_add(preimage.blob.len() as u32)
        }
    }

    let mut services: HashSet<ServiceId> = HashSet::new();
    // The core and service activity statistics are tracked only on a per-block basis unlike the validator statistics
    // which are tracked over the whole epoch.
    statistics.cores = CoresStatistics::default();
    statistics.services = ServicesStatistics::default();

    for validator_index in 0..VALIDATORS_COUNT {
        if reporters.contains(&curr_validators.list[validator_index].ed25519) {
            if let Some(record) = statistics.curr.records.get_mut(validator_index as usize) {
                record.guarantees = record.guarantees.saturating_add(1);
            }
        }
    }

    for guarantee in &block.extrinsic.guarantees {
        
        if let Some(record) = statistics.cores.records.get_mut(guarantee.report.core_index as usize) {
            record.imports = record.imports.saturating_add(guarantee.report.results.iter().map(|result| result.refine_load.imports).sum::<u16>());
        }
        if let Some(record) = statistics.cores.records.get_mut(guarantee.report.core_index as usize) {
            record.extrinsic_count = record.extrinsic_count.saturating_add(guarantee.report.results.iter().map(|result| result.refine_load.extrinsic_count).sum::<u16>());
        }
        if let Some(record) = statistics.cores.records.get_mut(guarantee.report.core_index as usize) {
            record.extrinsic_size = record.extrinsic_size.saturating_add(guarantee.report.results.iter().map(|result| result.refine_load.extrinsic_size).sum::<u32>());
        }
        if let Some(record) = statistics.cores.records.get_mut(guarantee.report.core_index as usize) {
            record.exports = record.exports.saturating_add(guarantee.report.results.iter().map(|result| result.refine_load.exports).sum::<u16>());
        }
        if let Some(record) = statistics.cores.records.get_mut(guarantee.report.core_index as usize) {
            record.gas_used = record.gas_used.saturating_add(guarantee.report.results.iter().map(|result| result.refine_load.gas_used).sum::<u64>());
        }
        if let Some(record) = statistics.cores.records.get_mut(guarantee.report.core_index as usize) {
            record.bundle_size = record.bundle_size.saturating_add(guarantee.report.package_spec.length);
        }

        services.extend(guarantee.report.results.iter().map(|result| result.service));
    }
    
    services.extend(block.extrinsic.preimages.iter().map(|preimage| preimage.requester));
    services.extend(get_acc_stats().iter().map(|(service, _)| *service));
    
    for service in services.iter() {

        statistics.services.records.insert(*service, SeviceActivityRecord::default());

        for guarantee in &block.extrinsic.guarantees {
            for result in guarantee.report.results.iter() {
                if result.service == *service {
                    if let Some(record) = statistics.services.records.get_mut(service) {
                        record.imports = record.imports.saturating_add(result.refine_load.imports as u32);
                    }
                    if let Some(record) = statistics.services.records.get_mut(service) {
                        record.extrinsic_count = record.extrinsic_count.saturating_add(result.refine_load.extrinsic_count as u32);
                    }
                    if let Some(record) = statistics.services.records.get_mut(service) {
                        record.extrinsic_size = record.extrinsic_size.saturating_add(result.refine_load.extrinsic_size as u32);
                    }
                    if let Some(record) = statistics.services.records.get_mut(service) {
                        record.exports = record.exports.saturating_add(result.refine_load.exports as u32);
                    }
                    if let Some(record) = statistics.services.records.get_mut(service) {
                        record.refinement_count = record.refinement_count.saturating_add(1);
                    }
                    if let Some(record) = statistics.services.records.get_mut(service) {
                        record.refinement_gas_used = record.refinement_gas_used.saturating_add(result.refine_load.gas_used);
                    }
                }
            }
        }

        for preimage in &block.extrinsic.preimages {
            if preimage.requester == *service {
                if let Some(record) = statistics.services.records.get_mut(service) {
                    record.provided_count = record.provided_count.saturating_add(1);
                }
                if let Some(record) = statistics.services.records.get_mut(service) {
                    record.provided_size = record.provided_size.saturating_add(preimage.blob.len() as u32);
                }
            }
        }

        if let Some((acc_gas, acc_count)) = get_acc_stats().get(service) {
            if let Some(record) = statistics.services.records.get_mut(service) {
                record.accumulate_gas_used = record.accumulate_gas_used.saturating_add(*acc_gas as u64) // TODO fix this
            }
            if let Some(record) = statistics.services.records.get_mut(service) {
                record.accumulate_count = record.accumulate_count.saturating_add(*acc_count);
            }
        }
    }

    // The number of availability assurances made by the validator
    for assurance in block.extrinsic.assurances.iter() {
        if let Some(record) = statistics.curr.records.get_mut(assurance.validator_index as usize) {
            record.assurances = record.assurances.saturating_add(1);
        }
        for core_index in 0..CORES_COUNT {
            if assurance.bitfield[core_index / 8] & (1 << core_index % 8) != 0 {
                if let Some(record) = statistics.cores.records.get_mut(core_index as usize) {
                    record.popularity = record.popularity.saturating_add(1);
                }
            }
        }
    }

    for new_wr in new_available_wr.iter() {
        if let Some(record) = statistics.cores.records.get_mut(new_wr.core_index as usize) {
            record.da_load = record.da_load.saturating_add(new_wr.package_spec.length + SEGMENT_SIZE as u32 * (new_wr.package_spec.exports_count * (65/64)) as u32) // TODO revisar esta formula (la division)
        }
    }
}
