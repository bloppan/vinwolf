mod default;
// JAM Protocol Types
use std::collections::{HashMap, VecDeque};
use serde::Deserialize;

use constants::node::{ ENTROPY_POOL_SIZE, VALIDATORS_COUNT, CORES_COUNT, AVAIL_BITFIELD_BYTES, MAX_ITEMS_AUTHORIZATION_QUEUE, EPOCH_LENGTH, SEGMENT_SIZE };
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
pub type Balance = u64;

// ----------------------------------------------------------------------------------------------------------
// Block
// ----------------------------------------------------------------------------------------------------------
#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    pub header: Header,
    pub extrinsic: Extrinsic,
}

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

#[derive(Debug, PartialEq, Clone)]
pub struct Extrinsic {
    // Tickets, used for the mechanism which manages the selection of validators for the permissioning of block authoring.
    pub tickets: Vec<Ticket>,
    // Votes, by validators, on dispute(s) arising between them presently taking place.
    pub disputes: DisputesExtrinsic,
    // Static data which is presently being requested to be available for workloads to be able to fetch on demand.
    pub preimages: Vec<Preimage>,
    // Assurances by each validator concerning which of the input data of workloads they have correctly received and are storing locally.
    pub assurances: Vec<Assurance>,
    // Reports of newly completed workloads whose accuracy is guaranteed by specific validators.
    pub guarantees: Vec<Guarantee>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DisputesExtrinsic {
    pub verdicts: Vec<Verdict>,
    pub culprits: Vec<Culprit>,
    pub faults: Vec<Fault>,
}

#[derive(Debug, PartialEq)]
pub enum ReadError {
    NotEnoughData,
    InvalidData,
    ConversionError,
}

impl std::fmt::Display for ReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadError::NotEnoughData => write!(f, "Not enough data to decode."),
            ReadError::InvalidData => write!(f, "Invalid data encountered during decoding."),
            ReadError::ConversionError => write!(f, "Error occurred during data conversion."),
        }
    }
}

impl std::error::Error for ReadError {}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct Entropy {
    pub entropy: OpaqueHash,
}
#[derive(Debug, Clone, PartialEq)]
pub struct EntropyPool {
    pub buf: Box<[Entropy; ENTROPY_POOL_SIZE]>,
}

pub type Offenders = Vec<Ed25519Public>;

