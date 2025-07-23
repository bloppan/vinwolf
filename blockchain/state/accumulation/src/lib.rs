/* 
    Accumulation may be defined as some function whose arguments are W and δ together with selected portions of 
    (at times partially transitioned) state and which yields the posterior service state together with additional 
    state elements.

    The proposition of accumulation is in fact quite simple: we merely wish to execute the Accumulate logic of the 
    service code of each of the services which has at least one work output, passing to it the work outputs and useful 
    contextual information. However, there are three main complications. Firstly, we must define the execution environment 
    of this logic and in particular the host functions available to it. Secondly, we must define the amount of gas to be 
    allowed for each service’s execution. Finally, we must determine the nature of transfers within Accumulate which, as we 
    will see, leads to the need for a second entry-point, on-transfer.
*/

use std::collections::{HashSet, HashMap};
use ark_vrf::reexports::ark_std::iterable::Iterable;

use constants::node::{EPOCH_LENGTH, TOTAL_GAS_ALLOCATED, WORK_REPORT_GAS_LIMIT, CORES_COUNT};
use jam_types::{
    AccumulatedHistory, AccumulationOperand, AccumulationPartialState, AuthQueues, DeferredTransfer, OpaqueHash, Privileges, 
    ReadyQueue, ReadyRecord, ServiceAccounts, TimeSlot, ValidatorsData, WorkPackageHash, WorkReport, ServiceId, Gas, Account, 
    AccumulateErrorCode, StateKeyType, ProcessError
};
use handler::{get_current_block_slot, get_time};
use codec::Encode;
use utils::serialization::{StateKeyTrait, construct_lookup_key, construct_preimage_key};
use utils::trie;
use pvm::hostcall::accumulate::invoke_accumulation;
use pvm::hostcall::on_transfer::invoke_on_transfer;

// Accumulation of a work-package/work-report is deferred in the case that it has a not-yet-fulfilled dependency and is 
// cancelled entirely in the case of an invalid dependency. Dependencies are specified  as work-package hashes and in order 
// to know which work-packages have been accumulated already, we maintain a history of what has been accumulated. This history 
// (AccumulatedHistory), is sufficiently large for an epoch worth of work-reports.

// We also maintain knowledge of ready (i.e. available and/or audited) but not-yet-accumulated work-reports in the state ReadyQueue.
// Each of these were made available at most one epoch ago but have or had unfulfilled dependencies. Alongside the work-report itself, 
// we retain its unaccumulated dependencies, a set of work-package hashes.
pub fn process(
    accumulated_history: &mut AccumulatedHistory,
    ready_queue: &mut ReadyQueue,
    service_accounts: ServiceAccounts,
    next_validators: ValidatorsData,
    queues_auth: AuthQueues,
    privileges: Privileges,
    post_tau: &TimeSlot,
    new_available_reports: &[WorkReport],
) -> Result<(OpaqueHash, ServiceAccounts, ValidatorsData, AuthQueues, Privileges), ProcessError> {
  
    log::debug!("Process accumulation");
    // We define the final state of the ready queue and the accumulated map by integrating those work-reports which were accumulated in this 
    // block and shifting any from the prior state with the oldest such items being dropped entirely:
    let current_block_accumulatable = get_current_block_accumulatable(new_available_reports, ready_queue, accumulated_history, post_tau);

    let partial_state = AccumulationPartialState {
        service_accounts,
        next_validators,
        queues_auth,
        privileges,
    };

    let (num_wi_accumulated, 
         mut post_partial_state, 
         transfers, 
         mut service_hash_pairs, 
         service_gas_pairs) = outer_accumulation(
        &get_gas_limit(&partial_state.privileges),
        &current_block_accumulatable,
        partial_state.clone(),
        &partial_state.privileges.always_acc,
    )?;

    let accumulation_root = get_acc_root(&mut service_hash_pairs);
    log::debug!("Accumulation root: 0x{}", utils::print_hash!(accumulation_root));

    accumulate_history::update(accumulated_history, map_workreports(&current_block_accumulatable));
    // The newly available work-reports, are partitioned into two sequences based on the condition of having zero prerequisite work-reports.
    // Those meeting the condition are accumulated immediately. Those not, (reports_for_queue) are for queued execution.
    let reports_for_queue = get_reports_for_queue(new_available_reports, &accumulated_history);
    ready_queue::update(ready_queue, accumulated_history, post_tau, reports_for_queue);

    save_statistics(&mut post_partial_state, &transfers, &service_gas_pairs, &current_block_accumulatable, num_wi_accumulated);

    Ok((accumulation_root, 
        post_partial_state.service_accounts, 
        post_partial_state.next_validators, 
        post_partial_state.queues_auth, 
        post_partial_state.privileges))
}

