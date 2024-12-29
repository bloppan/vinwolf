
use vinwolf::utils::codec::{BytesReader, Decode, Encode, ReadError};
use vinwolf::types::{
    TimeSlot, GuaranteesExtrinsic, AvailabilityAssignments, EntropyPool, BlockHistory, AuthPools, ValidatorsData,
    Services, Offenders
};
use vinwolf::utils::codec::jam::work_report::{OutputDataReports, ReportErrorCode};

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

#[derive(Debug, Clone, PartialEq)]
pub struct WorkReportState {

    pub avail_assignments: AvailabilityAssignments,
    pub curr_validators: ValidatorsData,
    pub prev_validators: ValidatorsData,
    pub entropy: EntropyPool,
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
            entropy: EntropyPool::decode(blob)?,
            offenders: Offenders::decode(blob)?,
            recent_blocks: BlockHistory::decode(blob)?,
            auth_pools: AuthPools::decode(blob)?,
            services: Services::decode(blob)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum OutputWorkReport {
    Ok(OutputDataReports),
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
            Ok(OutputWorkReport::Ok(OutputDataReports::decode(output_blob)?))
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