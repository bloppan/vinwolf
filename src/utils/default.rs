use std::collections::VecDeque;
use std::collections::HashMap;
use std::array::from_fn;
use crate::constants::{EPOCH_LENGTH, MAX_ITEMS_AUTHORIZATION_POOL, RECENT_HISTORY_SIZE, VALIDATORS_COUNT, NUM_REG, PAGE_SIZE, NUM_PAGES};
use crate::types::ReportGuarantee;
use crate::types::ReportedWorkPackages;
use crate::types::{
    OpaqueHash, ReadyQueue, ReadyRecord, RefineContext, SegmentRootLookup, SegmentRootLookupItem, WorkPackageSpec, WorkReport, WorkResult,
    GlobalState, AvailabilityAssignments, EntropyPool, BlockHistory, AuthPools, AuthQueues, Statistics, ValidatorsData, DisputesRecords, Safrole,
    ServiceAccounts, AccumulatedHistory, Privileges, TimeSlot, WorkPackageHash, AuthPool, AuthQueue, AuthorizerHash, Entropy, TicketBody,
    BandersnatchEpoch, TicketsOrKeys, BandersnatchPublic, BandersnatchRingCommitment, Account, ActivityRecord, ActivityRecords, Metadata, BlsPublic,
    Ed25519Public, ValidatorData, TicketsMark, Context, Program, PageTable, Page, PageFlags, PageMap, RamMemory, MemoryChunk, GuaranteesExtrinsic,
    SerializedState
};
// ----------------------------------------------------------------------------------------------------------
// Jam Types
// ----------------------------------------------------------------------------------------------------------
impl Default for WorkReport {
    fn default() -> Self {
        WorkReport {
            package_spec: WorkPackageSpec::default(),
            context: RefineContext::default(),
            core_index: 0,
            authorizer_hash: OpaqueHash::default(),
            auth_output: Vec::new(),
            segment_root_lookup: SegmentRootLookup::default(),
            results: Vec::new(),
        }
    }
}

impl Default for SegmentRootLookupItem {
    fn default() -> Self {
        SegmentRootLookupItem {
            work_package_hash: OpaqueHash::default(),
            segment_tree_root: OpaqueHash::default(),
        }
    }
}

impl Default for SegmentRootLookup {
    fn default() -> Self {
        SegmentRootLookup {
            0: Vec::new(),
        }
    }
}

impl Default for WorkPackageSpec {
    fn default() -> Self {
        WorkPackageSpec {
            hash: OpaqueHash::default(),
            length: 0,
            erasure_root: OpaqueHash::default(),
            exports_root: OpaqueHash::default(),
            exports_count: 0,
        }
    }
}

impl Default for RefineContext {
    fn default() -> Self {
        RefineContext {
            anchor: OpaqueHash::default(),
            state_root: OpaqueHash::default(),
            beefy_root: OpaqueHash::default(),
            lookup_anchor: OpaqueHash::default(),
            lookup_anchor_slot: 0,
            prerequisites: Vec::new(),
        }
    }
}

impl Default for WorkResult {
    fn default() -> Self {
        WorkResult {
            service: 0,
            code_hash: OpaqueHash::default(),
            payload_hash: OpaqueHash::default(),
            gas: 0,
            result: Vec::new(),
        }
    }
}

impl Default for TicketBody {
    fn default() -> Self {
        TicketBody {
            id: [0u8; std::mem::size_of::<OpaqueHash>()],
            attempt: 0,
        }
    }
}

impl Default for TicketsMark {
    fn default() -> Self {
        TicketsMark{ tickets_mark: Box::new(std::array::from_fn(|_| TicketBody::default()))}
    }
}

impl Default for ReportedWorkPackages {
    fn default() -> Self {
        ReportedWorkPackages {
            0: Vec::new(),
        }
    }
}
// ----------------------------------------------------------------------------------------------------------
// Global State
// ----------------------------------------------------------------------------------------------------------
impl Default for GlobalState {
    fn default() -> Self {
        GlobalState {
            time: TimeSlot::default(),
            availability: AvailabilityAssignments::default(),
            entropy: EntropyPool::default(),
            recent_history: BlockHistory::default(),
            auth_pools: AuthPools::default(),
            auth_queues: AuthQueues::default(),
            statistics: Statistics::default(),
            prev_validators: ValidatorsData::default(),
            curr_validators: ValidatorsData::default(),
            next_validators: ValidatorsData::default(),
            disputes: DisputesRecords::default(),
            safrole: Safrole::default(),
            service_accounts: ServiceAccounts::default(),
            services_info: HashMap::new(),
            preimages: HashMap::new(),
            lookup_map: HashMap::new(),
            accumulation_history: AccumulatedHistory::default(),
            ready_queue: ReadyQueue::default(),
            privileges: Privileges::default(),
        }
    }
}

