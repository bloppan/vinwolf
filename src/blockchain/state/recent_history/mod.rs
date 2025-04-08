/* 
    We retain in state information on the most recent RECENT_HISTORY_SIZE blocks. This is used to preclude the 
    possibility of duplicate or out of date work-reports from being submitted.

    For each recent block, we retain its header hash, its state root, its accumulation-result mmr and the corresponding 
    work-package hashes of each item reported (which is no more than the total number of cores, C = 341).
*/

use sp_core::keccak_256;

use crate::types::{Hash, BlockHistory, BlockInfo, ReportedWorkPackages, Mmr};
use crate::constants::RECENT_HISTORY_SIZE;
use crate::utils::trie::append;

pub fn process_recent_history(
    recent_history_state: &mut BlockHistory,
    header_hash: &Hash, 
    parent_state_root: &Hash, 
    work_packages: &ReportedWorkPackages
) -> BlockHistory {

    let history_len = recent_history_state.blocks.len();

    if history_len == 0 {
        add_new_block(
            recent_history_state,
            header_hash,
            &Mmr { peaks: Vec::new() },
            &[0u8; std::mem::size_of::<Hash>()],
            &[0u8; std::mem::size_of::<Hash>()],
            work_packages,
        );
        
        return recent_history_state.clone();
    }
    
    recent_history_state.blocks[history_len - 1].state_root = *parent_state_root;

    return recent_history_state.clone();
}

pub fn finalize_recent_history(recent_history_state: &mut BlockHistory,
                               header_hash: &Hash, 
                               accumulate_root: &Hash, 
                               work_packages: &ReportedWorkPackages
) {

    let history_len = recent_history_state.blocks.len();

    if history_len == 1 && recent_history_state.blocks[0].state_root == [0u8; std::mem::size_of::<Hash>()] {
        recent_history_state.blocks[0].mmr = append(&Mmr { peaks: Vec::new() }, *accumulate_root, keccak_256);
        return;
    }

    let last_mmr = Mmr {
        peaks: recent_history_state.blocks[history_len - 1].mmr.peaks.clone(),
    };
    
    add_new_block(
        recent_history_state,
        header_hash,
        &last_mmr,
        accumulate_root,
        &[0u8; std::mem::size_of::<Hash>()],
        work_packages,
    );

    if history_len >= RECENT_HISTORY_SIZE {
        recent_history_state.blocks.pop_front();
    }
}

fn add_new_block(
    recent_history_state: &mut BlockHistory,
    header_hash: &Hash,
    mmr: &Mmr,
    accumulate_root: &Hash,
    state_root: &Hash,
    work_packages: &ReportedWorkPackages,
) {
    // We define an item n comprising the new block's header hash, its accumulation-result Merkle tree root and the set
    // of work-reports made into it (for which we use the guarantees extrinsic).
    recent_history_state.blocks.push_back(BlockInfo {
        header_hash: *header_hash,
        // Note that the accumulation-result tree root r is derived from C (section 12) using the basic binary Merklization 
        // function MB (defined in apendix E) and appending it using the mmr append function to form a Merkle mountain range.
        mmr: append(mmr, *accumulate_root, keccak_256),
        // The state-trie root is as being the zero hash, which while inaccurate at the end state of the block β', it is
        // nevertheless safe since β' is not utilized except to define the next block’s β†, which contains a corrected value for this
        state_root: *state_root,
        reported: work_packages.clone(),
    });
}

