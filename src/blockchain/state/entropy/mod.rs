use sp_core::blake2_256;
use std::sync::Mutex;
use once_cell::sync::Lazy;

use crate::types::{Entropy, EntropyPool, OpaqueHash};

// eta0
static RECENT_ENTROPY: Lazy<Mutex<OpaqueHash>> = Lazy::new(|| {
    Mutex::new(OpaqueHash::default())
});

pub fn get_recent_entropy() -> Entropy {
    let recent_entropy = RECENT_ENTROPY.lock().unwrap();
    let entropy = recent_entropy.clone();
    Entropy { entropy }
}

pub fn set_recent_entropy(entropy: OpaqueHash) {
    let mut recent_entropy = RECENT_ENTROPY.lock().unwrap();
    *recent_entropy = entropy;
}

impl EntropyPool {

    pub fn rotate(&mut self) {
        // In addition to the entropy accumulator eta0, we retain three additional historical values of the accumulator at the point of 
        // each of the three most recently ended epochs, eta1, eta2 and eta3. The second-oldest of these eta2 is utilized to help ensure 
        // future entropy is unbiased and seed the fallback seal-key generation function with randomness. The oldest is used to regenerate 
        // this randomness when verifying the seal gamma_s.
        self.buf[3] = self.buf[2].clone();
        self.buf[2] = self.buf[1].clone();
        self.buf[1] = self.buf[0].clone();
    }

    pub fn update_recent(&mut self, entropy_source: OpaqueHash) {
        // eta0 defines the state of the randomness accumulator to which the provably random output of the vrf, the signature over 
        // some unbiasable input, is combined each block. eta1 and eta2 meanwhile retain the state of this accumulator at the end 
        // of the two most recently ended epochs in order.
        self.buf[0] = Entropy { entropy: blake2_256(&[self.buf[0].entropy, entropy_source].concat())};
        set_recent_entropy(self.buf[0].entropy);
    }
}


