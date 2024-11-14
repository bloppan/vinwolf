use crate::types::{CoreIndex, OpaqueHash};
use crate::codec::{Encode, EncodeLen, Decode, DecodeLen, BytesReader, ReadError};
use crate::codec::refine_context::RefineContext;
use crate::codec::work_result::WorkResult;

// A work-report, of the set W, is defined as a tuple of the work-package specification, the
// refinement context, and the core-index (i.e. on which the work is done) as well as the 
// authorizer hash and output, a segment-root lookup dictionary, and finally the results of 
// the evaluation of each of the items in the package, which is always at least one item and 
// may be no more than I items.

pub struct WorkReport {
    package_spec: WorkPackageSpec,
    context: RefineContext,
    core_index: CoreIndex,
    authorizer_hash: OpaqueHash,
    auth_output: Vec<u8>,
    results: Vec<WorkResult>,
}

pub struct WorkPackageSpec {
    hash: OpaqueHash,
    len: u32,
    erasure_root: OpaqueHash,
    exports_root: OpaqueHash,
}

impl WorkReport {

    pub fn decode(work_report: &mut BytesReader) -> Result<Self, ReadError> {
        
        Ok(WorkReport {
            package_spec: WorkPackageSpec {
                hash: OpaqueHash::decode(work_report)?,
                len: u32::decode(work_report)?,
                erasure_root: OpaqueHash::decode(work_report)?,
                exports_root: OpaqueHash::decode(work_report)?,
            },
            context: RefineContext::decode(work_report)?,
            core_index: CoreIndex::decode(work_report)?,
            authorizer_hash: OpaqueHash::decode(work_report)?,
            auth_output: Vec::<u8>::decode_len(work_report)?,
            results: WorkResult::decode_len(work_report)?,
        })
    }
    

    pub fn encode(&self) -> Vec<u8> {
        let mut work_report_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<WorkReport>());
        
        self.package_spec.hash.encode_to(&mut work_report_blob);
        self.package_spec.len.encode_to(&mut work_report_blob);
        self.package_spec.erasure_root.encode_to(&mut work_report_blob);
        self.package_spec.exports_root.encode_to(&mut work_report_blob);
        self.context.encode_to(&mut work_report_blob);
        self.core_index.encode_to(&mut work_report_blob);
        self.authorizer_hash.encode_to(&mut work_report_blob);
        self.auth_output.as_slice().encode_len().encode_to(&mut work_report_blob);
        WorkResult::encode_len(&self.results).encode_to(&mut work_report_blob);

        return work_report_blob;
    }

    pub fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}
