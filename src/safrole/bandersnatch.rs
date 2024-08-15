use ark_ec_vrfs::suites::bandersnatch::edwards as bandersnatch;
use ark_ec_vrfs::{prelude::ark_serialize, suites::bandersnatch::edwards::RingContext};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use bandersnatch::{IetfProof, Input, Output, Public, RingProof, Secret};

use crate::block::TicketEnvelope;
use crate::safrole::{SafroleState, Input as InputSafrole, 
                    KeySet, TicketBody, bandersnatch_keys_collect,
                    Output as OutputSafrole, OutputMarks, ErrorType};

use std::collections::HashSet;

const RING_SIZE: usize = 6;

// This is the IETF `Prove` procedure output as described in section 2.2
// of the Bandersnatch VRFs specification
#[derive(CanonicalSerialize, CanonicalDeserialize)]
struct IetfVrfSignature {
    output: Output,
    proof: IetfProof,
}

// This is the IETF `Prove` procedure output as described in section 4.2
// of the Bandersnatch VRFs specification
#[derive(CanonicalSerialize, CanonicalDeserialize)]
struct RingVrfSignature {
    output: Output,
    // This contains both the Pedersen proof and actual ring proof.
    proof: RingProof,
}

// "Static" ring context data
fn ring_context() -> &'static RingContext {
    use std::sync::OnceLock;
    static RING_CTX: OnceLock<RingContext> = OnceLock::new();
    RING_CTX.get_or_init(|| {
        use bandersnatch::PcsParams;
        use std::{fs::File, io::Read};
        let manifest_dir =
            std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set");
        let filename = format!("{}/data/zcash-srs-2-11-uncompressed.bin", manifest_dir);
        let mut file = File::open(filename).unwrap();
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).unwrap();
        let pcs_params = PcsParams::deserialize_uncompressed_unchecked(&mut &buf[..]).unwrap();
        RingContext::from_srs(RING_SIZE, pcs_params).unwrap()
    })
}

// Construct VRF Input Point from arbitrary data (section 1.2)
fn vrf_input_point(vrf_input_data: &[u8]) -> Input {
    let point =
        <bandersnatch::BandersnatchSha512Ell2 as ark_ec_vrfs::Suite>::data_to_point(vrf_input_data)
            .unwrap();
    Input::from(point)
}

// Prover actor.
struct Prover {
    pub prover_idx: usize,
    pub secret: Secret,
    pub ring: Vec<Public>,
}

impl Prover {
    pub fn new(ring: Vec<Public>, prover_idx: usize) -> Self {
        Self {
            prover_idx,
            secret: Secret::from_seed(&prover_idx.to_le_bytes()),
            ring,
        }
    }

    /// Anonymous VRF signature.
    ///
    /// Used for tickets submission.
    pub fn ring_vrf_sign(&self, vrf_input_data: &[u8], aux_data: &[u8]) -> Vec<u8> {
        use ark_ec_vrfs::ring::Prover as _;

        let input = vrf_input_point(vrf_input_data);
        let output = self.secret.output(input);

        // Backend currently requires the wrapped type (plain affine points)
        let pts: Vec<_> = self.ring.iter().map(|pk| pk.0).collect();

        // Proof construction
        let ring_ctx = ring_context();
        let prover_key = ring_ctx.prover_key(&pts);
        let prover = ring_ctx.prover(prover_key, self.prover_idx);
        let proof = self.secret.prove(input, output, aux_data, &prover);

        // Output and Ring Proof bundled together (as per section 2.2)
        let signature = RingVrfSignature { output, proof };
        let mut buf = Vec::new();
        signature.serialize_compressed(&mut buf).unwrap();
        buf
    }

    /// Non-Anonymous VRF signature.
    ///
    /// Used for ticket claiming during block production.
    /// Not used with Safrole test vectors.
    pub fn ietf_vrf_sign(&self, vrf_input_data: &[u8], aux_data: &[u8]) -> Vec<u8> {
        use ark_ec_vrfs::ietf::Prover as _;

        let input = vrf_input_point(vrf_input_data);
        let output = self.secret.output(input);

        let proof = self.secret.prove(input, output, aux_data);

        // Output and IETF Proof bundled together (as per section 2.2)
        let signature = IetfVrfSignature { output, proof };
        let mut buf = Vec::new();
        signature.serialize_compressed(&mut buf).unwrap();
        buf
    }
}

type RingCommitment = ark_ec_vrfs::ring::RingCommitment<bandersnatch::BandersnatchSha512Ell2>;

// Verifier actor.
struct Verifier {
    pub commitment: RingCommitment,
    pub ring: Vec<Public>,
}

impl Verifier {
    pub fn new(ring: Vec<Public>) -> Self {
        // Backend currently requires the wrapped type (plain affine points)
        let pts: Vec<_> = ring.iter().map(|pk| pk.0).collect();
        let verifier_key = ring_context().verifier_key(&pts);
        let commitment = verifier_key.commitment();
        Self { ring, commitment }
    }

    /// Anonymous VRF signature verification.
    ///
    /// Used for tickets verification.
    ///
    /// On success returns the VRF output hash.
    pub fn ring_vrf_verify(
        &self,
        vrf_input_data: &[u8],
        aux_data: &[u8],
        signature: &[u8],
    ) -> Result<[u8; 32], ()> {
        use ark_ec_vrfs::ring::Verifier as _;

        let signature = RingVrfSignature::deserialize_compressed(signature).unwrap();

        let input = vrf_input_point(vrf_input_data);
        let output = signature.output;

        let ring_ctx = ring_context();

        // The verifier key is reconstructed from the commitment and the constant
        // verifier key component of the SRS in order to verify some proof.
        // As an alternative we can construct the verifier key using the
        // RingContext::verifier_key() method, but is more expensive.
        // In other words, we prefer computing the commitment once, when the keyset changes.
        let verifier_key = ring_ctx.verifier_key_from_commitment(self.commitment.clone());
        let verifier = ring_ctx.verifier(verifier_key);
        if Public::verify(input, output, aux_data, &signature.proof, &verifier).is_err() {
            println!("Ring signature verification failure");
            return Err(());
        }
        println!("Ring signature verified");

        // This truncated hash is the actual value used as ticket-id/score in JAM
        let vrf_output_hash: [u8; 32] = output.hash()[..32].try_into().unwrap();
        println!(" vrf-output-hash: {}", hex::encode(vrf_output_hash));
        Ok(vrf_output_hash)
    }

