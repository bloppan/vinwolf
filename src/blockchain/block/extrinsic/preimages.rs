use crate::types::ServiceId;
use crate::utils::codec::{Encode, EncodeLen, Decode, DecodeLen, BytesReader, ReadError};
use crate::utils::codec::{encode_unsigned, decode_unsigned};

// Preimages are static data which is presently being requested to be available for workloads to be able to 
// fetch on demand. Prior to accumulation, we must first integrate all preimages provided in the lookup extrinsic. 
// The lookup extrinsic is a sequence of pairs of service indices and data. These pairs must be ordered and without 
// duplicates. The data must have been solicited by a service but not yet be provided.
#[derive(Debug)]
pub struct PreimagesExtrinsic {
    preimages: Vec<Preimage>,
}

#[derive(Debug)]
struct Preimage {
    requester: ServiceId,
    blob: Vec<u8>,
}

impl Decode for PreimagesExtrinsic {

    fn decode(preimage_blob: &mut BytesReader) -> Result<Self, ReadError> {

        let num_preimages = decode_unsigned(preimage_blob)?;
        let mut preimg_extrinsic: Vec<Preimage> = Vec::with_capacity(num_preimages);

        for _ in 0..num_preimages {
            let requester = ServiceId::decode(preimage_blob)?;
            let blob = Vec::<u8>::decode_len(preimage_blob)?;
            preimg_extrinsic.push(Preimage { requester, blob });
        }

        Ok(PreimagesExtrinsic { preimages: preimg_extrinsic })
    }
}

impl Encode for PreimagesExtrinsic {

    fn encode(&self) -> Vec<u8> {

        let mut preimg_encoded = Vec::with_capacity(std::mem::size_of::<PreimagesExtrinsic>());
        encode_unsigned(self.preimages.len()).encode_to(&mut preimg_encoded);
        
        for preimage in &self.preimages {
            preimage.requester.encode_to(&mut preimg_encoded);
            preimage.blob.as_slice().encode_len().encode_to(&mut preimg_encoded);
        }
        
        return preimg_encoded;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode()); 
    }
}