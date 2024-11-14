use crate::types::{OpaqueHash, ValidatorIndex, Ed25519Signature, Ed25519Key};
use crate::constants::{VALIDATORS_SUPER_MAJORITY};
use crate::codec::{Encode, EncodeSize, Decode, BytesReader, ReadError};
use crate::codec::{encode_unsigned, decode_unsigned};

// Judgement statements come about naturally as part of the auditing process and are expected to be positive,
// further affirming the guarantors’ assertion that the workreport is valid. In the event of a negative judgment, 
// then all validators audit said work-report and we assume a verdict will be reached.

struct Judgement {
    vote: bool,
    index: ValidatorIndex,
    signature: Ed25519Signature,
}

impl Judgement {

    fn decode_len(judgement_blob: &mut BytesReader) -> Result<Vec<Self>, ReadError> {
        
        let mut votes: Vec<Judgement> = Vec::with_capacity(VALIDATORS_SUPER_MAJORITY);
        
        for _ in 0..VALIDATORS_SUPER_MAJORITY {
            let vote: bool = judgement_blob.read_byte()? != 0;
            let index = ValidatorIndex::decode(judgement_blob)?;
            let signature = Ed25519Signature::decode(judgement_blob)?; 
            votes.push(Judgement{vote, index, signature});
        }
        
        Ok(votes)
    }

    fn encode_len(judgments: &[Judgement]) -> Vec<u8> {
        
        let mut judgement_blob: Vec<u8> = Vec::new();
        
        for judgment in judgments.iter().take(VALIDATORS_SUPER_MAJORITY) {
            judgement_blob.push(judgment.vote as u8);
            judgment.index.encode_size(2).encode_to(&mut judgement_blob);
            judgment.signature.encode_to(&mut judgement_blob);
        }
        
        
        return judgement_blob;
    }
}

// A Verdict is a compilation of judgments coming from exactly two-thirds plus one of either the active validator set 
// or the previous epoch’s validator set, i.e. the Ed25519 keys of κ or λ. Verdicts contains only the report hash and 
// the sum of positive judgments. We require this total to be either exactly two-thirds-plus-one, zero or one-third 
// of the validator set indicating, respectively, that the report is good, that it’s bad, or that it’s wonky.

struct Verdict {
    target: OpaqueHash,
    age: u32,
    votes: Vec<Judgement>,
}

impl Verdict {

    fn decode_len(verdict_blob: &mut BytesReader) -> Result<Vec<Self>, ReadError> {

        let num_verdicts = decode_unsigned(verdict_blob)?;
        let mut verdicts: Vec<Verdict> = Vec::with_capacity(num_verdicts);

        for _ in 0..num_verdicts {
            let target = OpaqueHash::decode(verdict_blob)?;
            let age = u32::decode(verdict_blob)?;
            let votes = Judgement::decode_len(verdict_blob)?;
            verdicts.push(Verdict {target, age, votes});
        }

        Ok(verdicts)
    }

    fn encode_len(verdicts: &[Verdict]) -> Vec<u8> {

        let mut verdicts_blob: Vec<u8> = Vec::new();
        encode_unsigned(verdicts.len()).encode_to(&mut verdicts_blob);

        for verdict in verdicts {
            verdict.target.encode_to(&mut verdicts_blob);
            verdict.age.encode_size(4).encode_to(&mut verdicts_blob);
            Judgement::encode_len(&verdict.votes).encode_to(&mut verdicts_blob);
        }

        return verdicts_blob;
    }
}

// A culprit is a proofs of the misbehavior of one or more validators by guaranteeing a work-report found to be invalid.
// Is a offender signature.

struct Culprit {
    target: OpaqueHash,
    key: Ed25519Key,
    signature: Ed25519Signature,
}

impl Encode for Culprit {

    fn encode(&self) -> Vec<u8> {
        
        let mut culprit_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<Culprit>());

        self.target.encode_to(&mut culprit_blob);
        self.key.encode_to(&mut culprit_blob);
        self.signature.encode_to(&mut culprit_blob);
        
        return culprit_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode()); 
    }
}

impl Decode for Culprit {

    fn decode(culprit_blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(Culprit {
            target: OpaqueHash::decode(culprit_blob)?,
            key: Ed25519Key::decode(culprit_blob)?,
            signature : Ed25519Signature::decode(culprit_blob)?,
        })
    }
}

