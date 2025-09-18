use constants::node::MAX_AGE_LOOKUP_ANCHOR;
use super::Storage;
use jam_types::{TimeSlot, OpaqueHash};
use std::sync::{Mutex, LazyLock};
#[cfg(not(feature = "DB"))]
use std::collections::HashMap;
#[cfg(feature = "DB")]
use rocksdb::{DB, ColumnFamilyDescriptor, Options, IteratorMode};

#[cfg(feature = "DB")]
const ANCESTORS_CF: &str = "ancestors";

static ANCESTORS: LazyLock<Mutex<AncestorsInfo>> = LazyLock::new(|| Mutex::new(AncestorsInfo::default()));
static ANCESTORS_ENABLED: LazyLock<Mutex<bool>> = LazyLock::new(|| Mutex::new(false));

#[cfg(feature = "DB")]
static BUILD: &str = "DB";
#[cfg(not(feature = "DB"))]
static BUILD: &str = "vector";

#[cfg(not(feature = "DB"))]
pub type Ancestors = HashMap<TimeSlot, OpaqueHash>;

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

#[cfg(feature = "DB")]
#[derive(Debug)] 
pub struct RocksDBWrapper {
    pub db: DB,
    pub cf_name: &'static str,
    pub entries_count: usize,
}

#[cfg(feature = "DB")]
impl RocksDBWrapper {
    pub fn new(path: &str) -> Self {
        // Wipe existing DB directory to start fresh
        /*if std::fs::metadata(path).is_ok() {
            std::fs::remove_dir_all(path).expect("Failed to wipe existing DB directory");
        }*/
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);

        let cf_desc = ColumnFamilyDescriptor::new(ANCESTORS_CF, Options::default());
        let db = DB::open_cf_descriptors(&opts, path, vec![cf_desc])
            .expect("Failed to open DB");
        let cf = db.cf_handle(ANCESTORS_CF).expect("CF not found");
        let entries_count = db.iterator_cf(&cf, IteratorMode::Start).count();
        
        RocksDBWrapper {
            db,
            cf_name: ANCESTORS_CF,
            entries_count,
        }
    }

    pub fn cf_handle(&self) -> &rocksdb::ColumnFamily {
        self.db.cf_handle(self.cf_name).expect("CF not found")
    }
}

#[cfg(feature = "DB")]
impl Storage for RocksDBWrapper {
    fn get(&self, slot: &TimeSlot) -> Option<OpaqueHash> {
        let cf = self.cf_handle();
        self.db
            .get_cf(cf, slot.to_be_bytes())
            .expect("DB get failed")
            .and_then(|bytes| bytes.try_into().ok())
    }

    fn insert(&mut self, slot: TimeSlot, hash: OpaqueHash) {
        let cf = self.cf_handle();
        self.db
            .put_cf(cf, slot.to_be_bytes(), hash)
            .expect("DB put failed");
        self.entries_count += 1;
    }

    fn remove(&mut self, slot: &TimeSlot) {
        let cf = self.cf_handle();
        self.db
            .delete_cf(cf, slot.to_be_bytes())
            .expect("DB delete failed");
        self.entries_count -= 1;
    }

    fn len(&self) -> usize {
        self.entries_count
    }
}

#[derive(Debug)]
pub struct AncestorsInfo {
    #[cfg(not(feature = "DB"))]
    pub map: Ancestors,
    #[cfg(feature = "DB")]
    pub map: RocksDBWrapper,
    pub min_timeslot: TimeSlot,
    pub max_timeslot: TimeSlot,
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

pub fn is_ancestors_feature_enabled() -> bool {
    *ANCESTORS_ENABLED.lock().unwrap()
}

pub fn set_ancestors_feature(status: bool) {
    *ANCESTORS_ENABLED.lock().unwrap() = status;
}

pub fn get() -> &'static Mutex<AncestorsInfo> {
    &ANCESTORS
}

pub fn set(ancestors: AncestorsInfo) {
    *ANCESTORS.lock().unwrap() = ancestors;
}

pub fn lookup(slot: &TimeSlot, header_hash: &OpaqueHash) -> bool {

    let ancestors = get().lock().unwrap();

    if *slot < ancestors.min_timeslot || *slot > ancestors.max_timeslot {
        return false;
    }

    #[cfg(feature = "DB")]
    {
        ancestors.map.get(slot).map_or(false, |hash| hash == *header_hash)
    }
    #[cfg(not(feature = "DB"))]
    {
        ancestors.map.get(slot).map_or(false, |hash| *hash == *header_hash)
    }
}

pub fn update(slot: &TimeSlot, header_hash: &OpaqueHash) {

    utils::log::debug!("Update ancestors. Build type: {:?}. slot: {:?} hash: {}", BUILD, *slot, utils::hex::encode(header_hash));
    let mut ancestors = get().lock().unwrap();
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

        update(&slot, &hash);

        assert!(lookup(&slot, &hash));
        assert!(!lookup(&slot, &[2u8; 32]));
    }
}