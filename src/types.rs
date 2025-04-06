// JAM Protocol Types
use std::collections::{HashMap, HashSet, VecDeque};
use serde::Deserialize;

use crate::utils::codec::ReadError;
use crate::constants::{
    ENTROPY_POOL_SIZE, VALIDATORS_COUNT, CORES_COUNT, AVAIL_BITFIELD_BYTES, MAX_ITEMS_AUTHORIZATION_QUEUE, EPOCH_LENGTH,
    NUM_REG, PAGE_SIZE, SEGMENT_SIZE
};
// ----------------------------------------------------------------------------------------------------------
// Crypto
// ----------------------------------------------------------------------------------------------------------
pub type Ed25519Public = [u8; 32];
pub type BlsPublic = [u8; 144];
pub type BandersnatchPublic = [u8; 32];

pub type BandersnatchRingVrfSignature = [u8; 784];
pub type BandersnatchVrfSignature = [u8; 96];
pub type Ed25519Signature = [u8; 64];

pub type BandersnatchRingCommitment = [u8; 144];
// ----------------------------------------------------------------------------------------------------------
// Application Specific Core
// ----------------------------------------------------------------------------------------------------------
pub type OpaqueHash = [u8; 32];
pub type Metadata = [u8; 128];

pub type TimeSlot = u32;
pub type ValidatorIndex = u16;
pub type CoreIndex = u16;

pub type Hash = OpaqueHash;
pub type HeaderHash = OpaqueHash;
pub type StateRoot = OpaqueHash;
pub type BeefyRoot = OpaqueHash;
pub type WorkPackageHash = OpaqueHash;
pub type WorkReportHash = OpaqueHash;
pub type ExportsRoot = OpaqueHash;
pub type ErasureRoot = OpaqueHash;

