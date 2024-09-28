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
    pub fn decode(work_report: &[u8]) -> Self {
        let hash = work_report[0..32].try_into().expect("slice with incorrect length for hash");
        let len = decode_trivial(&work_report[32..36][..]) as u32;
        let erasure_root = work_report[36..68].try_into().expect("slice with incorrect length for erasure_root");
        let exports_root = work_report[68..100].try_into().expect("slice with incorrect length for export_root");
        let package_spec = WorkPackageSpec { hash, len, erasure_root, exports_root };
        let context: RefineContext = RefineContext::decode(&work_report[100..][..]);
        let mut index = 100;
        if context.prerequisite != None {
            index += 164 + 1;
        } else {
            index += 132 + 1;
        }
        let core_index = decode_trivial(&work_report[index..index + 2][..]) as u16;
        index += 2;
        let authorizer_hash = work_report[index..index + 32].try_into().expect("slice with incorrect lenth for authorizer hash");
        index += 32;
        let auth_output_usize: Vec<usize> = decode_len(&work_report[index..]);
        let auth_output: Vec<u8> = auth_output_usize.iter().map(|&x| x as u8).collect();
        index += auth_output.len() + 1;
        let num_results = work_report[index];
        index += 1;
        let mut results: Vec<WorkResult> = Vec::with_capacity(4);
        for _ in 0..num_results {
            let item = WorkResult::decode(&work_report[index..]);
            let item_size = 76 + item.result.len();
            results.push(item);
            index += item_size;  
        }
        WorkReport {
            package_spec,
            context,
            core_index,
            authorizer_hash,
            auth_output,
            results,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut work_report_blob: Vec<u8> = vec![];
        work_report_blob.extend_from_slice(&self.package_spec.hash);
        work_report_blob.extend_from_slice(&encode_trivial(self.package_spec.len as usize, 4));
        work_report_blob.extend_from_slice(&self.package_spec.erasure_root);
        work_report_blob.extend_from_slice(&self.package_spec.exports_root);
        work_report_blob.extend_from_slice(&self.context.encode());
        work_report_blob.extend_from_slice(&encode_trivial(self.core_index as usize, 2));
        work_report_blob.extend_from_slice(&self.authorizer_hash);
        work_report_blob.extend_from_slice(&(&self.auth_output[..]).encode_len());
        work_report_blob.push(self.results.len() as u8);
        for item in &self.results {
            work_report_blob.extend_from_slice(&item.encode());
        }

        work_report_blob
    }
}

impl WorkResult {
    pub fn decode(work_result: &[u8]) -> Self {
        let service = u32::from_le_bytes(work_result[0..4].try_into().expect("slice with incorrect length for service"));
        let code_hash: [u8; 32] = work_result[4..36].try_into().expect("slice with incorrect length for code_hash");
        let payload_hash: [u8; 32] = work_result[36..68].try_into().expect("slice with incorrect length for payload_hash");
        let gas_ratio: u64 = decode_trivial(&work_result[68..76][..]) as u64;
        let mut result: Vec<u8> = vec![];
        result.push(work_result[76]);
        match result[0] {
            0 => {
                let len = work_result[77];
                result.push(len);
                for i in 0..len {
                    result.push(work_result[78 + i as usize]);
                }
                WorkExecResult::Ok
            },
            1 => WorkExecResult::OutOfGas,
            2 => WorkExecResult::Panic,
            3 => WorkExecResult::BadCode,
            4 => WorkExecResult::CodeOversize,
            _ => panic!("Valor inválido para WorkExecResult: {}", result[0]),
        };
        WorkResult {
            service,
            code_hash,
            payload_hash,
            gas_ratio,
            result,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut work_res_blob: Vec<u8> = Vec::with_capacity(4 + 32 + 32 + 8 + 1);
        work_res_blob.extend_from_slice(&encode_trivial(self.service as usize, 4));
        work_res_blob.extend_from_slice(&self.code_hash[..]);
        work_res_blob.extend_from_slice(&self.payload_hash[..]);
        work_res_blob.extend_from_slice(&encode_trivial(self.gas_ratio as usize, 8));
        work_res_blob.push(self.result[0]);
        if self.result[0] == 0 {
            let len = self.result[1];
            work_res_blob.push(len);
            for i in 0..len {
                work_res_blob.push(self.result[2 + i as usize]);
            }
        }
        work_res_blob
    }
}

impl WorkPackage {
    pub fn encode(&self) -> Vec<u8> {
        // Preallocate initial capacity
        let mut work_pkg_blob: Vec<u8> = Vec::new();
        // Encode WorkItem params
        work_pkg_blob.push(self.authorization.len() as u8);
        work_pkg_blob.extend_from_slice(&self.authorization[..]);
        work_pkg_blob.extend_from_slice(&encode_trivial(self.auth_code_host as usize, 4));
        work_pkg_blob.extend_from_slice(&self.authorizer.code_hash[..]);
        work_pkg_blob.push(self.authorizer.params.len() as u8);
        work_pkg_blob.extend_from_slice(&self.authorizer.params[..]);
        work_pkg_blob.extend_from_slice(&self.context.encode());
        work_pkg_blob.push(self.items.len() as u8);
        for i in 0..self.items.len() {
            work_pkg_blob.extend_from_slice(&self.items[i].encode());
        }
        work_pkg_blob
    }

