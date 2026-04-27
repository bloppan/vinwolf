pub mod dev_accounts;
pub mod grid;
pub mod jamnp_codec;
pub mod jamnp_types;
pub mod message;
pub mod net_ctrl;
pub mod net_utils;
pub mod scheduler;

use jam_types::ValidatorIndex;
use std::sync::{LazyLock, Mutex};

pub use dev_accounts::{
    add_dev_account, get_dev_account_connection, parse_dev_accounts, remove_dev_account, Identity,
    RawIdentity,
};
pub use grid::{am_i_the_preferred_initiator, compute_proxy_index, is_neighbour, width};
pub use jamnp_types::{
    Announcement, ConnectionError, Handshake, ImportedBlocks, LastFinalizedBlock, Leaf,
    NetworkError, StreamError, StreamKind, TicketDistributed,
};
pub use message::{
    ConnectionInfo, NetworkMessage, BLOCK_ANNOUNCEMENT, BLOCK_REQUEST, STATE_REQUEST,
    TICKET_GENERATION, TICKET_PROXY,
};
pub use net_ctrl::{NetworkController, PeerInfo, PeerState};
pub use net_utils::{
    load_client_config, load_credentials, load_server_config, parse_pem_certs,
    parse_pem_private_key, SkipClientVerification, SkipServerVerification,
};

static ACCOUNT_ID: LazyLock<Mutex<ValidatorIndex>> =
    LazyLock::new(|| Mutex::new(ValidatorIndex::default()));

pub mod node_config {

    use super::*;

    pub fn get_account_id() -> ValidatorIndex {
        match ACCOUNT_ID.lock() {
            Ok(account_id) => *account_id,
            Err(poisoned) => *poisoned.into_inner(),
        }
    }

    pub fn set_account_id(id: ValidatorIndex) {
        match ACCOUNT_ID.lock() {
            Ok(mut account_id) => *account_id = id,
            Err(poisoned) => *poisoned.into_inner() = id,
        }
    }
}

pub use node_config::{get_account_id, set_account_id};
