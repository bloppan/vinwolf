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

use std::collections::{HashSet, VecDeque};
use crate::constants::EPOCH_LENGTH;
use crate::types::{
    AccumulatedHistory, EntropyPool, OpaqueHash, OutputAccumulation, Privileges, ReadyQueue, ReadyRecord, ServiceAccounts, TimeSlot, 
    WorkPackageHash, WorkReport
};
use super::{get_time, ProcessError};

// Accumulation of a work-package/work-report is deferred in the case that it has a not-yet-fulfilled dependency and is 
// cancelled entirely in the case of an invalid dependency. Dependencies are specified  as work-package hashes and in order 
// to know which work-packages have been accumulated already, we maintain a history of what has been accumulated. This history 
// (AccumulatedHistory), is sufficiently large for an epoch worth of work-reports.
// 
// We also maintain knowledge of ready (i.e. available and/or audited) but not-yet-accumulated work-reports in the state ReadyQueue.
// Each of these were made available at most one epoch ago but have or had unfulfilled dependencies. Alongside the work-report itself, 
// we retain its unaccumulated dependencies, a set of work-package hashes.
pub fn process_accumulation(
    accumulated_history: &mut AccumulatedHistory,
    ready_queue: &mut ReadyQueue,
    entropy_pool: &EntropyPool,
    privileges: &Privileges,
    services_accounts: &ServiceAccounts,
    post_tau: &TimeSlot,
    reports: &[WorkReport],
) -> Result<OutputAccumulation, ProcessError> {

    // The newly available work-reports, are partitioned into two sequences based on the condition of having zero prerequisite work-reports.
    // Those meeting the condition are accumulated immediately. Those not, (reports_for_queue) are for queued execution.
    let reports_for_queue = get_reports_for_queued(reports, &accumulated_history);
    
    // We define the final state of the ready queue and the accumulated map by integrating those work-reports which were accumulated in this 
    // block and shifting any from the prior state with the oldest such items being dropped entirely:
    let current_block_accumulatable = get_current_block_accumulatable(reports, ready_queue, accumulated_history, post_tau);
    accumulated_history.update(map_workreports(&current_block_accumulatable));
    ready_queue.update(accumulated_history, post_tau, reports_for_queue);

    Ok(OutputAccumulation::Ok([0; 32]))
}

// The newly available work-reports, are partitioned into two sequences based on the condition of having zero prerequisite work-reports.
// Those meeting the condition are accumulated immediately. 
fn get_reports_imm_accumulable(reports: &[WorkReport]) -> Vec<WorkReport> {
    let mut new_imm_available_work_reports = vec![];
    for report in reports.iter() {
        if report.context.prerequisites.len() == 0 && report.segment_root_lookup.0.len() == 0 {
            new_imm_available_work_reports.push(report.clone());
        }
    }
    return new_imm_available_work_reports;
}

// These reports are for queued execution.
fn get_reports_for_queued(reports: &[WorkReport], accumulated_history: &AccumulatedHistory) -> Vec<ReadyRecord> {

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

// Returns a sequence of accumulatable wor-reports in this block (W*)
fn get_current_block_accumulatable(
    reports: &[WorkReport], 
    ready: &ReadyQueue,
    accumulated_history: &AccumulatedHistory,
    slot: &TimeSlot)
    -> Vec<WorkReport> {
    
    let m = (*slot % EPOCH_LENGTH as TimeSlot) as usize;
    
    // W!
    let imm_accumulatable = get_reports_imm_accumulable(reports);
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
    let for_queue = get_reports_for_queued(reports, accumulated_history);
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

fn D(reports: &[WorkReport]) -> Vec<ReadyRecord> {

    let mut ready_records: Vec<ReadyRecord> = vec![];

    for report in reports.iter() {
        let mut lookup_dep = vec![];
        for dep in report.segment_root_lookup.0.iter() {
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
        edited_records.push(ReadyRecord{report: ready.report.clone(), dependencies: dependencies});
    }

    return edited_records;

}

// The mapping function extracts the corresponding work-package hashes from a set of work-reports.
fn map_workreports(reports: &[WorkReport]) -> Vec<WorkPackageHash> {
    reports.iter().map(|report| report.package_spec.hash).collect::<Vec<OpaqueHash>>()
}

impl ReadyQueue {
 
    fn update(
            &mut self, 
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
                new_ready_record = queue_edit(&self.queue[queue_position], &accumulated_history.queue[EPOCH_LENGTH - 1]);
            }
            self.queue[queue_position] = new_ready_record;
        }
        
    }
}

impl AccumulatedHistory {

    fn update(&mut self, hash_reports: Vec<WorkPackageHash>) {
        self.queue.pop_front();
        let mut sorted_reports: Vec<WorkPackageHash> = hash_reports.clone();
        sorted_reports.sort();
        self.queue.push_back(sorted_reports);
    }
}

impl Default for AccumulatedHistory {
    fn default() -> Self {
        AccumulatedHistory {
            queue: {
                let mut queue: VecDeque<Vec<WorkPackageHash>> = VecDeque::new();
                for _ in 0..EPOCH_LENGTH {
                    queue.push_back(vec![]);
                }
                queue
            }
        }
    }
}

