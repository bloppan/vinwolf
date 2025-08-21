use rocksdb::{Options, DB, IteratorMode};
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KvError {
    #[error(transparent)]
    Db(#[from] rocksdb::Error),
}

pub struct Kv {
    db: DB,
}

impl Kv {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, KvError> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        let db = DB::open(&opts, path)?;
        Ok(Self { db })
    }

    pub fn destroy(path: impl AsRef<Path>) -> Result<(), KvError> {
        let opts = Options::default();
        DB::destroy(&opts, path)?;
        Ok(())
    }

    pub fn put(&self, key: impl AsRef<[u8]>, value: impl AsRef<[u8]>) -> Result<(), KvError> {
        self.db.put(key, value)?;
        Ok(())
    }

    pub fn get(&self, key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>, KvError> {
        Ok(self.db.get(key)?)
    }

    pub fn delete(&self, key: impl AsRef<[u8]>) -> Result<(), KvError> {
        self.db.delete(key)?;
        Ok(())
    }

    pub fn flush(&self) -> Result<(), KvError> {
        self.db.flush()?;
        Ok(())
    }

    pub fn scan(&self) -> Result<(), KvError> {
        for item in self.db.iterator(IteratorMode::Start) {
            let (k, v) = item?;
            print_bytes("key", &k);
            print_bytes("value", &v);
        }
        Ok(())
    }
}

pub fn reset_db(path: &std::path::PathBuf) {
    let opts = Options::default();
    DB::destroy(&opts, path).unwrap();
    Kv::open(path).unwrap();
}

fn print_bytes(label: &str, bytes: &[u8]) {
    match std::str::from_utf8(bytes) {
        Ok(s) => println!("{label}: {s}"),
        Err(_) => println!("{label}: 0x{}", hex::encode(bytes)),
    }
}