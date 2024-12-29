// JAM Protocol Types
use std::collections::VecDeque;
use crate::constants::{ENTROPY_POOL_SIZE, VALIDATORS_COUNT, CORES_COUNT, AVAIL_BITFIELD_BYTES, MAX_ITEMS_AUTHORIZATION_QUEUE};
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
pub type Hash = [u8; 32];
pub type Metadata = [u8; 128];

pub type TimeSlot = u32;
pub type ValidatorIndex = u16;
pub type CoreIndex = u16;

pub type HeaderHash = OpaqueHash;
pub type StateRoot = OpaqueHash;
pub type BeefyRoot = OpaqueHash;
pub type WorkPackageHash = OpaqueHash;
pub type WorkReportHash = OpaqueHash;
pub type ExportsRoot = OpaqueHash;
pub type ErasureRoot = OpaqueHash;

pub type Gas = u64;

#[derive(Debug, Clone, PartialEq)]
pub struct Entropy(pub OpaqueHash);
#[derive(Debug, Clone, PartialEq)]
pub struct EntropyPool(pub Box<[Entropy; ENTROPY_POOL_SIZE]>);

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

#[derive(Clone, PartialEq, Debug)]
pub struct ValidatorsData(pub Box<[ValidatorData; VALIDATORS_COUNT]>);
// ----------------------------------------------------------------------------------------------------------
// Service
// ----------------------------------------------------------------------------------------------------------
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
#[derive(Debug)]
pub struct ImportSpec {
    pub tree_root: OpaqueHash,
    pub index: u16,
}

// The extrinsic spec is a sequence of blob hashes and lengths to be introduced in this block 
// (and which we assume the validator knows). It's a member of Work Item
#[derive(Debug)]
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
#[derive(Debug)]
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
#[derive(Debug)]
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
    pub segment_root_lookup: SegmentRootLookup,
    pub results: Vec<WorkResult>,
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
pub struct WorkResult {
    pub service: ServiceId,
    pub code_hash: OpaqueHash,
    pub payload_hash: OpaqueHash,
    pub gas: Gas,
    pub result: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WorkExecResult {
    Ok = 0,
    OutOfGas = 1,
    Panic = 2,
    BadCode = 3,
    CodeOversize = 4,
    UnknownError = 5,
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
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentRootLookup(pub Vec<SegmentRootLookupItem>);
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
pub struct Statistics {
    pub curr: ActivityRecords,
    pub prev: ActivityRecords,
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
    Keys(Vec<BandersnatchPublic>),
    Tickets(Vec<TicketBody>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TicketsExtrinsic { 
    pub tickets: Vec<TicketEnvelope>,
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
// Preimages
// ----------------------------------------------------------------------------------------------------------
#[derive(Debug, PartialEq, Clone)]
pub struct Preimage {
    pub requester: ServiceId,
    pub blob: Vec<u8>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct PreimagesExtrinsic {
    pub preimages: Vec<Preimage>,
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
// Header
// ----------------------------------------------------------------------------------------------------------
#[derive(Debug, PartialEq, Clone)]
pub struct EpochMark {
    pub entropy: OpaqueHash,
    pub tickets_entropy: OpaqueHash,
    pub validators: Vec<BandersnatchPublic>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TicketsMark {
    pub tickets_mark: Vec<TicketBody>,
}

pub type OffendersMark = Vec<Ed25519Public>;

#[derive(Debug, PartialEq, Clone)]
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
