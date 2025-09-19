#[cfg(test)]
mod tests {

    use std::path::PathBuf;
    use jam_types::{OpaqueHash, Entropy};
    use utils::{serde::{Deserialize, Value, from_json_str}, shuffle::shuffle};

    #[derive(Debug, PartialEq)]
    struct TestCase {
        input: u64,
        entropy: Entropy,
        output: Vec<u64>,
    }

    fn hex_to_bytes(s: &str) -> Result<OpaqueHash, String> {
        if s.len() != 64 {
            return Err("entropy must be 64 hex characters".into());
        }
        let mut bytes = OpaqueHash::default();
        for i in 0..32 {
            let byte_str = &s[i * 2..i * 2 + 2];
            bytes[i] = u8::from_str_radix(byte_str, 16).map_err(|_| "invalid hex character".to_string())?;
        }
        Ok(bytes)
    }

    impl Deserialize for TestCase {
        fn from_value(v: &Value) -> Result<Self, String> {
            let o = match v {
                Value::Object(o) => o,
                _ => return Err("expected object".into()),
            };
            let input = u64::from_value(o.get("input").ok_or("missing field input")?)?;
            let entropy_str = String::from_value(o.get("entropy").ok_or("missing field entropy")?)?;
            let entropy = Entropy { entropy: hex_to_bytes(&entropy_str)? };
            let output = Vec::<u64>::from_value(o.get("output").ok_or("missing field output")?)?;
            Ok(TestCase { input, entropy, output })
        }
    }

    #[test]
    fn run_shuffle_test() {

        // Set file path
        let file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("jamtestvectors/shuffle/shuffle_tests.json");
        // Read JSON file as string
        let file_content = std::fs::read_to_string(file_path).expect("Could not read JSON shuffle test file");

        let tests: Vec<TestCase> = from_json_str(&file_content).unwrap();
        for test_case in tests {
            let max_val = test_case.input.saturating_sub(1);
            if max_val <= u8::MAX as u64 {
                let sequence: Vec<u8> = (0..test_case.input).map(|i| i as u8).collect();
                let output_result: Vec<u8> = shuffle(&sequence, &test_case.entropy);
                let result_u64: Vec<u64> = output_result.iter().map(|&x| x as u64).collect();
                println!("expected: {:0x?}", test_case.output);
                println!("result:   {:0x?}", result_u64);
                assert_eq!(test_case.output, result_u64);
            } else if max_val <= u16::MAX as u64 {
                let sequence: Vec<u16> = (0..test_case.input).map(|i| i as u16).collect();
                let output_result: Vec<u16> = shuffle(&sequence, &test_case.entropy);
                let result_u64: Vec<u64> = output_result.iter().map(|&x| x as u64).collect();
                println!("expected: {:0x?}", test_case.output);
                println!("result:   {:0x?}", result_u64);
                assert_eq!(test_case.output, result_u64);
            } else if max_val <= u32::MAX as u64 {
                let sequence: Vec<u32> = (0..test_case.input).map(|i| i as u32).collect();
                let output_result: Vec<u32> = shuffle(&sequence, &test_case.entropy);
                let result_u64: Vec<u64> = output_result.iter().map(|&x| x as u64).collect();
                println!("expected: {:0x?}", test_case.output);
                println!("result:   {:0x?}", result_u64);
                assert_eq!(test_case.output, result_u64);
            } else {
                let sequence: Vec<u64> = (0..test_case.input).collect();
                let output_result: Vec<u64> = shuffle(&sequence, &test_case.entropy);
                println!("expected: {:0x?}", test_case.output);
                println!("result:   {:0x?}", output_result);
                assert_eq!(test_case.output, output_result);
            }
            println!("\n");
        }
    }
}
