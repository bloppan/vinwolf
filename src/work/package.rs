use crate::types::{OpaqueHash, Gas, ServiceId, CoreIndex};
use crate::refine::RefineContext;
use crate::codec::{ReadError, BytesReader, Decode, DecodeLen, Encode, EncodeLen, EncodeSize};

pub struct WorkPackageSpec {
    hash: OpaqueHash,
    len: u32,
    erasure_root: OpaqueHash,
    exports_root: OpaqueHash,
}

pub struct WorkReport {
    package_spec: WorkPackageSpec,
    context: RefineContext,
    core_index: CoreIndex,
    authorizer_hash: OpaqueHash,
    auth_output: Vec<u8>,
    results: Vec<WorkResult>,
}

impl WorkReport {
    pub fn decode(work_report: &mut BytesReader) -> Result<Self, ReadError> {
        let package_spec = WorkPackageSpec {
            hash: OpaqueHash::decode(work_report)?,
            len: u32::decode(work_report)?,
            erasure_root: OpaqueHash::decode(work_report)?,
            exports_root: OpaqueHash::decode(work_report)?,
        };
        
        let context = RefineContext::decode(work_report)?;
        let core_index = CoreIndex::decode(work_report)?;
        let authorizer_hash = OpaqueHash::decode(work_report)?;
        let auth_output = Vec::<u8>::decode_len(work_report)?;
        let results = WorkResult::decode_len(work_report)?;

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
        let mut work_report_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<WorkReport>());
        
        self.package_spec.hash.encode_to(&mut work_report_blob);
        self.package_spec.len.encode_to(&mut work_report_blob);
        self.package_spec.erasure_root.encode_to(&mut work_report_blob);
        self.package_spec.exports_root.encode_to(&mut work_report_blob);
        self.context.encode_to(&mut work_report_blob)?;
        self.core_index.encode_to(&mut work_report_blob);
        self.authorizer_hash.encode_to(&mut work_report_blob);
        self.auth_output.as_slice().encode_len().encode_to(&mut work_report_blob);
        WorkResult::encode_len(&self.results)?.encode_to(&mut work_report_blob);

        Ok(work_report_blob)
    }

    pub fn encode_to(&self, into: &mut Vec<u8>) -> Result<(), ReadError> {
        into.extend_from_slice(&self.encode()?);
        Ok(())
    }
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
    code_hash: OpaqueHash,
    payload_hash: OpaqueHash,
    gas_ratio: Gas,
    result: Vec<u8>,
}