impl Culprit {

    fn encode_len(culprits: &[Culprit]) -> Vec<u8> {
        
        let mut culprits_blob: Vec<u8> = Vec::with_capacity(culprits.len() * std::mem::size_of::<Fault>());
        encode_unsigned(culprits.len()).encode_to(&mut culprits_blob); 
        
        for culprit in culprits {
            culprit.encode_to(&mut culprits_blob);
        }
        
        return culprits_blob;
    }

    fn decode_len(culprits_blob: &mut BytesReader) -> Result<Vec<Self>, ReadError> {

        let num_culprits = decode_unsigned(culprits_blob)?; 
        let mut culprits: Vec<Culprit> = Vec::with_capacity(num_culprits);

        for _ in 0..num_culprits {
            culprits.push(Culprit::decode(culprits_blob)?);
        }

        Ok(culprits)
    }
}

// A fault is a proofs of the misbehavior of one or more validators by signing a judgment found to be contradiction to a 
// work-report’s validity. Is a offender signature. Must be ordered by validators Ed25519Key. There may be no duplicate
// report hashes within the extrinsic, nor amongst any past reported hashes.

struct Fault {
    target: OpaqueHash,
    vote: bool,
    key: Ed25519Key,
    signature: Ed25519Signature,
}

impl Encode for Fault {

    fn encode(&self) -> Vec<u8> {

        let mut fault_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<Fault>());
        
        self.target.encode_to(&mut fault_blob);
        self.vote.encode_to(&mut fault_blob);
        self.key.encode_to(&mut fault_blob);
        self.signature.encode_to(&mut fault_blob);

        return fault_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());       
    }
}

impl Decode for Fault {

    fn decode(fault_blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(Fault {
            target : OpaqueHash::decode(fault_blob)?,
            vote : fault_blob.read_byte()? != 0,
            key : OpaqueHash::decode(fault_blob)?,
            signature : Ed25519Signature::decode(fault_blob)?,
        })
    }
}

impl Fault {

    fn encode_len(faults: &[Fault]) -> Vec<u8> {

        let faults_len = faults.len();
        let mut faults_blob: Vec<u8> = Vec::with_capacity(faults_len * std::mem::size_of::<Fault>());
        encode_unsigned(faults_len).encode_to(&mut faults_blob);

        for fault in faults {
            fault.encode_to(&mut faults_blob);
        }

        return faults_blob;
    }

    fn decode_len(faults_blob: &mut BytesReader) -> Result<Vec<Self>, ReadError> {

        let num_faults = decode_unsigned(faults_blob)?;
        let mut faults: Vec<Fault> = Vec::with_capacity(num_faults);

        for _ in 0..num_faults {
            faults.push(Fault::decode(faults_blob)?);
        }

        Ok(faults)
    }
   
}

// The disputes extrinsic may contain one or more verdicts v as a compilation of judgments coming from exactly 
// two-thirds plus one of either the active validator set or the previous epoch’s validator set, i.e. the Ed25519 
// keys of κ or λ. Additionally, it may contain proofs of the misbehavior of one or more validators, either by 
// guaranteeing a work-report found to be invalid (culprits), or by signing a judgment found to be contradiction 
// to a work-report’s validity (faults). Both are considered a kind of offense.

pub struct DisputesExtrinsic {
    verdicts: Vec<Verdict>,
    culprits: Vec<Culprit>,
    faults: Vec<Fault>,
}

impl Encode for DisputesExtrinsic {

    fn encode(&self) -> Vec<u8> {

        let mut dispute_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<DisputesExtrinsic>());
        
        Verdict::encode_len(&self.verdicts).encode_to(&mut dispute_blob);
        Culprit::encode_len(&self.culprits).encode_to(&mut dispute_blob);
        Fault::encode_len(&self.faults).encode_to(&mut dispute_blob);
        
        return dispute_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode()); 
    }
}

impl Decode for DisputesExtrinsic {

    fn decode(dispute_blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(DisputesExtrinsic {
            verdicts : Verdict::decode_len(dispute_blob)?,
            culprits : Culprit::decode_len(dispute_blob)?,
            faults : Fault::decode_len(dispute_blob)?,
        })
    }
}
