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
    Account, AccumulateErrorCode, AccumulatedHistory, AccumulationOperand, AccumulationPartialState, AuthQueues, DeferredTransfer, Gas, 
    OpaqueHash, Privileges, ProcessError, ReadyQueue, ReadyRecord, RecentAccOutputs, ServiceAccounts, ServiceId, StateKeyType, TimeSlot, 
    ValidatorsData, WorkPackageHash, WorkReport
};
use codec::{Encode, EncodeLen};
use utils::serialization::{StateKeyTrait, construct_lookup_key, construct_preimage_key};
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
        manager: privileges.manager,
        assign: privileges.assign,
        designate: privileges.designate,
        always_acc: privileges.always_acc,
    };

    let (num_wi_accumulated, 
         mut post_partial_state, 
         transfers, 
         mut service_hash_pairs, 
         service_gas_pairs) = outer_accumulation(
        &get_gas_limit(&partial_state.always_acc),
        &current_block_accumulatable,
        partial_state.clone(),
        &partial_state.always_acc,
    )?;

    state_handler::acc_outputs::set(service_hash_pairs.clone());
    log::debug!("service-hash pairs: {:?}", service_hash_pairs);

    let acc_root = get_acc_root(&mut service_hash_pairs);
    log::debug!("Accumulation root: 0x{}", utils::print_hash!(acc_root));

    accumulate_history::update(accumulated_history, map_workreports(&current_block_accumulatable));
    // The newly available work-reports, are partitioned into two sequences based on the condition of having zero prerequisite work-reports.
    // Those meeting the condition are accumulated immediately. Those not, (reports_for_queue) are for queued execution.
    let reports_for_queue = get_reports_for_queue(new_available_reports, &accumulated_history);
    ready_queue::update(ready_queue, accumulated_history, post_tau, reports_for_queue);

    save_statistics(&mut post_partial_state, &transfers, &service_gas_pairs, &current_block_accumulatable, num_wi_accumulated);
    let post_privileges = Privileges {
        manager: post_partial_state.manager,
        assign: post_partial_state.assign,
        designate: post_partial_state.designate,
        always_acc: post_partial_state.always_acc,
    };

    Ok((acc_root, 
        post_partial_state.service_accounts, 
        post_partial_state.next_validators, 
        post_partial_state.queues_auth, 
        post_privileges))
}

fn outer_accumulation(
    gas_limit: &Gas,
    reports: &[WorkReport],
    partial_state: AccumulationPartialState,
    service_gas_dict: &HashMap<ServiceId, Gas>

) -> Result<(u32, AccumulationPartialState, Vec<DeferredTransfer>, RecentAccOutputs, Vec<(ServiceId, Gas)>), ProcessError>
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
        return Ok((0, partial_state.clone(), vec![], RecentAccOutputs::default(), vec![]));
    }

    let (star_partial_state,
         star_deferred_transfers, 
         star_service_hash, 
         star_gas_used) = parallelized_accumulation(partial_state, &reports[..i as usize], &service_gas_dict)?;

    let total_gas_used: Gas = star_gas_used.iter().map(|(_, gas)| *gas).sum();

    let (j, 
        prime_partial_state, 
        t_deferred_transfers,
        b_service_hash,
        u_gas_used) = outer_accumulation(&(*gas_limit - total_gas_used), 
                                                            &reports[i as usize..], 
                                                            star_partial_state, 
                                                            &HashMap::<ServiceId, Gas>::new())?;

    log::debug!("Finalized outer accumulation");

    let recent_acc_outputs = RecentAccOutputs {
            pairs: star_service_hash.pairs.iter().cloned()
                .chain(b_service_hash.pairs.iter().cloned())
                .collect(),
    };

    return Ok((i + j, 
               prime_partial_state, 
               [star_deferred_transfers, t_deferred_transfers].concat(), 
               recent_acc_outputs, 
               [star_gas_used, u_gas_used].concat()));
}