    pub fn decode(work_pkg_blob: &[u8]) -> Self {

        let authorization_usize: Vec<usize> = decode_len(&work_pkg_blob);
        let authorization: Vec<u8> = authorization_usize.iter().map(|&x| x as u8).collect();
        let mut index = authorization.len() + 1;
        let auth_code_host: u32 = decode_trivial(&work_pkg_blob[index..index + 4]) as u32;
        index += 4;
        let code_hash: [u8; 32] = work_pkg_blob[index..index + 32].try_into().expect("slice with incorrect length for code_hash");
        index += 32;
        let params_usize: Vec<usize> = decode_len(&work_pkg_blob[index..]);
        let params: Vec<u8> = params_usize.iter().map(|&x| x as u8).collect();
        index += params.len() + 1;
        let authorizer = Authorizer {code_hash, params};

        let context: RefineContext = RefineContext::decode(&work_pkg_blob[index..].to_vec());
        if context.prerequisite != None {
            index += 164 + 1;
        } else {
            index += 132 + 1;
        }
        let num_items = work_pkg_blob[index];
        index += 1;
        let mut items: Vec<WorkItem> = Vec::with_capacity(4);
        for _ in 0..num_items {
            let item = WorkItem::decode(&work_pkg_blob[index..]);
            let item_size = estimate_work_item_size(&item); 
            items.push(item);
            index += item_size;  
        }
        
        WorkPackage {
            authorization,
            auth_code_host,
            authorizer,
            context,
            items,
        }
    }
}

fn estimate_work_item_size(work_item: &WorkItem) -> usize {
    let mut size = 0;
    size += 4; // service
    size += 32; // code_hash
    size += 1 + work_item.payload.len(); // payload length + data
    size += 8; // gas_limit
    size += 1; // import_segments length
    size += work_item.import_segments.len() * (32 + 2); // Each ImportSpec
    size += 1; // extrinsic length
    size += work_item.extrinsic.len() * (32 + 4); // Each ExtrinsicSpec
    size += 2; // result
    size
}

impl WorkItem {
    pub fn encode(&self) -> Vec<u8> {
        // Preallocate initial capacity
        let mut work_item_blob: Vec<u8> = Vec::with_capacity(estimate_work_item_size(self));
        // Encode WorkItem params
        work_item_blob.extend_from_slice(&encode_trivial(self.service as usize, 4));
        work_item_blob.extend_from_slice(&self.code_hash);
        work_item_blob.extend_from_slice(&(&self.payload[..]).encode_len());
        work_item_blob.extend_from_slice(&encode_trivial(self.gas_limit as usize, 8));
        work_item_blob.push(self.import_segments.len() as u8);
        for segment in &self.import_segments {
            work_item_blob.extend_from_slice(&segment.tree_root);
            work_item_blob.extend_from_slice(&encode_trivial(segment.index as usize, 2));
        }
        work_item_blob.push(self.extrinsic.len() as u8);
        for ext in &self.extrinsic {
            work_item_blob.extend_from_slice(&ext.hash);
            work_item_blob.extend_from_slice(&encode_trivial(ext.len as usize, 4));
        }
        work_item_blob.extend_from_slice(&encode_trivial(self.export_count as usize, 2));
        work_item_blob
    }

    pub fn decode(work_item_blob: &[u8]) -> Self {
        let mut index = 0;

        let service = u32::from_le_bytes(work_item_blob[index..index + 4].try_into().expect("slice with incorrect length for service"));
        index += 4;
        let code_hash = work_item_blob[index..index + 32].try_into().expect("slice with incorrect length for code_hash");
        index += 32;
        let payload_usize: Vec<usize> = decode_len(&work_item_blob[index..]);
        let payload: Vec<u8> = payload_usize.iter().map(|&x| x as u8).collect();
        index += payload_usize.len() + 1;
        let gas_limit = u64::from_le_bytes(work_item_blob[index..index + 8].try_into().expect("slice with incorrect length for gas_limit"));
        index += 8;
        let num_segments = work_item_blob[index] as usize;
        index += 1;

        let mut import_segments = Vec::with_capacity(num_segments);
        for _ in 0..num_segments {
            let tree_root = work_item_blob[index..index + 32].try_into().expect("slice with incorrect length for tree_root");
            index += 32;
            let index_segment = u16::from_le_bytes(work_item_blob[index..index + 2].try_into().expect("slice with incorrect length for index"));
            index += 2;
            import_segments.push(ImportSpec {tree_root, index: index_segment});
        }

        let num_extrinsics = work_item_blob[index] as usize;
        index += 1;

        let mut extrinsic = Vec::with_capacity(num_extrinsics);
        for _ in 0..num_extrinsics {
            let hash = work_item_blob[index..index + 32].try_into().expect("slice with incorrect length for hash");
            index += 32;
            let len = u32::from_le_bytes(work_item_blob[index..index + 4].try_into().expect("slice with incorrect length for len"));
            index += 4;
            extrinsic.push(ExtrinsicSpec { hash, len });
        }

        let export_count = decode_trivial(&work_item_blob[index..index + 1]) as u16;

        WorkItem {
            service,
            code_hash,
            payload,
            gas_limit,
            import_segments,
            extrinsic,
            export_count,
        }
    }
}