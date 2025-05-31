use crate::types::{Authorizer, Hash, RefineContext, ReportedWorkPackage, ReportedWorkPackages, ServiceId, WorkItem, WorkPackage};
use crate::utils::codec::generic::decode;
use crate::utils::codec::{Encode, EncodeSize, EncodeLen, Decode, DecodeLen, BytesReader, ReadError};

impl Encode for WorkPackage {

    fn encode(&self) -> Vec<u8> {

        let mut work_pkg_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<WorkPackage>());

        self.authorization.as_slice().encode_len().encode_to(&mut work_pkg_blob);
        self.auth_code_host.encode_size(4).encode_to(&mut work_pkg_blob);
        self.authorizer.encode_to(&mut work_pkg_blob);
        self.context.encode_to(&mut work_pkg_blob);
        self.items.encode_len().encode_to(&mut work_pkg_blob);
        
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
            authorizer: Authorizer::decode(work_pkg_blob)?, 
            context : RefineContext::decode(work_pkg_blob)?,
            items : Vec::<WorkItem>::decode_len(work_pkg_blob)?,
        })
    }
}

impl Encode for ReportedWorkPackage {

    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::with_capacity(std::mem::size_of::<Self>());
        
        self.hash.encode_to(&mut blob);
        self.exports_root.encode_to(&mut blob);
        
        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

/*impl Encode for ReportedWorkPackages {

    fn encode(&self) -> Vec<u8> {
        let mut blob = Vec::new();

        self.0.encode_to(&mut blob);

        return blob;
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for ReportedWorkPackages {

    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        
        Ok( ReportedWorkPackages{0: Vec::<ReportedWorkPackage>::decode_len(reader)?})
    }
}*/

impl Decode for ReportedWorkPackage {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(ReportedWorkPackage{
            hash: Hash::decode(blob)?,
            exports_root: Hash::decode(blob)?,
        })
    }
}