fn parallelized_accumulation(
    partial_state: AccumulationPartialState,
    reports: &[WorkReport],
    service_gas_dict: &HashMap<ServiceId, Gas>,
) -> Result<(AccumulationPartialState, Vec<DeferredTransfer>, RecentAccOutputs, Vec<(ServiceId, Gas)>), ProcessError>
{
    //println!("\nParallelized accumulation");

    let mut s_services: Vec<ServiceId> = Vec::new();
    for report in reports.iter() {
        for result in report.results.iter() {
            if !s_services.contains(&result.service) {
                s_services.push(result.service);
            }
        }
    }

    for service_gas_dict in service_gas_dict.iter() {
        if !s_services.contains(service_gas_dict.0) {
            s_services.push(service_gas_dict.0.clone());
        }
    }

    let mut acc_output_map: HashMap<ServiceId, (AccumulationPartialState, Vec<DeferredTransfer>, Option<OpaqueHash>, Gas, Vec<(u32, Vec<u8>)>)> = HashMap::new();
    let mut u_gas_used: Vec<(ServiceId, Gas)> = vec![];
    let mut b_service_hash: RecentAccOutputs = RecentAccOutputs::default();
    let mut t_deferred_transfers: Vec<DeferredTransfer> = vec![];
    let mut n_service_accounts: ServiceAccounts = ServiceAccounts::default();
    let mut m_service_accounts = HashSet::new();
    let mut p_preimages: Vec<(ServiceId, Vec<u8>)> = Vec::new();
    
    log::info!("privileged services: manager: {:?}, assign: {:?}, designate: {:?}, always_acc: {:?}", 
                                partial_state.manager, partial_state.assign, partial_state.designate, partial_state.always_acc);

    log::info!("Services to accumulate: {:?}", s_services);

    for service in s_services.iter() {
        
        let acc_output = single_service_accumulation(partial_state.clone(), reports, service_gas_dict, service);

        // Insert the acc_output only if the gas used in acc is > 0
        if acc_output.3 > 0 {
            acc_output_map.insert(*service, acc_output.clone());
        }

        let post_partial_state = &acc_output.0;
        let transfers = &acc_output.1;
        let service_hash = &acc_output.2;
        let gas = &acc_output.3;
        let preimages = &acc_output.4;

        u_gas_used.push((*service, *gas));
        if let Some(hash) = service_hash {
            b_service_hash.pairs.push((*service, *hash));
        }
        t_deferred_transfers.extend(transfers.clone());
        p_preimages.extend(preimages.clone());

        let o_d_services_keys: HashSet<_> = post_partial_state
                                                .service_accounts
                                                .iter()
                                                .map(|key| *key.0)
                                                .collect();

        let m: HashSet<_> = partial_state
                                .service_accounts
                                .iter()
                                .filter(|(key, _)| !o_d_services_keys.contains(key))
                                .map(|(k, _)| k.clone())
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

    if !acc_output_map.contains_key(&partial_state.manager) {
        let acc_output = single_service_accumulation(partial_state.clone(), reports, service_gas_dict, &partial_state.manager);
        // Insert the acc_output only if the gas used in acc is > 0
        if acc_output.3 > 0 {
            acc_output_map.insert(partial_state.manager, acc_output);
        }
    } 
    
    let m_post_partial_state = if let Some(service) = acc_output_map.get(&partial_state.manager) {
        service.0.clone()
    } else {
        partial_state.clone()
    };

    /*let result = acc_output_map.get(&partial_state.manager).unwrap();
    let m_post_partial_state = result.0.clone();*/

    let (post_manager, star_assign, star_v_designate, post_always_acc) = (m_post_partial_state.manager, 
                                                                                    m_post_partial_state.assign.clone(),
                                                                                    m_post_partial_state.designate,
                                                                                    m_post_partial_state.always_acc);

    log::info!("post_manager: {:?}, star_assign: {:?}, star_v_designate: {:?}, post_always_acc: {:?}",
                post_manager, star_assign, star_v_designate, post_always_acc);
                                                                                
    let mut post_assign: Box<[ServiceId; CORES_COUNT]> = Box::new(std::array::from_fn(|_| ServiceId::default()));

    for core_index in 0..CORES_COUNT{

        if !acc_output_map.contains_key(&star_assign[core_index]) {
            let acc_output = single_service_accumulation(partial_state.clone(), reports, service_gas_dict, &star_assign[core_index]);
            // Insert the acc_output only if the gas used in acc is > 0
            if acc_output.3 > 0 {
                acc_output_map.insert(star_assign[core_index], acc_output);
            }
        }
        post_assign[core_index] = if let Some(service) = acc_output_map.get(&star_assign[core_index]) {
            service.0.assign[core_index]
        } else {
            star_assign[core_index]
        };
    }

    if !acc_output_map.contains_key(&star_v_designate) {
        let acc_output = single_service_accumulation(partial_state.clone(), reports, service_gas_dict, &star_v_designate);
        // Insert the acc_output only if the gas used in acc is > 0
        if acc_output.3 > 0 {
            acc_output_map.insert(star_v_designate, acc_output);
        }
    }

    let post_v_designate = if let Some(service) = acc_output_map.get(&star_v_designate) {
        service.0.designate
    } else {
        star_v_designate
    };
    
    if !acc_output_map.contains_key(&partial_state.designate) {
        let acc_output = single_service_accumulation(partial_state.clone(), reports, service_gas_dict, &partial_state.designate);
        // Insert the acc_output only if the gas used in acc is > 0
        if acc_output.3 > 0 {
            acc_output_map.insert(partial_state.designate, acc_output);    
        }
    }

    let post_next_validators = if let Some(service) = acc_output_map.get(&partial_state.designate) {
        service.0.next_validators.clone()
    } else {
        partial_state.next_validators.clone()
    };
    
    let mut post_queues_auth: AuthQueues = AuthQueues::default();

    for core_index in 0..CORES_COUNT {

        if !acc_output_map.contains_key(&partial_state.assign[core_index]) {
            let acc_output = single_service_accumulation(partial_state.clone(), reports, service_gas_dict, &partial_state.assign[core_index]);
            if acc_output.3 > 0 {
                acc_output_map.insert(partial_state.assign[core_index], acc_output);
            }
        }
        post_queues_auth.0[core_index] = if let Some(service) = acc_output_map.get(&partial_state.assign[core_index]) {
            service.0.queues_auth.0[core_index].clone()
        } else {
            partial_state.queues_auth.0[core_index].clone()
        };
    }

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
        manager: post_manager,
        assign: post_assign.clone(),
        designate: post_v_designate,
        always_acc: post_always_acc,
    };

    log::debug!("Finalized paralellized accumulation");
    return Ok((result_partial_state, t_deferred_transfers, b_service_hash, u_gas_used));
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
                    auth_trace: report.auth_trace.clone(),
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
        &state_handler::time::get_current(),
        service_id,
        total_gas,
        &accumulation_operands,
    )

}

