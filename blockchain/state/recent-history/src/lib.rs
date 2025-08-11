/* 
    We retain in state information on the most recent RECENT_HISTORY_SIZE blocks. This is used to preclude the 
    possibility of duplicate or out of date work-reports from being submitted.

    For each recent block, we retain its header hash, its state root, its accumulation-result mmr and the corresponding 
    work-package hashes of each item reported (which is no more than the total number of cores, C = 341).
*/

use sp_core::keccak_256;
use jam_types::{BlockInfo, Hash, Mmr, OpaqueHash, RecentBlocks, ReportedWorkPackages, RecentAccOutputs};
use constants::node::RECENT_HISTORY_SIZE;
use utils::trie::append;
use codec::Encode;

pub fn process(
    recent_history_state: &mut RecentBlocks,
    header_hash: &Hash, 
    parent_state_root: &Hash, 
    reported_wp: &ReportedWorkPackages
) -> RecentBlocks {

    let history_len = recent_history_state.history.len();

    if history_len == 0 {
        add_new_block(
            recent_history_state,
            header_hash,
            &[0u8; std::mem::size_of::<Hash>()],
            &[0u8; std::mem::size_of::<Hash>()],
            reported_wp,
        );
        
        recent_history_state.mmr = Mmr { peaks: Vec::new() };
        return recent_history_state.clone();
    }
    
    recent_history_state.history[history_len - 1].state_root = *parent_state_root;

    return recent_history_state.clone();
}

pub fn finalize(
        recent_history_state: &mut RecentBlocks,
        header_hash: &Hash,
        acc_outputs_result: &OpaqueHash, 
        work_packages: &ReportedWorkPackages
) {

    let history_len = recent_history_state.history.len();

    recent_history_state.history[history_len - 1].state_root = [0u8; std::mem::size_of::<Hash>()];
    recent_history_state.history[history_len - 1].header_hash = *header_hash;
    recent_history_state.history[history_len - 1].acc_result = *acc_outputs_result;
    recent_history_state.history[history_len - 1].reported_wp = work_packages.clone();


    if history_len >= RECENT_HISTORY_SIZE {
        recent_history_state.history.pop_front();
    }
}

fn add_new_block(
    recent_history_state: &mut RecentBlocks,
    header_hash: &Hash,
    acc_result: &Hash,
    state_root: &Hash,
    work_packages: &ReportedWorkPackages,
) {
    // We define an item n comprising the new block's header hash, its accumulation-result Merkle tree root and the set
    // of work-reports made into it (for which we use the guarantees extrinsic).
    recent_history_state.history.push_back(BlockInfo {
        header_hash: *header_hash,
        // Note that the accumulation-result tree root r is derived from C (section 12) using the basic binary Merklization 
        // function MB (defined in apendix E) and appending it using the mmr append function to form a Merkle mountain range.
        acc_result: *acc_result,
        // The state-trie root is as being the zero hash, which while inaccurate at the end state of the block β', it is
        // nevertheless safe since β' is not utilized except to define the next block’s β†, which contains a corrected value for this
        state_root: *state_root,
        reported_wp: work_packages.clone(),
    });
}

