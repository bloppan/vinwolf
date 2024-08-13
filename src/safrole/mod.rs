extern crate hex;
extern crate array_bytes;
mod bandersnatch;
mod time;
use serde::Deserialize;
use frame_support::Blake2_256;
use sp_core::blake2_256;

// The length of an epoch in timeslots
const E: u32 = 12; // The length of an epoch timeslots.
const Y: u32 = 10; // The number of slots into an epoch at which ticket-submission ends
const V: u32 = 6;  // Total number of validators

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct ValidatorData {
    bandersnatch: String,
    ed25519: String,
    bls: String,
    metadata: String,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct Keys {
    keys: Vec<String>,
}

/*
    @gamma_k:   validators's pending set
    @gamma_a:   ticket accumulator. A series of highestscoring ticket identifiers to be used for the next epoch
    @gamma_s:   current epoch's slot-sealer series
    @gamma_z:   epoch's root, a Bandersnatch ring root composed with the one Bandersnatch key of each of the next
                epoch’s validators
    @iota:      validator's staging set
    @kappa:     validator's active set
    @lambda:    validator's active set in the prior epoch
*/
#[allow(non_camel_case_types)]
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct SafroleState {
    pub tau: u32,
    pub eta: Vec<String>,
    pub lambda: Vec<ValidatorData>,
    pub kappa: Vec<ValidatorData>,
    pub gamma_k: Vec<ValidatorData>,
    pub iota: Vec<ValidatorData>,
    pub gamma_a: Vec<String>,
    pub gamma_s: Keys,
    pub gamma_z: String,
}

#[allow(non_camel_case_types)]
enum KeySet {
    gamma_k,
    kappa,
    lambda,
    iota,
    gamma_a,
    gamma_s,
}

pub struct Safrole {
    pub pre_state: SafroleState,
    pub post_state: SafroleState,
}

#[allow(non_camel_case_types)]
#[derive(Deserialize, Debug, PartialEq)]
enum ErrorType {
    bad_slot = 0, // Timeslot value must be strictly monotonic.
    unexpected_ticket = 1, // Received a ticket while in epoch's tail.
    bad_ticket_order = 2, // Tickets must be sorted.
    bad_ticket_proof = 3, // Invalid ticket ring proof.
    bad_ticket_attempt = 4, // Invalid ticket attempt value.
    reserved = 5, // Reserved.
    duplicate_ticket = 6, // Found a ticket duplicate.
}
#[derive(Deserialize, Debug, PartialEq)]
pub struct EpochMark {
    pub entropy: String,
    pub validators: Vec<String>,
}
#[derive(Deserialize, Debug, PartialEq)]
pub struct OutputMarks {
    pub epoch_mark: Option<EpochMark>,
    pub tickets_mark: Option<String>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct Input {
    pub slot: u32,
    pub entropy: String,
    pub extrinsic: Vec<String>,
}
#[allow(non_camel_case_types)]
#[derive(Deserialize, Debug, PartialEq)]
pub enum Output {
    ok(OutputMarks),
    err(ErrorType),
}

fn update_entropy_pool(state: &mut SafroleState) {
    // Update the three aditional accumulator's values (Eq 68)
    let eta_0 = state.eta[0].clone();
    let eta_1 = state.eta[1].clone();
    let eta_2 = state.eta[2].clone();
    state.eta[1] = eta_0.clone();
    state.eta[2] = eta_1.clone();
    state.eta[3] = eta_2.clone();
}

fn update_recent_entropy(input: Input, state: &mut SafroleState) {
    // update eta'[0] (Eq 67)
    let clean_eta0 = &state.eta[0][2..];
    let clean_entropy = &input.entropy[2..];
    let eta0_bytes = array_bytes::hex2bytes(clean_eta0).expect("Failed to convert hex to bytes");
    let entropy_bytes = array_bytes::hex2bytes(clean_entropy).expect("Failed to convert hex to bytes");
    let concatenated = [eta0_bytes, entropy_bytes].concat();
    let hash = blake2_256(&concatenated);
    state.eta[0] = format!("0x{}", hex::encode(hash));
}

fn bandersnatch_keys_collect(state: SafroleState, key_set: KeySet) -> Vec<String> {
    let bandersnatch_keys: Vec<String> = match key_set {
        KeySet::gamma_k => state.gamma_k.iter().map(|validator| validator.bandersnatch.clone()).collect(),
        KeySet::kappa => state.kappa.iter().map(|validator| validator.bandersnatch.clone()).collect(),
        KeySet::lambda => state.lambda.iter().map(|validator| validator.bandersnatch.clone()).collect(),
        KeySet::iota => state.iota.iter().map(|validator| validator.bandersnatch.clone()).collect(),
        KeySet::gamma_a => state.gamma_a.clone(), 
        KeySet::gamma_s => state.gamma_s.keys.clone(), 
    };

    bandersnatch_keys
}

fn key_rotation(state: &mut SafroleState) { // (Eq 58)

    let bandersnatch_keys = bandersnatch_keys_collect(state.clone(), KeySet::gamma_k);
    state.lambda = state.kappa.clone();
    state.kappa = state.gamma_k.clone();
    state.gamma_k = state.iota.clone();
    state.gamma_z = bandersnatch::ring_vrf_proof(bandersnatch_keys);
}

pub fn update_state(input: Input, state: &mut SafroleState) -> Output {

    let mut output: Option<Output> = None;
    let mut epoch_mark: Option<EpochMark> = None;
    let mut tickets_mark: Option<String> = None;

    if input.slot > state.tau {
        // Check if we are in a new epoch (e' > e)
        let e: u32 = state.tau / E;
        let m: u32 = state.tau % E;
        let post_e: u32 = input.slot / E;
        if post_e > e {
            update_entropy_pool(state); 
            key_rotation(state);
            state.tau = input.slot; // tau' = slot
            epoch_mark = Some(EpochMark {
                entropy: state.eta[1].clone(),
                validators: bandersnatch_keys_collect(state.clone(), KeySet::gamma_k),  
            });
            if post_e == e + 1 && m >= Y && state.gamma_a.len() as u32 == E {
                // Z(gamma_a)
            } else if post_e == e {
                // gamma_s' = gamma_s
            } else {
                let bandersnatch_keys = bandersnatch_keys_collect(state.clone(), KeySet::kappa);
                state.gamma_s = Keys { keys: fallback(state.eta[2].clone(), bandersnatch_keys.clone()) };
            } 
        }  
        // Update recent entropy eta[0]
        update_recent_entropy(input.clone(), state);
        return Output::ok(OutputMarks {epoch_mark, tickets_mark});
    } else if input.slot == state.tau {
        return Output::err(ErrorType::bad_slot);
    } else {
        return Output::ok(OutputMarks { epoch_mark: None, tickets_mark: None });
    }
}

pub fn fallback(entropy: String, keys: Vec<String>) -> Vec<String> {
    let mut new_keys = vec![String::new(); (E as u32).try_into().unwrap()];
    let clean_entropy = &entropy[2..];
    assert!(keys.len() >= 6, "The keys vector must have at least 6 elements");

    for i in 0u32..E as u32 { // Limita el bucle al tamaño de keys
        let index_le = i.to_le_bytes();
        let index_hex = hex::encode(index_le);
        let entropy_bytes = array_bytes::hex2bytes(clean_entropy).expect("Failed to convert hex to bytes");
        let index_bytes = array_bytes::hex2bytes(index_hex).expect("Failed to convert hex to bytes");
        let concatenated = [entropy_bytes, index_bytes].concat();
        let hash = blake2_256(&concatenated);
        let hash_4 = u32::from_be_bytes([hash[3], hash[2], hash[1], hash[0]]);
        let id = (hash_4 % V as u32) as usize;
        new_keys[i as usize] = keys[id].clone();
    }
    
    new_keys
}

pub fn test_bandersnatch() {
    let ring_set_hex = vec![
        "5e465beb01dbafe160ce8216047f2155dd0569f058afd52dcea601025a8d161d".to_string(),
        "3d5e5a51aab2b048f8686ecd79712a80e3265a114cc73f14bdb2a59233fb66d0".to_string(),
        "aa2b95f7572875b0d0f186552ae745ba8222fc0b5bd456554bfe51c68938f8bc".to_string(),
        "7f6190116d118d643a98878e294ccf62b509e214299931aad8ff9764181a4e33".to_string(),
        "48e5fcdce10e0b64ec4eebd0d9211c7bac2f27ce54bca6f7776ff6fee86ab3e3".to_string(),
        "f16e5352840afb47e206b5c89f560f2611835855cf2e6ebad1acc9520a72591d".to_string(),
    ];

    let gamma_z = bandersnatch::ring_vrf_proof(ring_set_hex);

    println!("gamma_z = {:?}", gamma_z);
}
