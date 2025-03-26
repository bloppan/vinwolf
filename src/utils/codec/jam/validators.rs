use crate::types::{BandersnatchPublic, Ed25519Public, BlsPublic, Metadata, ValidatorData, ValidatorsData};
use crate::utils::codec::{Encode, Decode, BytesReader, ReadError};

impl Encode for ValidatorData {
    
    fn encode(&self) -> Vec<u8> {

        let mut validator_data: Vec<u8> = Vec::with_capacity(std::mem::size_of::<ValidatorData>());
        
        self.bandersnatch.encode_to(&mut validator_data);
        self.ed25519.encode_to(&mut validator_data);
        self.bls.encode_to(&mut validator_data);
        self.metadata.encode_to(&mut validator_data);

        return validator_data;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for ValidatorData {

    fn decode(data_blob: &mut BytesReader) -> Result<Self, ReadError> {
    
        Ok(ValidatorData {
            bandersnatch: BandersnatchPublic::decode(data_blob)?,
            ed25519: Ed25519Public::decode(data_blob)?,
            bls: BlsPublic::decode(data_blob)?,
            metadata: Metadata::decode(data_blob)?,
        })
    }
}

impl Encode for ValidatorsData {

    fn encode(&self) -> Vec<u8> {

        let mut validators_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<Self>());

        for validator in self.0.iter() {
            validator.encode_to(&mut validators_blob);
        }

        return validators_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for ValidatorsData {

    fn decode(validators_blob: &mut BytesReader) -> Result<Self, ReadError> {

        let mut validators: ValidatorsData = ValidatorsData::default();

        for validator in validators.0.iter_mut() {
            *validator = ValidatorData::decode(validators_blob)?;
        }
        
        Ok(validators)
    }
}
