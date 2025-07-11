
use ark_vrf::reexports::ark_serialize::{self, CanonicalDeserialize, CanonicalSerialize};
use ark_vrf::suites::bandersnatch;
use bandersnatch::{BandersnatchSha512Ell2, IetfProof, Input, Output, Public, RingProof, RingProofParams, Secret};

use crate::constants::VALIDATORS_COUNT;
use crate::types::{ProcessError, SafroleErrorCode};
const RING_SIZE: usize = VALIDATORS_COUNT;


// This is the IETF `Prove` procedure output as described in section 2.2
// of the Bandersnatch VRF specification
#[derive(CanonicalSerialize, CanonicalDeserialize)]
struct IetfVrfSignature {
    output: Output,
    proof: IetfProof,
}

// This is the IETF `Prove` procedure output as described in section 4.2
// of the Bandersnatch VRF specification
#[derive(CanonicalSerialize, CanonicalDeserialize)]
struct RingVrfSignature {
    output: Output,
    // This contains both the Pedersen proof and actual ring proof.
    proof: RingProof,
}

// "Static" ring proof parameters.
fn ring_proof_params() -> &'static RingProofParams {
    use std::sync::OnceLock;
    static PARAMS: OnceLock<RingProofParams> = OnceLock::new();
    PARAMS.get_or_init(|| {
        use bandersnatch::PcsParams;
        let file_data: &[u8] = include_bytes!("../../../../tests/test_vectors/w3f/jamtestvectors/safrole/zcash-srs-2-11-uncompressed.bin");
        let pcs_params = PcsParams::deserialize_uncompressed_unchecked(&mut &file_data[..]).unwrap();
        RingProofParams::from_pcs_params(RING_SIZE, pcs_params).unwrap()
    })
}

// Construct VRF Input Point from arbitrary data (section 1.2)
fn vrf_input_point(vrf_input_data: &[u8]) -> Input {
    Input::new(vrf_input_data).unwrap()
}

#[allow(dead_code)]
// Prover actor.
struct Prover {
    pub prover_idx: usize,
    pub secret: Secret,
    pub ring: Vec<Public>,
}

impl Prover {
    #[allow(dead_code)]
    pub fn new(ring: Vec<Public>, prover_idx: usize) -> Self {
        Self {
            prover_idx,
            secret: Secret::from_seed(&prover_idx.to_le_bytes()),
            ring,
        }
    }

    /// VRF output hash.
    #[allow(dead_code)]
    pub fn vrf_output(&self, vrf_input_data: &[u8]) -> Vec<u8> {
        let input = vrf_input_point(vrf_input_data);
        let output = self.secret.output(input);
        output.hash()[..32].try_into().unwrap()
    }

    /// Anonymous VRF signature.
    ///
    /// Used for tickets submission.
    #[allow(dead_code)]
    pub fn ring_vrf_sign(&self, vrf_input_data: &[u8], aux_data: &[u8]) -> Vec<u8> {
        use ark_vrf::ring::Prover as _;

        let input = vrf_input_point(vrf_input_data);
        let output = self.secret.output(input);

        // Backend currently requires the wrapped type (plain affine points)
        let pts: Vec<_> = self.ring.iter().map(|pk| pk.0).collect();

        // Proof construction
        let params = ring_proof_params();
        let prover_key = params.prover_key(&pts);
        let prover = params.prover(prover_key, self.prover_idx);
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
    #[allow(dead_code)]
    pub fn ietf_vrf_sign(&self, vrf_input_data: &[u8], aux_data: &[u8]) -> Vec<u8> {
        use ark_vrf::ietf::Prover as _;

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

type RingCommitment = ark_vrf::ring::RingCommitment<BandersnatchSha512Ell2>;

// Verifier actor.
pub struct Verifier {
    pub commitment: RingCommitment,
    pub ring: Vec<Public>,
}

impl Verifier {
    pub fn new(ring: Vec<Public>) -> Self {
        // Backend currently requires the wrapped type (plain affine points)
        let pts: Vec<_> = ring.iter().map(|pk| pk.0).collect();
        let verifier_key = ring_proof_params().verifier_key(&pts);
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
    ) -> Result<[u8; 32], ProcessError> {
        use ark_vrf::ring::Verifier as _;

        let signature = RingVrfSignature::deserialize_compressed(signature).map_err(|_| ProcessError::SafroleError(SafroleErrorCode::InvalidRingVrfSignature))?;

        let input = vrf_input_point(vrf_input_data);
        let output = signature.output;

        let params = ring_proof_params();

        // The verifier key is reconstructed from the commitment and the constant
        // verifier key component of the SRS in order to verify some proof.
        // As an alternative we can construct the verifier key using the
        // RingProofParams::verifier_key() method, but is more expensive.
        // In other words, we prefer computing the commitment once, when the keyset changes.
        let verifier_key = params.verifier_key_from_commitment(self.commitment.clone());
        let verifier = params.verifier(verifier_key);
        if Public::verify(input, output, aux_data, &signature.proof, &verifier).is_err() {
            log::debug!("Ring signature verification failure");
            return Err(ProcessError::SafroleError(SafroleErrorCode::RingSignatureVerificationFail));
        }
        //println!("Ring signature verified");

        // This truncated hash is the actual value used as ticket-id/score in JAM
        let vrf_output_hash: [u8; 32] = output.hash()[..32].try_into().unwrap();
        //println!(" vrf-output-hash: {}", hex::encode(vrf_output_hash));
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
    ) -> Result<[u8; 32], ProcessError> {
        use ark_vrf::ietf::Verifier as _;

        //println!("signer_key index: {signer_key_index}, ring len: {:?}", self.ring.len());
        if signer_key_index >= self.ring.len()  {
            return Err(ProcessError::SafroleError(SafroleErrorCode::InvalidSignerKeyIndex));
        }
        
        let signature = IetfVrfSignature::deserialize_compressed(signature)
                                                    .map_err(|_| ProcessError::SafroleError(SafroleErrorCode::InvalidIetffSignature))?;

        let input = vrf_input_point(vrf_input_data);
        let output = signature.output;

        let public = &self.ring[signer_key_index];
        if public
            .verify(input, output, aux_data, &signature.proof)
            .is_err()
        {
            println!("Ring signature verification failure");
            return Err(ProcessError::SafroleError(SafroleErrorCode::IetfSignatureVerificationFail));
        }
        //println!("Ietf signature verified");

        // This is the actual value used as ticket-id/score
        // NOTE: as far as vrf_input_data is the same, this matches the one produced
        // using the ring-vrf (regardless of aux_data).
        let vrf_output_hash: [u8; 32] = output.hash()[..32].try_into().unwrap();
        //println!(" vrf-output-hash: {}", hex::encode(vrf_output_hash));
        Ok(vrf_output_hash)
    }
}

#[allow(unused_macros)]
macro_rules! measure_time {
    ($func_name:expr, $func_call:expr) => {{
        let start = std::time::Instant::now();
        let result = $func_call;
        let duration = start.elapsed();
        println!("* Time taken by {}: {:?}", $func_name, duration);
        result
    }};
}

