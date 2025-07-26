use jam_types::{OpaqueHash, Ed25519Signature, ValidatorIndex, Assurance, WorkReport, OutputDataAssurances, OutputAssurances, AssurancesErrorCode};
use crate::{Encode, EncodeLen, EncodeSize, Decode, DecodeLen, BytesReader, ReadError};
use constants::node::AVAIL_BITFIELD_BYTES;

impl Encode for Assurance {
    
    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::new();

        self.anchor.encode_to(&mut blob);
        self.bitfield.encode_to(&mut blob);
        self.validator_index.encode_size(2).encode_to(&mut blob);
        self.signature.encode_to(&mut blob);

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode()); 
    }
}

impl Decode for Assurance {

    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(Assurance { 
            anchor: OpaqueHash::decode(reader)?, 
            bitfield: <[u8; AVAIL_BITFIELD_BYTES]>::decode(reader)?, 
            validator_index: ValidatorIndex::decode(reader)?, 
            signature: Ed25519Signature::decode(reader)?
        })
    }
}

impl Encode for OutputDataAssurances {
    
    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::with_capacity(std::mem::size_of::<WorkReport>() * self.reported.len());
        self.reported.encode_len().encode_to(&mut blob);

        return blob;
    }

    fn encode_to(&self, writer: &mut Vec<u8>) {
        writer.extend_from_slice(&self.encode());
    }
}

impl Decode for OutputDataAssurances {
    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(OutputDataAssurances {
            reported: Vec::<WorkReport>::decode_len(reader)?,
        })
    }
}

impl Encode for OutputAssurances {

    fn encode(&self) -> Vec<u8> {
        let mut output = Vec::with_capacity(std::mem::size_of::<OutputAssurances>());
        match self {
            OutputAssurances::Ok(data) => {
                output.push(0);
                data.encode_to(&mut output);
            },
            OutputAssurances::Err(code) => {
                output.push(1);
                output.push(*code as u8);
            }
        }
        return output;
    }
    fn encode_to(&self, writer: &mut Vec<u8>) {
        writer.extend_from_slice(&self.encode());
    }
}

impl Decode for OutputAssurances {
    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        let result = reader.read_byte()?;
        match result {
            0 => Ok(OutputAssurances::Ok(OutputDataAssurances::decode(reader)?)),
            1 => {
                let code = reader.read_byte()?;
                let error_code = match code {
                    0 => AssurancesErrorCode::BadAttestationParent,
                    1 => AssurancesErrorCode::BadValidatorIndex,
                    2 => AssurancesErrorCode::CoreNotEngaged,
                    3 => AssurancesErrorCode::BadSignature,
                    4 => AssurancesErrorCode::NotSortedOrUniqueAssurers,
                    5 => AssurancesErrorCode::TooManyAssurances,
                    6 => AssurancesErrorCode::WrongBitfieldLength,
                    _ => return Err(ReadError::InvalidData),
                };
                Ok(OutputAssurances::Err(error_code))
            },
            _ => Err(ReadError::InvalidData)
        }
    }
}
