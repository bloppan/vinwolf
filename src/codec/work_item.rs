use crate::types::{ServiceId, OpaqueHash, Gas};
use crate::codec::{Encode, EncodeLen, EncodeSize, Decode, DecodeLen, BytesReader, ReadError};
use crate::codec::{encode_unsigned, decode_unsigned};

// A Work Item includes: the identifier of the service to which it relates, the code hash of the service at 
// the time of reporting (whose preimage must be available from the perspective of the lookup anchor block), 
// a payload blob, a gas limit, and the three elements of its manifest, a sequence of imported data segments, 
// which identify a prior exported segment through an index and the identity of an exporting work-package, 
// a sequence of blob hashes and lengths to be introduced in this block (and which we assume the validator knows) 
// and the number of data segments exported by this work item.

pub struct WorkItem {
    service: ServiceId,
    code_hash: OpaqueHash,
    payload: Vec<u8>,
    gas_limit: Gas,
    import_segments: Vec<ImportSpec>,
    extrinsic: Vec<ExtrinsicSpec>,
    export_count: u16,
}

impl Encode for WorkItem {
    
    fn encode(&self) -> Vec<u8> {
        
        let mut work_item_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<WorkItem>());
        
        self.service.encode_size(4).encode_to(&mut work_item_blob);
        self.code_hash.encode_to(&mut work_item_blob);
        self.payload.as_slice().encode_len().encode_to(&mut work_item_blob);
        self.gas_limit.encode_size(8).encode_to(&mut work_item_blob);
        ImportSpec::encode_len(&self.import_segments).encode_to(&mut work_item_blob);
        ExtrinsicSpec::encode_len(&self.extrinsic).encode_to(&mut work_item_blob);
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
            service : ServiceId::decode(work_item_blob)?,
            code_hash : OpaqueHash::decode(work_item_blob)?,
            payload : Vec::<u8>::decode_len(work_item_blob)?,
            gas_limit : Gas::decode(work_item_blob)?,
            import_segments : ImportSpec::decode_len(work_item_blob)?,
            extrinsic : ExtrinsicSpec::decode_len(work_item_blob)?,
            export_count : u16::decode(work_item_blob)?,
        })
    }
}

impl WorkItem {

    pub fn decode_len(work_item_blob: &mut BytesReader) -> Result<Vec<WorkItem>, ReadError> {

        let num_items = decode_unsigned(work_item_blob)?;
        let mut items = Vec::with_capacity(num_items);

        for _ in 0..num_items {
            items.push(WorkItem::decode(work_item_blob)?);
        }

        Ok(items)
    }

    pub fn encode_len(work_item: &[WorkItem]) -> Vec<u8> {

        let mut blob: Vec<u8> = Vec::new();
        encode_unsigned(work_item.len()).encode_to(&mut blob);

        for item in work_item {
            item.encode_to(&mut blob);
        }

        return blob;
    }
}

// The Import Spec is a sequence of imported data segments, which identify a prior exported segment 
// through an index and the identity of an exporting work-package. Its a member of Work Item.

pub struct ImportSpec {
    pub tree_root: OpaqueHash,
    pub index: u16,
}

impl Encode for ImportSpec {

    fn encode(&self) -> Vec<u8> {

        let mut import_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<ImportSpec>());
        self.encode_to(&mut import_blob);

        return import_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.tree_root.encode()); 
        into.extend_from_slice(&self.index.encode()); 
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

impl ImportSpec {

    fn encode_len(import_segments: &[ImportSpec]) -> Vec<u8> {

        let mut import_blob_len: Vec<u8> = Vec::with_capacity(1 + import_segments.len() * std::mem::size_of::<ImportSpec>());
        encode_unsigned(import_segments.len()).encode_to(&mut import_blob_len);

        for import in import_segments {
            import.encode_to(&mut import_blob_len);
        }

        return import_blob_len;
    }

    fn decode_len(spec_blob: &mut BytesReader) -> Result<Vec<Self>, ReadError> {

        let num_segments = spec_blob.read_byte()? as usize;
        let mut import_segments: Vec<ImportSpec> = Vec::with_capacity(num_segments);

        for _ in 0..num_segments {
            import_segments.push(ImportSpec::decode(spec_blob)?);
        } 

        return Ok(import_segments);
    }
}

// The extrinsic spec is a sequence of blob hashes and lengths to be introduced in this block 
// (and which we assume the validator knows). It's a member of Work Item

pub struct ExtrinsicSpec {
    pub hash: OpaqueHash,
    pub len: u32,
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

impl ExtrinsicSpec {

    fn encode_len(extrinsics: &[ExtrinsicSpec]) -> Vec<u8> {

        let mut ext_blob_len: Vec<u8> = Vec::with_capacity(1 + extrinsics.len() * std::mem::size_of::<ExtrinsicSpec>());
        encode_unsigned(extrinsics.len()).encode_to(&mut ext_blob_len);
        
        for ext in extrinsics {
            ext.encode_to(&mut ext_blob_len);
        }
        
        return ext_blob_len;
    }

    fn decode_len(ext_blob: &mut BytesReader) -> Result<Vec<Self>, ReadError> {

        let num_extrinsics = decode_unsigned(ext_blob)?;
        let mut extrinsic: Vec<ExtrinsicSpec> = Vec::with_capacity(num_extrinsics);
        
        for _ in 0..num_extrinsics {
            extrinsic.push(ExtrinsicSpec::decode(ext_blob)?);
        }
        
        Ok(extrinsic)
    }
}