use crate::{Encode, EncodeLen, EncodeSize, Decode, DecodeLen, BytesReader, ReadError};
use jam_types::{TimeSlot, WorkReport, ValidatorIndex, Ed25519Signature, Guarantee, ValidatorSignature};

impl Encode for Guarantee {

    fn encode(&self) -> Vec<u8> {
        
        let mut blob = Vec::new();

        self.report.encode_to(&mut blob);
        self.slot.encode_size(4).encode_to(&mut blob);
        self.signatures.encode_len().encode_to(&mut blob);

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for Guarantee {

    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(Guarantee { 
            report: WorkReport::decode(reader)?, 
            slot: TimeSlot::decode(reader)?, 
            signatures: Vec::<ValidatorSignature>::decode_len(reader)?,
        })
    }
}

impl Encode for ValidatorSignature {

    fn encode(&self) -> Vec<u8> {
        
        let mut blob = Vec::new();

        self.validator_index.encode_size(2).encode_to(&mut blob);
        self.signature.encode_to(&mut blob);

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for ValidatorSignature {

    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(ValidatorSignature { 
            validator_index: ValidatorIndex::decode(reader)?, 
            signature: Ed25519Signature::decode(reader)?, 
        })
    }
}


