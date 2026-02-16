use constants::node::VALIDATORS_COUNT;
use jam_types::{ValidatorIndex, Ed25519Public};

pub fn am_i_the_preferred_initiator(my_key: &Ed25519Public, peer_key: &Ed25519Public) -> bool {
    let cond = ((my_key[31] > 127) ^ (peer_key[31] > 127)) ^ (my_key < peer_key);
    cond
}

/// Grid width W = floor(sqrt(V)) as defined in JAMNP-S.
pub const fn grid_width() -> usize {
    // Integer sqrt: largest w such that w*w <= V
    let mut w = 1;
    while (w + 1) * (w + 1) <= VALIDATORS_COUNT {
        w += 1;
    }
    w
}

/// Two validators in the same epoch are grid neighbours if they share
/// the same row (index / W) or the same column (index % W).
pub fn is_grid_neighbour(my_index: ValidatorIndex, other_index: ValidatorIndex) -> bool {
    if my_index == other_index {
        return false;
    }
    let w = grid_width();
    let same_row = (my_index as usize / w) == (other_index as usize / w);
    let same_col = (my_index as usize % w) == (other_index as usize % w);
    same_row || same_col
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::dev_accounts;

    #[test]
    fn preferred_initiator_test() {

        let identities = dev_accounts::parse_dev_accounts();
        let ed25519_public: Vec<_> = identities.iter().map(|key| key.ed25519_public).collect();
        let node_id: usize = 5;

        for id in ed25519_public.iter() {
            let is_preferred = am_i_the_preferred_initiator(id, &ed25519_public[node_id]);
            if is_preferred {
                println!("mismo");
            } else {
                println!("el otro");
            }
        }
    }
}
