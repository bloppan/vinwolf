use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use vinwolf::safrole::{SafroleState, Input, Output};
use vinwolf::safrole::update_state;


#[derive(Deserialize, Debug, PartialEq)]
struct JsonData {
    input: Input,
    output: Output,
    pre_state: SafroleState,
    post_state: SafroleState,
}

fn load_json_data(filename: &str) -> Result<JsonData, Box<dyn std::error::Error>> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR")); // root project's directory
    path.push("data/safrole/tiny/");
    path.push(filename);

    let mut file = File::open(&path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let data: JsonData = serde_json::from_str(&contents)?;
    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn run_safrole_test_file(filename: &str) {
        let test = load_json_data(filename).expect("Failed to load JSON data");
        let mut post_state: SafroleState = test.pre_state.clone();
        let res = JsonData {
            input: test.input.clone(),
            output: update_state(test.input.clone(), &mut post_state),
            pre_state: test.pre_state.clone(),
            post_state,
        };
        assert_eq!(test.post_state.tau, res.post_state.tau);
        assert_eq!(test.post_state.eta, res.post_state.eta);
        assert_eq!(test.post_state.lambda, res.post_state.lambda);
        assert_eq!(test.post_state.kappa, res.post_state.kappa);
        assert_eq!(test.post_state.gamma_k, res.post_state.gamma_k);
        assert_eq!(test.post_state.iota, res.post_state.iota);
        assert_eq!(test.post_state.gamma_a, res.post_state.gamma_a);
        assert_eq!(test.post_state.gamma_s, res.post_state.gamma_s);
    }

    #[test]
    fn test_safrole_json() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("data/safrole/tiny/");
        // Read each file from data directory
        for entry in fs::read_dir(path).expect("Directory not found") {
            let entry = entry.expect("Failed to read directory entry");
            let path = entry.path();
            // Ensure that the file is json type
            if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("json") {
                let filename = path.file_name().and_then(|name| name.to_str()).expect("Invalid filename");
                println!("Running test for file: {}", filename);
                run_safrole_test_file(filename);
            }
        }
    }
}