use crate::types::{
    ServiceId, Gas, CoreIndex, OpaqueHash, TimeSlot, Ed25519Public, WorkPackageHash, ValidatorsData, ServiceInfo, 
    AvailabilityAssignments, RefineContext, AuthPool, AuthPools, WorkResult, SegmentRootLookupItem, SegmentRootLookup, 
    WorkReport, WorkPackageSpec, BlockHistory, GuaranteesExtrinsic
};
use crate::constants::CORES_COUNT;
use crate::utils::codec::{Encode, EncodeLen, Decode, DecodeLen, BytesReader, ReadError};
use crate::utils::codec::{encode_unsigned, decode_unsigned};

#[derive(Debug, Clone, PartialEq)]
pub struct InputWorkReport {
    pub guarantees: GuaranteesExtrinsic,
    pub slot: TimeSlot,
}

impl Encode for InputWorkReport {

    fn encode(&self) -> Vec<u8> {
        let mut blob = Vec::with_capacity(std::mem::size_of::<InputWorkReport>());
        self.guarantees.encode_to(&mut blob);
        self.slot.encode_to(&mut blob);
        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for InputWorkReport {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(InputWorkReport{
            guarantees: GuaranteesExtrinsic::decode(blob)?,
            slot: TimeSlot::decode(blob)?,
        })
    }
}

impl Encode for ServiceInfo {

    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::with_capacity(std::mem::size_of::<Self>());
        
