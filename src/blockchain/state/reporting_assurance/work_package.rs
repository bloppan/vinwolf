use crate::types::{ServiceId, OpaqueHash, RefineContext, WorkPackage, Authorizer, WorkItem};
use crate::utils::codec::{Encode, EncodeSize, EncodeLen, Decode, DecodeLen, BytesReader, ReadError};

impl Encode for WorkPackage {

    fn encode(&self) -> Vec<u8> {

        let mut work_pkg_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<WorkPackage>());

        self.authorization.as_slice().encode_len().encode_to(&mut work_pkg_blob);
        self.auth_code_host.encode_size(4).encode_to(&mut work_pkg_blob);
        self.authorizer.code_hash.encode_to(&mut work_pkg_blob);
        self.authorizer.params.as_slice().encode_len().encode_to(&mut work_pkg_blob);
        self.context.encode_to(&mut work_pkg_blob);
        WorkItem::encode_len(&self.items).encode_to(&mut work_pkg_blob);
        
        return work_pkg_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for WorkPackage {

    fn decode(work_pkg_blob: &mut BytesReader) -> Result<Self, ReadError> {
        
        Ok(WorkPackage {
            authorization : Vec::<u8>::decode_len(work_pkg_blob)?,
            auth_code_host : ServiceId::decode(work_pkg_blob)?,
            authorizer : {
                let code_hash = OpaqueHash::decode(work_pkg_blob)?;
                let params = Vec::<u8>::decode_len(work_pkg_blob)?;
                Authorizer {code_hash, params}
            },
            context : RefineContext::decode(work_pkg_blob)?,
            items : WorkItem::decode_len(work_pkg_blob)?,
        })
    }
}