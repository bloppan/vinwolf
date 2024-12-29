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

