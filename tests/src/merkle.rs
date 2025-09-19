#[cfg(test)]
mod tests {

    use std::fs::File;
    use std::io::Read;
    use std::path::PathBuf;
    use std::collections::HashMap;
    use utils::{trie::merkle, hex, serde::{Deserialize, Value, from_json_str}};
    use jam_types::{OpaqueHash};

    #[derive(Debug, PartialEq)]
    struct TestCase {
        input: HashMap<Vec<u8>, Vec<u8>>,
        output: OpaqueHash,
    }

    fn hex_to_hash(s: &str) -> Result<OpaqueHash, String> {
        if s.len() != 64 {
            return Err("must be 64 hex characters".into());
        }
        let mut bytes = OpaqueHash::default();
        for i in 0..32 {
            let byte_str = &s[i * 2..i * 2 + 2];
            bytes[i] = u8::from_str_radix(byte_str, 16).map_err(|_| "invalid hex character".to_string())?;
        }
        Ok(bytes)
    }

    fn hex_to_vec(s: &str) -> Result<Vec<u8>, String> {
        if s.len() % 2 != 0 {
            return Err("hex string must have even length".into());
        }
        let mut bytes = Vec::with_capacity(s.len() / 2);
        for i in 0..s.len() / 2 {
            let byte_str = &s[i * 2..i * 2 + 2];
            bytes.push(u8::from_str_radix(byte_str, 16).map_err(|_| "invalid hex character".to_string())?);
        }
        Ok(bytes)
    }

    impl Deserialize for TestCase {
        fn from_value(v: &Value) -> Result<Self, String> {
            let o = match v {
                Value::Object(o) => o,
                _ => return Err("expected object".into()),
            };
            let input_obj = match o.get("input").ok_or("missing field input")? {
                Value::Object(obj) => obj,
                _ => return Err("expected object for input".into()),
            };
            let mut input = HashMap::new();
            for (key_hex, val_value) in input_obj {
                let key = hex_to_vec(key_hex)?;
                let val_hex = String::from_value(val_value)?;
                let val = hex_to_vec(&val_hex)?;
                input.insert(key, val);
            }
            let output_hex = String::from_value(o.get("output").ok_or("missing field output")?)?;
            let output = hex_to_hash(&output_hex)?;
            Ok(TestCase { input, output })
        }
    }

    #[test]
    fn run_trie_test() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("jamtestvectors/trie/trie.json");
        let mut file = File::open(&path).expect("Couldn't open trie json test file");
        // Read test json content
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("Couldn't read the file to string");
        // Deserialize JSON
        let tests: Vec<TestCase> = from_json_str(&contents).unwrap();  
        for (index, entry) in tests.iter().enumerate() {
            let kvs: Vec<(Vec<u8>, Vec<u8>)> = entry.input.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
            let merkle_root = merkle(&kvs, 0);  
            let res_hex = hex::encode(merkle_root);
            let test_hex = hex::encode(entry.output);
            println!("Test case {}: Expected output = {}, Merkle result = {}", index + 1, test_hex, res_hex);
            assert_eq!(test_hex, res_hex, "Test case {}: Merkle root mismatch!", index + 1);
        }
    }
}


