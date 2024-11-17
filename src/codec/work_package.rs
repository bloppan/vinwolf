use crate::types::{ServiceId, OpaqueHash};
use crate::codec::{Encode, EncodeLen, EncodeSize, Decode, DecodeLen, BytesReader, ReadError};
use crate::codec::refine_context::RefineContext;
use crate::codec::work_item::WorkItem;

// A work-package includes a simple blob acting as an authorization token, the index of the service which
// hosts the authorization code, an authorization code hash and a parameterization blob, a context and a 
// sequence of work items:

pub struct WorkPackage {
    authorization: Vec<u8>,
    auth_code_host: ServiceId,
    authorizer: Authorizer,
    context: RefineContext,
    pub items: Vec<WorkItem>,
}

struct Authorizer {
    code_hash: OpaqueHash,
    params: Vec<u8>,
}

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