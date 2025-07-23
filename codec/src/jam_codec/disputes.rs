use jam_types::{Judgement, ValidatorIndex, Ed25519Signature, Ed25519Public, WorkReportHash, OffendersMark, OutputDataDisputes, DisputesRecords};
use constants::node::VALIDATORS_SUPER_MAJORITY;
use crate::{BytesReader, Decode, DecodeLen, Encode, EncodeLen, EncodeSize, ReadError};

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

