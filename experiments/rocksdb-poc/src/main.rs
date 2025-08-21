use rocksdb::{DB, Error};

fn main() -> Result<(), Error> {
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

