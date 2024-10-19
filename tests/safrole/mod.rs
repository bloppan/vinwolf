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
    path.push("data/safrole/full/");
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

    const TEST_TYPE: &str = "tiny";

    fn run_safrole_json_file(filename: &str) {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join(format!("data/safrole/{}/{}", TEST_TYPE, filename));

        let test = load_json_data(path.to_str().unwrap()).expect("Failed to load JSON data");

        let mut post_state: SafroleState = test.pre_state.clone();
        // Exec update state safrole
        let res = JsonData {
            input: test.input.clone(),
            output: update_state(test.input.clone(), &mut post_state),
            pre_state: test.pre_state.clone(),
            post_state,
        };

        println!("state result = {:?}", res.output);
        assert_eq!(test.post_state.tau, res.post_state.tau);
        assert_eq!(test.post_state.eta, res.post_state.eta);
        assert_eq!(test.post_state.lambda, res.post_state.lambda);
        assert_eq!(test.post_state.kappa, res.post_state.kappa);
        assert_eq!(test.post_state.gamma_k, res.post_state.gamma_k);
        assert_eq!(test.post_state.iota, res.post_state.iota);
        assert_eq!(test.post_state.gamma_a, res.post_state.gamma_a);
        assert_eq!(test.post_state.gamma_s, res.post_state.gamma_s);
    }

    /*#[test]
    fn test_enact_epoch_change_with_no_tickets_1() {
        run_safrole_json_file("enact-epoch-change-with-no-tickets-1.json");
    }
   
    #[test]
    fn test_enact_epoch_change_with_no_tickets_2() {
        run_safrole_json_file("enact-epoch-change-with-no-tickets-2.json");
    }

    #[test]
    fn test_enact_epoch_change_with_no_tickets_3() {
        run_safrole_json_file("enact-epoch-change-with-no-tickets-3.json");
    }

    #[test]
    fn test_enact_epoch_change_with_no_tickets_4() {
        run_safrole_json_file("enact-epoch-change-with-no-tickets-4.json");
    }

    #[test]
    fn test_publish_tickets_no_mark_1() {
        run_safrole_json_file("publish-tickets-no-mark-1.json");
    }

    #[test]
    fn test_publish_tickets_no_mark_2() {
        run_safrole_json_file("publish-tickets-no-mark-2.json");
    }

    #[test]
    fn test_publish_tickets_no_mark_3() {
        run_safrole_json_file("publish-tickets-no-mark-3.json");
    }

    #[test]
    fn test_publish_tickets_no_mark_4() {
        run_safrole_json_file("publish-tickets-no-mark-4.json");
    }
    
    #[test]
    fn test_publish_tickets_no_mark_5() {
        run_safrole_json_file("publish-tickets-no-mark-5.json");
    }

    #[test]
    fn test_publish_tickets_no_mark_6() {
        run_safrole_json_file("publish-tickets-no-mark-6.json");
    }

    #[test]
    fn test_publish_tickets_no_mark_7() {
        run_safrole_json_file("publish-tickets-no-mark-7.json");
    }

    #[test]
    fn test_publish_tickets_no_mark_8() {
        run_safrole_json_file("publish-tickets-no-mark-8.json");
    }

    #[test]
    fn test_publish_tickets_no_mark_9() {
        run_safrole_json_file("publish-tickets-no-mark-9.json");
    }

    #[test]
    fn test_publish_tickets_with_mark_1() {
        run_safrole_json_file("publish-tickets-with-mark-1.json");
    }

    #[test]
    fn test_publish_tickets_with_mark_2() {
        run_safrole_json_file("publish-tickets-with-mark-2.json");
    }

    #[test]
    fn test_publish_tickets_with_mark_3() {
        run_safrole_json_file("publish-tickets-with-mark-3.json");
    }

    #[test]
    fn test_publish_tickets_with_mark_4() {
        run_safrole_json_file("publish-tickets-with-mark-4.json");
    }

    #[test]
    fn test_publish_tickets_with_mark_5() {
        run_safrole_json_file("publish-tickets-with-mark-5.json");
    }

    #[test]
    fn test_skip_epoch_tail_1() {
        run_safrole_json_file("skip-epoch-tail-1.json");
    }

    #[test]
    fn test_skip_epochs_1() {
        run_safrole_json_file("skip-epochs-1.json");
    }*/

    #[test]
    fn test_enact_epoch_change_with_padding_1() {
        run_safrole_json_file("enact-epoch-change-with-padding-1.json");
    }
}