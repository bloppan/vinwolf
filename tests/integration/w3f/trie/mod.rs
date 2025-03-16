use serde::Deserialize;
use std::collections::HashMap;
use std::convert::TryInto;
use serde::de::{self, Deserializer};

extern crate vinwolf;
use vinwolf::utils::trie::merkle;

#[derive(Debug, Deserialize)]
struct Entry {
    #[serde(deserialize_with = "deserialize_input")]
    input: HashMap<Vec<u8>, Vec<u8>>, 

    #[serde(deserialize_with = "deserialize_output")]
    output: [u8; 32], 
}

#[derive(Debug, Deserialize)]
struct JsonData(Vec<Entry>);

fn deserialize_input<'de, D>(deserializer: D) -> Result<HashMap<Vec<u8>, Vec<u8>>, D::Error>
where
    D: Deserializer<'de>,
{
    let map: HashMap<String, String> = HashMap::deserialize(deserializer)?;
    let mut result = HashMap::new();

    for (key_hex, value_hex) in map {
        let key_bytes = hex::decode(key_hex).map_err(de::Error::custom)?;
        let value_bytes = if value_hex.is_empty() {
            Vec::new()  // If the value is empty then represent it as empty Vec<u8>
        } else {
            hex::decode(value_hex).map_err(de::Error::custom)?  // Convert hex value to Vec<u8>
        };
        result.insert(key_bytes, value_bytes);
    }

    Ok(result)
}

fn deserialize_output<'de, D>(deserializer: D) -> Result<[u8; 32], D::Error>
where
    D: Deserializer<'de>,
{
    let hex_str: String = String::deserialize(deserializer)?;
    let output_bytes = hex::decode(&hex_str)
        .map_err(de::Error::custom)?
        .try_into()
        .map_err(|_| de::Error::custom("Output must be 32 bytes long"))?;
    Ok(output_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use std::fs::File;
    use std::io::Read;
    use std::path::PathBuf;

    #[test]
    fn run_trie_test() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests/test_vectors/w3f/jamtestvectors/trie/trie.json");
        let mut file = File::open(&path).expect("Couldn't open trie json test file");
        // Read test json content
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("Couldn't read the file to string");
        // Deserialize JSON
        let data: JsonData = serde_json::from_str(&contents).expect("Couldn't deserialize JSON");
        // Iterate over each JSON input
        for (index, entry) in data.0.iter().enumerate() {
            let mut kvs = Vec::new();
            // Fill kvs
            for (key, value) in &entry.input {
                kvs.push((key.clone(), value.clone()));
            }
            // Calculate merkle result
            let res = merkle(&kvs, 0);
            // Convert Merkle result and spected output to hex
            match res {
                Ok(merkle_root) => {
                    let res_hex = hex::encode(merkle_root);
                    let test_hex = hex::encode(entry.output);
                    println!("Test case {}: Expected output = {}, Merkle result = {}", index + 1, res_hex, test_hex);
                    assert_eq!(test_hex, res_hex, "Test case {}: Merkle root mismatch!", index + 1);
                } Err(e) => {println!("Test case {}: Failed to calculate Merkle root. Error: {:?}", index + 1, e);
                    panic!("Test case {} failed due to Merkle calculation error", index + 1);
                }
            }
        }
    }
}


