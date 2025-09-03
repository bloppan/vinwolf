use jam_types::{ServiceId, Preimage, PreimagesErrorCode, OutputPreimages};
use crate::{Encode, EncodeLen, EncodeSize, Decode, DecodeLen, BytesReader, ReadError};

// Preimages are static data which is presently being requested to be available for workloads to be able to 
// fetch on demand. Prior to accumulation, we must first integrate all preimages provided in the lookup extrinsic. 
// The lookup extrinsic is a sequence of pairs of service indices and data. These pairs must be ordered and without 
// duplicates. The data must have been solicited by a service but not yet be provided.


impl Encode for Preimage {

    fn encode(&self) -> Vec<u8> {
        
        let mut blob = Vec::with_capacity(std::mem::size_of::<Self>());

        self.requester.encode_size(4).encode_to(&mut blob);
        self.blob.encode_len().encode_to(&mut blob);

        return blob;
    }

    fn encode_to(&self, writer: &mut Vec<u8>) {
        writer.extend_from_slice(&self.encode());
    }
}

impl Decode for Preimage {

    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(Self { requester: ServiceId::decode(reader)?, blob: Vec::<u8>::decode_len(reader)? })
    }
}

impl Encode for PreimagesErrorCode {
    fn encode(&self) -> Vec<u8> {
        let mut blob = Vec::new();
        match self {
            PreimagesErrorCode::PreimageUnneeded => {
                blob.push(0);
            },
            PreimagesErrorCode::PreimagesNotSortedOrUnique => {
                blob.push(1);
            },
            PreimagesErrorCode::RequesterNotFound => {
                blob.push(2);
            },
        }
        return blob;
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for PreimagesErrorCode {
    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        match reader.read_byte()? {
            0 => Ok(PreimagesErrorCode::PreimageUnneeded),
            1 => Ok(PreimagesErrorCode::PreimagesNotSortedOrUnique),
            2 => Ok(PreimagesErrorCode::RequesterNotFound),
            _ => Err(ReadError::InvalidData),
        }
    }
}

impl Encode for OutputPreimages {
    fn encode(&self) -> Vec<u8> {
        let mut blob = Vec::new();
        match self {
            OutputPreimages::Ok() => {
                blob.push(0);
            }
            OutputPreimages::Err(code) => {
                blob.push(1);
                code.encode_to(&mut blob);
            }
        }
        return blob;
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for OutputPreimages {
    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        match reader.read_byte()? {
            0 => Ok(OutputPreimages::Ok()),
            1 => Ok(OutputPreimages::Err(PreimagesErrorCode::decode(reader)?)),
            _ => Err(ReadError::InvalidData),
        }
    }
}