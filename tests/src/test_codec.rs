use jam_types::{
    AccumulatedHistory, Assurance, AuthPools, AuthQueues, AvailabilityAssignments, BandersnatchRingCommitment, RecentBlocks, CodeAuthorizers, CoresStatistics, 
    DisputesErrorCode, DisputesRecords, Ed25519Public, Entropy, EntropyPool, Guarantee, HeaderHash, OpaqueHash, OutputDataDisputes, OutputDataReports, 
    Privileges, ReadError, ReadyQueue, ReportErrorCode, ReportedWorkPackage, ServiceId, ServiceInfo, Services, ServicesStatistics, 
    ServicesStatisticsMapEntry, TicketBody, TicketsOrKeys, TimeSlot, ValidatorIndex, ValidatorStatistics, ValidatorsData, WorkPackageHash, WorkReport,
    Preimage, Ticket, Extrinsic
};
use codec::{Encode, EncodeLen, Decode, DecodeLen, BytesReader};
use crate::test_types::{
    InputAuthorizations, StateAuthorizations, InputAssurances, StateAssurances, DisputesState, OutputDisputes, InputHistory, InputPreimages, AccountsMapEntry,
    PreimagesState, AccountTest, LookupMetaMapEntry, LookupMetaMapKeyTest, InputWorkReport, WorkReportState, OutputWorkReport, InputSafrole, SafroleState,
    InputStatistics, StateStatistics, AccountAccTest, StorageMapEntry, StateAccumulate, InputAccumulate, AccountsAccMapEntry, PreimagesMapEntry
};

// ----------------------------------------------------------------------------------------------------------
// Authorizations
// ----------------------------------------------------------------------------------------------------------
impl Encode for InputAuthorizations {
    fn encode(&self) -> Vec<u8> {
        let mut input = Vec::with_capacity(std::mem::size_of::<InputAuthorizations>());
        self.slot.encode_to(&mut input);
        self.auths.encode_to(&mut input);
        return input;
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for InputAuthorizations {
    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(InputAuthorizations {
            slot: TimeSlot::decode(reader)?,
            auths: CodeAuthorizers::decode(reader)?,
        })
    }
}

impl Encode for StateAuthorizations {
    fn encode(&self) -> Vec<u8> {
        let mut state = Vec::with_capacity(std::mem::size_of::<StateAuthorizations>());
        self.auth_pools.encode_to(&mut state);
        self.auth_queues.encode_to(&mut state);
        return state;
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for StateAuthorizations {
    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(StateAuthorizations {
            auth_pools: AuthPools::decode(blob)?,
            auth_queues: AuthQueues::decode(blob)?,
        })
    }
}
// ----------------------------------------------------------------------------------------------------------
// Authorizations
// ----------------------------------------------------------------------------------------------------------
impl Encode for InputAssurances {
    fn encode(&self) -> Vec<u8> {
        let mut input = Vec::with_capacity(std::mem::size_of::<InputAssurances>());
        self.assurances.encode_len().encode_to(&mut input);
        self.slot.encode_to(&mut input);
        self.parent.encode_to(&mut input);
        return input;
    }
    fn encode_to(&self, writer: &mut Vec<u8>) {
        writer.extend_from_slice(&self.encode());
    }
}

impl Decode for InputAssurances {
    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        
        Ok(InputAssurances {
            assurances: Vec::<Assurance>::decode_len(reader)?,
            slot: TimeSlot::decode(reader)?,
            parent: HeaderHash::decode(reader)?,
        })
    }
}

impl Encode for StateAssurances {
    fn encode(&self) -> Vec<u8> {
        let mut state = Vec::with_capacity(std::mem::size_of::<StateAssurances>());
        self.avail_assignments.encode_to(&mut state);
        self.curr_validators.encode_to(&mut state);
        return state;
    }
    fn encode_to(&self, writer: &mut Vec<u8>) {
        writer.extend_from_slice(&self.encode());
    }
}

impl Decode for StateAssurances {
    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        
        Ok(StateAssurances {
            avail_assignments: AvailabilityAssignments::decode(reader)?,
            curr_validators: ValidatorsData::decode(reader)?,
        })
    }
}
// ----------------------------------------------------------------------------------------------------------
// Disputes
// ----------------------------------------------------------------------------------------------------------
impl Encode for DisputesState {

