/*
    Our state may be logically partitioned into several largely independent segments which can both help avoid visual clutter 
    within our protocol description and provide formality over elements of computation which may be simultaneously calculated 
    (i.e. parallelized). We therefore pronounce an equivalence between σ (some complete state) and a tuple of partitioned 
    segments of that state:

    σ ≡ (α, β, γ, δ, η, ι, κ, λ, ρ, τ, φ, χ, ψ, π, ϑ, ξ)

    In summary, δ is the portion of state dealing with services, analogous in Jam to the Yellow Paper’s (smart contract) accounts, 
    the only state of the YP’s Ethereum. The identities of services which hold some privileged status are tracked in χ.

    Validators, who are the set of economic actors uniquely privileged to help build and maintain the Jam chain, are identified 
    within κ, archived in λ and enqueued from ι. All other state concerning the determination of these keys is held within γ. 
    Note this is a departure from the YP proofof-work definitions which were mostly stateless, and this set was not enumerated 
    but rather limited to those with sufficient compute power to find a partial hash-collision in the sha2-256 cryptographic hash 
    function. An on-chain entropy pool is retained in η.

    Our state also tracks two aspects of each core: α, the authorization requirement which work done on that core must satisfy at 
    the time of being reported on-chain, together with the queue which fills this, φ; and ρ, each of the cores currently assigned 
    report, the availability of whose work-package must yet be assured by a super-majority of validators.

    Finally, details of the most recent blocks and timeslot index are tracked in β and τ respectively, work-reports which are 
    ready to be accumulated and work-packages which were recently accumulated are tracked in ϑ and ξ respectively and, judgments 
    are tracked in ψ and validator statistics are tracked in π.
*/
use {once_cell::sync::Lazy, sp_core::blake2_256, std::sync::Mutex};

use crate::types::{
    AccumulatedHistory, AuthPools, AuthQueues, AvailabilityAssignments, Block, BlockHistory, DisputesRecords, EntropyPool, GlobalState, 
    OpaqueHash, Privileges, ProcessError, ReadyQueue, Safrole, SerializedState, ServiceAccounts, ServiceInfo, StateKey, 
    Statistics, TimeSlot, ValidatorSet, ValidatorsData, 
};
use crate::constants::{
    ACCUMULATION_HISTORY, AUTH_POOLS, AUTH_QUEUE, AVAILABILITY, CURR_VALIDATORS, DISPUTES, ENTROPY, NEXT_VALIDATORS, PREV_VALIDATORS, PRIVILEGES, 
    READY_QUEUE, RECENT_HISTORY, SAFROLE, STATISTICS, TIME
};
use crate::utils::codec::{Encode, EncodeLen};
use crate::utils::codec::jam::global_state::{construct_preimage_key, construct_lookup_key, construct_storage_key, StateKeyTrait};

pub mod accumulation; pub mod authorization; pub mod disputes; pub mod entropy; pub mod safrole; pub mod recent_history; pub mod reporting_assurance;
pub mod services; pub mod time; pub mod statistics; pub mod validators; 

static GLOBAL_STATE: Lazy<Mutex<GlobalState>> = Lazy::new(|| {
    Mutex::new(GlobalState::default())
});

static STATE_ROOT: Lazy<Mutex<OpaqueHash>> = Lazy::new(|| {
    Mutex::new(OpaqueHash::default())
});

