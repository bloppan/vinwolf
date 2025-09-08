use {std::sync::LazyLock, std::sync::Mutex};

use jam_types::{
    AccumulatedHistory, AuthPools, AuthQueues, AvailabilityAssignments, RecentBlocks, DisputesRecords, EntropyPool, GlobalState, CoreIndex,
    OpaqueHash, Privileges, ReadyQueue, Safrole, ServiceAccounts, Statistics, TimeSlot, ValidatorSet, ValidatorsData, AvailabilityAssignment,
    Offenders, WorkReportHash, ProcessError, DisputesErrorCode, Entropy
};
use codec::Encode;

static GLOBAL_STATE: LazyLock<Mutex<GlobalState>> = LazyLock::new(|| {
    Mutex::new(GlobalState::default())
});

static STATE_ROOT: LazyLock<Mutex<OpaqueHash>> = LazyLock::new(|| {
    Mutex::new(OpaqueHash::default())
});

// Global state
pub fn get_global_state() -> &'static Mutex<GlobalState> {
    &GLOBAL_STATE
}
pub fn set_global_state(new_state: GlobalState) {
    *GLOBAL_STATE.lock().unwrap() = new_state;
}
// State root
pub fn set_state_root(new_root: OpaqueHash) {
    *STATE_ROOT.lock().unwrap() = new_root;
}
pub fn get_state_root() -> &'static Mutex<OpaqueHash> {
    &STATE_ROOT
}

static CURRENT_SLOT: LazyLock<Mutex<TimeSlot>> = LazyLock::new(|| {
    Mutex::new(TimeSlot::default())
});

pub mod time {

    use super::*;

    pub fn set(new_time: TimeSlot) {
        GLOBAL_STATE.lock().unwrap().time = new_time;
    }
    pub fn get() -> TimeSlot {
        GLOBAL_STATE.lock().unwrap().time
    }
    pub fn set_current(slot: &TimeSlot) {
        *CURRENT_SLOT.lock().unwrap() = *slot;
    }
    pub fn get_current() -> TimeSlot {
        *CURRENT_SLOT.lock().unwrap()
    }
}

static RECENT_ENTROPY: LazyLock<Mutex<OpaqueHash>> = LazyLock::new(|| {
    Mutex::new(OpaqueHash::default())
});

pub mod entropy {

    use super::*;

    pub fn set(new_entropy: EntropyPool) {
        GLOBAL_STATE.lock().unwrap().entropy = new_entropy;
    }
    pub fn get() -> EntropyPool {
        GLOBAL_STATE.lock().unwrap().entropy.clone()
    }
    pub fn get_recent() -> Entropy {
        Entropy { entropy: RECENT_ENTROPY.lock().unwrap().clone() }
    }
    pub fn set_recent(entropy: OpaqueHash) {
        *RECENT_ENTROPY.lock().unwrap() = entropy;
    }
}

static CURR_BLOCK_HISTORY: LazyLock<Mutex<RecentBlocks>> = LazyLock::new(|| {
    Mutex::new(RecentBlocks::default())
});

pub mod recent_history {
    
    use super::*;

    pub fn set(new_recent_history: RecentBlocks) {
        GLOBAL_STATE.lock().unwrap().recent_history = new_recent_history;
    }
    pub fn get() -> RecentBlocks {
        GLOBAL_STATE.lock().unwrap().recent_history.clone()
    }
    pub fn get_current() -> &'static Mutex<RecentBlocks> {
        &CURR_BLOCK_HISTORY
    }
    pub fn set_current(new_state: RecentBlocks) {
        *CURR_BLOCK_HISTORY.lock().unwrap() = new_state;
    }
}

pub mod reports {

    use super::*;

    pub fn set(new_availability: AvailabilityAssignments) {
        GLOBAL_STATE.lock().unwrap().availability = new_availability;
    }
    pub fn get() -> AvailabilityAssignments {
        GLOBAL_STATE.lock().unwrap().availability.clone()
    }
    pub fn add_assignment(assignment: &AvailabilityAssignment, state: &mut AvailabilityAssignments) {
        state.list[assignment.report.core_index as usize] = Some(assignment.clone());
    }
    pub fn remove_assignment(core_index: &CoreIndex, state: &mut AvailabilityAssignments) {
        state.list[*core_index as usize] = None;
    }
    pub fn update_first_step(availability_state: &mut AvailabilityAssignments, disputes_state: &DisputesRecords) {
        // We clear any work-reports which we judged as uncertain or invalid from their core:
        for assignment in availability_state.list.iter_mut() {
            if let Some(availability_assignment) = assignment {
                // Calculate target hash
                let target_hash = sp_core::blake2_256(&availability_assignment.report.encode());
                // Check if the hash is contained in bad or wonky sets
                if disputes_state.bad.contains(&target_hash)
                    || disputes_state.wonky.contains(&target_hash)
                {
                    *assignment = None; // Set to None
                }
            }
        }
    }
}

pub mod disputes {

    use super::*;

