/* 
    We have previously discussed the model of workpackages and services, however we have yet to make a substantial discussion 
    of exactly how some coretime resource may be apportioned to some work-package and its associated service. 
    In the YP Ethereum model, the underlying resource, gas, is procured at the point of introduction on-chain and the purchaser 
    is always the same agent who authors the data which describes the work to be done (i.e. the transaction). 
    Conversely, in Polkadot the underlying resource, a parachain slot, is procured with a substantial deposit for typically 
    24 months at a time and the procurer, generally a parachain team, will often have no direct relation to the author of the 
    work to be done (i.e. a parachain block).

    On a principle of flexibility, we would wish Jam capable of supporting a range of interaction patterns both Ethereum-style 
    and Polkadot-style. In an effort to do so, we introduce the authorization system, a means of disentangling the intention 
    of usage for some coretime from the specification and submission of a particular workload to be executed on it. We are thus 
    able to disassociate the purchase and assignment of coretime from the specific determination of work to be done with it, 
    and so are able to support both Ethereum-style and Polkadot-style interaction patterns.

    The authorization system involves two key concepts: authorizers and authorizations. An authorization is simply a piece of opaque
    data to be included with a work-package. An authorizer meanwhile, is a piece of pre-parameterized logic which accepts as an 
    additional parameter an authorization and, when executed within a vm of prespecified computational limits, provides a Boolean 
    output denoting the veracity of said authorization.

    Authorizers are identified as the hash of their logic (specified as the vm code) and their pre-parameterization. The process 
    by which work-packages are determined to be authorized (or not) is not the competence of on-chain logic and happens entirely 
    in-core. However, on-chain logic must identify each set of authorizers assigned to each core in order to verify that a 
    work-package is legitimately able to utilize that resource. It is this subsystem we will now define.
*/

use std::array::from_fn;
use std::mem::size_of;
use std::collections::VecDeque;

use crate::constants::{CORES_COUNT, MAX_ITEMS_AUTHORIZATION_POOL, MAX_ITEMS_AUTHORIZATION_QUEUE};
use crate::types::{AuthorizerHash, AuthPool, AuthPools, AuthQueue, AuthQueues, CodeAuthorizers, TimeSlot};
use crate::blockchain::state::get_authqueues;

mod codec;

pub fn process_authorizations(
    auth_pool_state: &mut AuthPools, 
    slot: &TimeSlot, 
    code_authorizers: &CodeAuthorizers) {
    // We define the set of authorizers allowable for a particular core as the authorizer pool

    // To maintain this value, a futher portion of state is tracked for each core: The core's authorizer queue,
    // from which we draw values, to fill the pool.
    // Note: The portion of state AUTH_QUEUE_STATE may be altered only through an exogenous call made from the accumulate logic
    // of an appropriately privileged service.

    // We utilize the CodeAuthorizers (from guarantees extrinsic) to remove the oldest authorizer which has 
    // been used to justify a guaranteed work-package in the current block.
    for auth in code_authorizers.authorizers.iter() {
        if auth_pool_state.auth_pools[auth.core as usize].auth_pool.contains(&auth.auth_hash) {
            auth_pool_state.auth_pools[auth.core as usize].auth_pool.retain(|&x| x != auth.auth_hash);
        }
    }

    // Since AUTH_POOL_STATE is dependent on AUTH_QUEUE_STATE, practically speaking, this step must be computed 
    // after accumulation, the stage in which AUTH_QUEUE_STATE is defined.
    let auth_queues = get_authqueues();

    // The state transition of a block involves placing a new authorization into the pool from the queue
    for core in 0..CORES_COUNT {
        let new_auth = auth_queues.auth_queues[core].auth_queue[*slot as usize % MAX_ITEMS_AUTHORIZATION_QUEUE];
        auth_pool_state.auth_pools[core].auth_pool.push_back(new_auth);
        while auth_pool_state.auth_pools[core].auth_pool.len() > MAX_ITEMS_AUTHORIZATION_POOL {
            auth_pool_state.auth_pools[core].auth_pool.pop_front();
        }
    }
}

impl Default for AuthPool {
    fn default() -> Self {
        AuthPool {
            auth_pool: VecDeque::with_capacity(MAX_ITEMS_AUTHORIZATION_POOL),
        }
    }
}

impl Default for AuthPools {
    fn default() -> Self {
        AuthPools {
            auth_pools: Box::new(from_fn(|_| AuthPool::default())),
        }
    }
}

impl Default for AuthQueue {
    fn default() -> Self {
        AuthQueue {
            auth_queue: Box::new(from_fn(|_| [0; size_of::<AuthorizerHash>()])),
        }
    }
}

impl Default for AuthQueues {
    fn default() -> Self {
        AuthQueues {
            auth_queues: Box::new(from_fn(|_| AuthQueue::default())),
        }
    }
}
