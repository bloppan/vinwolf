use std::collections::HashSet;
use std::hash::Hash;
use sp_core::{ed25519, Pair};

use crate::types::{BandersnatchPublic, BlsPublic, Ed25519Public, Ed25519Signature, Metadata, ValidatorsData};

pub fn is_sorted_and_unique<T: PartialOrd + Hash + Eq>(vec: &[T]) -> bool {
    let mut seen = HashSet::new();

    if vec.len() < 2 {
        return true;
    }
    
    vec.windows(2).all(|window| window[0] < window[1]) && vec.iter().all(|x| seen.insert(x))
}

pub fn has_duplicates<T: Eq + std::hash::Hash + Clone>(items: &[T]) -> bool {
    let mut seen = HashSet::<T>::new();
    for i in items {
        if !seen.insert(i.clone()) {
            return true;
        }
    }
    false
}

pub trait VerifySignature {
    fn verify_signature(&self, message: &[u8], public_key: &Ed25519Public) -> bool;
}

impl VerifySignature for Ed25519Signature {
    
    fn verify_signature(&self, message: &[u8], public_key: &Ed25519Public) -> bool {

        let signature = ed25519::Signature::from_raw(*self);
        let public_key = ed25519::Public::from_raw(*public_key);

        ed25519::Pair::verify(&signature, message, &public_key)
    }
}

pub fn set_offenders_null(validators_data: &mut ValidatorsData, offenders: &[Ed25519Public]) {
    
    // We return the same keyset if there aren't offenders
    if offenders.is_empty() {
        return;
    }

    // For each offender set ValidatorData to zero
    'next_offender: for offender in offenders {
        for validator in validators_data.0.iter_mut() {
            if *offender == validator.ed25519 {
                validator.bandersnatch = [0u8; std::mem::size_of::<BandersnatchPublic>()];
                validator.ed25519 = [0u8; std::mem::size_of::<Ed25519Public>()];
                validator.bls = [0u8; std::mem::size_of::<BlsPublic>()];
                validator.metadata = [0u8; std::mem::size_of::<Metadata>()];
                continue 'next_offender;
            }
        }
    }
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn integer_sorted() {
        let integers = vec![1, 2, 4, 6, 8];
        assert_eq!(true, is_sorted_and_unique(&integers));
        let integers2 = vec![1, 2, 3, 3, 4, 6];
        assert_eq!(false, is_sorted_and_unique(&integers2));
    }

    #[test]
    fn array_sorted() {
        let array_1: [u8; 5] = [0, 1, 2, 3, 4];
        let array_2: [u8; 5] = [1, 2, 3, 4, 5];
        let array_3: [u8; 5] = [0, 1, 3, 3, 4];
        let array_4: [u8; 5] = [0, 1, 3, 3, 4];

        let vector: Vec<[u8; 5]> = vec![array_1, array_2];
        let vector2: Vec<[u8; 5]> = vec![array_2, array_1];
        let vector3: Vec<[u8; 5]> = vec![array_1, array_3];
        let vector4: Vec<[u8; 5]> = vec![array_3, array_4];
        let vector5: Vec<[u8; 5]> = vec![array_2, array_4];

        assert_eq!(true, is_sorted_and_unique(&vector));
        assert_eq!(false, is_sorted_and_unique(&vector2));
        assert_eq!(true, is_sorted_and_unique(&vector3));
        assert_eq!(false, is_sorted_and_unique(&vector4));
        assert_eq!(false, is_sorted_and_unique(&vector5));
    }
}
