use crate::types::*;
use crate::refine::RefineContext;
use crate::codec::*;

#[derive(Default, Clone)]
pub struct ImportSpec {
    pub tree_root: [u8; 32],
    pub index: u16,
}

#[derive(Default, Clone)]
pub struct ExtrinsicSpec {
    pub hash: [u8; 32],
    pub len: u32,
}

struct Authorizer {
    code_hash: [u8; 32],
    params: Vec<u8>,
}

pub struct WorkItem {
    service: ServiceId,
    code_hash: [u8; 32],
    payload: Vec<u8>,
    gas_limit: u64,
    import_segments: Vec<ImportSpec>,
    extrinsic: Vec<ExtrinsicSpec>,
    export_count: u16,
}

pub struct WorkPackage {
    authorization: Vec<u8>,
    auth_code_host: ServiceId,
    authorizer: Authorizer,
    context: RefineContext,
    pub items: Vec<WorkItem>,
}

enum WorkExecResult {
    Ok = 0,
    OutOfGas = 1,
    Panic = 2,
    BadCode = 3,
    CodeOversize = 4,
}

pub struct WorkResult {
    service: ServiceId,
    code_hash: [u8; 32],
    payload_hash: [u8; 32],
    gas_ratio: u64,
    result: Vec<u8>,
}
/*
WorkPackageSpec ::= SEQUENCE {
    hash OpaqueHash,
    len U32,
    erasure-root OpaqueHash,
    exports-root OpaqueHash
}

WorkReport ::= SEQUENCE {
    package-spec WorkPackageSpec,
    context RefineContext,
    core-index CoreIndex,
    authorizer-hash OpaqueHash,
    auth-output ByteSequence,
    results SEQUENCE (SIZE(1..4)) OF WorkResult
}*/

pub struct WorkPackageSpec {
    hash: [u8; 32],
    len: u32,
    erasure_root: [u8; 32],
    exports_root: [u8; 32],
}

pub struct WorkReport {
    package_spec: WorkPackageSpec,
    context: RefineContext,
    core_index: CoreIndex,
    authorizer_hash: [u8; 32],
    auth_output: Vec<u8>,
    results: Vec<WorkResult>,
}

impl WorkReport {
    pub fn decode(work_report: &[u8]) -> Result<Self, ReadError> {

        let mut blob = SliceReader::new(work_report);

        let hash = blob.read_32bytes()?; 
        let len = blob.read_u32()?; 
        let erasure_root = blob.read_32bytes()?; 
        let exports_root = blob.read_32bytes()?; 
        let package_spec = WorkPackageSpec { hash, len, erasure_root, exports_root };
        let context: RefineContext = RefineContext::decode(&blob.current_slice())?;
        blob.inc_pos(context.len())?;
        let core_index = blob.read_u16()?; 
        let authorizer_hash = blob.read_32bytes()?; 
        let auth_output_usize: Vec<usize> = decode_len(&blob.current_slice());
        let auth_output: Vec<u8> = auth_output_usize.iter().map(|&x| x as u8).collect();
        blob.inc_pos(auth_output.len() + 1);
        
        let num_results = blob.read_next_byte()?;
        let mut results: Vec<WorkResult> = Vec::with_capacity(4);
        for _ in 0..num_results {
            let item = WorkResult::decode(&blob.current_slice())?;
            let item_size = WorkResult::len(&item);
            results.push(item);
            blob.inc_pos(item_size);
        }

        Ok(WorkReport {
            package_spec,
            context,
            core_index,
            authorizer_hash,
            auth_output,
            results,
        })
    }

