use crate::jam_types::{
    OpaqueHash, Ed25519Public, WorkPackageHash, RefineContext, WorkResult, SegmentRootLookupItem, WorkReport, WorkPackageSpec, 
    ReportedPackage, OutputDataReports, Gas
};
use crate::utils::codec::{Encode, EncodeLen, Decode, DecodeLen, BytesReader, ReadError};
use crate::utils::codec::generic::{encode_unsigned, decode_unsigned};

impl Encode for WorkReport {

    fn encode(&self) -> Vec<u8> {

        let mut work_report_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<WorkReport>());
        
        self.package_spec.hash.encode_to(&mut work_report_blob);
        self.package_spec.length.encode_to(&mut work_report_blob);
        self.package_spec.erasure_root.encode_to(&mut work_report_blob);
        self.package_spec.exports_root.encode_to(&mut work_report_blob);
        self.package_spec.exports_count.encode_to(&mut work_report_blob);
        self.context.encode_to(&mut work_report_blob);
        encode_unsigned(self.core_index as usize).encode_to(&mut work_report_blob);
        self.authorizer_hash.encode_to(&mut work_report_blob);
        self.auth_output.encode_len().encode_to(&mut work_report_blob);
        self.segment_root_lookup.encode_len().encode_to(&mut work_report_blob);
        self.results.encode_len().encode_to(&mut work_report_blob);
        encode_unsigned(self.auth_gas_used as usize).encode_to(&mut work_report_blob);
        
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
            core_index: decode_unsigned(work_report)? as u16,
            authorizer_hash: OpaqueHash::decode(work_report)?,
            auth_output: Vec::<u8>::decode_len(work_report)?,
            segment_root_lookup: Vec::<SegmentRootLookupItem>::decode_len(work_report)?,
            results: Vec::<WorkResult>::decode_len(work_report)?,
            auth_gas_used: decode_unsigned(work_report)? as Gas,
        })
    }
}

impl Encode for ReportedPackage {

    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::with_capacity(std::mem::size_of::<Self>());

        self.work_package_hash.encode_to(&mut blob);
        self.segment_tree_root.encode_to(&mut blob);

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for ReportedPackage {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(ReportedPackage{
            work_package_hash: WorkPackageHash::decode(blob)?,
            segment_tree_root: OpaqueHash::decode(blob)?,
        })
    }
}

impl Encode for OutputDataReports {

    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::with_capacity(std::mem::size_of::<ReportedPackage>() * self.reported.len() + std::mem::size_of::<Ed25519Public>() * self.reporters.len());

        self.reported.encode_len().encode_to(&mut blob);
        self.reporters.encode_len().encode_to(&mut blob);
        
        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for OutputDataReports {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(OutputDataReports{
            reported: Vec::<ReportedPackage>::decode_len(blob)?,
            reporters: Vec::<Ed25519Public>::decode_len(blob)?,
        })
    }
}

impl Encode for SegmentRootLookupItem {

    fn encode(&self) -> Vec<u8> {
        
        let mut item = Vec::with_capacity(std::mem::size_of::<Self>());

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

