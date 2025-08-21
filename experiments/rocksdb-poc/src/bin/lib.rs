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

fn main() -> Result<(), rocksdb::Error> {
    let path = "_kvdb";
    let db = DB::open_default(path)?;
    db.put(b"foo", b"bar")?;
    let key: Vec<u8> = vec![1,3,4];
    let value: String = "Valor".to_string();
    db.put(key, value)?;
    let value = db.get(b"foo")?;
    if let Some(v) = value {
        println!("{:?}", String::from_utf8_lossy(&v));
    }
    Ok(())
}


#[cfg(test)]
mod tests {
    use std::time::UNIX_EPOCH;

    use super::*;
    use tempfile::TempDir;

    #[test]
    fn create_db() {
        let path = std::env::current_dir().unwrap().join("mi_db");
        println!("Created db at dir: {:?}", path);       
    }

    #[test]
    fn flush_db() {
        let path = std::env::current_dir().unwrap().join("mi_db");
        let db: Kv = Kv::open(&path).unwrap();
        db.flush().unwrap();
    }

    #[test]
    fn insert_value() {
        let path = std::env::current_dir().unwrap().join("mi_db");
        let db: Kv = Kv::open(&path).unwrap();
        let key = "clave2".to_string();
        let valor = std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs().to_le_bytes();
        db.put(&key,&valor).unwrap();
        println!("Key {:?} Value inserted: {:?}", key, valor);
    }

    #[test]
    fn read_key_value() {
        let path = std::env::current_dir().unwrap().join("mi_db");
        let db: Kv = Kv::open(&path).unwrap();
        let key = "clave".to_string();
        let value = db.get(&key).unwrap();
        println!("get value: {:?}", value);
    }

    #[test]
    fn scan_values() {
        let path = std::env::current_dir().unwrap().join("mi_db");
        let db: Kv = Kv::open(&path).unwrap();
        let _ = db.flush();
        let _ = db.scan();
    }

    #[test]
    fn delete_value() {
        let path = std::env::current_dir().unwrap().join("mi_db");
        let db: Kv = Kv::open(&path).unwrap();
        let key = "clave".to_string();
        let value = db.get(&key).unwrap();
        println!("Trying to delete the key {:?} value {:?}", key, value);
        db.delete(&key).unwrap();
        let value = db.get(&key).unwrap();
        println!("Now the value is: {:?}", value);
    }

    #[test]
    fn reset_db_test() {
        let path = std::env::current_dir().unwrap().join("mi_db");
        reset_db(&path);
    }

    #[test]
    fn destroy_db() {
        let path = std::env::current_dir().unwrap().join("mi_db");
        Kv::destroy(&path).unwrap();
    }

    #[test]
    fn basic_put_get_delete() {
        let dir = TempDir::new().unwrap();
        let db = Kv::open(dir.path()).unwrap();
        db.put(b"k", b"v").unwrap();
        let got = db.get(b"k").unwrap();
        assert_eq!(got.as_deref(), Some(&b"v"[..]));
        db.delete(b"k").unwrap();
        let got = db.get(b"k").unwrap();
        assert!(got.is_none());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_roundtrip() {
        let dir = TempDir::new().unwrap();
        let db = Kv::open(dir.path()).unwrap();
        #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
        struct Item { n: u32 }
        db.put_value(b"it", &Item { n: 7 }).unwrap();
        let got: Option<Item> = db.get_value(b"it").unwrap();
        assert_eq!(got, Some(Item { n: 7 }));
    }


}
