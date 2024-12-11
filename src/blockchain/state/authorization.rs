use once_cell::sync::Lazy;
use std::sync::Mutex;

use crate::types::OpaqueHash;
use crate::constants::{MAX_ITEMS_AUTHORIZATION_POOL, CORES_COUNT};
use crate::codec::work_report::AuthPools;

static AUTHORIZER_POOL_STATE: Lazy<Mutex<AuthPools>> = Lazy::new(|| Mutex::new(AuthPools{auth_pools: Vec::with_capacity(MAX_ITEMS_AUTHORIZATION_POOL * CORES_COUNT)}));

pub fn set_authpool_state(post_state: &AuthPools) {
    let mut state = AUTHORIZER_POOL_STATE.lock().unwrap();
    *state = post_state.clone();
}

pub fn get_authpool_state() -> AuthPools {
    let state = AUTHORIZER_POOL_STATE.lock().unwrap(); 
    return state.clone();
}

/*
#[derive(Debug, PartialEq, Clone)]
struct Authorizer {
    authorizer_hash: Vec<OpaqueHash>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct AuthorizerPool {
    pub authorizer_pool: Vec<Authorizer>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct AuthorizerQueue {
    pub authorizer_queue: Vec<Authorizer>,
}
*/