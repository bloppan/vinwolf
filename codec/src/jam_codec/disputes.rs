use jam_types::{
    Judgement, ValidatorIndex, Ed25519Signature, Ed25519Public, WorkReportHash, OffendersMark, OutputDataDisputes, DisputesRecords, Fault, Culprit, Verdict,
    OpaqueHash, 
};
use constants::node::VALIDATORS_SUPER_MAJORITY;
use crate::{BytesReader, Decode, DecodeLen, Encode, EncodeLen, EncodeSize, ReadError};

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



