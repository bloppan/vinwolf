use jam_types::{HeaderHash, StateRoot, Seal, TimeSlot};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Clone, Debug)]
pub struct BlockInfo {
    pub parent_header: HeaderHash,
    pub state_root: StateRoot,
    pub slot: TimeSlot,
    pub seal: Seal,
}

#[derive(Clone, Debug)]
pub struct BlockTree {
    finalized: HeaderHash,
    blocks: HashMap<HeaderHash, BlockInfo>,
    children: HashMap<HeaderHash, Vec<HeaderHash>>,
    tips: HashSet<HeaderHash>,
    by_slot: HashMap<TimeSlot, Vec<HeaderHash>>,
    duplicate_slots: HashSet<TimeSlot>,
    audited: HashSet<HeaderHash>,
}

impl BlockTree {

    pub fn new(finalized: HeaderHash, state_root: StateRoot, slot: TimeSlot, seal: Seal) -> Self {

        let mut blocks = HashMap::new();
        blocks.insert(finalized, BlockInfo { parent_header: HeaderHash::default(), state_root, slot, seal });
        let mut tips = HashSet::new();
        tips.insert(finalized);
        let mut by_slot = HashMap::new();
        by_slot.insert(slot, vec![finalized]);

        Self {finalized, blocks, children: HashMap::new(), tips, by_slot, duplicate_slots: HashSet::new(), audited: HashSet::new()}
    }
}

pub mod block_tree {

    use super::*;
    use std::sync::{LazyLock, Mutex};

    pub static BLOCK_TREE: LazyLock<Mutex<BlockTree>> = LazyLock::new(|| {
        Mutex::new(BlockTree::new(HeaderHash::default(), StateRoot::default(), TimeSlot::default(), Seal::None))
    });

    pub fn reset(genesis: HeaderHash, state_root: StateRoot, slot: TimeSlot, seal: Seal) {
        let mut tree = block_tree::BLOCK_TREE.lock().unwrap();
        *tree = BlockTree::new(genesis, state_root, slot, seal);
    }

    pub fn insert_block(
        header_hash: HeaderHash, 
        parent_header: HeaderHash, 
        state_root: StateRoot,
        slot: TimeSlot,
        seal: Seal,
    ) -> bool {

        let mut tree = BLOCK_TREE.lock().unwrap();

        if !tree.blocks.contains_key(&parent_header) { 
            return false; 
        }

        if let Some(existing) = tree.blocks.get(&header_hash) {
            if existing.state_root != state_root || existing.slot != slot || existing.seal != seal {
                return false;
            } else {
                return true;
            }
        }

        tree.blocks.insert(header_hash, BlockInfo { parent_header, state_root, slot, seal });
        tree.children.entry(parent_header).or_default().push(header_hash);
        tree.tips.remove(&parent_header);
        tree.tips.insert(header_hash);
        tree.by_slot.entry(slot).or_default().push(header_hash);

        if let Some(v) = tree.by_slot.get(&slot) { 
            if v.len() > 1 { 
                tree.duplicate_slots.insert(slot); 
            } 
        }

        maybe_finalize_locked(&mut tree);

        return true;
    }

    pub fn get_state_root_of(hash: &HeaderHash) -> Option<StateRoot> {
        let tree = BLOCK_TREE.lock().unwrap();
        tree.blocks.get(hash).map(|m| m.state_root)
    }

    pub fn get_block_tree() -> BlockTree {
        BLOCK_TREE.lock().unwrap().clone()
    }

    pub fn finalize(hash: HeaderHash) {
        let mut tree = BLOCK_TREE.lock().unwrap();
        finalize_locked(&mut tree, hash);
    }

    fn collect_descendants(children: &HashMap<HeaderHash, Vec<HeaderHash>>, hash: &HeaderHash) -> HashSet<HeaderHash> {

        let mut live = HashSet::new();
        let mut q = VecDeque::new();

        q.push_back(hash);

        while let Some(h) = q.pop_front() {

            live.insert(*h);

            if let Some(childrens) = children.get(h) {
                for c in childrens {
                    if !live.contains(c) {
                        q.push_back(c);
                    }
                }
            }
        }

        return live;
    }

    // TODO
    fn is_audited(_m: &BlockInfo) -> bool {
        true
    }