    pub fn encode(&self) -> Result<Vec<u8>, ReadError> {
        let mut work_report_blob: Vec<u8> = vec![];
        work_report_blob.extend_from_slice(&self.package_spec.hash);
        work_report_blob.extend_from_slice(&self.package_spec.len.to_le_bytes());
        work_report_blob.extend_from_slice(&self.package_spec.erasure_root);
        work_report_blob.extend_from_slice(&self.package_spec.exports_root);
        work_report_blob.extend_from_slice(&self.context.encode()?);
        work_report_blob.extend_from_slice(&self.core_index.to_le_bytes());
        work_report_blob.extend_from_slice(&self.authorizer_hash);
        work_report_blob.extend_from_slice(&(&self.auth_output[..]).encode_len());
        work_report_blob.push(self.results.len() as u8);
        for item in &self.results {
            work_report_blob.extend_from_slice(&item.encode()?);
        }

        Ok(work_report_blob)
    }
}

impl WorkResult {
    pub fn decode(work_result: &[u8]) -> Result<Self, ReadError> {
        
        let mut blob = SliceReader::new(work_result);

        let service = blob.read_u32()?; 
        let code_hash: [u8; 32] = blob.read_32bytes()?; 
        let payload_hash: [u8; 32] = blob.read_32bytes()?; 
        let gas_ratio: u64 = blob.read_u64()?;
        let mut result: Vec<u8> = vec![];
        result.push(blob.read_next_byte()?);
        let exec_result = result[0];
        match exec_result {
            0 => {
                let len = blob.read_next_byte()?;
                result.push(len);
                for i in 0..len {
                    result.push(blob.read_next_byte()?); 
                }
                WorkExecResult::Ok
            },
            1 => WorkExecResult::OutOfGas,
            2 => WorkExecResult::Panic,
            3 => WorkExecResult::BadCode,
            4 => WorkExecResult::CodeOversize,
            _ => panic!("Valor inválido para WorkExecResult: {}", exec_result),
        };
        Ok(WorkResult {
            service,
            code_hash,
            payload_hash,
            gas_ratio,
            result,
        })
    }

    pub fn encode(&self) -> Result<Vec<u8>, ReadError> {
        let mut work_res_blob: Vec<u8> = Vec::with_capacity(self.len());
        work_res_blob.extend_from_slice(&self.service.to_le_bytes());
        work_res_blob.extend_from_slice(&self.code_hash[..]);
        work_res_blob.extend_from_slice(&self.payload_hash[..]);
        work_res_blob.extend_from_slice(&self.gas_ratio.to_le_bytes());
        let exec_result = self.result[0];
        work_res_blob.push(exec_result);
        if exec_result == 0 {
            let len = self.result[1];
            work_res_blob.push(len);
            for i in 0..len {
                work_res_blob.push(self.result[2 + i as usize]);
            }
        }
        
        Ok(work_res_blob)
    }

    pub fn len(&self) -> usize {
        return 4 + 32 + 32 + 8 + self.result.len();
    }
}

impl WorkPackage {
    pub fn encode(&self) -> Result<Vec<u8>, ReadError> {
        // Preallocate initial capacity
        let mut work_pkg_blob: Vec<u8> = Vec::new();
        // Encode WorkItem params
        work_pkg_blob.push(self.authorization.len() as u8);
        work_pkg_blob.extend_from_slice(&self.authorization[..]);
        work_pkg_blob.extend_from_slice(&self.auth_code_host.to_le_bytes());
        work_pkg_blob.extend_from_slice(&self.authorizer.code_hash[..]);
        work_pkg_blob.push(self.authorizer.params.len() as u8);
        work_pkg_blob.extend_from_slice(&self.authorizer.params[..]);
        work_pkg_blob.extend_from_slice(&self.context.encode()?);
        work_pkg_blob.push(self.items.len() as u8);

        for i in 0..self.items.len() {
            work_pkg_blob.extend_from_slice(&self.items[i].encode()?);
        }
        
        Ok(work_pkg_blob)
    }