    pub fn set(new_disputes: DisputesRecords) {
        GLOBAL_STATE.lock().unwrap().disputes = new_disputes;
    }
    pub fn get() -> DisputesRecords {
        GLOBAL_STATE.lock().unwrap().disputes.clone()
    }
    pub fn update(
        disputes_state: &mut DisputesRecords, 
        new_wr_reported: &DisputesRecords, 
        culprits_keys: &[WorkReportHash],
        faults_keys: &[WorkReportHash]) 
    -> Result<Offenders, ProcessError> {

        let new_offenders = Vec::from([culprits_keys, faults_keys].concat());

        // In the disputes extrinsic can not be offenders already reported
        let all_offenders = Vec::from([disputes_state.offenders.clone(), new_offenders.clone()].concat());
        if utils::common::has_duplicates(&all_offenders) {
            return Err(ProcessError::DisputesError(DisputesErrorCode::OffenderAlreadyReported));
        }   

        // If the state was initialized, then we save the auxiliar records in the state
        disputes_state.good.extend_from_slice(&new_wr_reported.good);
        disputes_state.bad.extend_from_slice(&new_wr_reported.bad);
        disputes_state.wonky.extend_from_slice(&new_wr_reported.wonky);
        let mut offenders = new_offenders.clone();
        offenders.sort();
        disputes_state.offenders.extend_from_slice(&offenders);

        Ok(new_offenders)
    }
}


pub mod auth_pools {
    
    use super::*;

    pub fn set(new_authpool: AuthPools) {
        GLOBAL_STATE.lock().unwrap().auth_pools = new_authpool;
    }
    pub fn get() -> AuthPools {
        GLOBAL_STATE.lock().unwrap().auth_pools.clone()
    }
}

pub mod auth_queues {

    use super::*;

    pub fn set(new_authqueue: AuthQueues) {
        GLOBAL_STATE.lock().unwrap().auth_queues = new_authqueue;
    }
    pub fn get() -> AuthQueues {
        GLOBAL_STATE.lock().unwrap().auth_queues.clone()
    }
}

pub mod acc_outputs {

    use jam_types::RecentAccOutputs;

    use super::*;

    pub fn set(new_acc_outputs: RecentAccOutputs) {
        GLOBAL_STATE.lock().unwrap().recent_acc_outputs = new_acc_outputs;
    }
    pub fn get() -> RecentAccOutputs {
        GLOBAL_STATE.lock().unwrap().recent_acc_outputs.clone()
    }
}

pub mod statistics {

    use super::*;

    pub fn set(new_statistics: Statistics) {
        GLOBAL_STATE.lock().unwrap().statistics = new_statistics;
    }
    pub fn get() -> Statistics {
        GLOBAL_STATE.lock().unwrap().statistics.clone()
    }
}

pub mod safrole {
    
    use super::*;

    pub fn set(new_safrole: Safrole) {
        GLOBAL_STATE.lock().unwrap().safrole = new_safrole;
    }
    pub fn get() -> Safrole {
        GLOBAL_STATE.lock().unwrap().safrole.clone()
    }
}

pub mod service_accounts {

    use super::*;

    pub fn set(new_service_accounts: ServiceAccounts) {
        GLOBAL_STATE.lock().unwrap().service_accounts = new_service_accounts;
    }
    pub fn get() -> ServiceAccounts {
        GLOBAL_STATE.lock().unwrap().service_accounts.clone()
    }
}

pub mod acc_history {

    use super::*;

    pub fn set(new_accumulation_history: AccumulatedHistory) {
        GLOBAL_STATE.lock().unwrap().accumulation_history = new_accumulation_history;
    }
    pub fn get() -> AccumulatedHistory {
        GLOBAL_STATE.lock().unwrap().accumulation_history.clone()
    }
}

pub mod ready_queue {

    use super::*;

    pub fn set(new_ready_queue: ReadyQueue) {
        GLOBAL_STATE.lock().unwrap().ready_queue = new_ready_queue;
    }
    pub fn get() -> ReadyQueue {
        GLOBAL_STATE.lock().unwrap().ready_queue.clone()
    }
}

pub mod privileges {

    use super::*;

    pub fn set(new_privileges: Privileges) {
        GLOBAL_STATE.lock().unwrap().privileges = new_privileges;
    }
    pub fn get() -> Privileges {
        GLOBAL_STATE.lock().unwrap().privileges.clone()
    }
}

pub mod validators {

    use super::*;

    pub fn set(new_validators: ValidatorsData, validator_set: ValidatorSet) {

        let mut state = GLOBAL_STATE.lock().unwrap();

        match validator_set {
            ValidatorSet::Previous => {
                state.prev_validators = new_validators;
            },
            ValidatorSet::Current => {
                state.curr_validators = new_validators;
            },
            ValidatorSet::Next => {
                state.next_validators = new_validators;
            },
        }    
    }
    pub fn get(validator_set: ValidatorSet) -> ValidatorsData {
        
        match validator_set {
            ValidatorSet::Previous => {
                return GLOBAL_STATE.lock().unwrap().prev_validators.clone();
            },
            ValidatorSet::Current => {
                return GLOBAL_STATE.lock().unwrap().curr_validators.clone();
            },
            ValidatorSet::Next => {
                return GLOBAL_STATE.lock().unwrap().next_validators.clone();
            },
        }
    }
}