use crate::types::{CoreIndex, OpaqueHash};
use crate::codec::{Encode, EncodeLen, Decode, DecodeLen, BytesReader, ReadError};
use crate::codec::{encode_unsigned, decode_unsigned};
use crate::codec::refine_context::RefineContext;
use crate::codec::work_result::WorkResult;

// A work-report, of the set W, is defined as a tuple of the work-package specification, the
// refinement context, and the core-index (i.e. on which the work is done) as well as the 
// authorizer hash and output, a segment-root lookup dictionary, and finally the results of 
// the evaluation of each of the items in the package, which is always at least one item and 
// may be no more than I items.
#[derive(Debug, Clone, PartialEq)]
pub struct WorkReport {
    package_spec: WorkPackageSpec,
    context: RefineContext,
    core_index: CoreIndex,
    authorizer_hash: OpaqueHash,
    auth_output: Vec<u8>,
    segment_root_lookup: SegmentRootLookup,
    results: Vec<WorkResult>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorkPackageSpec {
    hash: OpaqueHash,
    length: u32,
    erasure_root: OpaqueHash,
    exports_root: OpaqueHash,
    exports_count: u16,
}

#[derive(Debug, Clone, PartialEq)]
struct SegmentRootLookupItem {
    work_package_hash: OpaqueHash,
    segment_tree_root: OpaqueHash,
}

impl Encode for SegmentRootLookupItem {

    fn encode(&self) -> Vec<u8> {
        
        let mut item = Vec::with_capacity(std::mem::size_of::<SegmentRootLookupItem>());
        self.work_package_hash.encode_to(&mut item);
        self.segment_tree_root.encode_to(&mut item);
        
        return item;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for SegmentRootLookupItem {

    fn decode(item: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(SegmentRootLookupItem {
            work_package_hash: OpaqueHash::decode(item)?,
            segment_tree_root: OpaqueHash::decode(item)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
struct SegmentRootLookup {
    segment_root_lookup: Vec<SegmentRootLookupItem>,
}

impl Encode for SegmentRootLookup {
    
    fn encode(&self) -> Vec<u8> {

        let len = self.segment_root_lookup.len();
        let mut segment_root_lookup = Vec::with_capacity(len);
        encode_unsigned(len).encode_to(&mut segment_root_lookup);

        for item in &self.segment_root_lookup {
            item.encode_to(&mut segment_root_lookup);
        }

        return segment_root_lookup;
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for SegmentRootLookup {
    fn decode(segment: &mut BytesReader) -> Result<Self, ReadError> {
        let len = decode_unsigned(segment)?;
        let mut segment_root_lookup = SegmentRootLookup {
            segment_root_lookup: Vec::with_capacity(len),
        };

        for _ in 0..len {
            segment_root_lookup
                .segment_root_lookup
                .push(SegmentRootLookupItem::decode(segment)?);
        }

        Ok(segment_root_lookup)
    }
}

impl Encode for WorkReport {

    fn encode(&self) -> Vec<u8> {
        let mut work_report_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<WorkReport>());
        
        self.package_spec.hash.encode_to(&mut work_report_blob);
        self.package_spec.length.encode_to(&mut work_report_blob);
        self.package_spec.erasure_root.encode_to(&mut work_report_blob);
        self.package_spec.exports_root.encode_to(&mut work_report_blob);
        self.package_spec.exports_count.encode_to(&mut work_report_blob);
        self.context.encode_to(&mut work_report_blob);
        self.core_index.encode_to(&mut work_report_blob);
        self.authorizer_hash.encode_to(&mut work_report_blob);
        self.auth_output.as_slice().encode_len().encode_to(&mut work_report_blob);
        self.segment_root_lookup.encode_to(&mut work_report_blob);
        WorkResult::encode_len(&self.results).encode_to(&mut work_report_blob);

        return work_report_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for WorkReport {

    fn decode(work_report: &mut BytesReader) -> Result<Self, ReadError> {
        
        Ok(WorkReport {
            package_spec: WorkPackageSpec {
                hash: OpaqueHash::decode(work_report)?,
                length: u32::decode(work_report)?,
                erasure_root: OpaqueHash::decode(work_report)?,
                exports_root: OpaqueHash::decode(work_report)?,
                exports_count: u16::decode(work_report)?,
            },
            context: RefineContext::decode(work_report)?,
            core_index: CoreIndex::decode(work_report)?,
            authorizer_hash: OpaqueHash::decode(work_report)?,
            auth_output: Vec::<u8>::decode_len(work_report)?,
            segment_root_lookup: SegmentRootLookup::decode(work_report)?,
            results: WorkResult::decode_len(work_report)?,
        })
    }
}