// We specify the state transition function as the implication of formulating all items of posterior state in terms of the prior
// state and block. To aid the architecting of implementations which parallelize this computation, we minimize the depth of the
// dependency graph where possible. 
pub fn state_transition_function(block: &Block) -> Result<(), ProcessError> {
    
    block.header.verify(&block.extrinsic)?;

    let mut new_state = get_global_state().lock().unwrap().clone();
    
    time::set_current_slot(&block.header.unsigned.slot);

    let mut reported_work_packages = Vec::new();
    for report in &block.extrinsic.guarantees.report_guarantee {
        reported_work_packages.push((report.report.package_spec.hash, report.report.package_spec.exports_root));
    }
    reported_work_packages.sort_by_key(|(hash, _)| *hash);

    recent_history::process(
        &mut new_state.recent_history,
        &blake2_256(&block.header.encode()), 
        &block.header.unsigned.parent_state_root,
        &reported_work_packages);

    let _ = disputes::process(
        &mut new_state.disputes,
        &mut new_state.availability,
        &block.extrinsic.disputes,
    )?;
    
    safrole::process(
        &mut new_state.safrole,
        &mut new_state.entropy,
        &mut new_state.curr_validators,
        &mut new_state.prev_validators,
        &mut new_state.time,
        &block.header,
        &block.extrinsic.tickets,
        &new_state.disputes.offenders)?;

    let new_available_workreports = reporting_assurance::process_assurances(
        &mut new_state.availability,
        &block.extrinsic.assurances,
        &block.header.unsigned.slot,
        &block.header.unsigned.parent,
    )?;

    let _ = reporting_assurance::process_guarantees(
        &mut new_state.availability, 
        &block.extrinsic.guarantees,
        &block.header.unsigned.slot,
        &new_state.entropy,
        &new_state.prev_validators,
        &new_state.curr_validators,
    )?; 

    let (accumulation_root, 
         service_accounts, 
         next_validators, 
         queue_auth, 
         privileges) = accumulation::process(
                                        &mut new_state.accumulation_history,
                                        &mut new_state.ready_queue,
                                        new_state.service_accounts,
                                        new_state.next_validators,
                                        new_state.auth_queues,
                                        new_state.privileges,
                                        &block.header.unsigned.slot,
                                        &new_available_workreports.reported)?;

    new_state.service_accounts = service_accounts;
    new_state.next_validators = next_validators;
    new_state.auth_queues = queue_auth;
    new_state.privileges = privileges;
    
    recent_history::finalize(
        &mut new_state.recent_history,
        &blake2_256(&block.header.encode()),
        &accumulation_root,
        &reported_work_packages);

    services::process(
        &mut new_state.service_accounts, 
        &block.header.unsigned.slot,  
        &block.extrinsic.preimages)?;
    
    authorization::process(
        &mut new_state.auth_pools, 
        &block.header.unsigned.slot, 
        &block.extrinsic.guarantees);

    statistics::process(
        &mut new_state.statistics, 
        &block.header.unsigned.slot, 
        &block.header.unsigned.author_index, 
        &block.extrinsic,
        &new_available_workreports.reported,
    );
    
    set_global_state(new_state);    

    Ok(())
}

impl GlobalState {

