pub mod ancestors;
use jam_types::{OpaqueHash, TimeSlot};

pub trait Storage {
    fn get(&self, slot: &TimeSlot) -> Option<OpaqueHash>;
    fn insert(&mut self, slot: TimeSlot, hash: OpaqueHash);
    fn remove(&mut self, slot: &TimeSlot);
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}


