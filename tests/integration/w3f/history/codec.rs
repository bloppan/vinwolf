use vinwolf::types::{Hash, ReportedWorkPackage};
use vinwolf::utils::codec::{Encode, EncodeLen, Decode, DecodeLen, BytesReader, ReadError};

#[derive(Debug)]
pub struct InputHistory {
    pub header_hash: Hash,
    pub parent_state_root: Hash,
    pub accumulate_root: Hash,
    pub work_packages: Vec<ReportedWorkPackage>,
}

impl Encode for InputHistory {

    fn encode(&self) -> Vec<u8> {

        let mut input_blob = Vec::with_capacity(std::mem::size_of::<Self>());

        self.header_hash.encode_to(&mut input_blob);
        self.parent_state_root.encode_to(&mut input_blob);
        self.accumulate_root.encode_to(&mut input_blob);
        self.work_packages.encode_len().encode_to(&mut input_blob);

        return input_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for InputHistory {

    fn decode(input_blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(InputHistory {
            header_hash: Hash::decode(input_blob)?,
            parent_state_root: Hash::decode(input_blob)?,
            accumulate_root: Hash::decode(input_blob)?,
            work_packages: Vec::<ReportedWorkPackage>::decode_len(input_blob)?,
        })
    }
}