impl WorkResult {
    pub fn decode(work_result_blob: &mut BytesReader) -> Result<Self, ReadError> {
        
        let service = ServiceId::decode(work_result_blob)?;
        let code_hash = OpaqueHash::decode(work_result_blob)?;
        let payload_hash= OpaqueHash::decode(work_result_blob)?;
        let gas_ratio = Gas::decode(work_result_blob)?; 
        let mut result: Vec<u8> = vec![];

        result.push(work_result_blob.read_byte()?);
        let exec_result = result[0];
        match exec_result {
            0 => {
                let len = work_result_blob.read_byte()?;
                result.push(len);
                for i in 0..len {
                    result.push(work_result_blob.read_byte()?); 
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

    pub fn decode_len(work_result_blob: &mut BytesReader) -> Result<Vec<Self>, ReadError> {
        let num_results = work_result_blob.read_byte()? as usize;
        let mut results: Vec<WorkResult> = Vec::with_capacity(num_results);
        for _ in 0..num_results {
            let work_result = WorkResult::decode(work_result_blob)?;
            results.push(work_result);
        }
        Ok(results)
    }

    pub fn encode(&self) -> Result<Vec<u8>, ReadError> {

        let mut work_res_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<WorkResult>());

        self.service.encode_to(&mut work_res_blob);
        self.code_hash.encode_to(&mut work_res_blob);
        self.payload_hash.encode_to(&mut work_res_blob);
        self.gas_ratio.encode_to(&mut work_res_blob);

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

    pub fn encode_len(results: &[WorkResult]) -> Result<Vec<u8>, ReadError> {
        let mut encoded: Vec<u8> = Vec::new();
        encoded.push(results.len() as u8);
        for result in results {
            encoded.extend_from_slice(&result.encode()?);
        }
        Ok(encoded)
    }

    pub fn encode_to(&self, into: &mut Vec<u8>) -> Result<(), ReadError> {
        self.service.encode_to(into);
        self.code_hash.encode_to(into);
        self.payload_hash.encode_to(into);
        self.gas_ratio.encode_to(into);

        into.push(self.result[0]); 

        if self.result[0] == 0 {
            let len = self.result[1]; 
            into.push(len);
            for i in 2..(2 + len as usize) {
                into.push(self.result[i]); 
            }
        }

        Ok(())
    }
}

pub struct WorkPackage {
    authorization: Vec<u8>,
    auth_code_host: ServiceId,
    authorizer: Authorizer,
    context: RefineContext,
    pub items: Vec<WorkItem>,
}

impl WorkPackage {
    pub fn encode(&self) -> Result<Vec<u8>, ReadError> {
        // Preallocate initial capacity
        let mut work_pkg_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<WorkPackage>());
        // Encode WorkPackage params
        self.authorization.as_slice().encode_len().encode_to(&mut work_pkg_blob);
        self.auth_code_host.encode_size(4).encode_to(&mut work_pkg_blob);
        self.authorizer.code_hash.encode_to(&mut work_pkg_blob);
        self.authorizer.params.as_slice().encode_len().encode_to(&mut work_pkg_blob);
        self.context.encode_to(&mut work_pkg_blob)?;
        WorkItem::encode_len(&self.items)?.encode_to(&mut work_pkg_blob);
        
        Ok(work_pkg_blob)
    }

    pub fn decode(work_pkg_blob: &mut BytesReader) -> Result<Self, ReadError> {

        let authorization = Vec::<u8>::decode_len(work_pkg_blob)?;
        let auth_code_host = ServiceId::decode(work_pkg_blob)?;
        let code_hash = OpaqueHash::decode(work_pkg_blob)?;
        let params = Vec::<u8>::decode_len(work_pkg_blob)?;
        let authorizer = Authorizer {code_hash, params};
        let context = RefineContext::decode(work_pkg_blob)?;
        let items = WorkItem::decode_len(work_pkg_blob)?;
        
        Ok(WorkPackage {
            authorization,
            auth_code_host,
            authorizer,
            context,
            items,
        })
    }
}

#[derive(Default, Clone)]
pub struct ImportSpec {
    pub tree_root: OpaqueHash,
    pub index: u16,
}

impl ImportSpec {
    fn decode(spec_blob: &mut BytesReader) -> Result<Self, ReadError> {
        let tree_root = OpaqueHash::decode(spec_blob)?;
        let index = u16::decode(spec_blob)?;

        Ok(ImportSpec {
            tree_root,
            index,
        })
    }

    fn decode_len(spec_blob: &mut BytesReader) -> Result<Vec<Self>, ReadError> {
        let num_segments = spec_blob.read_byte()? as usize;
        let mut import_segments: Vec<ImportSpec> = Vec::with_capacity(num_segments);
        for _ in 0..num_segments {
            import_segments.push(ImportSpec::decode(spec_blob)?);
        } 
        return Ok(import_segments);
    }

    fn encode(&self) -> Result<Vec<u8>, ReadError> {
        let mut import_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<ImportSpec>());
        self.encode_to(&mut import_blob);
        Ok(import_blob)
    }

    fn encode_len(import_segments: &[ImportSpec]) -> Result<Vec<u8>, ReadError> {
        let mut import_blob_len: Vec<u8> = Vec::with_capacity(1 + import_segments.len() * std::mem::size_of::<ImportSpec>());
        import_blob_len.push(import_segments.len() as u8); 
        for import in import_segments {
            import_blob_len.extend_from_slice(&import.encode()?);
        }
        Ok(import_blob_len)
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.tree_root.encode()); 
        into.extend_from_slice(&self.index.encode()); 
    }
}

#[derive(Default, Clone)]
pub struct ExtrinsicSpec {
    pub hash: OpaqueHash,
    pub len: u32,
}

impl ExtrinsicSpec {
    fn decode(ext_blob: &mut BytesReader) -> Result<Self, ReadError> {
        let hash = OpaqueHash::decode(ext_blob)?;
        let len = u32::decode(ext_blob)?;

        Ok(ExtrinsicSpec {
            hash,
            len,
        })
    }

