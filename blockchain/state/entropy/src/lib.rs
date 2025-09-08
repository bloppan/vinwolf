use sp_core::blake2_256;
use utils::{{print_hash, print_hash_start}, log};

use jam_types::{Entropy, EntropyPool, OpaqueHash};

pub fn rotate_pool(entropy_pool: &mut EntropyPool) {
    // In addition to the entropy accumulator η0, we retain three additional historical values of the accumulator at the point of 
    // each of the three most recently ended epochs, η1, η2 and η3. The second-oldest of these η2 is utilized to help ensure 
    // future entropy is unbiased and seed the fallback seal-key generation function with randomness. The oldest is used to regenerate 
    // this randomness when verifying the seal gamma_s.
    entropy_pool.buf[3] = entropy_pool.buf[2].clone();
    entropy_pool.buf[2] = entropy_pool.buf[1].clone();
    entropy_pool.buf[1] = entropy_pool.buf[0].clone();
    log::debug!("Rotate entropy pool η=[{}, {}, {}, {}]", 
                print_hash_start!(entropy_pool.buf[0].entropy), 
                print_hash_start!(entropy_pool.buf[1].entropy), 
                print_hash_start!(entropy_pool.buf[2].entropy),
                print_hash_start!(entropy_pool.buf[3].entropy));
}

pub fn update_recent(entropy_pool: &mut EntropyPool, entropy_source: OpaqueHash) {
    // η0 defines the state of the randomness accumulator to which the provably random output of the vrf, the signature over 
    // some unbiasable input, is combined each block. η1 and η2 meanwhile retain the state of this accumulator at the end 
    // of the two most recently ended epochs in order.
    entropy_pool.buf[0] = Entropy { entropy: blake2_256(&[entropy_pool.buf[0].entropy, entropy_source].concat())};
    state_handler::entropy::set_recent(entropy_pool.buf[0].entropy);
    log::debug!("Update recent entropy η0: 0x{}", print_hash!(entropy_pool.buf[0].entropy));
}


