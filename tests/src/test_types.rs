use jam_types::{
    AccumulatedHistory, AuthPools, AuthQueues, Assurance, AvailabilityAssignments, BandersnatchRingCommitment, RecentBlocks, CodeAuthorizers, 
    CoresStatistics, DisputesErrorCode, DisputesRecords, Ed25519Public, Entropy, EntropyPool, HeaderHash, Offenders, OpaqueHash, OutputDataDisputes, 
    OutputDataReports, Privileges, ReadyQueue, ReportErrorCode, ReportedWorkPackage, ServiceId, ServiceInfo, Services, ServicesStatistics, 
    ServicesStatisticsMapEntry, TicketBody, TicketsOrKeys, TimeSlot, ValidatorIndex, ValidatorStatistics, ValidatorsData, WorkPackageHash, WorkReport,
    Guarantee, Preimage, Ticket, Extrinsic,
};

// ----------------------------------------------------------------------------------------------------------
// Authorizations
// ----------------------------------------------------------------------------------------------------------
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct InputAuthorizations {
    pub slot: TimeSlot,
    pub auths: CodeAuthorizers,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct StateAuthorizations {
    pub auth_pools: AuthPools,
    pub auth_queues: AuthQueues,
}
// ----------------------------------------------------------------------------------------------------------
// Assurances
// ----------------------------------------------------------------------------------------------------------
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct InputAssurances {
    pub assurances: Vec<Assurance>,
    pub slot: TimeSlot,
    pub parent: HeaderHash
}
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct StateAssurances {
    pub avail_assignments: AvailabilityAssignments,
    pub curr_validators: ValidatorsData
}
// ----------------------------------------------------------------------------------------------------------
// Disputes
// ----------------------------------------------------------------------------------------------------------
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct DisputesState {
    pub psi: DisputesRecords,
    pub rho: AvailabilityAssignments,
    pub tau: TimeSlot,
    pub kappa: ValidatorsData,
    pub lambda: ValidatorsData,
}
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum OutputDisputes {
    Ok(OutputDataDisputes),
    Err(DisputesErrorCode),
}
// ----------------------------------------------------------------------------------------------------------
// Block History
// ----------------------------------------------------------------------------------------------------------
#[allow(dead_code)]
#[derive(Debug)]
pub struct InputHistory {
    pub header_hash: OpaqueHash,
    pub parent_state_root: OpaqueHash,
    pub accumulate_root: OpaqueHash,
    pub work_packages: Vec<ReportedWorkPackage>,
}
// ----------------------------------------------------------------------------------------------------------
// Preimages
// ----------------------------------------------------------------------------------------------------------
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct InputPreimages {
    pub preimages: Vec<Preimage>,
    pub slot: TimeSlot,
}
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct PreimagesState {
    pub accounts: Vec<AccountsMapEntry>,
    pub statistics: Vec<ServicesStatisticsMapEntry>,
}
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct PreimagesMapEntry {
    pub hash: HeaderHash,
    pub blob: Vec<u8>,
}
impl Default for PreimagesMapEntry {
    fn default() -> Self {
        Self {
            hash: OpaqueHash::default(),
            blob: Vec::new(),
        }
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct AccountsMapEntry {
    pub id: ServiceId,
    pub data: AccountTest,
}

impl Default for AccountsMapEntry {
    fn default() -> Self {
        Self {
            id: ServiceId::default(),
            data: AccountTest { preimages: Vec::new(), lookup_meta: Vec::new() }
        }
    }   
}
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct AccountTest {
    pub preimages: Vec<PreimagesMapEntry>,
    pub lookup_meta: Vec<LookupMetaMapEntry>,
}

impl Default for AccountTest {
    fn default() -> Self {
        Self {
            preimages: Vec::new(),
            lookup_meta: Vec::new(),
        }
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LookupMetaMapKeyTest {
    pub hash: HeaderHash,
    pub length: u32,
}
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct LookupMetaMapEntry {
    pub key: LookupMetaMapKeyTest,
    pub value: Vec<TimeSlot>,
}
impl Default for LookupMetaMapEntry {
    fn default() -> Self {
        Self { key: LookupMetaMapKeyTest { hash: OpaqueHash::default(), length: u32::default() }, value: Vec::new() }    }
}
// ----------------------------------------------------------------------------------------------------------
// Reports
// ----------------------------------------------------------------------------------------------------------
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct InputWorkReport {
    pub guarantees: Vec<Guarantee>,
    pub slot: TimeSlot,
    pub known_packages: Vec<WorkPackageHash>,
}
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct WorkReportState {
    pub avail_assignments: AvailabilityAssignments,
    pub curr_validators: ValidatorsData,
    pub prev_validators: ValidatorsData,
    pub entropy: EntropyPool,
    pub offenders: Offenders,
    pub recent_blocks: RecentBlocks,
    pub auth_pools: AuthPools,
    pub services: Services,
    pub cores_statistics: CoresStatistics,
    pub services_statistics: ServicesStatistics,
}
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum OutputWorkReport {
    Ok(OutputDataReports),
    Err(ReportErrorCode),
}
// ----------------------------------------------------------------------------------------------------------
// Safrole
// ----------------------------------------------------------------------------------------------------------
#[allow(dead_code)]
#[derive(Debug)]
pub struct InputSafrole {
    pub slot: TimeSlot,
    pub entropy: Entropy,
    pub tickets_extrinsic: Vec<Ticket>,
}
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct SafroleState {
    pub tau: TimeSlot,
    pub eta: EntropyPool,
    pub lambda: ValidatorsData,
    pub kappa: ValidatorsData,
    pub gamma_k: ValidatorsData,
    pub iota: ValidatorsData,
    pub gamma_a: Vec<TicketBody>,
    pub gamma_s: TicketsOrKeys,
    pub gamma_z: BandersnatchRingCommitment,
    pub post_offenders: Vec<Ed25519Public>,
}
// ----------------------------------------------------------------------------------------------------------
// Statistics
// ----------------------------------------------------------------------------------------------------------
#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone)]
pub struct InputStatistics {
    pub slot: TimeSlot,
    pub author_index: ValidatorIndex,
    pub extrinsic: Extrinsic
}
#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone)]
pub struct StateStatistics {
    pub curr_stats: ValidatorStatistics,
    pub prev_stats: ValidatorStatistics,
    pub tau: TimeSlot,
    pub curr_validators: ValidatorsData
}
// ----------------------------------------------------------------------------------------------------------
// Accumulate
// ----------------------------------------------------------------------------------------------------------
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct InputAccumulate {
    pub slot: TimeSlot,
    pub reports: Vec<WorkReport>,
}
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct StorageMapEntry {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct AccountsAccMapEntry {
    pub id: ServiceId,
    pub data: AccountAccTest,
}

impl Default for AccountsAccMapEntry {
    fn default() -> Self {
        Self {
            id: ServiceId::default(),
            data: AccountAccTest { service: ServiceInfo::default(), storage: Vec::new(), preimages: Vec::new() }
        }
    }   
}
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct AccountAccTest {
    pub service: ServiceInfo,
    pub storage: Vec<StorageMapEntry>,
    pub preimages: Vec<PreimagesMapEntry>,
}

impl Default for StorageMapEntry {
    fn default() -> Self {
        Self { key: vec![], value: vec![] }
    }
}

impl Default for AccountAccTest {
    fn default() -> Self {
        Self {
            service: ServiceInfo::default(),
            storage: vec![],
            preimages: vec![],
        }
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct StateAccumulate {
    pub slot: TimeSlot,
    pub entropy: OpaqueHash,
    pub ready: ReadyQueue,
    pub accumulated: AccumulatedHistory,
    pub privileges: Privileges,
    pub statistics: ServicesStatistics,
    pub accounts: Vec<AccountsAccMapEntry>,
}