fn outer_accumulation(
    gas_limit: &Gas,
    reports: &[WorkReport],
    partial_state: AccumulationPartialState,
    service_gas_dict: &HashMap<ServiceId, Gas>

) -> Result<(u32, AccumulationPartialState, Vec<DeferredTransfer>, Vec<(ServiceId, OpaqueHash)>, Vec<(ServiceId, Gas)>), ProcessError>
{

    log::debug!("Outer accumulation");

    let mut i: u32 = 0;
    let mut gas_to_use = 0;

    for report in reports.iter() {
        for result in report.results.iter() {
            if result.gas + gas_to_use > *gas_limit {
                break;
            } 
            gas_to_use += result.gas;
        }
        i += 1;
    }

    if i == 0 {
        log::debug!("Exit outer accumulation: i = 0");
        return Ok((0, partial_state.clone(), vec![], vec![], vec![]));
    }

    let (star_partial_state,
         star_deferred_transfers, 
         star_service_gas_pairs, 
         star_gas_used) = parallelized_accumulation(partial_state, &reports[..i as usize], &service_gas_dict)?;

    let total_gas_used: Gas = star_gas_used.iter().map(|(_, gas)| *gas).sum();

    let (j, 
        prime_partial_state, 
        t_deferred_transfers,
        b_service_hash_pairs,
        u_gas_used) = outer_accumulation(&(*gas_limit - total_gas_used), 
                                                            &reports[i as usize..], 
                                                            star_partial_state, 
                                                            &HashMap::<ServiceId, Gas>::new())?;

    
    log::debug!("Finalized outer accumulation");
    return Ok((i + j, 
               prime_partial_state, 
               [star_deferred_transfers, t_deferred_transfers].concat(), 
               [star_service_gas_pairs, b_service_hash_pairs].concat(), 
               [star_gas_used, u_gas_used].concat()));
}