    /// Non-Anonymous VRF signature verification.
    ///
    /// Used for ticket claim verification during block import.
    /// Not used with Safrole test vectors.
    ///
    /// On success returns the VRF output hash.
    pub fn ietf_vrf_verify(
        &self,
        vrf_input_data: &[u8],
        aux_data: &[u8],
        signature: &[u8],
        signer_key_index: usize,
    ) -> Result<[u8; 32], ()> {
        use ark_ec_vrfs::ietf::Verifier as _;

        let signature = IetfVrfSignature::deserialize_compressed(signature).unwrap();

        let input = vrf_input_point(vrf_input_data);
        let output = signature.output;

        let public = &self.ring[signer_key_index];
        if public
            .verify(input, output, aux_data, &signature.proof)
            .is_err()
        {
            println!("Ring signature verification failure");
            return Err(());
        }
        println!("Ietf signature verified");

        // This is the actual value used as ticket-id/score
        // NOTE: as far as vrf_input_data is the same, this matches the one produced
        // using the ring-vrf (regardless of aux_data).
        let vrf_output_hash: [u8; 32] = output.hash()[..32].try_into().unwrap();
        println!(" vrf-output-hash: {}", hex::encode(vrf_output_hash));
        Ok(vrf_output_hash)
    }
}

pub fn create_root_epoch(ring_set_hex: Vec<String>) -> String {

    let ring_set: Vec<Public> = ring_set_hex
        .iter()
        .map(|hex_str| {
            // Si la cadena comienza con "0x" o "0X", eliminamos esos dos caracteres
            let clean_hex = if hex_str.starts_with("0x") || hex_str.starts_with("0X") {
                &hex_str[2..]
            } else {
                hex_str
            };
            // Filtramos solo los caracteres hexadecimales válidos
            let clean_hex: String = clean_hex.chars().filter(|c| c.is_digit(16)).collect();
            let bytes = hex::decode(&clean_hex).expect("Decoding hex string failed");
            let point = bandersnatch::Public::deserialize_compressed(&bytes[..])
                .expect("Deserialization failed");
            point
        /*.map(|hex_str| {
            let bytes = hex::decode(hex_str).expect("Decoding hex string failed");
            let point = bandersnatch::Public::deserialize_compressed(&bytes[..])
                .expect("Deserialization failed");
            point*/
        })
        .collect();

    let verifier = Verifier::new(ring_set);
    let mut proof = vec![];
    verifier.commitment.serialize_compressed(&mut proof).unwrap();
    hex::encode(proof)
}

fn has_duplicates(id: &Vec<String>) -> bool {
    let mut seen = HashSet::new();
    for ticket in id {
        if !seen.insert(ticket) {
            // Si no se puede insertar significa que ya estaba en el HashSet
            return true; // Hay duplicados
        }
    }
    false // No hay duplicados
}

pub fn verify_tickets(input: InputSafrole, state: &mut SafroleState) -> OutputSafrole {

    for i in 0..input.extrinsic.len() {
        if input.extrinsic[i].attempt < 0 || input.extrinsic[i].attempt > 1 {
            return OutputSafrole::err(ErrorType::bad_ticket_attempt);
        }
    }
    let ring_keys = bandersnatch_keys_collect(state.clone(), KeySet::gamma_k);
    let ring_set: Vec<Public> = ring_keys
        .iter()
        .map(|key| {
            let bytes = hex::decode(&key[2..]).expect("Decoding hex string failed");
            let point = bandersnatch::Public::deserialize_compressed(&bytes[..])
                .expect("Deserialization failed");
            point
        })
        .collect();
    
    let verifier = Verifier::new(ring_set);
    let mut aux_gamma_a = state.gamma_a.clone();

    for i in 0..input.extrinsic.len() {
        let mut vrf_input_data = Vec::from(b"jam_ticket_seal");
        vrf_input_data.extend_from_slice(&hex::decode(&state.eta[2][2..]).expect("Decoding hex string failed"));
        vrf_input_data.push(input.extrinsic[i].attempt.try_into().unwrap());
        let aux_data = vec![];
        let signature_hex = hex::decode(&input.extrinsic[i].signature[2..]).expect("Decoding hex string failed");
        let res = verifier.ring_vrf_verify(&vrf_input_data, &aux_data, &signature_hex);
        match res {
            Ok(result) => {
                println!("VRF verification succeeded with result: {:?}", hex::encode(result));
                aux_gamma_a.push(TicketBody {
                    id: format!("0x{}", hex::encode(result)),
                    attempt: input.extrinsic[i].attempt,
                })
            },
            Err(_) => {
                println!("VRF verification failed");
                return OutputSafrole::err(ErrorType::bad_ticket_proof);
            }
        }
    }
    let ids: Vec<String> = aux_gamma_a.iter().map(|ticket| ticket.id.clone()).collect();
    if has_duplicates(&ids) {
        return OutputSafrole::err(ErrorType::duplicate_ticket);
    }
    OutputSafrole::ok(OutputMarks {
        epoch_mark: None,
        tickets_mark: None,
    })
}

