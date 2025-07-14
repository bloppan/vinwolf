use serde::Deserialize;
use std::path::PathBuf;

use vinwolf::jam_types::Entropy;
use vinwolf::utils::shuffle::shuffle;

// Test case struct
#[derive(Debug, Deserialize)]
struct TestCase<T> {
    input: usize,
    #[serde(with = "hex_array")]
    entropy: Entropy,
    output: Vec<T>,
}

// Entropy deserializer module
mod hex_array {
    use serde::{self, Deserialize, Deserializer};
    use vinwolf::jam_types::Entropy;
    use hex;
    use std::fmt;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Entropy, D::Error>
    where
        D: Deserializer<'de>,
    {
        let hex_str: &str = Deserialize::deserialize(deserializer)?;
        let bytes = hex::decode(hex_str).map_err(serde::de::Error::custom)?;
        if bytes.len() != 32 {
            return Err(serde::de::Error::invalid_length(bytes.len(), &HexLen));
        }
        let mut array = [0u8; 32];
        array.copy_from_slice(&bytes);
        Ok(Entropy{ entropy: array })
    }

    struct HexLen;

    impl serde::de::Expected for HexLen {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "expected 32 bytes")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn run_shuffle_test() {

        // Set file path
        let file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/test_vectors/w3f/jamtestvectors/shuffle/shuffle_tests.json");
        // Read JSON file as string
        let file_content = fs::read_to_string(file_path).expect("Could not read JSON shuffle test file");

        // Try to deserialize as u8
        match serde_json::from_str::<Vec<TestCase<u8>>>(&file_content) {
            Ok(test_cases_u8) => {
                for test_case in test_cases_u8 {
                    let sequence: Vec<u8> = (0..test_case.input)
                        .map(|x| x as u8) // Convert usize to u8
                        .collect();
                    let output_result = shuffle(&sequence, &test_case.entropy);
                    println!("expected: {:0x?}", test_case.output);
                    println!("result:   {:0x?}", output_result);
                    assert_eq!(test_case.output, output_result);
                    println!("\n");
                }
            }
            Err(_e) => {
                //println!("Could not deserialize as u8: {}", e);
            }
        }

        // Try to deserialize as u16
        match serde_json::from_str::<Vec<TestCase<u16>>>(&file_content) {
            Ok(test_cases_u16) => {
                for test_case in test_cases_u16 {
                    let sequence: Vec<u16> = (0..test_case.input)
                        .map(|x| x as u16) // Convertir usize to u16
                        .collect();
                    let output_result = shuffle(&sequence, &test_case.entropy);
                    println!("expected: {:0x?}", test_case.output);
                    println!("result:   {:0x?}", output_result);
                    assert_eq!(test_case.output, output_result);
                    println!("\n");
                }
            }
            Err(_e) => {
                //println!("Could not convert to u16: {}", e);
            }
        }
    }
}