    pub fn decode(work_pkg_blob: &[u8]) -> Result<Self, ReadError> {

        let mut blob = SliceReader::new(work_pkg_blob);

        let authorization_usize: Vec<usize> = decode_len(&blob.current_slice());
        let authorization: Vec<u8> = authorization_usize.iter().map(|&x| x as u8).collect();
        let mut index = authorization.len() + 1;
        blob.inc_pos(authorization.len() + 1)?;
        let auth_code_host: u32 = blob.read_u32()?; 
        let code_hash: [u8; 32] = blob.read_32bytes()?; 
        let params_usize: Vec<usize> = decode_len(&blob.current_slice());
        let params: Vec<u8> = params_usize.iter().map(|&x| x as u8).collect();
        blob.inc_pos(params.len() + 1);
        let authorizer = Authorizer {code_hash, params};

        let context: RefineContext = RefineContext::decode(&blob.current_slice())?;
        blob.inc_pos(context.len())?;

        let num_items = blob.read_next_byte()?; 
        let mut items: Vec<WorkItem> = Vec::with_capacity(4);
        for _ in 0..num_items {
            let item = WorkItem::decode(&blob.current_slice())?;
            let item_size = WorkItem::len(&item); 
            items.push(item);
            blob.inc_pos(item_size);
        }
        
        Ok(WorkPackage {
            authorization,
            auth_code_host,
            authorizer,
            context,
            items,
        })
    }
}

impl WorkItem {
    pub fn encode(&self) -> Result<Vec<u8>, ReadError> {
        // Preallocate initial capacity
        let mut work_item_blob: Vec<u8> = Vec::with_capacity(self.len());
        // Encode WorkItem params
        work_item_blob.extend_from_slice(&self.service.to_le_bytes());
        work_item_blob.extend_from_slice(&self.code_hash);
        work_item_blob.extend_from_slice(&(&self.payload[..]).encode_len());
        work_item_blob.extend_from_slice(&self.gas_limit.to_le_bytes());
        work_item_blob.push(self.import_segments.len() as u8);
        for segment in &self.import_segments {
            work_item_blob.extend_from_slice(&segment.tree_root);
            work_item_blob.extend_from_slice(&segment.index.to_le_bytes());
        }
        work_item_blob.push(self.extrinsic.len() as u8);
        for ext in &self.extrinsic {
            work_item_blob.extend_from_slice(&ext.hash);
            work_item_blob.extend_from_slice(&ext.len.to_le_bytes());
        }
        work_item_blob.extend_from_slice(&self.export_count.to_le_bytes());
        
        Ok(work_item_blob)
    }

    pub fn decode(work_item_blob: &[u8]) -> Result<Self, ReadError> {

        let mut blob = SliceReader::new(work_item_blob);

        let service = blob.read_u32()?;
        let code_hash = blob.read_32bytes()?;
        let payload_usize: Vec<usize> = decode_len(&blob.current_slice());
        let payload: Vec<u8> = payload_usize.iter().map(|&x| x as u8).collect();
        blob.inc_pos(payload_usize.len() + 1)?;
        let gas_limit = blob.read_u64()?;
        let num_segments = blob.read_next_byte()? as usize;

        let mut import_segments = Vec::with_capacity(num_segments);
        for _ in 0..num_segments {
            let tree_root = blob.read_32bytes()?; 
            let index_segment = blob.read_u16()?;
            import_segments.push(ImportSpec {tree_root, index: index_segment});
        }

        let num_extrinsics = blob.read_next_byte()? as usize; 
        let mut extrinsic = Vec::with_capacity(num_extrinsics);
        for _ in 0..num_extrinsics {
            let hash = blob.read_32bytes()?;
            let len = blob.read_u32()?;
            extrinsic.push(ExtrinsicSpec { hash, len });
        }

        let export_count = blob.read_u16()?;

        Ok(WorkItem {
            service,
            code_hash,
            payload,
            gas_limit,
            import_segments,
            extrinsic,
            export_count,
        })
    }

    pub fn len(&self) -> usize {
        return 4 + 32 + 1 + self.payload.len() + 8 + 1 + self.import_segments.len() * (32 + 2) + 1 + self.extrinsic.len() * (32 + 4) + 2;
    }
}