    fn encode(&self) -> Vec<u8> {

        let mut state_blob = Vec::with_capacity(std::mem::size_of::<Self>());

        self.psi.encode_to(&mut state_blob);
        self.rho.encode_to(&mut state_blob);
        self.tau.encode_to(&mut state_blob);
        self.kappa.encode_to(&mut state_blob);
        self.lambda.encode_to(&mut state_blob);

        return state_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for DisputesState {

    fn decode(state_blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(DisputesState{
            psi: DisputesRecords::decode(state_blob)?,
            rho: AvailabilityAssignments::decode(state_blob)?,
            tau: TimeSlot::decode(state_blob)?,
            kappa: ValidatorsData::decode(state_blob)?,
            lambda: ValidatorsData::decode(state_blob)?,
        })
    }
}

impl Encode for OutputDisputes {

    fn encode(&self) -> Vec<u8> {

        let mut output_blob: Vec<u8> = Vec::new();

        match self {
            OutputDisputes::Ok(output_data) => {
                output_blob.push(0); // 0 = OK
                output_data.encode_to(&mut output_blob);
            }
            OutputDisputes::Err(error_code) => {
                output_blob.push(1); // 1 = ERROR
                output_blob.push(*error_code as u8); 
            }
        }

        return output_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for OutputDisputes {

    fn decode(output_blob: &mut BytesReader) -> Result<Self, ReadError> {

        let result = output_blob.read_byte()?;
        if result == 0 {
            Ok(OutputDisputes::Ok(OutputDataDisputes::decode(output_blob)?))  
        } else if result == 1 {
            let error_type = output_blob.read_byte()?;
            let error = match error_type {
                0 => DisputesErrorCode::AlreadyJudged,
                1 => DisputesErrorCode::BadVoteSplit,
                2 => DisputesErrorCode::VerdictsNotSortedUnique,
                3 => DisputesErrorCode::JudgementsNotSortedUnique,
                4 => DisputesErrorCode::CulpritsNotSortedUnique,
                5 => DisputesErrorCode::FaultsNotSortedUnique,
                6 => DisputesErrorCode::NotEnoughCulprits,
                7 => DisputesErrorCode::NotEnoughFaults,
                8 => DisputesErrorCode::CulpritsVerdictNotBad,
                9 => DisputesErrorCode::FaultVerdictWrong,
                10 => DisputesErrorCode::OffenderAlreadyReported,
                11 => DisputesErrorCode::BadJudgementAge,
                12 => DisputesErrorCode::BadValidatorIndex,
                13 => DisputesErrorCode::BadSignature,
                14 => DisputesErrorCode::BadGuarantoorKey,
                15 => DisputesErrorCode::BadAuditorKey,
                16 => DisputesErrorCode::NoVerdictsFound,
                17 => DisputesErrorCode::AgesNotEqual,
                18 => DisputesErrorCode::CulpritKeyNotFound,
                19 => DisputesErrorCode::FaultKeyNotFound,
                _ => return Err(ReadError::InvalidData),
            };
            Ok(OutputDisputes::Err(error))
        } else {
            return Err(ReadError::InvalidData);
        }
    }
}
// ----------------------------------------------------------------------------------------------------------
// Block History
// ----------------------------------------------------------------------------------------------------------
impl Encode for InputHistory {

    fn encode(&self) -> Vec<u8> {

        let mut input_blob = Vec::with_capacity(std::mem::size_of::<Self>());

        self.header_hash.encode_to(&mut input_blob);
        self.parent_state_root.encode_to(&mut input_blob);
        self.accumulate_root.encode_to(&mut input_blob);
        self.work_packages.encode_len().encode_to(&mut input_blob);

        return input_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for InputHistory {

    fn decode(input_blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(InputHistory {
            header_hash: OpaqueHash::decode(input_blob)?,
            parent_state_root: OpaqueHash::decode(input_blob)?,
            accumulate_root: OpaqueHash::decode(input_blob)?,
            work_packages: Vec::<ReportedWorkPackage>::decode_len(input_blob)?,
        })
    }
}
// ----------------------------------------------------------------------------------------------------------
// Preimages
// ----------------------------------------------------------------------------------------------------------
impl Encode for InputPreimages {
    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::new();
        
        self.preimages.encode_len().encode_to(&mut blob);
        self.slot.encode_to(&mut blob);

        return blob;
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for InputPreimages {
    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(InputPreimages { 
            preimages: Vec::<Preimage>::decode_len(reader)?,
            slot: TimeSlot::decode(reader)?,
        })
    }
}

impl Default for PreimagesState {
    fn default() -> Self {
        Self {
            accounts: Vec::new(),
            statistics: Vec::new(),
        }
    }
}

impl Encode for PreimagesState {
    fn encode(&self) -> Vec<u8> {
        let mut blob = Vec::new();

        self.accounts.encode_len().encode_to(&mut blob);
        self.statistics.encode_len().encode_to(&mut blob);
        
        return blob;
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for PreimagesState {
    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(PreimagesState { 
            accounts: Vec::<AccountsMapEntry>::decode_len(blob)?,
            statistics: Vec::<ServicesStatisticsMapEntry>::decode_len(blob)?,
        })
    }
}

impl Encode for PreimagesMapEntry {

    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::new();

        self.hash.encode_to(&mut blob);
        self.blob.encode_len().encode_to(&mut blob);
        
        return blob;
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for PreimagesMapEntry {
    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(PreimagesMapEntry { 
            hash: HeaderHash::decode(reader)?,
            blob: Vec::<u8>::decode_len(reader)?,
        })
    }
}

impl Encode for AccountsMapEntry {
    fn encode(&self) -> Vec<u8> {
        let mut blob = Vec::new();
        self.id.encode_to(&mut blob);
        self.data.encode_to(&mut blob);
        return blob;
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for AccountsMapEntry {
    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(AccountsMapEntry { 
            id: ServiceId::decode(blob)?,
            data: AccountTest::decode(blob)?,
        })
    }
}

impl Encode for AccountTest {
    fn encode(&self) -> Vec<u8> {
        let mut blob = Vec::new();

        self.preimages.encode_len().encode_to(&mut blob);
        self.lookup_meta.encode_len().encode_to(&mut blob);

        return blob;
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for AccountTest {
    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(AccountTest { 
            preimages: Vec::<PreimagesMapEntry>::decode_len(blob)?,
            lookup_meta: Vec::<LookupMetaMapEntry>::decode_len(blob)?,
        })
    }
}

impl Encode for LookupMetaMapKeyTest {
    fn encode(&self) -> Vec<u8> {
        let mut blob = Vec::new();
        self.hash.encode_to(&mut blob);
        self.length.encode_to(&mut blob);
        return blob;
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for LookupMetaMapKeyTest {
    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(LookupMetaMapKeyTest { 
            hash: HeaderHash::decode(reader)?,
            length: u32::decode(reader)?,
        })
    }
}

impl Encode for LookupMetaMapEntry {
    fn encode(&self) -> Vec<u8> {
        let mut blob = Vec::new();
        
        self.key.encode_to(&mut blob);
        self.value.encode_len().encode_to(&mut blob);

        return blob;
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for LookupMetaMapEntry {
    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(LookupMetaMapEntry { 
            key: LookupMetaMapKeyTest::decode(reader)?,
            value: Vec::<TimeSlot>::decode(reader)?,
        })
    }
}
// ----------------------------------------------------------------------------------------------------------
// Reports
// ----------------------------------------------------------------------------------------------------------
impl Encode for InputWorkReport {

    fn encode(&self) -> Vec<u8> {
        let mut blob = Vec::with_capacity(std::mem::size_of::<InputWorkReport>());
        self.guarantees.encode_len().encode_to(&mut blob);
        self.slot.encode_to(&mut blob);
        self.known_packages.encode_len().encode_to(&mut blob);
        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for InputWorkReport {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(InputWorkReport{
            guarantees: Vec::<Guarantee>::decode_len(blob)?,
            slot: TimeSlot::decode(blob)?,
            known_packages: Vec::<WorkPackageHash>::decode_len(blob)?,
        })
    }
}

impl Encode for WorkReportState {

    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::with_capacity(std::mem::size_of::<Self>());

        self.avail_assignments.encode_to(&mut blob);
        self.curr_validators.encode_to(&mut blob);
        self.prev_validators.encode_to(&mut blob);
        self.entropy.encode_to(&mut blob);
        self.offenders.encode_len().encode_to(&mut blob);
        self.recent_blocks.encode_to(&mut blob);
        self.auth_pools.encode_to(&mut blob);
        self.services.encode_to(&mut blob);
        self.cores_statistics.encode_to(&mut blob);
        self.services_statistics.encode_to(&mut blob);

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for WorkReportState {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(WorkReportState{
            avail_assignments: {
                let avail = AvailabilityAssignments::decode(blob)?;
                //println!("avail: {:x?}", avail);
                avail
            },
            curr_validators: ValidatorsData::decode(blob)?,
            prev_validators: ValidatorsData::decode(blob)?,
            entropy: EntropyPool::decode(blob)?,
            offenders: Vec::<Ed25519Public>::decode_len(blob)?,
            recent_blocks: RecentBlocks::decode(blob)?,
            auth_pools: {
                let auth_pools = AuthPools::decode(blob)?;
                //println!("\nAuth pools: {:x?}", auth_pools);
                auth_pools
            },
            services: {
                let services = Services::decode(blob)?;
                //println!("\nServices: {:x?}", services);
                services
            },
            cores_statistics: {
                let core_statistics = CoresStatistics::decode(blob)?;
                //println!("\nCore statistics: {:?}", core_statistics);
                core_statistics
            },
            services_statistics: {
                let services_statistics = ServicesStatistics::decode(blob)?;
                //println!("\nServices statistics: {:?}", services_statistics);
                services_statistics
            },
        })
    }
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
// ----------------------------------------------------------------------------------------------------------
// Safrole
// ----------------------------------------------------------------------------------------------------------
impl Encode for InputSafrole {

    fn encode(&self) -> Vec<u8> {

        let mut input_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<InputSafrole>());
        self.slot.encode_to(&mut input_blob);
        self.entropy.encode_to(&mut input_blob);
        self.tickets_extrinsic.encode_len().encode_to(&mut input_blob);

        return input_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for InputSafrole {

    fn decode(input_blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(InputSafrole {
            slot: TimeSlot::decode(input_blob)?,
            entropy: Entropy::decode(input_blob)?,
            tickets_extrinsic: Vec::<Ticket>::decode_len(input_blob)?,
        })
    }
}

impl Encode for SafroleState {

    fn encode(&self) -> Vec<u8> {

        let mut state_encoded = Vec::new();

        self.tau.encode_to(&mut state_encoded);
        self.eta.encode_to(&mut state_encoded);
        self.lambda.encode_to(&mut state_encoded);
        self.kappa.encode_to(&mut state_encoded);
        self.gamma_k.encode_to(&mut state_encoded);
        self.iota.encode_to(&mut state_encoded);
        self.gamma_a.encode_len().encode_to(&mut state_encoded);
        self.gamma_s.encode_to(&mut state_encoded);
        self.gamma_z.encode_to(&mut state_encoded);
        self.post_offenders.encode_len().encode_to(&mut state_encoded);

        return state_encoded;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for SafroleState {

    fn decode(state_blob: &mut BytesReader) -> Result<Self, ReadError> {
        
        Ok(SafroleState {
            tau: TimeSlot::decode(state_blob)?, 
            eta: EntropyPool::decode(state_blob)?,
            lambda: ValidatorsData::decode(state_blob)?,
            kappa: ValidatorsData::decode(state_blob)?,
            gamma_k: ValidatorsData::decode(state_blob)?,
            iota: ValidatorsData::decode(state_blob)?,
            gamma_a: Vec::<TicketBody>::decode_len(state_blob)?,
            gamma_s: TicketsOrKeys::decode(state_blob)?,
            gamma_z: BandersnatchRingCommitment::decode(state_blob)?,  
            post_offenders: Vec::<Ed25519Public>::decode_len(state_blob)?,
        })
    }
}
// ----------------------------------------------------------------------------------------------------------
// Statistics
// ----------------------------------------------------------------------------------------------------------
impl Encode for InputStatistics {
    fn encode(&self) -> Vec<u8> {
        let mut blob = Vec::with_capacity(std::mem::size_of::<InputStatistics>());

        self.slot.encode_to(&mut blob);
        self.author_index.encode_to(&mut blob);
        self.extrinsic.encode_to(&mut blob);

        return blob;
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for InputStatistics {
    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        let slot = TimeSlot::decode(blob)?;
        let author_index = ValidatorIndex::decode(blob)?;
        let extrinsic = Extrinsic::decode(blob)?;

        Ok(InputStatistics {
            slot,
            author_index,
            extrinsic
        })
    }
}

impl Encode for StateStatistics {
    fn encode(&self) -> Vec<u8> {
        let mut blob = Vec::with_capacity(std::mem::size_of::<Self>());

        self.curr_stats.encode_to(&mut blob);
        self.prev_stats.encode_to(&mut blob);
        self.tau.encode_to(&mut blob);
        self.curr_validators.encode_to(&mut blob);

        return blob;
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for StateStatistics {
    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(StateStatistics {
            curr_stats: ValidatorStatistics::decode(blob)?,
            prev_stats: ValidatorStatistics::decode(blob)?,
            tau: TimeSlot::decode(blob)?,
            curr_validators: ValidatorsData::decode(blob)?
        })
    }
}
// ----------------------------------------------------------------------------------------------------------
// Accumulate
// ----------------------------------------------------------------------------------------------------------
impl Encode for AccountAccTest {

    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::new();

        self.service.encode_to(&mut blob);
        self.storage.encode_len().encode_to(&mut blob);
        self.preimages.encode_len().encode_to(&mut blob);

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for AccountAccTest {
    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(AccountAccTest { 
            service: ServiceInfo::decode(blob)?,
            storage: Vec::<StorageMapEntry>::decode_len(blob)?,
            preimages: Vec::<PreimagesMapEntry>::decode_len(blob)?,
        })
    }
}

impl Encode for StorageMapEntry {

    fn encode(&self) -> Vec<u8> {
        
        let mut blob = Vec::new();

        self.key.encode_len().encode_to(&mut blob);
        self.value.encode_len().encode_to(&mut blob);

        return blob;
    }

    fn encode_to(&self, writer: &mut Vec<u8>) {
        writer.extend_from_slice(&self.encode());
    }
}

impl Decode for StorageMapEntry {

    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(Self { key: Vec::<u8>::decode_len(reader)?, value: Vec::<u8>::decode_len(reader)? })
    }
}

impl Encode for StateAccumulate {

    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::new();

        self.slot.encode_to(&mut blob);
        self.entropy.encode_to(&mut blob);
        self.ready.encode_to(&mut blob);
        self.accumulated.encode_to(&mut blob);
        self.privileges.encode_to(&mut blob);
        self.statistics.encode_to(&mut blob);
        self.accounts.encode_len().encode_to(&mut blob);
                
        return blob;
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for StateAccumulate {
    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(StateAccumulate { 
            slot: TimeSlot::decode(blob)?,
            entropy: OpaqueHash::decode(blob)?,
            ready: ReadyQueue::decode(blob)?,
            accumulated: AccumulatedHistory::decode(blob)?,
            privileges: Privileges::decode(blob)?,
            statistics: ServicesStatistics::decode(blob)?,
            accounts: Vec::<AccountsAccMapEntry>::decode_len(blob)?,
        })
    }
}

impl Encode for AccountsAccMapEntry {

    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::new();

        self.id.encode_to(&mut blob);
        self.data.encode_to(&mut blob);

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for AccountsAccMapEntry {
    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(AccountsAccMapEntry { 
            id: ServiceId::decode(blob)?,
            data: AccountAccTest::decode(blob)?,
        })
    }
}

impl Encode for InputAccumulate {
    fn encode(&self) -> Vec<u8> {
        
        let mut blob = Vec::new();

        self.slot.encode_to(&mut blob);
        self.reports.encode_len().encode_to(&mut blob);

        return blob;
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for InputAccumulate {
    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(InputAccumulate { 
            slot: TimeSlot::decode(blob)?,
            reports: Vec::<WorkReport>::decode_len(blob)?,
        })
    }
}