fn parallelized_accumulation(
    partial_state: AccumulationPartialState,
    reports: &[WorkReport],
    service_gas_dict: &HashMap<ServiceId, Gas>,
) -> Result<(AccumulationPartialState, Vec<DeferredTransfer>, Vec<(ServiceId, OpaqueHash)>, Vec<(ServiceId, Gas)>), ProcessError>
{
    //println!("\nParallelized accumulation");

    let mut s_services: Vec<ServiceId> = Vec::new();
    for report in reports.iter() {
        for result in report.results.iter() {
            //println!("service: {}", result.service);
            s_services.push(result.service);
        }
    }

    for service_gas_dict in service_gas_dict.iter() {
        //println!("service gas dict: {}", service_gas_dict.0);
        s_services.push(service_gas_dict.0.clone());
    }

    let mut u_gas_used: Vec<(ServiceId, Gas)> = vec![];
    let mut b_service_hash_pairs: Vec<(ServiceId, OpaqueHash)> = vec![];
    let mut t_deferred_transfers: Vec<DeferredTransfer> = vec![];
    let mut n_service_accounts: ServiceAccounts = ServiceAccounts::default();
    let mut m_service_accounts = HashSet::new();
    let mut p_preimages: Vec<(ServiceId, Vec<u8>)> = Vec::new();
    
    for service in s_services.iter() {
        
        //println!("Service: {}", *service);
        let (post_partial_state, 
            transfers, 
            service_hash, 
            gas, 
            preimages) = single_service_accumulation(partial_state.clone(), reports, service_gas_dict, service);

        u_gas_used.push((*service, gas));
        if let Some(hash) = service_hash {
            b_service_hash_pairs.push((*service, hash));
        }
        t_deferred_transfers.extend(transfers);
        p_preimages.extend(preimages);

        let o_d_services_keys: HashSet<_> = post_partial_state
                                                .service_accounts
                                                .iter()
                                                .map(|key| *key.0)
                                                .collect();

        let m: HashSet<_> = partial_state
                                .service_accounts
                                .iter()
                                .filter(|(key, _)| !o_d_services_keys.contains(key))
                                .map(|(k, _)| (k.clone()))
                                .collect();
        // Removed services
        m_service_accounts.extend(m);
        
        let mut d_services = partial_state.service_accounts.clone();
        d_services.remove(service);
        let d_keys_excluding_s: HashSet<_> = d_services.iter().map(|key| *key.0).collect();
        
        let n: HashMap<_, _> = post_partial_state
                                    .service_accounts
                                    .iter()
                                    .filter(|(key, _)| !d_keys_excluding_s.contains(key))
                                    .map(|(k, v)| (k.clone(), v.clone()))
                                    .collect();
        
        // New and modified services
        n_service_accounts.extend(n);

    }
    
    // Different services may not each contribute the same index for a new, altered or removed service. This cannot happen for the set of
    // removed and altered services since the code hash of removable services has no known preimage and thus cannot execute itself to make
    // an alteration. For new services this should also never happen since new indices are explicitly selected to avoid such conflicts.
    // In the unlikely event it does happen, the block must be considered invalid.
    for key in n_service_accounts.keys() {
        if m_service_accounts.contains(key) {
            log::error!("Service conflict: key {:?}", *key);
            return Err(ProcessError::AccumulateError(AccumulateErrorCode::ServiceConflict)); // Collision
        }
    }

    //let i_next_validatos = partial_state.next_validators.clone();
    //let q_queues_auth = partial_state.queues_auth.clone();
    let m_bless = partial_state.privileges.bless.clone();
    let a_assign = partial_state.privileges.assign.clone();
    let v_designate = partial_state.privileges.designate.clone();
    //let z_always_acc = partial_state.privileges.always_acc.clone();

    let (_post_partial_state, 
        _transfers, 
        _service_hash, 
        _gas,
        _preimages) = single_service_accumulation(partial_state.clone(), reports, service_gas_dict, &m_bless);

    let post_privileges = partial_state.privileges.clone();
    
    let (_post_partial_state, 
        _transfers, 
        _service_hash, 
        _gas,
        _preimages) = single_service_accumulation(partial_state.clone(), reports, service_gas_dict, &v_designate);

    let post_next_validators = partial_state.next_validators.clone();

    let (_post_partial_state, 
        _transfers, 
        _service_hash, 
        _gas, 
        _preimages) = single_service_accumulation(partial_state.clone(), reports, service_gas_dict, &a_assign);
    
    let post_queues_auth = partial_state.queues_auth.clone();

    let mut d_services = partial_state.service_accounts.clone();
    d_services.extend(n_service_accounts);

    let result_services: ServiceAccounts = d_services
                                            .iter()
                                            .filter(|(key, _)| !m_service_accounts.contains(key))
                                            .map(|(k, v)| (k.clone(), v.clone()))
                                            .collect();

    let final_services = preimage_integration(&result_services, &p_preimages);
       
    
    let result_partial_state = AccumulationPartialState {
        service_accounts: final_services,
        next_validators: post_next_validators,
        queues_auth: post_queues_auth,
        privileges: post_privileges,
    };

    log::debug!("Finalized paralellized accumulation");
    return Ok((result_partial_state, t_deferred_transfers, b_service_hash_pairs, u_gas_used));
}

fn single_service_accumulation(
    partial_state: AccumulationPartialState,
    reports: &[WorkReport],
    service_gas_dict: &HashMap<ServiceId, Gas>,
    service_id: &ServiceId,
) -> (AccumulationPartialState, Vec<DeferredTransfer>, Option<OpaqueHash>, Gas, Vec<(ServiceId, Vec<u8>)>)
{
    log::debug!("Single service accumulation. Service {:?}", *service_id);
    
    let mut total_gas = 0;
    let mut accumulation_operands: Vec<AccumulationOperand> = vec![];
    for report in reports.iter() {
        for result in report.results.iter() {
            if *service_id == result.service {
                total_gas += result.gas;
                //println!("total_gas: {:?}", total_gas);
                accumulation_operands.push(AccumulationOperand {
                    result: result.result.clone(),
                    exports_root: report.package_spec.exports_root,
                    auth_output: report.auth_output.clone(),
                    payload_hash: result.payload_hash,
                    code_hash: report.package_spec.hash,
                    authorizer_hash: report.authorizer_hash,
                    gas_limit: result.gas,
                });
            }
        }
    }

    if let Some(gas) = service_gas_dict.get(service_id) {
        total_gas += *gas;
    } 

    invoke_accumulation(
        partial_state,
        &get_current_block_slot(),
        service_id,
        total_gas,
        &accumulation_operands,
    )

}

