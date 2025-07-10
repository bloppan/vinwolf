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

use crate::blockchain::state::recent_history::set_current_block_history;
use crate::utils::trie::merkle_state;
use crate::types::{
    AccumulatedHistory, AuthPools, AuthQueues, AvailabilityAssignments, Block, BlockHistory, DisputesRecords, EntropyPool, GlobalState, 
    OpaqueHash, Privileges, ProcessError, ReadyQueue, Safrole, ServiceAccounts, Statistics, TimeSlot, ValidatorSet, ValidatorsData, 
};

use crate::utils::codec::Encode;

pub mod accumulation; pub mod authorization; pub mod disputes; pub mod entropy; pub mod safrole; pub mod recent_history; pub mod reports;
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
    
    let header_hash = blake2_256(&block.header.encode());
    log::info!("Importing new block: 0x{}", hex::encode(header_hash));
    
    block.header.verify(&block.extrinsic)?;

    let mut new_state = get_global_state().lock().unwrap().clone();
    
    time::set_current_slot(&block.header.unsigned.slot);

    let mut reported_work_packages = Vec::new();
    for report in &block.extrinsic.guarantees.report_guarantee {
        reported_work_packages.push((report.report.package_spec.hash, report.report.package_spec.exports_root));
    }
    reported_work_packages.sort_by_key(|(hash, _)| *hash);

    let curr_block_history = recent_history::process(
        &mut new_state.recent_history,
        &header_hash, 
        &block.header.unsigned.parent_state_root,
        &reported_work_packages);
        
    set_current_block_history(curr_block_history);

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
        &block,
        &new_state.disputes.offenders)?;

    let new_available_workreports = reports::assurance::process(
        &mut new_state.availability,
        &block.extrinsic.assurances,
        &block.header.unsigned.slot,
        &block.header.unsigned.parent,
    )?;

    let _ = reports::guarantee::process(
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
        &header_hash,
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
    
    set_state_root(merkle_state(&new_state.serialize().map, 0).unwrap());
    set_global_state(new_state);

    Ok(())
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
pub fn get_state_root() -> &'static Mutex<OpaqueHash> {
    &STATE_ROOT
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