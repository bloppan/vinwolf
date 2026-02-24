use constants::node::VALIDATORS_COUNT;
use jam_types::{ValidatorIndex, Ed25519Public};

pub fn am_i_the_preferred_initiator(my_key: &Ed25519Public, peer_key: &Ed25519Public) -> bool {
    let cond = ((my_key[31] > 127) ^ (peer_key[31] > 127)) ^ (my_key < peer_key);
    cond
}

/// Grid width W = floor(sqrt(V)) as defined in JAMNP-S.
pub const fn width() -> usize {
    // largest w such that w*w <= V
    let mut w = 1;
    while (w + 1) * (w + 1) <= VALIDATORS_COUNT {
        w += 1;
    }
    w
}

/// Two validators in the same epoch are grid neighbours if they share
/// the same row (index / W) or the same column (index % W).
pub fn is_neighbour(my_index: ValidatorIndex, other_index: ValidatorIndex) -> bool {
    if my_index == other_index {
        return false;
    }
    let w = width();
    let same_row = (my_index as usize / w) == (other_index as usize / w);
    let same_col = (my_index as usize % w) == (other_index as usize % w);
    same_row || same_col
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::dev_accounts;

    #[test]
    fn width_test() {
        let w = width();
        // w must be floor(sqrt(VALIDATORS_COUNT))
        assert!(w * w <= VALIDATORS_COUNT, "w^2 must be <= V");
        assert!((w + 1) * (w + 1) > VALIDATORS_COUNT, "(w+1)^2 must be > V");

        #[cfg(not(feature = "full"))]
        assert_eq!(w, 2, "tiny config: floor(sqrt(6)) == 2");

        #[cfg(feature = "full")]
        assert_eq!(w, 31, "full config: floor(sqrt(1023)) == 31");
    }

    #[test]
    fn is_neighbour_test() {
        // tiny config: V=6, W=2
        //        col0  col1
        // row0: [  0,    1 ]
        // row1: [  2,    3 ]
        // row2: [  4,    5 ]

        // Same row neighbours
        assert!(is_neighbour(0, 1));
        assert!(is_neighbour(2, 3));
        assert!(is_neighbour(4, 5));

        // Same column neighbours
        assert!(is_neighbour(0, 2));
        assert!(is_neighbour(0, 4));
        assert!(is_neighbour(1, 3));
        assert!(is_neighbour(1, 5));
        assert!(is_neighbour(2, 4));
        assert!(is_neighbour(3, 5));

        // Symmetry: if a is neighbour of b, b is neighbour of a
        assert!(is_neighbour(4, 0));
        assert!(is_neighbour(5, 1));

        // Not neighbours (different row AND different column)
        assert!(!is_neighbour(0, 3)); // row0,col0 vs row1,col1
        assert!(!is_neighbour(1, 2)); // row0,col1 vs row1,col0
        assert!(!is_neighbour(1, 4)); // row0,col1 vs row2,col0
        assert!(!is_neighbour(2, 5)); // row1,col0 vs row2,col1

        // A validator is not its own neighbour
        for i in 0..VALIDATORS_COUNT as u16 {
            assert!(!is_neighbour(i, i));
        }
    }

    #[test]
    fn preferred_initiator_test() {
        // P(a,b) = (a[31]>127) XOR (b[31]>127) XOR (a<b)
        // Exactly one of the pair must be the preferred initiator (anti-symmetry).

        let mut key_a = Ed25519Public::default();
        let mut key_b = Ed25519Public::default();

        // Case 1: both high bits 0, a < b  =>  XOR(false, false, true) = true
        key_a[0] = 1; key_a[31] = 0;
        key_b[0] = 2; key_b[31] = 0;
        assert!(am_i_the_preferred_initiator(&key_a, &key_b));
        assert!(!am_i_the_preferred_initiator(&key_b, &key_a));

        // Case 2: both high bits 1, a < b  =>  XOR(true, true, true) = true
        key_a[0] = 1; key_a[31] = 200;
        key_b[0] = 2; key_b[31] = 200;
        assert!(am_i_the_preferred_initiator(&key_a, &key_b));
        assert!(!am_i_the_preferred_initiator(&key_b, &key_a));

        // Case 3: a high bit 0, b high bit 1, a < b  =>  XOR(false, true, true) = false
        key_a[0] = 1; key_a[31] = 0;
        key_b[0] = 2; key_b[31] = 200;
        assert!(!am_i_the_preferred_initiator(&key_a, &key_b));
        assert!(am_i_the_preferred_initiator(&key_b, &key_a));

        // Case 4: a high bit 1, b high bit 0, a < b  =>  XOR(true, false, true) = false
        key_a[0] = 1; key_a[31] = 200;
        key_b[0] = 2; key_b[31] = 0;
        assert!(!am_i_the_preferred_initiator(&key_a, &key_b));
        assert!(am_i_the_preferred_initiator(&key_b, &key_a));

        // Anti-symmetry with dev accounts: for every pair exactly one is initiator
        let identities = dev_accounts::parse_dev_accounts();
        let keys: Vec<_> = identities.iter().map(|id| id.ed25519_public).collect();
        for i in 0..keys.len() {
            for j in (i + 1)..keys.len() {
                let ab = am_i_the_preferred_initiator(&keys[i], &keys[j]);
                let ba = am_i_the_preferred_initiator(&keys[j], &keys[i]);
                assert_ne!(ab, ba, "pair ({i},{j}) must have exactly one initiator");
            }
        }
    }
}