fn get_acc_root(service_hash_pairs: &mut Vec<(ServiceId, OpaqueHash)>) -> OpaqueHash {

    service_hash_pairs.sort_by_key(|(service_id, _)| *service_id);
    
    let mut pairs_blob: Vec<Vec<u8>> = Vec::new();
    for (service_id, hash) in service_hash_pairs.iter() {
        pairs_blob.push([service_id.encode(), hash.encode()].concat());
    }
    /*for pair in pairs_blob.iter() {
        println!("encoded: {:x?}", pair);
    }*/
    let accumulation_root = trie::merkle_balanced(pairs_blob, sp_core::keccak_256);
    //println!("accumulation_root: {:x?}", accumulation_root);

    return accumulation_root;
}

// The preimage integration transforms a dictionary of service states and a set of service/hash pairs into a new 
// dictionary of service states. Preimage provisions into services which no longer exist or whose relevant request
// is dropped are disregarded.
fn preimage_integration(services: &ServiceAccounts, preimages: &[(ServiceId, Vec<u8>)]) -> ServiceAccounts {

    let mut services_result = services.clone();

    for pair in preimages.iter() {

        if services.contains_key(&pair.0) { 
                        
            let timeslots = services.get(&pair.0)
                                                       .unwrap()
                                                       .lookup
                                                       .get(&construct_lookup_key(&sp_core::blake2_256(&pair.1), pair.1.len() as u32));
            
            if timeslots.is_none() || (timeslots.is_some() && timeslots.len() == 0) {

                services_result.get_mut(&pair.0)
                               .unwrap()
                               .lookup
                               .insert(construct_lookup_key(&sp_core::blake2_256(&pair.1), pair.1.len() as u32), vec![get_current_block_slot()]);
                
                let preimage_hash = sp_core::blake2_256(&pair.1);
                let preimage_key = StateKeyType::Account(pair.0, construct_preimage_key(&preimage_hash).to_vec()).construct();
                services_result.get_mut(&pair.0)
                               .unwrap()
                               .preimages
                               .insert(preimage_key, pair.1.clone());
            }
        } 
    }

    return services_result;
}

fn select_deferred_transfers(deferred_transfers: &[DeferredTransfer], to_service: &ServiceId) -> Vec<DeferredTransfer> {

    let mut selected_transfers: Vec<DeferredTransfer> = vec![];

    for transfer in deferred_transfers.iter() {
        if transfer.to == *to_service {
            selected_transfers.push(transfer.clone());
        }
    }

    selected_transfers.sort_by_key(|transfer| transfer.from);

    return selected_transfers;
}

fn save_statistics(
    post_partial_state: &mut AccumulationPartialState,
    transfers: &Vec<DeferredTransfer>,
    service_gas_pairs: &Vec<(ServiceId, Gas)>,
    current_block_accumulatable: &Vec<WorkReport>,
    num_wi_accumulated: u32
) {

    let mut acc_stats: HashMap<ServiceId, (Gas, u32)> = HashMap::new();
    for (service_id, gas) in service_gas_pairs.iter() {
        let mut acc_curr_block_reports: Vec<WorkReport> = vec![];
        for report in current_block_accumulatable[..num_wi_accumulated as usize].iter() {
            for result in report.results.iter() {
                if *service_id == result.service {
                    acc_curr_block_reports.push(report.clone());
                }
            }
        }
        if acc_curr_block_reports.len() > 0 {
            acc_stats.insert(*service_id, (*gas, acc_curr_block_reports.len() as u32));
        }
    }

    statistics::set_acc_stats(acc_stats);


    let mut xfers_info: HashMap<ServiceId, (Account, Gas)> = HashMap::new();
    let mut xfers_stats: HashMap<ServiceId, (u32, Gas)> = HashMap::new();

    for service in post_partial_state.service_accounts.iter() {
        let service_id = service.0;
        let selected_transfers = select_deferred_transfers(&transfers, &service_id);
        let num_tranfers = selected_transfers.len();
        let xfer_result = invoke_on_transfer(
            &post_partial_state.service_accounts,
            &get_current_block_slot(),
            service_id,
            selected_transfers,
        );

        xfers_info.insert(*service_id,xfer_result.clone());

        if num_tranfers > 0 && xfers_info.get(service_id).is_some(){
            xfers_stats.insert(*service_id, (num_tranfers as u32, xfer_result.1));
        }
    }
    
    statistics::set_xfer_stats(xfers_stats);
    
    for service in xfers_info.iter() {
        post_partial_state.service_accounts.insert(*service.0, service.1.0.clone());
    }

}

