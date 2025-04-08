use std::collections::HashMap;

use crate::types::{Authorizer, Hash, OpaqueHash, RefineContext, ReportedWorkPackage, ReportedWorkPackages, ServiceId, WorkItem, WorkPackage};
use crate::utils::codec::{Encode, EncodeSize, EncodeLen, Decode, DecodeLen, BytesReader, ReadError};
use crate::utils::codec::generic::{encode_unsigned, decode_unsigned};

impl Encode for WorkPackage {

    fn encode(&self) -> Vec<u8> {

        let mut work_pkg_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<WorkPackage>());

        self.authorization.as_slice().encode_len().encode_to(&mut work_pkg_blob);
        self.auth_code_host.encode_size(4).encode_to(&mut work_pkg_blob);
        self.authorizer.encode_to(&mut work_pkg_blob);
        self.context.encode_to(&mut work_pkg_blob);
        self.items.encode_to(&mut work_pkg_blob);
        
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
            items : Vec::<WorkItem>::decode(work_pkg_blob)?,
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

impl Decode for ReportedWorkPackage {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(ReportedWorkPackage{
            hash: Hash::decode(blob)?,
            exports_root: Hash::decode(blob)?,
        })
    }
}

impl Encode for Vec<ReportedWorkPackage> {

    fn encode(&self) -> Vec<u8> {

        let len = self.len();
        let mut work_packages = Vec::with_capacity(len * std::mem::size_of::<ReportedWorkPackage>());
        encode_unsigned(len).encode_to(&mut work_packages); 

        for work_package in self.iter() {
            work_package.encode_to(&mut work_packages);
        }

        return work_packages;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for Vec<ReportedWorkPackage> {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        
        let len = decode_unsigned(blob)?;
        let mut work_packages = Vec::with_capacity(len as usize);

        for _ in 0..len {
            let work_package = ReportedWorkPackage::decode(blob)?;
            work_packages.push(work_package);
        }

        Ok(work_packages)
    }
}

impl Encode for ReportedWorkPackages {

    fn encode(&self) -> Vec<u8> {

        let len = self.map.len();
        let mut reported_work_packages = Vec::with_capacity(len * std::mem::size_of::<Hash>() * 2);
        encode_unsigned(len).encode_to(&mut reported_work_packages); 

        for (hash, exports_root) in self.map.iter() {
            hash.encode_to(&mut reported_work_packages);
            exports_root.encode_to(&mut reported_work_packages);
        }

        return reported_work_packages;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for ReportedWorkPackages {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        
        let len = decode_unsigned(blob)?;
        let mut reported_work_packages = HashMap::with_capacity(len as usize);

        for _ in 0..len {
            let hash = OpaqueHash::decode(blob)?;
            let exports_root = OpaqueHash::decode(blob)?;
            reported_work_packages.insert(hash, exports_root);
        }

        Ok(ReportedWorkPackages{
            map: reported_work_packages
        })
    }
}