    fn is_acceptable(tree: &BlockTree, header_hash: &HeaderHash) -> bool {

        let mut cur = *header_hash;

        loop {
            
            let block_info = match tree.blocks.get(&cur) { 
                Some(block) => block, 
                None => return false 
            };
            
            if tree.duplicate_slots.contains(&block_info.slot) { 
                return false; 
            }
            
            if !is_audited(block_info) { 
                return false; 
            }
            
            if cur == tree.finalized { 
                return true; 
            }
            
            cur = block_info.parent_header;
        }
    }

    fn ticket_weight(seal: &Seal) -> u64 {
        match seal {
            Seal::None => 0,
            Seal::Keys(_) => 0,
            Seal::Tickets(_) => 1,
        }
    }
    
    fn score_to_finalized(tree: &BlockTree, tip: &HeaderHash) -> u64 {

        let mut cur = *tip;
        let mut score = 0u64;

        loop {
            let block_info = tree.blocks.get(&cur).unwrap();
            score += ticket_weight(&block_info.seal);
            if cur == tree.finalized { break; }
            cur = block_info.parent_header;
        }

        return score;
    }

    fn choose_best_head_locked(tree: &BlockTree) -> Option<HeaderHash> {

        let mut best: Option<(HeaderHash, u64, TimeSlot)> = None;

        for tip in tree.tips.iter() {

            if !is_acceptable(tree, tip) { 
                continue; 
            }

            let score = score_to_finalized(tree, tip);
            let slot = tree.blocks.get(tip).unwrap().slot;

            // TODO podrian tener 2 el mismo score?
            match best {
                None => best = Some((*tip, score, slot)),
                Some((_bh, bs, bls)) => {
                    if score > bs || (score == bs && slot > bls) {
                        best = Some((*tip, score, slot));
                    }
                }
            }
        }

        best.map(|x| x.0)
    }

    pub fn best_block_with_state() -> Option<(HeaderHash, StateRoot)> {
        let tree = BLOCK_TREE.lock().unwrap();
        let best = choose_best_head_locked(&tree)?;
        let root = tree.blocks.get(&best).unwrap().state_root;
        Some((best, root))
    }

    fn finalize_locked(tree: &mut BlockTree, hash: HeaderHash) {

        let live = collect_descendants(&tree.children, &hash);

        tree.finalized = hash;
        tree.blocks.retain(|h, _| live.contains(h));
        tree.children.retain(|h, _| live.contains(h));
        tree.tips.retain(|h| live.contains(h));
        tree.by_slot.clear();
        tree.duplicate_slots.clear();
        
        let blocks = tree.blocks.clone();

        for (hash, block_info) in blocks.iter() {
            tree.by_slot.entry(block_info.slot).or_default().push(*hash);
        }

        let by_slot = tree.by_slot.clone();

        for (slot, v) in by_slot.iter() {
            if v.len() > 1 { 
                tree.duplicate_slots.insert(*slot); 
            }
        }
    }

    fn maybe_finalize_locked(tree: &mut BlockTree) {

        if let Some(best) = choose_best_head_locked(tree) {

            if !tree.duplicate_slots.is_empty() {
                return;
            }

            let mut path_rev = Vec::new();
            let mut cur = best;

            while cur != tree.finalized {
                path_rev.push(cur);
                cur = tree.blocks.get(&cur).unwrap().parent_header;
            }

            if path_rev.len() < 3 {
                return;
            }

            let child_of_finalized = *path_rev.last().unwrap();

            finalize_locked(tree, child_of_finalized);
        }
    }

    
    pub fn finalized() -> HeaderHash {
        let tree = BLOCK_TREE.lock().unwrap();
        tree.finalized
    }

    pub fn tips() -> HashSet<HeaderHash> {
        let tree = BLOCK_TREE.lock().unwrap();
        tree.tips.clone()
    }
}

#[cfg(test)]
mod tests {

    use super::block_tree as bt;
    use super::*;
    
    fn hh(x: u8) -> HeaderHash { [x; 32] }
    fn sr(x: u8) -> StateRoot { [x; 32] }

    #[test]
    fn run_all_grandpa_tests() {
        no_auto_finalize_before_three_descendants();
        auto_finalize_child_after_three_descendants();
        auto_finalize_moves_step_by_step_with_enough_descendants();
        forks_with_same_slot_update_tips_but_do_not_finalize();
        finalize_prunes_non_descendants();
        reject_unknown_parent();
    }