fn get_gas_limit(privileges: &Privileges) -> Gas {
    
    let mut gas_privilege_services = 0;
    
    for gas in privileges.always_acc.iter() {
        gas_privilege_services += gas.1;
    }

    return std::cmp::max(TOTAL_GAS_ALLOCATED, (WORK_REPORT_GAS_LIMIT * CORES_COUNT as Gas) + gas_privilege_services);
}

// The newly available work-reports, are partitioned into two sequences based on the condition of having zero prerequisite work-reports.
// Those meeting the condition are accumulated immediately. 
fn get_reports_imm_accumulatable(reports: &[WorkReport]) -> Vec<WorkReport> {
    let mut new_imm_available_work_reports = vec![];
    for report in reports.iter() {
        if report.context.prerequisites.len() == 0 && report.segment_root_lookup.len() == 0 {
            new_imm_available_work_reports.push(report.clone());
        }
    }
    return new_imm_available_work_reports;
}

// These reports are for queued execution.
fn get_reports_for_queue(reports: &[WorkReport], accumulated_history: &AccumulatedHistory) -> Vec<ReadyRecord> {

    let new_ready_records: Vec<ReadyRecord> = D(reports);
    let mut records_with_dependencies = vec![];
    for record in new_ready_records.iter() {
        if record.dependencies.len() > 0 {
            records_with_dependencies.push(record.clone());
        }
    }

    let mut history: Vec<WorkPackageHash> = vec![];
    for epoch in accumulated_history.queue.iter() {
        for item in epoch.iter() {
            history.push(*item);
        }
    }
    
    queue_edit(&records_with_dependencies, &history)
}

// Returns a sequence of accumulatable work-reports in this block (W*)
fn get_current_block_accumulatable(
    reports: &[WorkReport], 
    ready: &ReadyQueue,
    accumulated_history: &AccumulatedHistory,
    slot: &TimeSlot)
    -> Vec<WorkReport> {
    
    let m = (*slot % EPOCH_LENGTH as TimeSlot) as usize;
    
    // W!
    let imm_accumulatable = get_reports_imm_accumulatable(reports);
    let mut imm_reports: Vec<WorkReport> = vec![];
    for report in imm_accumulatable.iter() {
        imm_reports.push(report.clone());
    }
    // ready_records[m]
    let mut ready_records: Vec<ReadyRecord> = vec![];
    for i in m..EPOCH_LENGTH {
        ready_records.extend_from_slice(&ready.queue[i]);
    }
    for i in 0..m {
        ready_records.extend_from_slice(&ready.queue[i]);
    }
    // WQ
    let for_queue = get_reports_for_queue(reports, accumulated_history);
    // ready_records + for_queue
    ready_records.extend_from_slice(&for_queue);

    let q = queue_edit(&ready_records, &map_workreports(&imm_reports));
    // W* = W! + Q(q)
    let mut current_block_accumulatable_reports = imm_reports;
    current_block_accumulatable_reports.extend_from_slice(&Q(&q));
       
    return current_block_accumulatable_reports;
}

// We further define the accumulation priority queue function Q, which provides the sequence of work-reports which
// are accumulatable given a set of not-yet-accumulated work-reports and their dependencies.
#[allow(non_snake_case)]
fn Q(ready_reports: &[ReadyRecord]) -> Vec<WorkReport> {

    let mut g: Vec<WorkReport> = vec![];
    for record in ready_reports.iter() {
        if record.dependencies.len() == 0 {
            g.push(record.report.clone());
        }
    }

    if g.len() == 0 {
        return vec![];
    }

    g.extend_from_slice(&Q(&queue_edit(ready_reports, &map_workreports(&g)).as_slice()));

    return g;  
}
#[allow(non_snake_case)]
fn D(reports: &[WorkReport]) -> Vec<ReadyRecord> {

    let mut ready_records: Vec<ReadyRecord> = vec![];

    for report in reports.iter() {
        let mut lookup_dep = vec![];
        for dep in report.segment_root_lookup.iter() {
            lookup_dep.push(dep.work_package_hash);
        }
        let mut dependencies = vec![];
        dependencies.extend_from_slice(&lookup_dep);
        dependencies.extend_from_slice(&report.context.prerequisites);
        ready_records.push(ReadyRecord{report: report.clone(), dependencies: dependencies});
    }

    return ready_records;
}