pub type Gas = i64;
pub type RamAddress = u32;
pub type PageAddress = RamAddress;
pub type PageNumber = u32;
pub type RegSize = u64;
pub type RegSigned = i64;

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct Entropy {
    pub entropy: OpaqueHash,
}
#[derive(Debug, Clone, PartialEq)]
pub struct EntropyPool {
    pub buf: Box<[Entropy; ENTROPY_POOL_SIZE]>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Offenders(pub Vec<Ed25519Public>);
/// This is a combination of a set of cryptographic public keys and metadata which is an opaque octet sequence, 
/// but utilized to specify practical identifiers for the validator, not least a hardware address. The set of 
/// validator keys itself is equivalent to the set of 336-octet sequences. However, for clarity, we divide the
/// sequence into four easily denoted components. For any validator key k, the Bandersnatch key is is equivalent 
/// to the first 32-octets; the Ed25519 key, ke, is the second 32 octets; the bls key denoted bls is equivalent 
/// to the following 144 octets, and finally the metadata km is the last 128 octets.
#[derive(Debug, Clone, PartialEq)]
pub struct ValidatorData {
    pub bandersnatch: BandersnatchPublic,
    pub ed25519: Ed25519Public,
    pub bls: BlsPublic,
    pub metadata: Metadata,
}
pub type BandersnatchKeys = Box<[BandersnatchPublic; VALIDATORS_COUNT]>;
//pub type BandersnatchEpoch = Box<[BandersnatchPublic; EPOCH_LENGTH]>;

#[derive(Clone, PartialEq, Debug)]
pub struct BandersnatchEpoch(pub Box<[BandersnatchPublic; EPOCH_LENGTH]>);

#[derive(Clone, PartialEq, Debug)]
pub struct ValidatorsData(pub Box<[ValidatorData; VALIDATORS_COUNT]>);
#[derive(Clone, Debug, PartialEq)]
pub enum ValidatorSet {
    Previous,
    Current,
    Next,
}
// ----------------------------------------------------------------------------------------------------------
// Availability Assignments
// ----------------------------------------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq)]
pub struct AvailabilityAssignment {
    pub report: WorkReport,
    pub timeout: u32,
}

pub type AvailabilityAssignmentsItem = Option<AvailabilityAssignment>;

#[derive(Debug, Clone, PartialEq)]
pub struct AvailabilityAssignments(pub Box<[AvailabilityAssignmentsItem; CORES_COUNT]>);
// ----------------------------------------------------------------------------------------------------------
// Refine Context
// ----------------------------------------------------------------------------------------------------------
// A refinement context, denoted by the set X, describes the context of the chain at the point 
// that the report’s corresponding work-package was evaluated. It identifies two historical blocks, 
// the anchor, header hash a along with its associated posterior state-root s and posterior Beefy root b; 
// and the lookupanchor, header hash l and of timeslot t. Finally, it identifies the hash of an optional 
// prerequisite work-package p.
#[derive(Debug, Clone, PartialEq)]
pub struct RefineContext {
    pub anchor: OpaqueHash,
    pub state_root: OpaqueHash,
    pub beefy_root: OpaqueHash,
    pub lookup_anchor: OpaqueHash,
    pub lookup_anchor_slot: TimeSlot,
    pub prerequisites: Vec<OpaqueHash>,
}
// ----------------------------------------------------------------------------------------------------------
// Authorizations
// ----------------------------------------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq)]
pub struct Authorizer {
    pub code_hash: OpaqueHash,
    pub params: Vec<u8>,
}

pub type AuthorizerHash = OpaqueHash;

#[derive(Debug, Clone, PartialEq)]
pub struct AuthPool {
    pub auth_pool: VecDeque<OpaqueHash>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuthPools {
    pub auth_pools: Box<[AuthPool; CORES_COUNT]>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuthQueue {
    pub auth_queue: Box<[AuthorizerHash; MAX_ITEMS_AUTHORIZATION_QUEUE]>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuthQueues {
    pub auth_queues: Box<[AuthQueue; CORES_COUNT]>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CodeAuthorizers {
    pub authorizers: Vec<CodeAuthorizer>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CodeAuthorizer {
    pub core: CoreIndex,
    pub auth_hash: OpaqueHash,
}

// ----------------------------------------------------------------------------------------------------------
// Work Package
// ----------------------------------------------------------------------------------------------------------

// The Import Spec is a sequence of imported data segments, which identify a prior exported segment 
// through an index and the identity of an exporting work-package. Its a member of Work Item.
#[derive(Debug, Clone, PartialEq)]
pub struct ImportSpec {
    pub tree_root: OpaqueHash,
    pub index: u16,
}

// The extrinsic spec is a sequence of blob hashes and lengths to be introduced in this block 
// (and which we assume the validator knows). It's a member of Work Item
#[derive(Debug, Clone, PartialEq)]
pub struct ExtrinsicSpec {
    pub hash: OpaqueHash,
    pub len: u32,
}

// A Work Item includes: the identifier of the service to which it relates, the code hash of the service at 
// the time of reporting (whose preimage must be available from the perspective of the lookup anchor block), 
// a payload blob, a gas limit, and the three elements of its manifest, a sequence of imported data segments, 
// which identify a prior exported segment through an index and the identity of an exporting work-package, 
// a sequence of blob hashes and lengths to be introduced in this block (and which we assume the validator knows) 
// and the number of data segments exported by this work item.
#[derive(Debug, Clone, PartialEq)]
pub struct WorkItem {
    pub service: ServiceId,
    pub code_hash: OpaqueHash,
    pub payload: Vec<u8>,
    pub gas_limit: Gas,
    pub import_segments: Vec<ImportSpec>,
    pub extrinsic: Vec<ExtrinsicSpec>,
    pub export_count: u16,
}

// A work-package includes a simple blob acting as an authorization token, the index of the service which
// hosts the authorization code, an authorization code hash and a parameterization blob, a context and a 
// sequence of work items:
#[derive(Debug, Clone, PartialEq)]
pub struct WorkPackage {
    pub authorization: Vec<u8>,
    pub auth_code_host: ServiceId,
    pub authorizer: Authorizer,
    pub context: RefineContext,
    pub items: Vec<WorkItem>,
}
// ----------------------------------------------------------------------------------------------------------
// Work Report
// ----------------------------------------------------------------------------------------------------------
// A work-report, of the set W, is defined as a tuple of the work-package specification, the
// refinement context, and the core-index (i.e. on which the work is done) as well as the 
// authorizer hash and output, a segment-root lookup dictionary, and finally the results of 
// the evaluation of each of the items in the package, which is always at least one item and 
// may be no more than I items.
#[derive(Debug, Clone, PartialEq)]
pub struct WorkReport {
    pub package_spec: WorkPackageSpec,
    pub context: RefineContext,
    pub core_index: CoreIndex,
    pub authorizer_hash: OpaqueHash,
    pub auth_output: Vec<u8>,
    pub segment_root_lookup: Vec<SegmentRootLookupItem>,
    pub results: Vec<WorkResult>,
    pub auth_gas_used: Gas,
}
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct ReportedPackage {
    pub work_package_hash: WorkPackageHash,
    pub segment_tree_root: OpaqueHash,
}
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct OutputDataReports {
    pub reported: Vec<ReportedPackage>,
    pub reporters: Vec<Ed25519Public>,
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
// The Work Result is the data conduit by which services states may be altered through 
// the computation done within a work-package. 

// Work results are a tuple comprising several items. Firstly, the index of the service whose state 
// is to be altered and thus whose refine code was already executed. We include the hash of the code 
// of the service at the time of being reported, which must be accurately predicted within the 
// work-report; Next, the hash of the payload within the work item which was executed in the refine 
// stage to give this result. This has no immediate relevance, but is something provided to the 
// accumulation logic of the service. We follow with the gas prioritization ratio used when determining 
// how much gas should be allocated to execute of this item’s accumulate. Finally, there is the output 
// or error of the execution of the code, which may be either an octet sequence in case it was successful, 
// or a member of the set J (set of possible errors), if not. 
// Possible errors are:
//      Out-of-gas
//      Unexpected program termination
//      The code was not available for lookup in state at the posterior state of the lookup-anchor block.
//      The code was available but was beyond the maximun size allowed Wc.

#[derive(Debug, Clone, PartialEq)]
pub struct RefineLoad {
    pub gas_used: u64,
    pub imports: u16,
    pub extrinsic_count: u16,
    pub extrinsic_size: u32,
    pub exports: u16,
}
#[derive(Debug, Clone, PartialEq)]
pub struct WorkResult {
    pub service: ServiceId,
    pub code_hash: OpaqueHash,
    pub payload_hash: OpaqueHash,
    pub gas: Gas,
    pub result: Vec<u8>,
    pub refine_load: RefineLoad,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WorkExecResult {
    Ok(Vec<u8>),
    Error(WorkExecError),
}
#[derive(Debug, Clone, PartialEq)]
pub enum WorkExecError {
    OutOfGas = 1,
    Panic = 2,
    BadNumberExports = 3,
    BadCode = 4,
    CodeOversize = 5,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorkPackageSpec {
    pub hash: OpaqueHash,
    pub length: u32,
    pub erasure_root: OpaqueHash,
    pub exports_root: OpaqueHash,
    pub exports_count: u16,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentRootLookupItem {
    pub work_package_hash: OpaqueHash,
    pub segment_tree_root: OpaqueHash,
}
// ----------------------------------------------------------------------------------------------------------
// Block History
// ----------------------------------------------------------------------------------------------------------
pub type MmrPeak = Option<Hash>;

#[derive(Clone, Debug, PartialEq)]
pub struct Mmr{
    pub peaks: Vec<MmrPeak>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReportedWorkPackage {
    pub hash: Hash,
    pub exports_root: Hash,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReportedWorkPackages(pub Vec<ReportedWorkPackage>);

#[derive(Clone, Debug, PartialEq)]
pub struct BlockInfo {
    pub header_hash: Hash,
    pub mmr: Mmr,
    pub state_root: Hash,
    pub reported: ReportedWorkPackages,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BlockHistory {
    pub blocks: VecDeque<BlockInfo>,
}
// ----------------------------------------------------------------------------------------------------------
// Statistics
// ----------------------------------------------------------------------------------------------------------
#[derive(Clone, Debug, PartialEq, Copy)]
pub struct ActivityRecord {
    pub blocks: u32,
    pub tickets: u32,
    pub preimages: u32,
    pub preimages_size: u32,
    pub guarantees: u32,
    pub assurances: u32,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ActivityRecords {
    pub records: Box<[ActivityRecord; VALIDATORS_COUNT]>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CoreActivityRecord {
    pub gas_used: u64,        // Total gas consumed by core for reported work. Includes all refinement and authorizations.
    pub imports: u16,         // Number of segments imported from DA made by core for reported work.
    pub extrinsic_count: u16, // Total number of extrinsics used by core for reported work.
    pub extrinsic_size: u32,  // Total size of extrinsics used by core for reported work.
    pub exports: u16,         // Number of segments exported into DA made by core for reported work.
    pub bundle_size: u32,     // The work-bundle size. This is the size of data being placed into Audits DA by the core.
    pub da_load: u32,         // Amount of bytes which are placed into either Audits or Segments DA. This includes the work-bundle (including all extrinsics and imports) as well as all (exported) segments
    pub popularity: u16,      // Number of validators which formed super-majority for assurance.
}
#[derive(Clone, Debug, PartialEq)]
pub struct CoresStatistics {
    pub records: Box<[CoreActivityRecord; CORES_COUNT]>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct SeviceActivityRecord {
    pub provided_count: u16,        // Number of preimages provided to this service
    pub provided_size: u32,         // Total size of preimages provided to this service.
    pub refinement_count: u32,      // Number of work-items refined by service for reported work.
    pub refinement_gas_used: u64,   // Amount of gas used for refinement by service for reported work.
    pub imports: u32,               // Number of segments imported from the DL by service for reported work.
    pub extrinsic_count: u32,       // Total number of extrinsics used by service for reported work.
    pub extrinsic_size: u32,        // Total size of extrinsics used by service for reported work.
    pub exports: u32,               // Number of segments exported into the DL by service for reported work.
    pub accumulate_count: u32,      // Number of work-items accumulated by service.
    pub accumulate_gas_used: u64,   // Amount of gas used for accumulation by service.
    pub on_transfers_count: u32,    // Number of transfers processed by service.
    pub on_transfers_gas_used: u64, // Amount of gas used for processing transfers by service.
}
#[derive(Clone, Debug, PartialEq)]
pub struct ServicesStatisticsMapEntry {
    pub id: ServiceId,
    pub record: SeviceActivityRecord,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ServicesStatistics {
    pub records: HashMap<ServiceId, SeviceActivityRecord>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Statistics {
    pub curr: ActivityRecords,
    pub prev: ActivityRecords,
    pub cores: CoresStatistics,
    pub services: ServicesStatistics,
}
// ----------------------------------------------------------------------------------------------------------
// Tickets
// ----------------------------------------------------------------------------------------------------------
pub type TicketId = OpaqueHash;
pub type TicketAttempt = u8;

#[derive(Debug, Clone, PartialEq)]
pub struct TicketEnvelope {
    pub attempt: TicketAttempt,
    pub signature: BandersnatchRingVrfSignature,
}

#[derive(Debug, Clone, PartialEq, Ord, PartialOrd, Eq)]
pub struct TicketBody {
    pub id: OpaqueHash,
    pub attempt: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TicketsOrKeys {
    Keys(BandersnatchEpoch),
    Tickets(TicketsMark),
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TicketsExtrinsic { 
    pub tickets: Vec<TicketEnvelope>,
}
// ----------------------------------------------------------------------------------------------------------
// Safrole
// ----------------------------------------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq)]
pub struct Safrole {
    pub pending_validators: ValidatorsData,
    pub ticket_accumulator: Vec<TicketBody>,
    pub seal: TicketsOrKeys,
    pub epoch_root: BandersnatchRingCommitment,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OutputSafrole {
    Ok(OutputDataSafrole),
    Err(SafroleErrorCode),
}
#[derive(Debug, Clone, PartialEq)]
pub struct OutputDataSafrole {
    pub epoch_mark: Option<EpochMark>,
    pub tickets_mark: Option<TicketsMark>,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SafroleErrorCode {
    BadSlot = 0,           // Timeslot value must be strictly monotonic.
    UnexpectedTicket = 1,  // Received a ticket while in epoch's tail.
    BadTicketOrder = 2,   // Tickets must be sorted.
    BadTicketProof = 3,   // Invalid ticket ring proof.
    BadTicketAttempt = 4, // Invalid ticket attempt value.
    Reserved = 5,           // Reserved.
    DuplicateTicket = 6,   // Found a ticket duplicate.
    TooManyTickets = 7,    // Too many tickets in extrinsic.
    InvalidTicketSeal = 8,       // Invalid seal.
    InvalidKeySeal = 9,         // Invalid seal.
    InvalidEntropySource = 10, // Invalid entropy source.
    TicketsOrKeysNone = 11, // Tickets or keys is none.
    TicketNotMatch = 12,      // Seal does not match.
    KeyNotMatch = 13,        // Seal key does not match.
}
// ----------------------------------------------------------------------------------------------------------
// Disputes
// ----------------------------------------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq)]
pub struct OutputDataDisputes {
    pub offenders_mark: OffendersMark,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DisputesErrorCode {
    AlreadyJudged = 0,
    BadVoteSplit = 1,
    VerdictsNotSortedUnique = 2,
    JudgementsNotSortedUnique = 3,
    CulpritsNotSortedUnique = 4,
    FaultsNotSortedUnique = 5,
    NotEnoughCulprits = 6,
    NotEnoughFaults = 7,
    CulpritsVerdictNotBad = 8,
    FaultVerdictWrong = 9,
    OffenderAlreadyReported = 10,
    BadJudgementAge = 11,
    BadValidatorIndex = 12,
    BadSignature = 13,
    DisputeStateNotInitialized = 14,
    NoVerdictsFound = 15,
    AgesNotEqual = 16,
    CulpritKeyNotFound = 17,
    FaultKeyNotFound = 18,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Judgement {
    pub vote: bool,
    pub index: ValidatorIndex,
    pub signature: Ed25519Signature,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Verdict {
    pub target: OpaqueHash,
    pub age: u32,
    pub votes: Vec<Judgement>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Culprit {
    pub target: OpaqueHash,
    pub key: Ed25519Public,
    pub signature: Ed25519Signature,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Fault {
    pub target: OpaqueHash,
    pub vote: bool,
    pub key: Ed25519Public,
    pub signature: Ed25519Signature,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DisputesRecords {
    pub good: Vec<WorkReportHash>,
    pub bad: Vec<WorkReportHash>,
    pub wonky: Vec<WorkReportHash>,
    pub offenders: Vec<Ed25519Public>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DisputesExtrinsic {
    pub verdicts: Vec<Verdict>,
    pub culprits: Vec<Culprit>,
    pub faults: Vec<Fault>,
}
// ----------------------------------------------------------------------------------------------------------
// Service Accounts
// ----------------------------------------------------------------------------------------------------------
#[derive(Debug, PartialEq, Clone)]
pub struct ServiceAccounts {
    pub service_accounts: HashMap<ServiceId, Account>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    pub storage: HashMap<OpaqueHash, Vec<u8>>,
    pub preimages: HashMap<OpaqueHash, Vec<u8>>,
    pub lookup: HashMap<(OpaqueHash, u32), Vec<TimeSlot>>,
    pub code_hash: OpaqueHash,
    pub balance: u64,
    pub gas: Gas,
    pub min_gas: Gas,
    pub items: u32,
    pub bytes: u64,
}
pub type ServiceId = u32;

#[derive(Debug, Clone, PartialEq)]
pub struct ServiceInfo {
    pub code_hash: OpaqueHash,
    pub balance: u64,
    pub min_item_gas: Gas,
    pub min_memo_gas: Gas,
    pub bytes: u64,
    pub items: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ServiceItem {
    pub id: ServiceId,
    pub info: ServiceInfo,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Services(pub Vec<ServiceItem>);
// ----------------------------------------------------------------------------------------------------------
// Preimages
// ----------------------------------------------------------------------------------------------------------
#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, std::hash::Hash)]
pub struct Preimage {
    pub requester: ServiceId,
    pub blob: Vec<u8>,
}
#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, std::hash::Hash)]
pub struct PreimagesExtrinsic {
    pub preimages: Vec<Preimage>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct PreimagesMapEntry {
    pub hash: HeaderHash,
    pub blob: Vec<u8>,
}
#[derive(Debug, Clone, PartialEq)]
pub enum PreimagesErrorCode {
    PreimageUnneeded = 0,
    PreimagesNotSortedOrUnique = 1,
    RequesterNotFound = 2,
}
#[derive(Debug, Clone, PartialEq)]
pub enum OutputPreimages {
    Ok(),
    Err(PreimagesErrorCode),
}
// ----------------------------------------------------------------------------------------------------------
// Assurances
// ----------------------------------------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq)]
pub struct AssurancesExtrinsic {
    pub assurances: Vec<AvailAssurance>, 
}
#[derive(Debug, Clone, PartialEq)]
pub struct AvailAssurance {
    pub anchor: OpaqueHash,
    pub bitfield: [u8; AVAIL_BITFIELD_BYTES],
    pub validator_index: ValidatorIndex,
    pub signature: Ed25519Signature,
}
#[derive(Debug, Clone, PartialEq)]
pub struct OutputDataAssurances {
    pub reported: Vec<WorkReport>
}
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum AssurancesErrorCode {
    BadAttestationParent = 0,
    BadValidatorIndex = 1,
    CoreNotEngaged = 2,
    BadSignature = 3,
    NotSortedOrUniqueAssurers = 4,
    TooManyAssurances = 5,
    WrongBitfieldLength = 6,
}
#[derive(Debug, Clone, PartialEq)]
pub enum OutputAssurances {
    Ok(OutputDataAssurances),
    Err(AssurancesErrorCode)
}
// ----------------------------------------------------------------------------------------------------------
// Guarantees
// ----------------------------------------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq)]
pub struct ValidatorSignature {
    pub validator_index: ValidatorIndex,
    pub signature: Ed25519Signature,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReportGuarantee {
    pub report: WorkReport,
    pub slot: TimeSlot,
    pub signatures: Vec<ValidatorSignature>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GuaranteesExtrinsic {
    pub report_guarantee: Vec<ReportGuarantee>,
}
// ----------------------------------------------------------------------------------------------------------
// Accumulation
// ----------------------------------------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq)]
pub struct ReadyRecord {
    pub report: WorkReport,
    pub dependencies: Vec<WorkPackageHash>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct ReadyQueue {
    pub queue: Box<[Vec<ReadyRecord>; EPOCH_LENGTH]>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AccumulatedHistory {
    pub queue: VecDeque<Vec<WorkPackageHash>>,
}
/*#[derive(Debug, Clone, PartialEq)]
pub struct AlwaysAccumulateMapItem {
    pub id: ServiceId,
    pub gas: Gas,
}*/
#[derive(Debug, Clone, PartialEq)]
pub struct Privileges {
    pub bless: ServiceId,
    pub assign: ServiceId,
    pub designate: ServiceId,
    pub always_acc: HashMap<ServiceId, Gas>,
}

pub type AccumulateRoot = OpaqueHash;

#[derive(Debug, Clone, PartialEq)]
pub enum OutputAccumulation {
    Ok(AccumulateRoot),
    Err(),
}

// ----------------------------------------------------------------------------------------------------------
// Header
// ----------------------------------------------------------------------------------------------------------
#[derive(Debug, PartialEq, Clone)]
pub struct EpochMark {
    pub entropy: Entropy,
    pub tickets_entropy: Entropy,
    pub validators: Box<[(BandersnatchPublic, Ed25519Public); VALIDATORS_COUNT]>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TicketsMark {
    pub tickets_mark: Box<[TicketBody; EPOCH_LENGTH]>,
}

pub type OffendersMark = Vec<Ed25519Public>;

/*#[derive(Debug, PartialEq, Clone)]
pub struct Header {
    pub parent: HeaderHash,
    pub parent_state_root: StateRoot,
    pub extrinsic_hash: OpaqueHash,
    pub slot: TimeSlot,
    pub epoch_mark: Option<EpochMark>,
    pub tickets_mark: Option<TicketsMark>,
    pub offenders_mark: Vec<Ed25519Public>,
    pub author_index: ValidatorIndex,
    pub entropy_source: BandersnatchVrfSignature,
    pub seal: BandersnatchVrfSignature,
}*/
#[derive(Debug, PartialEq, Clone)]
pub struct Header {
    pub unsigned: UnsignedHeader,
    pub seal: BandersnatchVrfSignature,
}

#[derive(Debug, PartialEq, Clone)]
pub struct UnsignedHeader {
    pub parent: HeaderHash,
    pub parent_state_root: OpaqueHash,
    pub extrinsic_hash: OpaqueHash,
    pub slot: TimeSlot,
    pub epoch_mark: Option<EpochMark>,
    pub tickets_mark: Option<TicketsMark>,
    pub offenders_mark: Vec<Ed25519Public>,
    pub author_index: ValidatorIndex,
    pub entropy_source: BandersnatchVrfSignature,
}
// ----------------------------------------------------------------------------------------------------------
// Block
// ----------------------------------------------------------------------------------------------------------
#[derive(Debug, PartialEq, Clone)]
pub struct Extrinsic {
    pub tickets: TicketsExtrinsic,
    pub disputes: DisputesExtrinsic,
    pub preimages: PreimagesExtrinsic,
    pub assurances: AssurancesExtrinsic,
    pub guarantees: GuaranteesExtrinsic,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    pub header: Header,
    pub extrinsic: Extrinsic,
}
// ----------------------------------------------------------------------------------------------------------
// Global state
// ----------------------------------------------------------------------------------------------------------
#[derive(Clone, Debug)]
pub struct GlobalState {
    pub time: TimeSlot,
    pub availability: AvailabilityAssignments,
    pub entropy: EntropyPool,
    pub recent_history: BlockHistory,
    pub auth_pools: AuthPools,
    pub auth_queues: AuthQueues,
    pub statistics: Statistics,
    pub prev_validators: ValidatorsData,
    pub curr_validators: ValidatorsData,
    pub next_validators: ValidatorsData,
    pub disputes: DisputesRecords,
    pub safrole: Safrole,
    pub service_accounts: ServiceAccounts,
    pub services_info: HashMap<ServiceId, ServiceInfo>,
    pub preimages: HashMap<OpaqueHash, Vec<u8>>,
    pub lookup_map: HashMap<(OpaqueHash, u32), Vec<TimeSlot>>,
    pub storage_map: HashMap<OpaqueHash, Vec<u8>>,
    pub accumulation_history: AccumulatedHistory,
    pub ready_queue: ReadyQueue,
    pub privileges: Privileges,
}
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum StateKey {
    U8(u8),
    Service(u8, ServiceId),
    Account(ServiceId, Vec<u8>),
}
#[derive(Clone, Debug)]
pub struct SerializedState {
    pub map: HashMap<OpaqueHash, Vec<u8>>,
}

#[derive(Debug, PartialEq)]
pub enum ProcessError {
    ReadError(ReadError),
    SafroleError(SafroleErrorCode),
    DisputesError(DisputesErrorCode),
    ReportError(ReportErrorCode),
    AssurancesError(AssurancesErrorCode),
    PreimagesError(PreimagesErrorCode),
}
// ----------------------------------------------------------------------------------------------------------
// Polkadot Virtual Machine
// ----------------------------------------------------------------------------------------------------------
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct Program {
    pub code: Vec<u8>,          // Instruction data (c)
    pub bitmask: Vec<bool>,     // Bitmask (k)
    pub jump_table: Vec<usize>,    // Dynamic jump table (j)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Context {
    pub pc: RegSize,
    pub gas: Gas,
    pub ram: RamMemory,
    pub reg: [RegSize; NUM_REG as usize],
    pub page_fault: Option<RamAddress>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RamMemory {
    pub pages: Box<[Option<Page>]>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageTable {
    pub pages: HashMap<PageNumber, Page>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Page {
    pub flags: PageFlags,
    pub data: Box<[u8; PAGE_SIZE as usize]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageFlags {
    pub access: HashSet<RamAccess>,
    pub referenced: bool,
    pub modified: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, std::hash::Hash)]
pub enum RamAccess {
    Read,
    Write,
    None,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct PageMap {
    pub address: u32,
    pub length: u32,
    #[serde(rename = "is-writable")]
    pub is_writable: bool,
}
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct MemoryChunk {
    pub address: u32,
    pub contents: Vec<u8>,
}

#[warn(non_camel_case_types)]
#[derive(Deserialize, Debug, PartialEq, Eq)]
pub enum ExitReason {
    trap,
    halt,
    Continue,
    Branch,
    #[serde(rename = "halt")]
    Halt,
    panic,
    OutOfGas,
    #[serde(rename = "page-fault")]
    page_fault,
    PageFault(u32),     
    HostCall(HostCallType),      
}
// ----------------------------------------------------------------------------------------------------------
// Host Call
// ----------------------------------------------------------------------------------------------------------
#[derive(Deserialize, Eq, Debug, Clone, PartialEq)]
pub enum HostCallType {
    Gas = 0,
    Lookup = 1,
    Read = 2,
    Write = 3,
    Info = 4,
    Bless = 5,
    Assign = 6,
    Designate = 7,
    Checkpoint = 8,
    New = 9,
    Upgrade = 10,
    Transfer = 11,
    Eject = 12,
    Query = 13,
    Solicit = 14,
    Forget = 15,
    Yield = 16,
    HistoricalLookup = 17,
    Fetch = 18,
    Export = 19,
    Machine = 20,
    Peek = 21,
    Poke = 22,
    Zero = 23,
    Void = 24,
    Invoke = 25,
    Expugne = 26,
}

pub type Registers = [RegSize; NUM_REG as usize];

#[derive(Debug, Clone, PartialEq)]
pub struct StandardProgram {
    pub code: Vec<u8>,
    pub reg: [RegSize; NUM_REG],
    pub ram: RamMemory,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProgramFormat {
    pub c: Vec<u8>,
    pub o: Vec<u8>,
    pub w: Vec<u8>,
    pub z: u16,
    pub s: u32,
}
#[derive(Debug, Clone, PartialEq)]
pub struct AccumulationPartialState {
    pub services_accounts: ServiceAccounts,
    pub next_validators: ValidatorsData,
    pub queues_auth: AuthQueues,
    pub privileges: Privileges,
}
#[derive(Debug, Clone, PartialEq)]
pub struct DeferredTransfer {
    pub from: ServiceId,
    pub to: ServiceId,
    pub amount: u64,
    pub memo: u128,
    pub gas_limit: Gas,
}
#[derive(Debug, Clone, PartialEq)]
pub struct AccumulationContext {
    pub service_id: ServiceId,
    pub partial_state: AccumulationPartialState,
    pub index: ServiceId,
    pub deferred_transfers: Vec<DeferredTransfer>,
    pub y: Option<OpaqueHash>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct AccumulationOperand {
    code_hash: OpaqueHash,
    exports_root: OpaqueHash,
    authorizer_hash: OpaqueHash,
    auth_output: Vec<u8>,
    payload_hash: OpaqueHash,
    result: Vec<u8>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct RefineMemory {
    pub program: Vec<u8>,
    pub ram: RamMemory,
    pub pc: RegSize,
}

// The set of data segments, equivalent to octet sequences of length WG.(4104)
pub type DataSegment = [u8; SEGMENT_SIZE];
