use crate::types::{
    OpaqueHash, ValidatorIndex, Ed25519Signature, Ed25519Public, WorkReportHash, OffendersMark, OutputDataDisputes, 
    DisputesRecords, DisputesExtrinsic, Verdict, Judgement, Culprit, Fault
};
use crate::constants::VALIDATORS_SUPER_MAJORITY;
use crate::utils::codec::{BytesReader, Decode, DecodeLen, Encode, EncodeLen, EncodeSize, ReadError};

impl Encode for DisputesRecords {

    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::with_capacity(std::mem::size_of::<Self>());

        self.good.encode_len().encode_to(&mut blob);
        self.bad.encode_len().encode_to(&mut blob);
        self.wonky.encode_len().encode_to(&mut blob);
        self.offenders.encode_len().encode_to(&mut blob);

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for DisputesRecords {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(DisputesRecords {
            good: Vec::<WorkReportHash>::decode_len(blob)?,
            bad: Vec::<WorkReportHash>::decode_len(blob)?,
            wonky: Vec::<WorkReportHash>::decode_len(blob)?,
            offenders: Vec::<Ed25519Public>::decode_len(blob)?,
        })
    }
}

impl Encode for DisputesExtrinsic {

    fn encode(&self) -> Vec<u8> {

        let mut dispute_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<DisputesExtrinsic>());
        
        self.verdicts.encode_len().encode_to(&mut dispute_blob);
        self.culprits.encode_len().encode_to(&mut dispute_blob);
        self.faults.encode_len().encode_to(&mut dispute_blob);

        return dispute_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode()); 
    }
}

impl Decode for DisputesExtrinsic {

    fn decode(dispute_blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(DisputesExtrinsic {
            verdicts : Vec::<Verdict>::decode_len(dispute_blob)?,
            culprits : Vec::<Culprit>::decode_len(dispute_blob)?,
            faults : Vec::<Fault>::decode_len(dispute_blob)?,
        })
    }
}

impl Encode for OutputDataDisputes {

    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::with_capacity(std::mem::size_of::<OffendersMark>() * self.offenders_mark.len());
        self.offenders_mark.encode_len().encode_to(&mut blob);

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for OutputDataDisputes {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(OutputDataDisputes {
            offenders_mark: Vec::<Ed25519Public>::decode_len(blob)?,
        })
    }
}

// Judgement statements come about naturally as part of the auditing process and are expected to be positive,
// further affirming the guarantors’ assertion that the workreport is valid. In the event of a negative judgment, 
// then all validators audit said work-report and we assume a verdict will be reached.

impl Encode for Vec<Judgement> {

    fn encode(&self) -> Vec<u8> {

        let mut judgement_blob: Vec<u8> = Vec::new();
        
        for judgment in self.iter() {
            judgement_blob.push(judgment.vote as u8);
            judgment.index.encode_size(2).encode_to(&mut judgement_blob);
            judgment.signature.encode_to(&mut judgement_blob);
        }
        
        return judgement_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for Vec<Judgement> {

    fn decode(judgement_blob: &mut BytesReader) -> Result<Self, ReadError> {

        let mut votes: Vec<Judgement> = Vec::with_capacity(VALIDATORS_SUPER_MAJORITY);
        
        for _ in 0..VALIDATORS_SUPER_MAJORITY {
            let vote: bool = judgement_blob.read_byte()? != 0;
            let index = ValidatorIndex::decode(judgement_blob)?;
            let signature = Ed25519Signature::decode(judgement_blob)?; 
            votes.push(Judgement{vote, index, signature});
        }
        
        Ok(votes)
    }
}

// A Verdict is a compilation of judgments coming from exactly two-thirds plus one of either the active validator set 
// or the previous epoch’s validator set, i.e. the Ed25519 keys of κ or λ. Verdicts contains only the report hash and 
// the sum of positive judgments. We require this total to be either exactly two-thirds-plus-one, zero or one-third 
// of the validator set indicating, respectively, that the report is good, that it’s bad, or that it’s wonky.
impl Encode for Verdict {

    fn encode(&self) -> Vec<u8> {

        let mut verdicts_blob: Vec<u8> = Vec::new();

        self.target.encode_to(&mut verdicts_blob);
        self.age.encode_size(4).encode_to(&mut verdicts_blob);
        Vec::<Judgement>::encode(&self.votes).encode_to(&mut verdicts_blob);

        return verdicts_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for Verdict {

    fn decode(verdict_blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(Verdict{
            target: OpaqueHash::decode(verdict_blob)?,
            age: u32::decode(verdict_blob)?,
            votes: Vec::<Judgement>::decode(verdict_blob)?,
        })       
    }
}

// A culprit is a proofs of the misbehavior of one or more validators by guaranteeing a work-report found to be invalid.
// Is a offender signature.
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
            key: Ed25519Public::decode(culprit_blob)?,
            signature: Ed25519Signature::decode(culprit_blob)?,
        })
    }
}

// A fault is a proofs of the misbehavior of one or more validators by signing a judgment found to be contradiction to a 
// work-report’s validity. Is a offender signature. Must be ordered by validators Ed25519Key. There may be no duplicate
// report hashes within the extrinsic, nor amongst any past reported hashes.
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
            target: OpaqueHash::decode(fault_blob)?,
            vote: fault_blob.read_byte()? != 0,
            key: OpaqueHash::decode(fault_blob)?,
            signature: Ed25519Signature::decode(fault_blob)?,
        })
    }
}
