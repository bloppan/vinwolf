use jam_types::{ServiceId, OpaqueHash, Gas, WorkItem, ImportSpec, ExtrinsicSpec};
use crate::{Encode, EncodeLen, EncodeSize, Decode, DecodeLen, BytesReader, ReadError};

impl Encode for WorkItem {
    
    fn encode(&self) -> Vec<u8> {
        
        let mut work_item_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<WorkItem>());
        
        self.service.encode_size(4).encode_to(&mut work_item_blob);
        self.code_hash.encode_to(&mut work_item_blob);
        self.payload.as_slice().encode_len().encode_to(&mut work_item_blob);
        self.refine_gas_limit.encode_size(8).encode_to(&mut work_item_blob);
        self.acc_gas_limit.encode_size(8).encode_to(&mut work_item_blob);
        self.import_segments.encode_len().encode_to(&mut work_item_blob);
        self.extrinsic.encode_len().encode_to(&mut work_item_blob);
        self.export_count.encode_size(2).encode_to(&mut work_item_blob);
        
        return work_item_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for WorkItem {

    fn decode(work_item_blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(WorkItem {
            service: ServiceId::decode(work_item_blob)?,
            code_hash: OpaqueHash::decode(work_item_blob)?,
            payload: Vec::<u8>::decode_len(work_item_blob)?,
            refine_gas_limit: Gas::decode(work_item_blob)?,
            acc_gas_limit: Gas::decode(work_item_blob)?,
            import_segments: Vec::<ImportSpec>::decode_len(work_item_blob)?,
            extrinsic: Vec::<ExtrinsicSpec>::decode_len(work_item_blob)?,
            export_count: u16::decode(work_item_blob)?,
        })
    }
}

impl Encode for ImportSpec {

    fn encode(&self) -> Vec<u8> {

        let mut import_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<Self>());

        self.tree_root.encode_to(&mut import_blob);
        self.index.encode_to(&mut import_blob);

        return import_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode()); 
    }
}

impl Decode for ImportSpec {

    fn decode(spec_blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(ImportSpec {
            tree_root : OpaqueHash::decode(spec_blob)?,
            index : u16::decode(spec_blob)?,        
        })
    }
}

impl Decode for ExtrinsicSpec {
    fn decode(ext_blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(ExtrinsicSpec {
            hash : OpaqueHash::decode(ext_blob)?,
            len : u32::decode(ext_blob)?,
        })
    }
}

impl Encode for ExtrinsicSpec {

    fn encode(&self) -> Vec<u8> {

        let mut ext_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<ExtrinsicSpec>());

        self.hash.encode_to(&mut ext_blob);
        self.len.encode_size(4).encode_to(&mut ext_blob);

        return ext_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}