    fn decode_len(ext_blob: &mut BytesReader) -> Result<Vec<Self>, ReadError> {
        let num_extrinsics = ext_blob.read_byte()? as usize;
        let mut extrinsic: Vec<ExtrinsicSpec> = Vec::with_capacity(num_extrinsics);
        for _ in 0..num_extrinsics {
            extrinsic.push(ExtrinsicSpec::decode(ext_blob)?);
        }

        Ok(extrinsic)
    }

    fn encode(&self) -> Result<Vec<u8>, ReadError> {
        let mut ext_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<ExtrinsicSpec>());
        self.hash.encode_to(&mut ext_blob);
        self.len.encode_to(&mut ext_blob);
        Ok(ext_blob)
    }

    fn encode_len(extrinsics: &[ExtrinsicSpec]) -> Result<Vec<u8>, ReadError> {
        let mut ext_blob_len: Vec<u8> = Vec::with_capacity(1 + extrinsics.len() * std::mem::size_of::<ExtrinsicSpec>());
        ext_blob_len.push(extrinsics.len() as u8); 
        for ext in extrinsics {
            ext_blob_len.extend_from_slice(&ext.encode()?);
        }
        Ok(ext_blob_len)
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.hash.encode()); 
        into.extend_from_slice(&self.len.encode());
    }
}

struct Authorizer {
    code_hash: OpaqueHash,
    params: Vec<u8>,
}

pub struct WorkItem {
    service: ServiceId,
    code_hash: OpaqueHash,
    payload: Vec<u8>,
    gas_limit: Gas,
    import_segments: Vec<ImportSpec>,
    extrinsic: Vec<ExtrinsicSpec>,
    export_count: u16,
}

impl WorkItem {
    pub fn decode(work_item_blob: &mut BytesReader) -> Result<Self, ReadError> {

        let service = ServiceId::decode(work_item_blob)?;
        let code_hash = OpaqueHash::decode(work_item_blob)?;
        let payload = Vec::<u8>::decode_len(work_item_blob)?;
        let gas_limit = Gas::decode(work_item_blob)?;  
        let import_segments = ImportSpec::decode_len(work_item_blob)?;
        let extrinsic = ExtrinsicSpec::decode_len(work_item_blob)?;
        let export_count = u16::decode(work_item_blob)?;
    
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

    pub fn decode_len(work_item_blob: &mut BytesReader) -> Result<Vec<WorkItem>, ReadError> {
        let num_items = work_item_blob.read_byte()? as usize;
        let mut items = Vec::with_capacity(num_items);
        for _ in 0..num_items {
            items.push(WorkItem::decode(work_item_blob)?);
        }
        Ok(items)
    }

    pub fn encode(&self) -> Result<Vec<u8>, ReadError> {
        // Preallocate initial capacity
        let mut work_item_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<WorkItem>());
        // Encode WorkItem params
        self.service.encode_to(&mut work_item_blob);
        self.code_hash.encode_to(&mut work_item_blob);
        self.payload.as_slice().encode_len().encode_to(&mut work_item_blob);
        self.gas_limit.encode_to(&mut work_item_blob);
        ImportSpec::encode_len(&self.import_segments)?.encode_to(&mut work_item_blob);
        ExtrinsicSpec::encode_len(&self.extrinsic)?.encode_to(&mut work_item_blob);
        self.export_count.encode_to(&mut work_item_blob);      
        Ok(work_item_blob)
    }

    fn encode_len(items: &[WorkItem]) -> Result<Vec<u8>, ReadError> {
        let mut blob: Vec<u8> = Vec::new();
        blob.push(items.len() as u8);
        for item in items {
            blob.extend_from_slice(&item.encode()?);
        }
        Ok(blob)
    }

    fn encode_to(&self, into: &mut Vec<u8>) -> Result<(), ReadError> {
        into.extend_from_slice(&self.encode()?);
        Ok(())
    }
}
