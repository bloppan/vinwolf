use crate::types::{
    OpaqueHash, Ed25519Public, WorkPackageHash, RefineContext, WorkResult, SegmentRootLookupItem, WorkReport, WorkPackageSpec, 
    Offenders, ReportedPackage, OutputDataReports, Gas
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
        self.core_index.encode_to(&mut work_report_blob);
        self.authorizer_hash.encode_to(&mut work_report_blob);
        self.auth_output.as_slice().encode_len().encode_to(&mut work_report_blob);
        self.segment_root_lookup.encode_to(&mut work_report_blob);
        self.results.encode_to(&mut work_report_blob);
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
            core_index: u16::decode(work_report)?,
            authorizer_hash: OpaqueHash::decode(work_report)?,
            auth_output: Vec::<u8>::decode_len(work_report)?,
            segment_root_lookup: Vec::<SegmentRootLookupItem>::decode(work_report)?,
            results: Vec::<WorkResult>::decode(work_report)?,
            auth_gas_used: decode_unsigned(work_report)? as Gas,
        })
    }
}

impl Encode for Vec<WorkReport> {

    fn encode(&self) -> Vec<u8> {

        let mut work_reports_blob = Vec::with_capacity(std::mem::size_of::<WorkReport>() * self.len());

        encode_unsigned(self.len()).encode_to(&mut work_reports_blob);
        
        for work_report in self.iter() {
            work_report.encode_to(&mut work_reports_blob);
        }

        return work_reports_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for Vec<WorkReport> {
    
    fn decode(work_reports: &mut BytesReader) -> Result<Self, ReadError> {

        let len = decode_unsigned(work_reports)?;
        let mut work_reports_vec = Vec::with_capacity(len);

        for _ in 0..len {
            work_reports_vec.push(WorkReport::decode(work_reports)?);
        }

        Ok(work_reports_vec)
    }
}

/*impl Encode for Offenders {

    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::with_capacity(std::mem::size_of::<Self>() * self.len());

        encode_unsigned(self.len()).encode_to(&mut blob);

        for offender in self.iter() {
            offender.encode_to(&mut blob);
        }

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}*/

impl Decode for Offenders {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        let num_offenders = decode_unsigned(blob)?;
        let mut offenders = Vec::with_capacity(num_offenders);
        for _ in 0..num_offenders {
            offenders.push(Ed25519Public::decode(blob)?);
        }

        Ok( offenders )
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

        let blob_size = std::mem::size_of::<ReportedPackage>() * self.reported.len() + std::mem::size_of::<Ed25519Public>() * self.reporters.len(); 
        let mut blob = Vec::with_capacity(blob_size);

        encode_unsigned(self.reported.len()).encode_to(&mut blob);

        for package in &self.reported {
            package.encode_to(&mut blob);
        }
        
        encode_unsigned(self.reporters.len()).encode_to(&mut blob);

        for reporter in &self.reporters {
            reporter.encode_to(&mut blob);
        }
        
        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for OutputDataReports {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(OutputDataReports{
            reported: {
                let reported_len = decode_unsigned(blob)?;
                let mut reported = Vec::with_capacity(std::mem::size_of::<ReportedPackage>() * reported_len);
                for _ in 0..reported_len {
                    reported.push(ReportedPackage::decode(blob)?);
                }
                reported
            },
            reporters: {
                let reporters_len = decode_unsigned(blob)?;
                let mut reporters = Vec::with_capacity(std::mem::size_of::<Ed25519Public>() * reporters_len);
                for _ in 0..reporters_len {
                    reporters.push(Ed25519Public::decode(blob)?);
                }
                reporters
            }
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

impl Encode for Vec<SegmentRootLookupItem> {
    
    fn encode(&self) -> Vec<u8> {

        let len = self.len();
        let mut segment_root_lookup = Vec::with_capacity(len);
        encode_unsigned(len).encode_to(&mut segment_root_lookup);

        for item in self.iter() {
            item.encode_to(&mut segment_root_lookup);
        }

        return segment_root_lookup;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for Vec<SegmentRootLookupItem> {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        let len = decode_unsigned(blob)?;
        let mut segment_root_lookup =  Vec::with_capacity(len);

        for _ in 0..len {
            segment_root_lookup.push(SegmentRootLookupItem::decode(blob)?);
        }

        Ok(segment_root_lookup)
    }
}

