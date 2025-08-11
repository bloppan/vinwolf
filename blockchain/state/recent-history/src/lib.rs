/* 
    We retain in state information on the most recent RECENT_HISTORY_SIZE blocks. This is used to preclude the 
    possibility of duplicate or out of date work-reports from being submitted.

    For each recent block, we retain its header hash, its state root, its accumulation-result mmb and the corresponding 
    work-package hashes of each item reported (which is no more than the total number of cores, C = 341).
*/

use jam_types::{BlockInfo, Hash, Mmr, OpaqueHash, RecentBlocks, ReportedWorkPackages};
use constants::node::RECENT_HISTORY_SIZE;

pub fn process(
    recent_history_state: &mut RecentBlocks,
    header_hash: &Hash, 
    parent_state_root: &Hash, 
    reported_wp: &ReportedWorkPackages
) -> RecentBlocks {


    let history_len = recent_history_state.history.len();

    if history_len == 0 {
        recent_history_state.history.push_back(BlockInfo {
            header_hash: *header_hash,
            beefy_root: [0u8; std::mem::size_of::<Hash>()],
            state_root: [0u8; std::mem::size_of::<Hash>()],
            reported_wp: reported_wp.clone(),
        });
        
        recent_history_state.mmr = Mmr { peaks: Vec::new() };
        return recent_history_state.clone();
    }
    // During the accumulation stage, a value with the partial transition of this state is provided which contains the correction
    // for the newly-known state-root of the parent block
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

    if history_len == 1 && recent_history_state.history[0].state_root == [0u8; std::mem::size_of::<Hash>()] {
        recent_history_state.mmr = Mmr { peaks: vec![Some(*acc_outputs_result)] };
        recent_history_state.history[0].beefy_root = *acc_outputs_result;
        return;
    }

    // The final state transition for βH appends a new item including the new block's header hash, a Merkle commitment to the block's 
    // Accumulation Output Log and the set of work-reports made into it (for which we use the guarantees extrinsic, EG).
    recent_history_state.history.push_back(BlockInfo {
        header_hash: *header_hash,
        // Merkle commitment to the block's Accumulation Output Log 
        beefy_root: *acc_outputs_result,
        // The new state-trie root is the zero hash, H0, which is inaccurate but safe since β'H is not utilized except to define the next block's β†H, 
        // which contains a corrected value for this
        state_root: [0u8; std::mem::size_of::<Hash>()],

        reported_wp: work_packages.clone(),
    });

    if history_len >= RECENT_HISTORY_SIZE {
        recent_history_state.history.pop_front();
    }
}

