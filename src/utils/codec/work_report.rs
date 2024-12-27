use crate::types::{
    CoreIndex, OpaqueHash, Ed25519Public, WorkPackageHash, RefineContext, WorkResult, SegmentRootLookupItem, 
    SegmentRootLookup, WorkReport, WorkPackageSpec, Offenders
};
use crate::utils::codec::{Encode, EncodeLen, Decode, DecodeLen, BytesReader, ReadError};
use crate::utils::codec::{encode_unsigned, decode_unsigned};

impl Encode for Offenders {

    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::with_capacity(std::mem::size_of::<Self>() * self.0.len());

        encode_unsigned(self.0.len()).encode_to(&mut blob);

        for offender in &self.0 {
            offender.encode_to(&mut blob);
        }

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for Offenders {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(Offenders {
            0: {
                let num_offenders = decode_unsigned(blob)?;
                let mut offenders = Vec::with_capacity(num_offenders);
                for _ in 0..num_offenders {
                    offenders.push(Ed25519Public::decode(blob)?);
                }
                offenders
            },
        })
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct ReportedPackage {
    pub work_package_hash: WorkPackageHash,
    pub segment_tree_root: OpaqueHash,
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

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct OutputData {
    pub reported: Vec<ReportedPackage>,
    pub reporters: Vec<Ed25519Public>,
}

impl Encode for OutputData {

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

impl Decode for OutputData {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(OutputData{
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

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum ReportErrorCode {
    BadCoreIndex = 0,
    FutureReportSlot = 1,
    ReportEpochBeforeLast = 2,
    InsufficientGuarantees = 3,
    OutOfOrderGuarantee = 4,
    NotSortedOrUniqueGuarantors = 5,
    WrongAssignment = 6,
    CoreEngaged = 7,
    AnchorNotRecent = 8,
    BadServiceId = 9,
    BadCodeHash = 10,
    DependencyMissing = 11,
    DuplicatePackage = 12,
    BadStateRoot = 13,
    BadBeefyMmrRoot = 14,
    CoreUnauthorized = 15,
    BadValidatorIndex = 16,
    WorkReportGasTooHigh = 17,
    ServiceItemGasTooLow = 18,
    TooManyDependencies = 19,
    SegmentRootLookupInvalid = 20,
    BadSignature = 21,
    WorkReportTooBig = 22,
    TooManyGuarantees = 23,
    NoAuthorization = 24,
    BadNumberCredentials = 25,
    TooOldGuarantee = 26,
    GuarantorNotFound = 27,
    LengthNotEqual = 28,
    BadLookupAnchorSlot = 29,
    NoResults = 30,
    TooManyResults = 31,
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

impl Encode for SegmentRootLookup {
    
    fn encode(&self) -> Vec<u8> {

        let len = self.0.len();
        let mut segment_root_lookup = Vec::with_capacity(len);
        encode_unsigned(len).encode_to(&mut segment_root_lookup);

        for item in &self.0 {
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
            0: Vec::with_capacity(len),
        };

        for _ in 0..len {
            segment_root_lookup.0.push(SegmentRootLookupItem::decode(segment)?);
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

