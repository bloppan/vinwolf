use jam_types::{HeaderHash, StateRoot, Seal, TimeSlot};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Clone, Debug)]
pub struct BlockMeta {
    pub parent_header: HeaderHash,
    pub state_root: StateRoot,
    pub slot: TimeSlot,
    pub seal: Seal,
}

#[derive(Clone, Debug)]
pub struct BlockTree {
    finalized: HeaderHash,
    blocks: HashMap<HeaderHash, BlockMeta>,
    children: HashMap<HeaderHash, Vec<HeaderHash>>,
    tips: HashSet<HeaderHash>,
}

impl BlockTree {

    pub fn new(finalized: HeaderHash, state_root: StateRoot, slot: TimeSlot, seal: Seal) -> Self {

        let mut blocks = HashMap::new();
        blocks.insert(finalized, BlockMeta { parent_header: HeaderHash::default(), state_root, slot, seal });
        let mut tips = HashSet::new();
        tips.insert(finalized);

        Self {finalized, blocks, children: HashMap::new(), tips}
    }
}

pub mod block_tree {

    use super::*;
    use std::sync::{LazyLock, Mutex};

    pub static BLOCK_TREE: LazyLock<Mutex<BlockTree>> = LazyLock::new(|| {
        Mutex::new(BlockTree::new(HeaderHash::default(), StateRoot::default(), TimeSlot::default(), Seal::None))
    });

    pub fn reset(genesis: HeaderHash, state_root: StateRoot, slot: TimeSlot, seal: Seal) {
        let mut ft = block_tree::BLOCK_TREE.lock().unwrap();
        *ft = BlockTree::new(genesis, state_root, slot, seal);
    }

    pub fn insert_block(
        header_hash: HeaderHash, 
        parent_header: HeaderHash, 
        state_root: StateRoot,
        slot: TimeSlot,
        seal: Seal,
    ) -> bool {

        let mut ft = BLOCK_TREE.lock().unwrap();

        if !ft.blocks.contains_key(&parent_header) {
            return false;
        }

        ft.blocks.insert(header_hash, BlockMeta { parent_header, state_root, slot, seal });
        ft.children.entry(parent_header).or_default().push(header_hash);
        ft.tips.remove(&parent_header);
        ft.tips.insert(header_hash);
        
        return true;
    }

    pub fn get_state_root_of(hash: &HeaderHash) -> Option<StateRoot> {
        let ft = BLOCK_TREE.lock().unwrap();
        ft.blocks.get(hash).map(|m| m.state_root)
    }

    pub fn get_block_tree() -> BlockTree {
        BLOCK_TREE.lock().unwrap().clone()
    }

    pub fn finalize(hash: HeaderHash) {
        let mut ft = BLOCK_TREE.lock().unwrap();
        let live = collect_descendants(&ft.children, &hash);
        ft.finalized = hash;
        ft.blocks.retain(|h, _| live.contains(h));
        ft.children.retain(|h, _| live.contains(h));
        ft.tips.retain(|h| live.contains(h));
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

    pub fn finalized() -> HeaderHash {
        let ft = BLOCK_TREE.lock().unwrap();
        ft.finalized
    }

    pub fn tips() -> HashSet<HeaderHash> {
        let ft = BLOCK_TREE.lock().unwrap();
        ft.tips.clone()
    }
}

#[cfg(test)]
mod tests {

    use super::block_tree as bt;
    use super::*;
    
    fn hh(x: u8) -> HeaderHash { [x; 32] }
    fn sr(x: u8) -> StateRoot { [x; 32] }

    #[test]
    fn insert_and_lookup_state_root_test() {
        let g = hh(0);
        let r0 = sr(9);
        bt::reset(g, r0, 0, Seal::None);

        let h1 = hh(1);
        let r1 = sr(11);
        assert!(bt::insert_block(h1, g, r1, 0, Seal::None));
        assert_eq!(bt::get_state_root_of(&h1), Some(r1));
        assert_eq!(bt::finalized(), g);
    }

    #[test]
    fn forks_update_tips_test() {
        let g = hh(0);
        let r0 = sr(0);
        bt::reset(g, r0, 0, Seal::None);

        let a = hh(1);
        let b = hh(2);
        assert!(bt::insert_block(a, g, sr(1), 0, Seal::None));
        assert!(bt::insert_block(b, g, sr(2), 0, Seal::None));

        let tips = bt::tips();
        assert_eq!(tips.len(), 2);
        assert!(tips.contains(&a));
        assert!(tips.contains(&b));
    }

    #[test]
    fn finalize_prunes_non_descendants() {
        let g = hh(0);
        bt::reset(g, sr(0), 0, Seal::None);

        let a = hh(1);
        let b = hh(2);
        let a1 = hh(3);

        assert!(bt::insert_block(a, g, sr(1), 0, Seal::None));
        assert!(bt::insert_block(b, g, sr(2), 0, Seal::None));
        assert!(bt::insert_block(a1, a, sr(3), 0, Seal::None));

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
        assert!(!bt::insert_block(hh(1), unknown_parent, sr(7), 0, Seal::None));
        assert!(bt::insert_block(hh(2), g, sr(2), 0, Seal::None));
        assert_eq!(bt::get_state_root_of(&hh(1)), None);
        assert!(bt::get_state_root_of(&hh(2)).is_some());
    }
}
