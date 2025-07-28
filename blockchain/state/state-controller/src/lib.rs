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

use sp_core::blake2_256;
use jam_types::{Block, ProcessError};
use utils::trie::merkle_state;
use block::header;
use codec::Encode;

// We specify the state transition function as the implication of formulating all items of posterior state in terms of the prior
// state and block. To aid the architecting of implementations which parallelize this computation, we minimize the depth of the
// dependency graph where possible. 
pub fn state_transition_function(block: &Block) -> Result<(), ProcessError> {
    
    let header_hash = blake2_256(&block.header.encode());
    log::debug!("Importing new block: 0x{}", utils::print_hash!(header_hash));
    
    header::verify(&block)?;

    let mut new_state = state_handler::get_global_state().lock().unwrap().clone();
    
    state_handler::time::set_current(&block.header.unsigned.slot);

    let mut reported_work_packages = Vec::new();
    for report in &block.extrinsic.guarantees {
        reported_work_packages.push((report.report.package_spec.hash, report.report.package_spec.exports_root));
    }
    reported_work_packages.sort_by_key(|(hash, _)| *hash);

    let curr_block_history = recent_history::process(
        &mut new_state.recent_history,
        &header_hash, 
        &block.header.unsigned.parent_state_root,
        &reported_work_packages);
        
    state_handler::recent_history::set_current(curr_block_history);

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

    let new_available_workreports = reports::assurances::process(
        &mut new_state.availability,
        &block.extrinsic.assurances,
        &block.header.unsigned.slot,
        &block.header.unsigned.parent,
    )?;

    let _ = reports::guarantees::process(
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
    
    state_handler::set_state_root(merkle_state(&utils::serialization::serialize(&new_state).map, 0));
    state_handler::set_global_state(new_state);

    log::debug!("Block 0x{} processed succesfully", utils::print_hash!(header_hash));
    
    Ok(())
}
