use std::default::Default;
use core::array::from_fn;

use crate::types::{OpaqueHash, Entropy, EntropyPool};

impl Default for EntropyPool {
    fn default() -> Self {
        EntropyPool(Box::new(from_fn(|_| Entropy::default())))
    }
}

impl Default for Entropy {
    fn default() -> Self {
        Entropy(OpaqueHash::default())
    }
}

pub fn rotate_entropy_pool(entropy_state: &mut EntropyPool) {
    // In addition to the entropy accumulator eta0, we retain three additional historical values of the accumulator at the point of 
    // each of the three most recently ended epochs, eta1, eta2 and eta3. The second-oldest of these eta2 is utilized to help ensure 
    // future entropy is unbiased and seed the fallback seal-key generation function with randomness. The oldest is used to regenerate 
    // this randomness when verifying the seal gamma_s.   
    entropy_state.0[3] = entropy_state.0[2].clone();
    entropy_state.0[2] = entropy_state.0[1].clone();
    entropy_state.0[1] = entropy_state.0[0].clone();
}