impl Default for SerializedState {
    fn default() -> Self {
        SerializedState {
            map: HashMap::new(),
        }
    }
}
// ----------------------------------------------------------------------------------------------------------
// Accumulation
// ----------------------------------------------------------------------------------------------------------
impl Default for AccumulatedHistory {
    fn default() -> Self {
        AccumulatedHistory {
            queue: {
                let mut queue: VecDeque<Vec<WorkPackageHash>> = VecDeque::new();
                for _ in 0..EPOCH_LENGTH {
                    queue.push_back(vec![]);
                }
                queue
            }
        }
    }
}

impl Default for ReadyQueue {
    fn default() -> Self {
        ReadyQueue {
            queue: Box::new(std::array::from_fn(|_| Vec::with_capacity(EPOCH_LENGTH))),
        }
    }
}

impl Default for ReadyRecord {
    fn default() -> Self {
        ReadyRecord {
            report: WorkReport::default(),
            dependencies: Vec::new(),
        }
    }
}
// ----------------------------------------------------------------------------------------------------------
// Authorization
// ----------------------------------------------------------------------------------------------------------
impl Default for AuthPool {
    fn default() -> Self {
        AuthPool {
            auth_pool: VecDeque::with_capacity(MAX_ITEMS_AUTHORIZATION_POOL),
        }
    }
}

impl Default for AuthPools {
    fn default() -> Self {
        AuthPools {
            auth_pools: Box::new(from_fn(|_| AuthPool::default())),
        }
    }
}

impl Default for AuthQueue {
    fn default() -> Self {
        AuthQueue {
            auth_queue: Box::new(from_fn(|_| [0; size_of::<AuthorizerHash>()])),
        }
    }
}

impl Default for AuthQueues {
    fn default() -> Self {
        AuthQueues {
            auth_queues: Box::new(from_fn(|_| AuthQueue::default())),
        }
    }
}
// ----------------------------------------------------------------------------------------------------------
// Disputes
// ----------------------------------------------------------------------------------------------------------
impl Default for DisputesRecords {
    fn default() -> Self {
        DisputesRecords {
            good: vec![],
            bad: vec![],
            wonky: vec![],
            offenders: vec![],
        }
    }
}
// ----------------------------------------------------------------------------------------------------------
// On-chain Entropy
// ----------------------------------------------------------------------------------------------------------
impl Default for EntropyPool {
    fn default() -> Self {
        EntropyPool { buf: Box::new(from_fn(|_| Entropy::default())) }
    }
}

impl Default for Entropy {
    fn default() -> Self {
        Entropy { entropy: OpaqueHash::default() }
    }
}
// ----------------------------------------------------------------------------------------------------------
// Privileges
// ----------------------------------------------------------------------------------------------------------
impl Default for Privileges {
    fn default() -> Self {
        Privileges {
            bless: 0,
            assign: 0,
            designate: 0,
            always_acc: HashMap::new(),
        }
    }
}
// ----------------------------------------------------------------------------------------------------------
// Block History
// ----------------------------------------------------------------------------------------------------------
impl Default for BlockHistory {
    fn default() -> Self {
        BlockHistory {
            blocks: VecDeque::with_capacity(RECENT_HISTORY_SIZE),
        }
    }
}
// ----------------------------------------------------------------------------------------------------------
// Assignments
// ----------------------------------------------------------------------------------------------------------
impl Default for AvailabilityAssignments {

    fn default() -> Self {
        AvailabilityAssignments {
            0: Box::new(std::array::from_fn(|_| None)),
        }
    }
}
// ----------------------------------------------------------------------------------------------------------
// Safrole
// ----------------------------------------------------------------------------------------------------------
impl Default for Safrole {
    fn default() -> Self {
        Safrole {
            pending_validators: ValidatorsData::default(),
            ticket_accumulator: vec![TicketBody::default()],
            seal: TicketsOrKeys::None,
            epoch_root: [0u8; std::mem::size_of::<BandersnatchRingCommitment>()],
        }
    }
}

