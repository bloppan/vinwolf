use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

#[derive(Deserialize, Debug)]
struct Input {
    slot: u32,
    entropy: String,
    extrinsic: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct OkData {
    epoch_mark: Option<String>,
    tickets_mark: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Output {
    ok: OkData,
}

#[derive(Deserialize, Debug)]
struct ValidatorKeys {
    bandersnatch: String,
    ed25519: String,
    bls: String,
    metadata: String,
}

#[derive(Deserialize, Debug)]
struct Keys {
    keys: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct State {
    tau: u32,
    eta: Vec<String>,
    lambda: Vec<ValidatorKeys>,
    kappa: Vec<ValidatorKeys>,
    gamma_k: Vec<ValidatorKeys>,
    iota: Vec<ValidatorKeys>,
    gamma_a: Vec<String>,
    gamma_s: Keys,
    gamma_z: String,
}

#[derive(Deserialize, Debug)]
struct JsonData {
    input: Input,
    output: Output,
    pre_state: State,
    post_state: State,
}

pub fn load_json_data() -> Result<JsonData, Box<dyn std::error::Error>> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR")); // root project's directory
    path.push("data/enact-epoch-change-with-no-tickets-1.json");

    let mut file = File::open(&path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let data: JsonData = serde_json::from_str(&contents)?;
    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_json_data() {
        let data = load_json_data().expect("Failed to load JSON data");
        println!("{:?}", data);
        
    }
}