        self.code_hash.encode_to(&mut blob);
        self.balance.encode_to(&mut blob);
        self.min_item_gas.encode_to(&mut blob);
        self.min_memo_gas.encode_to(&mut blob);
        self.bytes.encode_to(&mut blob);
        self.items.encode_to(&mut blob);
        
        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for ServiceInfo {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(ServiceInfo {
            code_hash: OpaqueHash::decode(blob)?,
            balance: u64::decode(blob)?,
            min_item_gas: Gas::decode(blob)?,
            min_memo_gas: Gas::decode(blob)?,
            bytes: u64::decode(blob)?,
            items: u32::decode(blob)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ServiceItem {
    pub id: ServiceId,
    pub info: ServiceInfo,
}

impl Encode for ServiceItem {

    fn encode(&self) -> Vec<u8> {
        
        let mut blob = Vec::with_capacity(std::mem::size_of::<Self>());
        self.id.encode_to(&mut blob);
        self.info.encode_to(&mut blob);
        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for ServiceItem {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(ServiceItem{
            id: ServiceId::decode(blob)?,
            info: ServiceInfo::decode(blob)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Services {
    pub services: Vec<ServiceItem>,
}

impl Encode for Services {

    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::with_capacity(std::mem::size_of::<Self>() * self.services.len());

        encode_unsigned(self.services.len()).encode_to(&mut blob);

        for item in &self.services {
            item.encode_to(&mut blob);
        }

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for Services {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        let len = decode_unsigned(blob)?;
        let mut services = Vec::with_capacity(std::mem::size_of::<Self>() * len);

        for _ in 0..len {
            services.push(ServiceItem::decode(blob)?);
        }

        Ok(Services{services: services})
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Offenders {
    pub offenders: Vec<Ed25519Public>,
}

impl Encode for Offenders {

    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::with_capacity(std::mem::size_of::<Self>() * self.offenders.len());

        encode_unsigned(self.offenders.len()).encode_to(&mut blob);

        for offender in &self.offenders {
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
            offenders: {
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

#[derive(Debug, Clone, PartialEq)]
pub struct WorkReportState {

    pub avail_assignments: AvailabilityAssignments,
    pub curr_validators: ValidatorsData,
    pub prev_validators: ValidatorsData,
    pub entropy: Box<[OpaqueHash; 4]>,
    pub offenders: Offenders,
    pub recent_blocks: BlockHistory,
    pub auth_pools: AuthPools,
    pub services: Services,
}

impl Encode for WorkReportState {

    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::with_capacity(std::mem::size_of::<Self>());

        self.avail_assignments.encode_to(&mut blob);
        self.curr_validators.encode_to(&mut blob);
        self.prev_validators.encode_to(&mut blob);
        self.entropy.encode_to(&mut blob);
        self.offenders.encode_to(&mut blob);
        self.recent_blocks.encode_to(&mut blob);
        self.auth_pools.encode_to(&mut blob);
        self.services.encode_to(&mut blob);

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for WorkReportState {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(WorkReportState{
            avail_assignments: AvailabilityAssignments::decode(blob)?,
            curr_validators: ValidatorsData::decode(blob)?,
            prev_validators: ValidatorsData::decode(blob)?,
            entropy: Box::new(<[OpaqueHash; 4]>::decode(blob)?),
            offenders: Offenders::decode(blob)?,
            recent_blocks: BlockHistory::decode(blob)?,
            auth_pools: AuthPools::decode(blob)?,
            services: Services::decode(blob)?,
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

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum OutputWorkReport {
    Ok(OutputData),
    Err(ReportErrorCode),
}

impl Encode for OutputWorkReport {

    fn encode(&self) -> Vec<u8> {

        let mut output_blob = Vec::new();

        match self {
            OutputWorkReport::Ok(output_data) => {
                output_blob.push(0);   // OK
                output_data.encode_to(&mut output_blob);
            }
            OutputWorkReport::Err(error_code) => {
                output_blob.push(1);   // ERROR
                output_blob.push(*error_code as u8);
            }
        }

        return output_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for OutputWorkReport {

    fn decode(output_blob: &mut BytesReader) -> Result<Self, ReadError> {

        let result = output_blob.read_byte()?;
        if result == 0 {
            Ok(OutputWorkReport::Ok(OutputData::decode(output_blob)?))
        } else if result == 1 {
            let error_type = output_blob.read_byte()?;
            let error = match error_type {
                0 => ReportErrorCode::BadCoreIndex,
                1 => ReportErrorCode::FutureReportSlot,
                2 => ReportErrorCode::ReportEpochBeforeLast,
                3 => ReportErrorCode::InsufficientGuarantees,
                4 => ReportErrorCode::OutOfOrderGuarantee,
                5 => ReportErrorCode::NotSortedOrUniqueGuarantors,
                6 => ReportErrorCode::WrongAssignment,
                7 => ReportErrorCode::CoreEngaged,
                8 => ReportErrorCode::AnchorNotRecent,
                9 => ReportErrorCode::BadServiceId,
                10 => ReportErrorCode::BadCodeHash,
                11 => ReportErrorCode::DependencyMissing,
                12 => ReportErrorCode::DuplicatePackage,
                13 => ReportErrorCode::BadStateRoot,
                14 => ReportErrorCode::BadBeefyMmrRoot,
                15 => ReportErrorCode::CoreUnauthorized,
                16 => ReportErrorCode::BadValidatorIndex,
                17 => ReportErrorCode::WorkReportGasTooHigh,
                18 => ReportErrorCode::ServiceItemGasTooLow,
                19 => ReportErrorCode::TooManyDependencies,
                20 => ReportErrorCode::SegmentRootLookupInvalid,
                21 => ReportErrorCode::BadSignature,
                22 => ReportErrorCode::WorkReportTooBig,
                23 => ReportErrorCode::TooManyGuarantees,
                24 => ReportErrorCode::NoAuthorization,
                25 => ReportErrorCode::BadNumberCredentials,
                26 => ReportErrorCode::TooOldGuarantee,
                27 => ReportErrorCode::GuarantorNotFound,
                28 => ReportErrorCode::LengthNotEqual,    
                29 => ReportErrorCode::BadLookupAnchorSlot,     
                30 => ReportErrorCode::NoResults,
                31 => ReportErrorCode::TooManyResults,               
                _ => return Err(ReadError::InvalidData),
            };
            Ok(OutputWorkReport::Err(error))
        } else {
            return Err(ReadError::InvalidData);
        }
    }
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