fn get_acc_root(service_hash: &mut RecentAccOutputs) -> OpaqueHash {

    service_hash.pairs.sort_by_key(|(service_id, _)| *service_id);
    
    let mut pairs_blob: Vec<Vec<u8>> = Vec::new();

    for (service_id, hash) in &service_hash.pairs {
        pairs_blob.push([service_id.encode(), hash.encode()].concat());
    }

    utils::trie::merkle_balanced(pairs_blob, sp_core::keccak_256)
}

// The preimage integration transforms a dictionary of service states and a set of service/hash pairs into a new 
// dictionary of service states. Preimage provisions into services which no longer exist or whose relevant request
// is dropped are disregarded.
fn preimage_integration(services: &ServiceAccounts, preimages: &[(ServiceId, Vec<u8>)]) -> ServiceAccounts {

    let mut services_result = services.clone();

    for service_value in preimages.iter() {

        if services.contains_key(&service_value.0) { 

            let lookup_key = StateKeyType::Account(service_value.0, construct_lookup_key(&sp_core::blake2_256(&service_value.1), service_value.1.len() as u32)).construct();

            let timeslots = services.get(&service_value.0)
                                                       .unwrap()
                                                       .storage
                                                       .get(&lookup_key);
            
            if timeslots.is_none() || (timeslots.is_some() && timeslots.len() == 0) {

                services_result.get_mut(&service_value.0)
                               .unwrap()
                               .storage
                               .insert(lookup_key, Vec::<TimeSlot>::from([state_handler::time::get_current()]).encode_len());
                
                let preimage_hash = sp_core::blake2_256(&service_value.1);
                let preimage_key = StateKeyType::Account(service_value.0, construct_preimage_key(&preimage_hash)).construct();
                services_result.get_mut(&service_value.0)
                               .unwrap()
                               .storage
                               .insert(preimage_key, service_value.1.clone());
            }
        } 
    }

    return services_result;
}

// We define a selection function which maps a sequence of deferred transfers and a desired destination service index into the sequence 
// of transfers targeting said service, ordered primarily according to the source service index and secondary their order within the 
// sequence of implied transfers
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

    // We compose our accumulation statistics, which is a mapping from the service indices which were accumulated to the amount of 
    // gas used throughout accumulation and the number of work-items accumulated.
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

    let mut xfers_info: HashMap<ServiceId, (Account, Gas)> = HashMap::new();
    // Furthermore we build the deferred transfers statistics as the number of transfers and the total gas used in transfer processing 
    // for each destination service index.
    let mut xfers_stats: HashMap<ServiceId, (u32, Gas)> = HashMap::new();

    for service_account in post_partial_state.service_accounts.iter() {
        let service_id = service_account.0;
        let selected_transfers = select_deferred_transfers(&transfers, &service_id);
        let num_tranfers = selected_transfers.len();
        let xfer_result = invoke_on_transfer(
            &post_partial_state.service_accounts,
            &state_handler::time::get_current(),
            service_id,
            selected_transfers,
        );

        xfers_info.insert(*service_id,xfer_result.clone());

        if num_tranfers > 0 && xfers_info.get(service_id).is_some(){
            xfers_stats.insert(*service_id, (num_tranfers as u32, xfer_result.1));
        }
    }
    
    // The second intermediate state may then be defined with all the deferred effects of the transfers applied followed by the
    // last-accumulation record being updated for all accumulated services
    for service in xfers_info.iter() {
        if acc_stats.contains_key(service.0) {
            let mut post_account = service.1.0.clone();
            post_account.last_acc = state_handler::time::get_current();
            post_partial_state.service_accounts.insert(*service.0, post_account);
        } else {
            post_partial_state.service_accounts.insert(*service.0, service.1.0.clone());
        }
    }
    
    statistics::set_acc_stats(acc_stats);
    statistics::set_xfer_stats(xfers_stats);
}

fn get_gas_limit(always_acc: &HashMap<ServiceId, Gas>) -> Gas {
    
    let mut gas_privilege_services = 0;
    
    for gas in always_acc.iter() {
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
        let tau = state_handler::time::get();

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