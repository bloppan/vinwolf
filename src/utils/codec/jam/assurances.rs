use crate::types::{
    OpaqueHash, Ed25519Signature, ValidatorIndex, AssurancesExtrinsic, AvailAssurance, WorkReport, 
    OutputDataAssurances, OutputAssurances, AssurancesErrorCode
};
use crate::constants::AVAIL_BITFIELD_BYTES;
use crate::utils::codec::{Encode, EncodeSize, Decode, BytesReader, ReadError};
use crate::utils::codec::generic::{encode_unsigned, decode_unsigned};

impl Encode for AssurancesExtrinsic {
    
    fn encode(&self) -> Vec<u8> {

        let mut assurances_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<AssurancesExtrinsic>() * self.assurances.len());
        encode_unsigned(self.assurances.len()).encode_to(&mut assurances_blob);

        for assurance in &self.assurances {
            assurance.anchor.encode_to(&mut assurances_blob);
            assurance.bitfield.encode_to(&mut assurances_blob);
            assurance.validator_index.encode_size(2).encode_to(&mut assurances_blob);
            assurance.signature.encode_to(&mut assurances_blob);
        }

        return assurances_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode()); 
    }
}

impl Decode for AssurancesExtrinsic {

    fn decode(assurances_blob: &mut BytesReader) -> Result<Self, ReadError> {

        let num_assurances = decode_unsigned(assurances_blob)?;
        let mut assurances = Vec::with_capacity(num_assurances);  

        for _ in 0..num_assurances {
            assurances.push(AvailAssurance {
                anchor: OpaqueHash::decode(assurances_blob)?,
                bitfield: <[u8; AVAIL_BITFIELD_BYTES]>::decode(assurances_blob)?,
                validator_index: ValidatorIndex::decode(assurances_blob)?,
                signature: Ed25519Signature::decode(assurances_blob)?,
            });
        }
        
        Ok(AssurancesExtrinsic { assurances })
    }
}

impl Encode for OutputDataAssurances {
    fn encode(&self) -> Vec<u8> {
        let mut output = Vec::with_capacity(std::mem::size_of::<WorkReport>() * self.reported.len());
        
        encode_unsigned(self.reported.len()).encode_to(&mut output);
        
        for report in &self.reported {
            report.encode_to(&mut output);
        }

        return output;
    }
    fn encode_to(&self, writer: &mut Vec<u8>) {
        writer.extend_from_slice(&self.encode());
    }
}

impl Decode for OutputDataAssurances {
    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(OutputDataAssurances {
            reported: {
                let len = decode_unsigned(reader)?;
                let mut reported = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    reported.push(WorkReport::decode(reader)?);
                }
                reported
            }
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
