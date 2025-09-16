use std::sync::{LazyLock, Mutex};
use std::collections::HashMap;

use constants::node::MAX_AGE_LOOKUP_ANCHOR;
use jam_types::{TimeSlot, OpaqueHash};

#[cfg(feature = "DB")]
use rocksdb::{DB, ColumnFamilyDescriptor, Options};
#[cfg(feature = "DB")]
const ANCESTORS_CF: &str = "ancestors";

// Storage trait to abstract HashMap and RocksDB
trait Storage {
    fn get(&self, slot: &TimeSlot) -> Option<OpaqueHash>;
    fn insert(&mut self, slot: TimeSlot, hash: OpaqueHash);
    fn remove(&mut self, slot: &TimeSlot);
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(not(feature = "DB"))]
type Ancestors = HashMap<TimeSlot, OpaqueHash>;

#[cfg(not(feature = "DB"))]
impl Storage for HashMap<TimeSlot, OpaqueHash> {
    fn get(&self, slot: &TimeSlot) -> Option<OpaqueHash> {
        self.get(slot).copied()
    }

    fn insert(&mut self, slot: TimeSlot, hash: OpaqueHash) {
        self.insert(slot, hash);
    }

    fn remove(&mut self, slot: &TimeSlot) {
        self.remove(slot);
    }

    fn len(&self) -> usize {
        HashMap::len(self)
    }
}

// RocksDB wrapper
#[cfg(feature = "DB")]
struct RocksDBWrapper {
    db: DB,
    cf_handle: rocksdb::BoundColumnFamily<'static>,
}

#[cfg(feature = "DB")]
impl RocksDBWrapper {
    fn new(path: &str) -> Self {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);

        let cf_desc = ColumnFamilyDescriptor::new(ANCESTORS_CF, Options::default());
        let db = DB::open_cf_descriptors(&opts, path, vec![cf_desc]).expect("Failed to open DB");

        // Safety: We ensure the column family exists and lives as long as the DB
        let cf_handle = unsafe {
            std::mem::transmute(db.cf_handle(ANCESTORS_CF).expect("CF not found"))
        };

        RocksDBWrapper { db, cf_handle }
    }
}

#[cfg(feature = "DB")]
impl Storage for RocksDBWrapper {
    fn get(&self, slot: &TimeSlot) -> Option<OpaqueHash> {
        self.db
            .get_cf(&self.cf_handle, slot.to_be_bytes())
            .expect("DB get failed")
            .and_then(|bytes| bytes.try_into().ok())
    }

    fn insert(&mut self, slot: TimeSlot, hash: OpaqueHash) {
        self.db
            .put_cf(&self.cf_handle, slot.to_be_bytes(), hash)
            .expect("DB put failed");
    }

    fn remove(&mut self, slot: &TimeSlot) {
        self.db
            .delete_cf(&self.cf_handle, slot.to_be_bytes())
            .expect("DB delete failed");
    }

    fn len(&self) -> usize {
        // Approximate count by iterating (not ideal, but RocksDB doesn't provide direct count)
        self.db
            .iterator_cf(&self.cf_handle, rocksdb::IteratorMode::Start)
            .count()
    }
}

#[derive(Debug)]
pub struct AncestorsInfo {
    #[cfg(not(feature = "DB"))]
    pub map: Ancestors,
    #[cfg(feature = "DB")]
    pub map: RocksDBWrapper,
    pub min_timeslot: u32,
    pub max_timeslot: u32,
}

impl AncestorsInfo {
    #[cfg(not(feature = "DB"))]
    pub fn default() -> Self {
        AncestorsInfo {
            map: HashMap::with_capacity(MAX_AGE_LOOKUP_ANCHOR as usize),
            min_timeslot: 0,
            max_timeslot: 0,
        }
    }

    #[cfg(feature = "DB")]
    pub fn default() -> Self {
        AncestorsInfo {
            map: RocksDBWrapper::new("./ancestors_db"),
            min_timeslot: 0,
            max_timeslot: 0,
        }
    }
}

static ANCESTORS: LazyLock<Mutex<AncestorsInfo>> = LazyLock::new(|| Mutex::new(AncestorsInfo::default()));
static ANCESTORS_ENABLED: LazyLock<Mutex<bool>> = LazyLock::new(|| Mutex::new(false));

pub fn is_ancestors_feature_enabled() -> bool {
    *ANCESTORS_ENABLED.lock().unwrap()
}

pub fn set_ancestors_feature(status: bool) {
    *ANCESTORS_ENABLED.lock().unwrap() = status;
}

pub fn get_ancestors() -> &'static Mutex<AncestorsInfo> {
    &ANCESTORS
}

pub fn set_ancestors(ancestors: AncestorsInfo) {
    *ANCESTORS.lock().unwrap() = ancestors;
}

pub fn lookup_ancestor(slot: &TimeSlot, header_hash: &OpaqueHash) -> bool {
    let ancestors = get_ancestors().lock().unwrap();

    if ancestors.map.is_empty() {
        return true;
    }

    if *slot < ancestors.min_timeslot || *slot > ancestors.max_timeslot {
        return false;
    }

    ancestors.map.get(slot).map_or(false, |hash| *hash == *header_hash)
}

pub fn update_ancestors(slot: &TimeSlot, header_hash: &OpaqueHash) {
    let mut ancestors = get_ancestors().lock().unwrap();
    ancestors.map.insert(*slot, *header_hash);
    ancestors.max_timeslot = ancestors.max_timeslot.max(*slot);

    if ancestors.min_timeslot == 0 {
        ancestors.min_timeslot = *slot;
    }

    let mut to_remove: Vec<TimeSlot> = vec![];
    let mut ancestors_len = ancestors.map.len();

    while ancestors_len > MAX_AGE_LOOKUP_ANCHOR as usize {
        to_remove.push(ancestors.min_timeslot);
        ancestors.min_timeslot += 1;
        ancestors_len -= 1;
    }

    for item in to_remove.iter() {
        ancestors.map.remove(item);
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ancestors() {
        set_ancestors_feature(true);
        assert!(is_ancestors_feature_enabled());

        let slot = 1000;
        let hash = [1u8; 32];
        update_ancestors(&slot, &hash);

        assert!(lookup_ancestor(&slot, &hash));
        assert!(!lookup_ancestor(&slot, &[2u8; 32]));
        assert!(!lookup_ancestor(&(slot + MAX_AGE_LOOKUP_ANCHOR + 1), &hash));
    }
}
