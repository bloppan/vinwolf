use sp_core::blake2_256;

use crate::types::{Entropy, EntropyPool};

pub fn rotate_entropy_pool(entropy_pool: &mut EntropyPool) {
    // In addition to the entropy accumulator eta0, we retain three additional historical values of the accumulator at the point of 
    // each of the three most recently ended epochs, eta1, eta2 and eta3. The second-oldest of these eta2 is utilized to help ensure 
    // future entropy is unbiased and seed the fallback seal-key generation function with randomness. The oldest is used to regenerate 
    // this randomness when verifying the seal gamma_s.   
    entropy_pool.buf[3] = entropy_pool.buf[2].clone();
    entropy_pool.buf[2] = entropy_pool.buf[1].clone();
    entropy_pool.buf[1] = entropy_pool.buf[0].clone();
}

pub fn update_recent_entropy(entropy_pool: &mut EntropyPool, entropy_source: [u8; 32]) {
    // eta0 defines the state of the randomness accumulator to which the provably random output of the vrf, the signature over 
    // some unbiasable input, is combined each block. eta1 and eta2 meanwhile retain the state of this accumulator at the end 
    // of the two most recently ended epochs in order.
    entropy_pool.buf[0] = Entropy { entropy: blake2_256(&[entropy_pool.buf[0].entropy, entropy_source].concat())};
}