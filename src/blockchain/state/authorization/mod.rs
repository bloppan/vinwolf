use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::array::from_fn;
use std::mem::size_of;

use crate::types::{AuthorizerHash, AuthPool, AuthPools, AuthQueue, AuthQueues};

mod codec;

static AUTHORIZER_POOL_STATE: Lazy<Mutex<AuthPools>> = Lazy::new(|| Mutex::new(AuthPools{auth_pools: Box::new(from_fn(|_| AuthPool { auth_pool: Vec::new() }))}));
static AUTHORIZER_QUEUE_STATE: Lazy<Mutex<AuthQueues>> = Lazy::new(|| Mutex::new(AuthQueues{auth_queues: Box::new(from_fn(|_| AuthQueue { auth_queue: Box::new(from_fn(|_| [0; size_of::<AuthorizerHash>()])) }))}));


pub fn set_authpool_state(new_state: &AuthPools) {
    let mut state = AUTHORIZER_POOL_STATE.lock().unwrap();
    *state = new_state.clone();
}

pub fn get_authpool_state() -> AuthPools {
    let state = AUTHORIZER_POOL_STATE.lock().unwrap(); 
    return state.clone();
}

pub fn set_authqueue_state(new_state: &AuthQueues) {
    let mut state = AUTHORIZER_QUEUE_STATE.lock().unwrap();
    *state = new_state.clone();
}

pub fn get_authqueue_state() -> AuthQueues {
    let state = AUTHORIZER_QUEUE_STATE.lock().unwrap(); 
    return state.clone();
}

