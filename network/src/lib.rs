pub mod client;
pub mod dev_accounts;
pub mod jamnp_codec;
pub mod jamnp_types;
pub mod message;
pub mod net_controller;
pub mod net_utils;
pub mod scheduler;

use jam_types::ValidatorIndex;
use rustls::lock::Mutex;
use std::sync::LazyLock;

static ACCOUNT_ID: LazyLock<Mutex<ValidatorIndex>> = LazyLock::new(| | Mutex::new(ValidatorIndex::default()));

pub mod node_config {

    use super::*;

    pub fn get_account_id() -> ValidatorIndex {
        ACCOUNT_ID.lock().unwrap().clone()
    }

    pub fn set_account_id(id: ValidatorIndex) {
        *ACCOUNT_ID.lock().unwrap() = id;
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn preferred_initiator<'a>(a: &'a[u8; 32], b: &'a[u8; 32]) -> &'a[u8; 32] {
        let cond = ((a[31] > 127) ^ (b[31] > 127)) ^ (a < b);
        if cond { a } else { b }
    }

    #[test]
    fn preferred_initiator_test() {

        let identities = dev_accounts::parse_dev_accounts();

        let ed25519_public: Vec<_> = identities.iter().map(|key| key.ed25519_public).collect();
        let node_id: usize = 5;

        for id in ed25519_public.iter() {
            let preferred = preferred_initiator(id, &ed25519_public[node_id]);
            if *preferred == ed25519_public[node_id] {
                println!("mismo");
            } else {
                println!("el otro");
            }
        }
    }
}