/// This is a combination of a set of cryptographic public keys and metadata which is an opaque octet sequence, 
/// but utilized to specify practical identifiers for the validator, not least a hardware address. The set of 
/// validator keys itself is equivalent to the set of 336-octet sequences.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ValidatorData {
    // The Bandersnatch key is equivalent to the first 32 octets
    pub bandersnatch: BandersnatchPublic,
    // The Ed25519 is the second 32 octets
    pub ed25519: Ed25519Public,
    // The bls key is equivalent to the following 144 octets
    pub bls: BlsPublic,
    // The metadata is the last 128 octets
    pub metadata: Metadata,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ValidatorsData {
    pub list: Box<[ValidatorData; VALIDATORS_COUNT]>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BandersnatchKeys {
    pub key: Box<[BandersnatchPublic; VALIDATORS_COUNT]>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BandersnatchEpoch {
    pub epoch: Box<[BandersnatchPublic; EPOCH_LENGTH]>,
}

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
    pub timeout: TimeSlot,
}

pub type AvailabilityAssignmentsItem = Option<AvailabilityAssignment>;

#[derive(Debug, Clone, PartialEq)]
pub struct AvailabilityAssignments {
    pub list: Box<[AvailabilityAssignmentsItem; CORES_COUNT]>,
}
// ----------------------------------------------------------------------------------------------------------
// Refine Context
// ----------------------------------------------------------------------------------------------------------
// A refinement context, denoted by the set X, describes the context of the chain at the point 
// that the report’s corresponding work-package was evaluated. 
#[derive(Debug, Clone, PartialEq)]
pub struct RefineContext {
    // Anchor block header hash
    pub anchor: OpaqueHash,
    // Posterior block state root
    pub state_root: OpaqueHash,
    // Posterior BEEFY root
    pub beefy_root: OpaqueHash,
    // Lookup anchor header hash
    pub lookup_anchor: OpaqueHash,
    // Lookup anchor timeslot
    pub lookup_anchor_slot: TimeSlot,
    // Sequence of hashes of any prerequisite work packages
    pub prerequisites: Vec<OpaqueHash>,
}
// ----------------------------------------------------------------------------------------------------------
// Authorizations
// ----------------------------------------------------------------------------------------------------------
pub type AuthorizerHash = OpaqueHash;

#[derive(Debug, Clone, PartialEq)]
pub struct Authorizer {
    pub code_hash: OpaqueHash,
    pub params: Vec<u8>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct AuthPools(pub Box<[AuthPool; CORES_COUNT]>);
pub type AuthPool = VecDeque<OpaqueHash>;

#[derive(Debug, Clone, PartialEq)]
pub struct AuthQueues(pub Box<[AuthQueue; CORES_COUNT]>);
pub type AuthQueue = Box<[AuthorizerHash; MAX_ITEMS_AUTHORIZATION_QUEUE]>;

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

#[derive(Debug, Clone, PartialEq)]
pub struct WorkItem {
    // Identifier of the service to which it relates
    pub service: ServiceId,
    // Code hash of the service at the time of reporting (whose preimage must be available from the perspective of the lookup achor block)
    pub code_hash: OpaqueHash,
    // Payload blob
    pub payload: Vec<u8>,
    // Gas limit for refine
    pub refine_gas_limit: Gas,
    // Gas limit for accumulate
    pub acc_gas_limit: Gas,
    // Sequence of imported segments which identify a prior exported segment through an index
    pub import_segments: Vec<ImportSpec>,
    // Sequence of blob hashes and lengths (which we assume the validators knows)
    pub extrinsic: Vec<ExtrinsicSpec>,
    // Number of data segments exported by this work item
    pub export_count: u16,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorkPackage {
    // Simple blob acting as an authorization token
    pub authorization: Vec<u8>,
    // Index of the service which hosts the authorization code
    pub auth_code_host: ServiceId,
    // Authorization code hash and configuration blob
    pub authorizer: Authorizer,
    // Refine context
    pub context: RefineContext,
    // Sequence of work items
    pub items: Vec<WorkItem>,
}
// ----------------------------------------------------------------------------------------------------------
// Work Report
// ----------------------------------------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq)]
pub struct WorkReport {
    // Work package specification
    pub package_spec: WorkPackageSpec,
    // Refine context
    pub context: RefineContext,
    // Core index
    pub core_index: CoreIndex,
    // Authorizer hash
    pub authorizer_hash: OpaqueHash,
    // Authorization output
    pub auth_output: Vec<u8>,
    // Segment root lookup dictionary
    pub segment_root_lookup: Vec<SegmentRootLookupItem>, // TODO mejor con un hashmap?
    // Sequence of work results of the evaluation of each of the items in the package together with some associated data
    pub results: Vec<WorkResult>,
    // Gas used
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

// The Work Result is the data conduit by which services states may be altered through the computation done within a work-package. 
#[derive(Debug, Clone, PartialEq)]
pub struct WorkResult {
    // Index of the service whose state is to be altered and thus whose refine code was already executed
    pub service: ServiceId,
    // Hash of the code service at the time of being reported, which must be accurately predicted within the work report
    pub code_hash: OpaqueHash,
    // Hash of the payload within the work item which was executed in the refine stage to give this result
    pub payload_hash: OpaqueHash,
    // Gas limit for executing this item's accumulate
    pub gas: Gas,
    // Output blob of error of the execution of the code which may be either an octed sequence in case it was successfull or a member of J (possible errors) if not
    // Possible errors are:
    //      Out-of-gas
    //      Unexpected program termination
    //      The code was not available for lookup in state at the posterior state of the lookup-anchor block.
    //      The code was available but was beyond the maximun size allowed Wc.
    pub result: Vec<u8>,
    // Level of activity which this workload imposed on the core in bringing the result to bear
    pub refine_load: RefineLoad,
}
#[derive(Debug, Clone, PartialEq)]
pub struct RefineLoad {
    // Gas used during refinement
    pub gas_used: u64,
    // Number of segments imported from DA
    pub imports: u16,
    // Number of the extrinsics used in computing the workload
    pub extrinsic_count: u16,
    // Total size in octets of the extrinsics used in computing the workload
    pub extrinsic_size: u32,
    // Number of segments exported into DA
    pub exports: u16,
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
    ServiceCodeNotAvailableForLookup = 4,
    BadCode = 5,
    CodeOversize = 6,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorkPackageSpec {
    // Work package hash
    pub hash: OpaqueHash,
    // Work bundle length
    pub length: u32,
    // Erasure root
    pub erasure_root: OpaqueHash,
    // Segment root
    pub exports_root: OpaqueHash,
    // Segment count
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
    pub hash: OpaqueHash,
    pub exports_root: OpaqueHash,
}

/*#[derive(Clone, Debug, PartialEq)]
pub struct ReportedWorkPackages(pub Vec<ReportedWorkPackage>);*/
pub type ReportedWorkPackages = Vec<(OpaqueHash, OpaqueHash)>;

#[derive(Clone, Debug, PartialEq)]
pub struct BlockInfo {
    // Block's header hash
    pub header_hash: Hash,
    // Accumulation-result MMR 
    pub mmr: Mmr,
    // Block's state root
    pub state_root: Hash,
    // Work package hashes of each item reported (which is no more than the CORES_COUNT)
    pub reported_wp: ReportedWorkPackages,
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
pub struct ValidatorStatistics {
    pub records: Box<[ActivityRecord; VALIDATORS_COUNT]>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct CoreActivityRecord {
    // Number of segments imported from DA made by core for reported work.
    pub imports: u16,         
    // Total number of extrinsics used by core for reported work.
    pub extrinsic_count: u16, 
    // Total size of extrinsics used by core for reported work.
    pub extrinsic_size: u32,  
    // Number of segments exported into DA made by core for reported work.
    pub exports: u16,         
    // Total gas consumed by core for reported work. Includes all refinement and authorizations.   
    pub gas_used: u64,        
    // The work-bundle size. This is the size of data being placed into Audits DA by the core.
    pub bundle_size: u32,     
    // Amount of bytes which are placed into either Audits or Segments DA. This includes the work-bundle (including all extrinsics and imports) as well as all (exported) segments
    pub da_load: u32,         
    // Number of validators which formed super-majority for assurance.
    pub popularity: u16,      
}
#[derive(Clone, Debug, PartialEq)]
pub struct CoresStatistics {
    pub records: Box<[CoreActivityRecord; CORES_COUNT]>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct SeviceActivityRecord {
    // Number of preimages provided to this service
    pub provided_count: u16,        
    // Total size of preimages provided to this service.    
    pub provided_size: u32,    
    // Number of work-items refined by service for reported work.
    pub refinement_count: u32,      
    // Amount of gas used for refinement by service for reported work.
    pub refinement_gas_used: u64,   
    // Number of segments imported from the DL by service for reported work.
    pub imports: u32,   
    // Number of segments exported into the DL by service for reported work.
    pub exports: u32,       
    // Total size of extrinsics used by service for reported work.
    pub extrinsic_size: u32,       
    // Total number of extrinsics used by service for reported work.
    pub extrinsic_count: u32,       
    // Number of work-items accumulated by service.
    pub accumulate_count: u32,      
    // Amount of gas used for accumulation by service.
    pub accumulate_gas_used: u64,   
    // Number of transfers processed by service.
    pub on_transfers_count: u32,    
    // Amount of gas used for processing transfers by service.
    pub on_transfers_gas_used: u64, 
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
    pub curr: ValidatorStatistics,
    pub prev: ValidatorStatistics,
    pub cores: CoresStatistics,
    pub services: ServicesStatistics,
}
// ----------------------------------------------------------------------------------------------------------
// Tickets
// ----------------------------------------------------------------------------------------------------------
// Tickets Extrinsic is a sequence of proofs of valid tickets; a ticket implies an entry in our epochal “contest” 
// to determine which validators are privileged to author a block for each timeslot in the following epoch. 
// Tickets specify an entry index together with a proof of ticket’s validity. The proof implies a ticket identifier, 
// a high-entropy unbiasable 32-octet sequence, which is used both as a score in the aforementioned contest and as 
// input to the on-chain vrf. 

// Towards the end of the epoch (i.e. Y slots from the start) this contest is closed implying successive blocks 
// within the same epoch must have an empty tickets extrinsic. At this point, the following epoch’s seal key sequence 
// becomes fixed. 
// We define the extrinsic as a sequence of proofs of valid tickets, each of which is a tuple of an entry index 
// (a natural number less than N) and a proof of ticket validity.
pub type TicketId = OpaqueHash;
pub type TicketAttempt = u8;

#[derive(Debug, Clone, PartialEq)]
pub struct Ticket {
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

// ----------------------------------------------------------------------------------------------------------
// Safrole
// ----------------------------------------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq)]
pub struct Safrole {
    // Internal to the Safrole state we retain a pending set of validators. The active set is the set of keys identifiying
    // the nodes which are currently privileged to author blocks and carry out the validation processes, whereas the pending
    // set, which is reset to next_validators (iota) at the beginning of each epoch, is the set of keys which will be active
    // in the next epoch and which determine the Bandersnatch ring root which authorizes tickets into the sealing-key contest
    // for the next epoch.
    pub pending_validators: ValidatorsData,
    // Sequence of highest-scoring ticket identifiers to be used for the next epoch
    pub ticket_accumulator: Vec<TicketBody>,
    // Current epoch's slot-sealer. Can be either a full complement of EPOCH_LENGTH tickets or, in the case of fallback mode,
    // a series of EPOCH_LENGTH Bandersnatch keys 
    pub seal: TicketsOrKeys,
    // Bandersnatch ring root composed with the one Bandersnatch key of each of the next epoch's validators
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
    // Timeslot value must be strictly monotonic
    BadSlot = 0,        
    // Received a ticket while in epoch's tail
    UnexpectedTicket = 1,  
    // Tickets must be sorted
    BadTicketOrder = 2,   
    // Invalid ticket ring proof
    BadTicketProof = 3,   
    // Invalid ticket attempt value
    BadTicketAttempt = 4, 
    // Reserved
    Reserved = 5,           
    // Found a ticket duplicate
    DuplicateTicket = 6,   
    // Too many tickets in extrinsic
    TooManyTickets = 7,    
    // Invalid seal
    InvalidTicketSeal = 8,      
    // Invalid seal 
    InvalidKeySeal = 9,         
    // Invalid entropy source
    InvalidEntropySource = 10, 
    // Tickets or keys is none
    TicketsOrKeysNone = 11, 
    // Seal does not match
    TicketNotMatch = 12,      
    // Seal key does not match
    KeyNotMatch = 13,        
    InvalidRingVrfSignature = 14,
    RingSignatureVerificationFail = 15,
    InvalidIetffSignature = 16,
    IetfSignatureVerificationFail = 17,
    InvalidSignerKeyIndex = 18,
}
// ----------------------------------------------------------------------------------------------------------
// Disputes
// ----------------------------------------------------------------------------------------------------------
// The disputes state includes four items, three of which concern verdicts
#[derive(Debug, Clone, PartialEq)]
pub struct DisputesRecords {
    // Good set
    pub good: Vec<WorkReportHash>,
    // Bad set
    pub bad: Vec<WorkReportHash>,
    // Wonky set containing the hashes of all work reports which were respectively judged to be correct, incorrect
    // or that it appears impossible to judge
    pub wonky: Vec<WorkReportHash>,
    // The offenders or punish set is a set of Ed25519 keys representing validators which were found to have misjudged a 
    // work report
    pub offenders: Vec<Ed25519Public>,
}
// A Verdict is a compilation of judgments coming from exactly two-thirds plus one of either the active validator set 
// or the previous epoch’s validator set, i.e. the Ed25519 keys of κ or λ. Verdicts contains only the report hash and 
// the sum of positive judgments. We require this total to be either exactly two-thirds-plus-one, zero or one-third 
// of the validator set indicating, respectively, that the report is good, that it’s bad, or that it’s wonky.
#[derive(Debug, Clone, PartialEq)]
pub struct Verdict {
    pub target: OpaqueHash,
    pub age: u32,
    pub votes: Vec<Judgement>,
}
// A culprit is a proofs of the misbehavior of one or more validators by guaranteeing a work-report found to be invalid.
// Is a offender signature.
#[derive(Debug, Clone, PartialEq)]
pub struct Culprit {
    pub target: OpaqueHash,
    pub key: Ed25519Public,
    pub signature: Ed25519Signature,
}
// A fault is a proofs of the misbehavior of one or more validators by signing a judgment found to be contradiction to a 
// work-report’s validity. Is a offender signature. Must be ordered by validators Ed25519Key. There may be no duplicate
// report hashes within the extrinsic, nor amongst any past reported hashes.
#[derive(Debug, Clone, PartialEq)]
pub struct Fault {
    pub target: OpaqueHash,
    pub vote: bool,
    pub key: Ed25519Public,
    pub signature: Ed25519Signature,
}
// Judgement statements come about naturally as part of the auditing process and are expected to be positive,
// further affirming the guarantors’ assertion that the workreport is valid. In the event of a negative judgment, 
// then all validators audit said work-report and we assume a verdict will be reached.
#[derive(Debug, Clone, PartialEq)]
pub struct Judgement {
    pub vote: bool,
    pub index: ValidatorIndex,
    pub signature: Ed25519Signature,
}

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
    BadGuarantoorKey = 14,
    BadAuditorKey = 15,
    NoVerdictsFound = 16,
    AgesNotEqual = 17,
    CulpritKeyNotFound = 18,
    FaultKeyNotFound = 19,
    BadVotesCount = 20,
}
// ----------------------------------------------------------------------------------------------------------
// Service Accounts
// ----------------------------------------------------------------------------------------------------------

pub type ServiceAccounts = HashMap<ServiceId, Account>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    // Storage dictionary
    pub storage: HashMap<StorageKey, Vec<u8>>,
    // Preimages dictionary
    pub preimages: HashMap<StorageKey, Vec<u8>>,
    // Lookup dictionary
    pub lookup: HashMap<StorageKey, Vec<TimeSlot>>,
    // Code hash
    pub code_hash: OpaqueHash,
    // Account balance
    pub balance: u64,
    // Minimum gas required in order to execute the accumulate entry-point of the service's code
    pub acc_min_gas: Gas,
    // Minimum gas required for the on transfer entry-point
    pub xfer_min_gas: Gas,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreimageData {
    pub metadata: Vec<u8>,
    pub code: Vec<u8>,
}
pub type ServiceId = u32;

#[derive(Debug, Clone, PartialEq)]
pub struct ServiceInfo {
    // Code hash
    pub code_hash: OpaqueHash,
    // Account balance
    pub balance: u64,
    // Minimum gas required in order to execute the accumulate entry-point of the service's code
    pub acc_min_gas: Gas,
    // Minimum gas required for the on transfer entry-point 
    pub xfer_min_gas: Gas,
    // Number of octets in the storage
    pub bytes: u64,
    // Number of items in the storage
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
pub struct Assurance {
    // Assurance anchor hash must be the parent header
    pub anchor: OpaqueHash,
    // Sequence of binary values (once per core) which a value of 1 at any given index implies that the validator assures
    // they are contributing to its availability
    pub bitfield: [u8; AVAIL_BITFIELD_BYTES],
    // Index of validator who is assuring
    pub validator_index: ValidatorIndex,
    // Validator public Ed25519 signature
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
pub struct Guarantee {
    pub report: WorkReport,
    pub slot: TimeSlot,
    pub signatures: Vec<ValidatorSignature>,
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

#[derive(Debug, Clone, PartialEq)]
pub struct Privileges {
    // Index of the manager service which is the service able to effect an alteration of privileges state component from block to block
    pub bless: ServiceId,
    // Index of service able to alter the authorizer queue state component
    pub assign: ServiceId,
    // Index of service able to alter the next validators state component
    pub designate: ServiceId,
    // Indices of services which automaticaly accumulate in each block together with a basic amount of gas with which each accumulates
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
// The epoch and winning-tickets markers are information placed in the header in order to minimize 
// data transfer necessary to determine the validator keys associated with any given epoch. They 
// are particularly useful to nodes which do not synchronize the entire state for any given block 
// since they facilitate the secure tracking of changes to the validator key sets using only the 
// chain of headers.

// The epoch marker specifies key and entropy relevant to the following epoch in case the ticket 
// contest does not complete adequately (a very much unexpected eventuality).The epoch marker is
// either empty or, if the block is the first in a new epoch, then a tuple of the epoch randomness 
// and a sequence of Bandersnatch keys defining the Bandersnatch validator keys (kb) beginning in 
// the next epoch.
#[derive(Debug, PartialEq, Clone)]
pub struct EpochMark {
    pub entropy: Entropy,
    pub tickets_entropy: Entropy,
    pub validators: Box<[(BandersnatchPublic, Ed25519Public); VALIDATORS_COUNT]>,
}
// The Tickets Marker provides the series of EPOCH_LENGTH (600) slot sealing “tickets” for the next epoch. Is either 
// empty or, if the block is the first after the end of the submission period for tickets and if the ticket accumulator 
// is saturated, then the final sequence of ticket identifiers.
#[derive(Debug, PartialEq, Clone)]
pub struct TicketsMark {
    pub tickets_mark: Box<[TicketBody; EPOCH_LENGTH]>,
}

pub type OffendersMark = Vec<Ed25519Public>;
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum StateKeyType {
    U8(u8),
    Service(u8, ServiceId),
    Account(ServiceId, Vec<u8>),
}
#[derive(Clone, Debug, PartialEq)]
pub struct SerializedState {
    pub map: HashMap<StateKey, Vec<u8>>,
}
#[derive(Clone, Debug, PartialEq)]
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
    pub accumulation_history: AccumulatedHistory,
    pub ready_queue: ReadyQueue,
    pub privileges: Privileges,
}
#[derive(Debug, PartialEq)]
pub enum ProcessError {
    ReadError(ReadError),
    HeaderError(HeaderErrorCode),
    SafroleError(SafroleErrorCode),
    DisputesError(DisputesErrorCode),
    ReportError(ReportErrorCode),
    AssurancesError(AssurancesErrorCode),
    PreimagesError(PreimagesErrorCode),
    AccumulateError(AccumulateErrorCode),
}
#[derive(Debug, Clone, PartialEq)]
pub enum AccumulateErrorCode {
    ServiceConflict = 0,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HeaderErrorCode {
    BadParentStateRoot = 0,
    BadValidatorIndex = 1,
    BadBlockAuthor = 2,
    BadExtrinsicHash = 3,
    BadOffenders = 4,
}
// ----------------------------------------------------------------------------------------------------------
// Polkadot Virtual Machine
// ----------------------------------------------------------------------------------------------------------
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


#[derive(Debug, Clone, PartialEq)]
pub struct AccumulationPartialState {
    pub service_accounts: ServiceAccounts,
    pub next_validators: ValidatorsData,
    pub queues_auth: AuthQueues,
    pub privileges: Privileges,
}
#[derive(Debug, Clone, PartialEq)]
pub struct DeferredTransfer {
    // Service index of the sender
    pub from: ServiceId,
    // Service index of the receiver
    pub to: ServiceId,
    // Amount to send
    pub amount: u64,
    // Memo component 
    pub memo: Vec<u8>,
    // Gas limit for the transfer
    pub gas_limit: Gas,
}
#[derive(Debug, Clone, PartialEq)]
pub struct AccumulationContext {
    pub service_id: ServiceId,
    pub partial_state: AccumulationPartialState,
    pub index: ServiceId,
    pub deferred_transfers: Vec<DeferredTransfer>,
    pub y: Option<OpaqueHash>,
    pub preimages: Vec<(ServiceId, Vec<u8>)>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct AccumulationOperand {
    pub code_hash: OpaqueHash,
    pub exports_root: OpaqueHash,
    pub authorizer_hash: OpaqueHash,
    pub payload_hash: OpaqueHash,
    pub gas_limit: Gas,
    pub result: Vec<u8>,
    pub auth_output: Vec<u8>,
}

// The set of data segments, equivalent to octet sequences of length WG.(4104)
pub type DataSegment = [u8; SEGMENT_SIZE];
pub type DataSegments = Vec<DataSegment>;

pub type TrieKey = [u8; 31];
pub type StateKey = [u8; 31];
pub type SimpleKey = StateKey;

pub type PreimageKey = StateKey;
pub type LookupKey = StateKey;
pub type StorageKey = StateKey;
pub type ServiceInfoKey = StateKey;

#[derive(Debug, Clone, PartialEq)]
pub struct KeyValue {
    pub key: TrieKey,
    pub value: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RawState {
    pub state_root: StateRoot,
    pub keyvals: Vec<KeyValue>,
}