impl Default for BandersnatchEpoch {
    fn default() -> Self {
        let keys: [BandersnatchPublic; EPOCH_LENGTH] = std::array::from_fn(|_| BandersnatchPublic::default());
        BandersnatchEpoch(Box::new(keys))
    }
}
// ----------------------------------------------------------------------------------------------------------
// Service Accounts
// ----------------------------------------------------------------------------------------------------------
impl Default for ServiceAccounts {
    fn default() -> Self {
        ServiceAccounts {
            service_accounts: HashMap::new(),
        }
    }
}

impl Default for Account {
    fn default() -> Self {
        Account {
            storage: HashMap::new(),
            preimages: HashMap::new(),
            lookup: HashMap::new(),
            code_hash: OpaqueHash::default(),
            balance: 0,
            gas: 0,
            min_gas: 0,
            items: 0,
            bytes: 0,
        }
    }
}
// ----------------------------------------------------------------------------------------------------------
// Statistics
// ----------------------------------------------------------------------------------------------------------
impl Default for ActivityRecord {
    fn default() -> Self {
        Self {
            blocks: 0,
            tickets: 0,
            preimages: 0,
            preimages_size: 0,
            guarantees: 0,
            assurances: 0,
        }
    }
}

impl Default for ActivityRecords {
    fn default() -> Self {
        Self {
            records: Box::new([ActivityRecord::default(); VALIDATORS_COUNT]),
        }
    }
}

impl Default for Statistics {
    fn default() -> Self {
        Self {
            curr: ActivityRecords::default(),
            prev: ActivityRecords::default(),
        }
    }
}
// ----------------------------------------------------------------------------------------------------------
// Validators
// ----------------------------------------------------------------------------------------------------------
impl Default for ValidatorsData {
    fn default() -> Self {
        ValidatorsData {
            0: Box::new(from_fn(|_| ValidatorData {
                bandersnatch: [0u8; std::mem::size_of::<BandersnatchPublic>()],
                ed25519: [0u8; std::mem::size_of::<Ed25519Public>()],
                bls: [0u8; std::mem::size_of::<BlsPublic>()],
                metadata: [0u8; std::mem::size_of::<Metadata>()],
            }))
        }
    }
}
// ----------------------------------------------------------------------------------------------------------
// Guarantees Extrinsic
// ----------------------------------------------------------------------------------------------------------
impl Default for GuaranteesExtrinsic {
    fn default() -> Self {
        GuaranteesExtrinsic {
            report_guarantee: Vec::new(),
        }
    }
}
impl Default for ReportGuarantee {
    fn default() -> Self {
        ReportGuarantee {
            report: WorkReport::default(),
            slot: 0,
            signatures: Vec::new(),
        }
    }
}
// ----------------------------------------------------------------------------------------------------------
// Polkadot Virtual Machine
// ----------------------------------------------------------------------------------------------------------
impl Default for Context {
    fn default() -> Self {
        Context {
            pc: 0,
            gas: 0,
            reg: [0; NUM_REG as usize],
            page_table: PageTable::default(),
            page_fault: None,
        }
    }
}

impl Default for Program {
    fn default() -> Self {
        Program {
            code: vec![],
            bitmask: vec![],
            jump_table: vec![],
        }
    }
}

impl Default for PageTable {
    fn default() -> Self {
        PageTable {
            pages: HashMap::new(),
        }
    }
}

impl Default for Page {
    fn default() -> Self {
        Page {
            flags: PageFlags::default(),
            data: Box::new([0u8; PAGE_SIZE as usize]),
        }
    }
}

impl Default for PageFlags {
    fn default() -> Self {
        PageFlags {
            is_writable: false,
            referenced: false,
            modified: false,
        }
    }
}

impl Default for PageMap {
    fn default() -> Self {
        PageMap {
            address: 0,
            length: 0,
            is_writable: false,
        }
    }
}

impl Default for RamMemory {
    fn default() -> Self {
        let mut v: Vec<Option<Page>> = Vec::with_capacity(NUM_PAGES as usize);
        for _ in 0..NUM_PAGES {
            v.push(None);
        }
        RamMemory {
            pages: v.into_boxed_slice(),
        }
    }
}

impl Default for MemoryChunk {
    fn default() -> Self {
        MemoryChunk {
            address: 0,
            contents: vec![],
        }
    }
}