pub mod dev_accounts;
pub mod jamnp_codec;
pub mod jamnp_types;
pub mod message;
pub mod net_ctrl;
pub mod net_utils;
pub mod scheduler;
pub mod grid;

use jam_types::ValidatorIndex;
use rustls::lock::Mutex;
use std::sync::LazyLock;

static ACCOUNT_ID: LazyLock<Mutex<ValidatorIndex>> = LazyLock::new(|| Mutex::new(ValidatorIndex::default()));

pub mod node_config {

    use super::*;

    pub fn get_account_id() -> ValidatorIndex {
        ACCOUNT_ID.lock().unwrap().clone()
    }

    pub fn set_account_id(id: ValidatorIndex) {
        *ACCOUNT_ID.lock().unwrap() = id;
    }
}
