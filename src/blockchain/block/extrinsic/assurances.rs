use crate::types::{OpaqueHash, Ed25519Signature, ValidatorIndex, AssurancesExtrinsic, AvailAssurance};
use crate::constants::AVAIL_BITFIELD_BYTES;
use crate::utils::codec::{Encode, EncodeSize, Decode, BytesReader, ReadError};
use crate::utils::codec::{encode_unsigned, decode_unsigned};

// The assurances extrinsic are the input data of workloads they have correctly received and are storing locally.
// The assurances extrinsic is a sequence of assurance values, at most one per validator. Each assurance is a 
// sequence of binary values (i.e. a bitstring), one per core, together with a signature and the index of the 
// validator who is assuring. A value of 1 at any given index implies that the validator assures they are contributing 
// to its availability.

impl Encode for AssurancesExtrinsic {
    
    fn encode(&self) -> Vec<u8> {

        let mut assurances_blob: Vec<u8> = Vec::new();
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