    #[test]
    fn no_auto_finalize_before_three_descendants() {
        let g = hh(0);
        bt::reset(g, sr(0), 0, Seal::None);

        let a = hh(1);
        let b = hh(2);

        assert!(bt::insert_block(a, g, sr(1), 1, Seal::None));
        assert_eq!(bt::finalized(), g);

        assert!(bt::insert_block(b, a, sr(2), 2, Seal::None));
        assert_eq!(bt::finalized(), g);
    }

    #[test]
    fn auto_finalize_child_after_three_descendants() {
        let g = hh(0);
        bt::reset(g, sr(0), 0, Seal::None);

        let a = hh(1);
        let b = hh(2);
        let c = hh(3);

        assert!(bt::insert_block(a, g, sr(1), 1, Seal::None));
        assert!(bt::insert_block(b, a, sr(2), 2, Seal::None));
        assert_eq!(bt::finalized(), g);

        assert!(bt::insert_block(c, b, sr(3), 3, Seal::None));

        assert_eq!(bt::finalized(), a);
        assert!(bt::get_state_root_of(&a).is_some());
        assert!(bt::get_state_root_of(&b).is_some());
        assert!(bt::get_state_root_of(&c).is_some());
        assert!(bt::get_state_root_of(&g).is_none());

        let tips = bt::tips();
        assert_eq!(tips.len(), 1);
        assert!(tips.contains(&c));
    }

    #[test]
    fn auto_finalize_moves_step_by_step_with_enough_descendants() {
        let g = hh(0);
        bt::reset(g, sr(0), 0, Seal::None);

        let a = hh(1);
        let b = hh(2);
        let c = hh(3);
        let d = hh(4);

        assert!(bt::insert_block(a, g, sr(1), 1, Seal::None));
        assert!(bt::insert_block(b, a, sr(2), 2, Seal::None));
        assert!(bt::insert_block(c, b, sr(3), 3, Seal::None));

        assert_eq!(bt::finalized(), a);

        assert!(bt::insert_block(d, c, sr(4), 4, Seal::None));

        assert_eq!(bt::finalized(), b);
    }

    #[test]
    fn forks_with_same_slot_update_tips_but_do_not_finalize() {
        let g = hh(0);
        bt::reset(g, sr(0), 0, Seal::None);

        let a = hh(1);
        let b = hh(2);

        assert!(bt::insert_block(a, g, sr(1), 1, Seal::None));
        assert!(bt::insert_block(b, g, sr(2), 1, Seal::None));

        let tips = bt::tips();
        assert_eq!(tips.len(), 2);
        assert!(tips.contains(&a));
        assert!(tips.contains(&b));

        assert_eq!(bt::finalized(), g);

        let tree = bt::get_block_tree();
        assert!(tree.duplicate_slots.contains(&1));
    }

    #[test]
    fn finalize_prunes_non_descendants() {
        let g = hh(0);
        bt::reset(g, sr(0), 0, Seal::None);

        let a = hh(1);
        let b = hh(2);
        let a1 = hh(3);

        assert!(bt::insert_block(a, g, sr(1), 1, Seal::None));
        assert!(bt::insert_block(b, g, sr(2), 1, Seal::None));
        assert!(bt::insert_block(a1, a, sr(3), 2, Seal::None));

        bt::finalize(a);

        assert!(bt::get_state_root_of(&a).is_some());
        assert!(bt::get_state_root_of(&a1).is_some());
        assert!(bt::get_state_root_of(&g).is_none());
        assert!(bt::get_state_root_of(&b).is_none());

        let tips = bt::tips();
        assert_eq!(tips.len(), 1);
        assert!(tips.contains(&a1));
        assert_eq!(bt::finalized(), a);
    }

    #[test]
    fn reject_unknown_parent() {
        let g = hh(0);
        bt::reset(g, sr(0), 0, Seal::None);

        let unknown_parent = hh(42);
        assert!(!bt::insert_block(hh(1), unknown_parent, sr(7), 1, Seal::None));
        assert!(bt::insert_block(hh(2), g, sr(2), 1, Seal::None));
        assert_eq!(bt::get_state_root_of(&hh(1)), None);
        assert!(bt::get_state_root_of(&hh(2)).is_some());
    }
}
