use crate::types::{ServiceId, OpaqueHash, Gas, WorkItem, ImportSpec, ExtrinsicSpec};
use crate::utils::codec::{Encode, EncodeLen, EncodeSize, Decode, DecodeLen, BytesReader, ReadError};
use crate::utils::codec::generic::{encode_unsigned, decode_unsigned};

impl Encode for WorkItem {
    
    fn encode(&self) -> Vec<u8> {
        
        let mut work_item_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<WorkItem>());
        
        self.service.encode_size(4).encode_to(&mut work_item_blob);
        self.code_hash.encode_to(&mut work_item_blob);
        self.payload.as_slice().encode_len().encode_to(&mut work_item_blob);
        self.gas_limit.encode_size(8).encode_to(&mut work_item_blob);
        self.import_segments.encode_to(&mut work_item_blob);
        self.extrinsic.encode_to(&mut work_item_blob);
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
            gas_limit: Gas::decode(work_item_blob)?,
            import_segments: Vec::<ImportSpec>::decode(work_item_blob)?,
            extrinsic: Vec::<ExtrinsicSpec>::decode(work_item_blob)?,
            export_count: u16::decode(work_item_blob)?,
        })
    }
}

impl Encode for Vec<WorkItem> {

    fn encode(&self) -> Vec<u8> {

        let mut blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<WorkItem>() * self.len());
        encode_unsigned(self.len()).encode_to(&mut blob);

        for item in self.iter() {
            item.encode_to(&mut blob);
        }

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for Vec<WorkItem> {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        let num_items = decode_unsigned(blob)?;
        let mut items = Vec::with_capacity(num_items);

        for _ in 0..num_items {
            items.push(WorkItem::decode(blob)?);
        }

        Ok(items)
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

impl Encode for Vec<ImportSpec> {
    
    fn encode(&self) -> Vec<u8> {

        let mut import_blob_len: Vec<u8> = Vec::with_capacity(std::mem::size_of::<ImportSpec>() * self.len());
        encode_unsigned(self.len()).encode_to(&mut import_blob_len);

        for import in self.iter() {
            import.encode_to(&mut import_blob_len);
        }

        return import_blob_len;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for Vec<ImportSpec> {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        
        let num_segments = decode_unsigned(blob)? as usize;
        let mut import_segments: Vec<ImportSpec> = Vec::with_capacity(num_segments);

        for _ in 0..num_segments {
            import_segments.push(ImportSpec::decode(blob)?);
        } 

        return Ok(import_segments);
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

impl Encode for Vec<ExtrinsicSpec> {

    fn encode(&self) -> Vec<u8> {

        let mut blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<ExtrinsicSpec>() * self.len());
        encode_unsigned(self.len()).encode_to(&mut blob);
        
        for ext in self.iter() {
            ext.encode_to(&mut blob);
        }
        
        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for Vec<ExtrinsicSpec> {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        let num_extrinsics = decode_unsigned(blob)?;
        let mut extrinsic: Vec<ExtrinsicSpec> = Vec::with_capacity(num_extrinsics);
        
        for _ in 0..num_extrinsics {
            extrinsic.push(ExtrinsicSpec::decode(blob)?);
        }
        
        Ok(extrinsic)
    }
}