    pub fn serialize(&self) -> SerializedState {

        let mut state = SerializedState::default();

        state.map.insert(StateKey::U8(AUTH_POOLS).construct(), self.auth_pools.encode());
        state.map.insert(StateKey::U8(AUTH_QUEUE).construct(), self.auth_queues.encode());
        state.map.insert(StateKey::U8(RECENT_HISTORY).construct(), self.recent_history.encode());
        state.map.insert(StateKey::U8(SAFROLE).construct(), self.safrole.encode());
        state.map.insert(StateKey::U8(DISPUTES).construct(), self.disputes.encode());
        state.map.insert(StateKey::U8(ENTROPY).construct(), self.entropy.encode());
        state.map.insert(StateKey::U8(NEXT_VALIDATORS).construct(), self.next_validators.encode());
        state.map.insert(StateKey::U8(CURR_VALIDATORS).construct(), self.curr_validators.encode());
        state.map.insert(StateKey::U8(PREV_VALIDATORS).construct(), self.prev_validators.encode());
        state.map.insert(StateKey::U8(AVAILABILITY).construct(), self.availability.encode());
        state.map.insert(StateKey::U8(TIME).construct(), self.time.encode());
        state.map.insert(StateKey::U8(PRIVILEGES).construct(), self.privileges.encode());
        state.map.insert(StateKey::U8(STATISTICS).construct(), self.statistics.encode());
        state.map.insert(StateKey::U8(READY_QUEUE).construct(), self.ready_queue.encode());
        state.map.insert(StateKey::U8(ACCUMULATION_HISTORY).construct(), self.accumulation_history.encode());
        
        for (service_id, account) in self.service_accounts.iter() {
            let key = StateKey::Service(255, *service_id).construct();       
            let service_info = ServiceInfo {
                balance: account.balance,
                code_hash: account.code_hash,
                acc_min_gas: account.acc_min_gas,
                xfer_min_gas: account.xfer_min_gas,
                bytes: account.get_footprint_and_threshold().1, // TODO bytes y items se calcula con la eq de threshold account (9.3)
                items: account.get_footprint_and_threshold().0,
            };
            println!("service: {} items: {} bytes: {}", service_id, service_info.items, service_info.bytes);
            // TODO revisar esto y ver si se puede hacer con encode account
            state.map.insert(key, service_info.encode());

            for preimage in account.preimages.iter() {
                let key = StateKey::Account(*service_id, construct_preimage_key(preimage.0).to_vec()).construct();
                state.map.insert(key, preimage.1.encode());
            }
            
            for lookup in account.lookup.iter() {
                let key = StateKey::Account(*service_id, construct_lookup_key(&lookup.0.0, lookup.0.1).to_vec()).construct();
                state.map.insert(key, lookup.1.encode_len());
            }

            for item in account.storage.iter() {
                let key = StateKey::Account(*service_id, construct_storage_key(item.0).to_vec()).construct();
                state.map.insert(key, item.1.encode());
            }
        }

        /*for item in state.map.iter() {
            println!("0x{}", item.0.iter().map(|b| format!("{:02x}", b)).collect::<String>());
            println!("0x{}", item.1.iter().map(|b| format!("{:02x}", b)).collect::<String>());
            println!("\n");
        }*/
        
        return state;
    }
}

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
pub fn get_state_root() -> OpaqueHash {
    let state = STATE_ROOT.lock().unwrap();
    state.clone()
}
// Time
pub fn set_time(new_time: TimeSlot) {
    let mut state = GLOBAL_STATE.lock().unwrap();
    state.time = new_time;
}
pub fn get_time() -> TimeSlot {
    let state = GLOBAL_STATE.lock().unwrap();
    state.time.clone()
}
// Entropy
pub fn set_entropy(new_entropy: EntropyPool) {
    let mut state = GLOBAL_STATE.lock().unwrap();
    state.entropy = new_entropy;
}
pub fn get_entropy() -> EntropyPool {
    let state = GLOBAL_STATE.lock().unwrap();
    state.entropy.clone()
}
// Authorization Pools
pub fn set_auth_pools(new_authpool: AuthPools) {
    let mut state = GLOBAL_STATE.lock().unwrap();
    state.auth_pools = new_authpool;
}
pub fn get_auth_pools() -> AuthPools {
    let state = GLOBAL_STATE.lock().unwrap();
    state.auth_pools.clone()
}
// Authorizations Queues
pub fn set_auth_queues(new_authqueue: AuthQueues) {
    let mut state = GLOBAL_STATE.lock().unwrap();
    state.auth_queues = new_authqueue;
}
pub fn get_auth_queues() -> AuthQueues {
    let state = GLOBAL_STATE.lock().unwrap();
    state.auth_queues.clone()
}
// Disputes
pub fn set_disputes(new_disputes: DisputesRecords) {
    let mut state = GLOBAL_STATE.lock().unwrap();
    state.disputes = new_disputes;
}
pub fn get_disputes() -> DisputesRecords {
    let state = GLOBAL_STATE.lock().unwrap();
    state.disputes.clone()
}
// Reporting and assurance
pub fn set_reporting_assurance(new_availability: AvailabilityAssignments) {
    let mut state = GLOBAL_STATE.lock().unwrap();
    state.availability = new_availability;
}
pub fn get_reporting_assurance() -> AvailabilityAssignments {
    let state = GLOBAL_STATE.lock().unwrap();
    state.availability.clone()
}
// Statistics
pub fn set_statistics(new_statistics: Statistics) {
    let mut state = GLOBAL_STATE.lock().unwrap();
    state.statistics = new_statistics;
}
pub fn get_statistics() -> Statistics {
    let state = GLOBAL_STATE.lock().unwrap();
    state.statistics.clone()
}
// Recent History
pub fn set_recent_history(new_recent_history: BlockHistory) {
    let mut state = GLOBAL_STATE.lock().unwrap();
    state.recent_history = new_recent_history;
}
pub fn get_recent_history() -> BlockHistory {
    let state = GLOBAL_STATE.lock().unwrap();
    state.recent_history.clone()
}
// Safrole
pub fn set_safrole(new_safrole: Safrole) {
    let mut state = GLOBAL_STATE.lock().unwrap();
    state.safrole = new_safrole;
}
pub fn get_safrole() -> Safrole {
    let state = GLOBAL_STATE.lock().unwrap();
    state.safrole.clone()
}
// Service Accounts
pub fn set_service_accounts(new_service_accounts: ServiceAccounts) {
    let mut state = GLOBAL_STATE.lock().unwrap();
    state.service_accounts = new_service_accounts;
}
pub fn get_service_accounts() -> ServiceAccounts {
    let state = GLOBAL_STATE.lock().unwrap();
    state.service_accounts.clone()
}
// Accumulation History
pub fn set_accumulation_history(new_accumulation_history: AccumulatedHistory) {
    let mut state = GLOBAL_STATE.lock().unwrap();
    state.accumulation_history = new_accumulation_history;
}
pub fn get_accumulation_history() -> AccumulatedHistory {
    let state = GLOBAL_STATE.lock().unwrap();
    state.accumulation_history.clone()
}
// Ready Queue
pub fn set_ready_queue(new_ready_queue: ReadyQueue) {
    let mut state = GLOBAL_STATE.lock().unwrap();
    state.ready_queue = new_ready_queue;
}
pub fn get_ready_queue() -> ReadyQueue {
    let state = GLOBAL_STATE.lock().unwrap();
    state.ready_queue.clone()
}
// Privileges
pub fn set_privileges(new_privileges: Privileges) {
    let mut state = GLOBAL_STATE.lock().unwrap();
    state.privileges = new_privileges;
}
pub fn get_privileges() -> Privileges {
    let state = GLOBAL_STATE.lock().unwrap();
    state.privileges.clone()
}
// Validators
pub fn set_validators(new_validators: ValidatorsData, validator_set: ValidatorSet) {

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
pub fn get_validators(validator_set: ValidatorSet) -> ValidatorsData {
    
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