// We define the queue-editing function which is essentially a mutator function for items such as those of ready work reports
// parameterized by sets of now-accumulated work-package hashes, those in immediate available work reports. It is used to update queues
// of work-reports  when some of them are accumulated. Functionally, it removes all entries whose work-report’s hash is in
// the set provided as a parameter, and removes any dependencies which appear in said set.
fn queue_edit(ready_reports: &[ReadyRecord], hashes_to_remove: &[WorkPackageHash]) -> Vec<ReadyRecord> {

    let mut hashes: HashSet<WorkPackageHash> = HashSet::new();
    for hash in hashes_to_remove.iter() {
        hashes.insert(*hash);
    }

    let mut edited_records: Vec<ReadyRecord> = vec![];

    for ready in ready_reports.iter() {
        
        if hashes.contains(&ready.report.package_spec.hash) {
            continue;
        }

        let mut dependencies = vec![];
        
        for dep in ready.dependencies.iter() {
            if hashes.contains(dep) {
                continue;
            }
            dependencies.push(*dep);
        }
        edited_records.push(ReadyRecord{report: ready.report.clone(), dependencies});
    }

    return edited_records;

}

// The mapping function extracts the corresponding work-package hashes from a set of work-reports.
fn map_workreports(reports: &[WorkReport]) -> Vec<WorkPackageHash> {
    reports.iter().map(|report| report.package_spec.hash).collect::<Vec<OpaqueHash>>()
}

mod ready_queue {
    
    use super::*;

    pub fn update(ready_queue: &mut ReadyQueue, 
              accumulated_history: &AccumulatedHistory,
              post_tau: &TimeSlot,
              reports_for_queue: Vec<ReadyRecord>) 
    {
        let m = (*post_tau % EPOCH_LENGTH as TimeSlot) as usize;
        let tau = get_time();

        for i in 0..EPOCH_LENGTH {
            let queue_position = (EPOCH_LENGTH + m - i) % EPOCH_LENGTH as usize;
            let mut new_ready_record: Vec<ReadyRecord> = vec![];
            if i == 0 {
                new_ready_record = queue_edit(&reports_for_queue, &accumulated_history.queue[EPOCH_LENGTH - 1]);       
            } else if 1 <= i && i < (*post_tau - tau) as usize {
                //new_ready_record: Vec<ReadyRecord> = vec![];
            } else if i >= (*post_tau - tau) as usize {
                new_ready_record = queue_edit(&ready_queue.queue[queue_position], &accumulated_history.queue[EPOCH_LENGTH - 1]);
            }
            ready_queue.queue[queue_position] = new_ready_record;
        }
    }
}

mod accumulate_history {

    use super::*;

    pub fn update(acc_history: &mut AccumulatedHistory, hash_reports: Vec<WorkPackageHash>) {
        acc_history.queue.pop_front();
        let mut sorted_reports: Vec<WorkPackageHash> = hash_reports.clone();
        sorted_reports.sort();
        acc_history.queue.push_back(sorted_reports);
    }
}

mod test {
    
    #[test]
    fn select_deferred_transfers_test() {
        let mut transfer1 = super::DeferredTransfer::default();
        transfer1.from = 2;
        transfer1.to = 2;
        transfer1.amount = 100;

        let mut transfer2 = super::DeferredTransfer::default();
        transfer2.from = 3;
        transfer2.to = 4;
        
        let mut transfer3 = super::DeferredTransfer::default();
        transfer3.from = 1;
        transfer3.to = 2;
        transfer3.amount = 300;

        let mut transfer4 = super::DeferredTransfer::default();
        transfer4.from = 1;
        transfer4.to = 2;
        transfer4.amount = 400;

        let transfers = [transfer1, transfer2, transfer3, transfer4].to_vec();
        let selected_transfers = super::select_deferred_transfers(&transfers, &2);

        println!("selected transfers: {:?}", selected_transfers);
        // selected transfers: 
        // [DeferredTransfer { from: 1, to: 2, amount: 300, memo: 0, gas_limit: 0 }, 
        //  DeferredTransfer { from: 1, to: 2, amount: 400, memo: 0, gas_limit: 0 }, 
        //  DeferredTransfer { from: 2, to: 2, amount: 100, memo: 0, gas_limit: 0 }]
    }
}