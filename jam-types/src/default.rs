use std::collections::VecDeque;
use std::collections::HashMap;
use std::array::from_fn;

use constants::node::{AVAIL_BITFIELD_BYTES, EPOCH_LENGTH, MAX_ITEMS_AUTHORIZATION_QUEUE, RECENT_HISTORY_SIZE, VALIDATORS_COUNT};
use crate::Hash;
use crate::{
    BandersnatchRingVrfSignature, Ticket, Account, AccumulatedHistory, AccumulationPartialState, ActivityRecord, Assurance, AuthPool, AuthPools, AuthQueues, 
    AuthorizerHash, AvailabilityAssignments, BandersnatchEpoch, BandersnatchPublic, BandersnatchRingCommitment, BlsPublic, CodeAuthorizer, 
    CodeAuthorizers, CoreActivityRecord, CoresStatistics, DeferredTransfer, DisputesRecords, Ed25519Public, Ed25519Signature, Entropy, EntropyPool, EpochMark, 
    ExtrinsicSpec, GlobalState, Guarantee, ImportSpec, Judgement, KeyValue, MemoryChunk, Metadata, OpaqueHash, PageMap, Preimage, Privileges, AccumulationContext,
    ReadyQueue, ReadyRecord, RefineContext, RefineLoad, ReportedPackage, ReportedWorkPackage, Safrole, SegmentRootLookupItem, SerializedState, ServiceAccounts, 
    ServiceId, ServiceInfo, ServiceItem, ServicesStatistics, ServicesStatisticsMapEntry, SeviceActivityRecord, Statistics, TicketBody, TicketsMark, TicketsOrKeys, 
    TimeSlot, ValidatorData, ValidatorSignature, ValidatorStatistics, ValidatorsData, WorkItem, WorkPackageHash, WorkPackageSpec, WorkReport, WorkResult, Block,
    Header, Extrinsic, UnsignedHeader, DisputesExtrinsic, Verdict, Culprit, Fault, RecentBlocks, Mmr, RecentAccOutputs,
};

impl Default for GlobalState {
    fn default() -> Self {
        GlobalState {
            time: TimeSlot::default(),
            availability: AvailabilityAssignments::default(),
            entropy: EntropyPool::default(),
            recent_history: RecentBlocks::default(),
            auth_pools: AuthPools::default(),
            auth_queues: AuthQueues::default(),
            statistics: Statistics::default(),
            prev_validators: ValidatorsData::default(),
            curr_validators: ValidatorsData::default(),
            next_validators: ValidatorsData::default(),
            disputes: DisputesRecords::default(),
            safrole: Safrole::default(),
            service_accounts: ServiceAccounts::default(),
            accumulation_history: AccumulatedHistory::default(),
            ready_queue: ReadyQueue::default(),
            recent_acc_outputs: RecentAccOutputs::default(),
            privileges: Privileges::default(),
        }
    }
}
// ----------------------------------------------------------------------------------------------------------
// Block
// ----------------------------------------------------------------------------------------------------------
impl Default for Block {
    fn default() -> Self {
        Self { header: Header::default(), extrinsic: Extrinsic::default(), }
    }
}
impl Default for Extrinsic {
    fn default() -> Self {
        Extrinsic {
            tickets: Vec::new(),
            disputes: DisputesExtrinsic::default(),
            preimages: Vec::new(),
            guarantees: Vec::new(),
            assurances: Vec::new(),
        }
    }
}
impl Default for Header {
    fn default() -> Self {
        Self {
            unsigned: UnsignedHeader::default(),
            seal: [0u8; 96],
        }
    }
}

impl Default for UnsignedHeader {
    fn default() -> Self {
        Self {
            parent: OpaqueHash::default(),
            parent_state_root: OpaqueHash::default(),
            extrinsic_hash: OpaqueHash::default(),
            slot: 0,
            epoch_mark: None,
            tickets_mark: None,
            offenders_mark: Vec::new(),
            author_index: 0,
            entropy_source: [0u8; 96],
        }
    }
}
// ----------------------------------------------------------------------------------------------------------
// Disputes Extrinsic
// ----------------------------------------------------------------------------------------------------------
impl Default for DisputesExtrinsic {
    fn default() -> Self {
        DisputesExtrinsic {
            verdicts: Vec::new(),
            culprits: Vec::new(),
            faults: Vec::new(),
        }
    }
}

impl Default for Verdict {
    fn default() -> Self {
        Self {
            target: OpaqueHash::default(),
            age: 0,
            votes: Vec::new(),
        }
    }
}
impl Default for Culprit {
    fn default() -> Self {
        Self {
            target: OpaqueHash::default(),
            key: OpaqueHash::default(),
            signature: [0u8; std::mem::size_of::<Ed25519Signature>()],
        }
    }
}
impl Default for Fault {
    fn default() -> Self {
        Self {
            target: OpaqueHash::default(),
            vote: false,
            key: OpaqueHash::default(),
            signature: [0u8; std::mem::size_of::<Ed25519Signature>()],
        }
    }
}
// ----------------------------------------------------------------------------------------------------------
// Guarantees Extrinsic
// ----------------------------------------------------------------------------------------------------------

impl Default for Guarantee {
    fn default() -> Self {
        Guarantee {
            report: WorkReport::default(),
            slot: 0,
            signatures: Vec::new(),
        }
    }
}

impl Default for ValidatorSignature {
    fn default() -> Self {
        ValidatorSignature {
            validator_index: 0,
            signature: [0u8; std::mem::size_of::<Ed25519Signature>()],
        }
    }
}

impl Default for WorkItem {
    fn default() -> Self {
        Self {
            service: ServiceId::default(),
            code_hash: OpaqueHash::default(),
            payload: Vec::new(),
            refine_gas_limit: 0,
            acc_gas_limit: 0,
            import_segments: Vec::new(),
            extrinsic: Vec::new(),
            export_count: 0,
        }
    }
}
impl Default for WorkReport {
    fn default() -> Self {
        WorkReport {
            package_spec: WorkPackageSpec::default(),
            context: RefineContext::default(),
            core_index: 0,
            authorizer_hash: OpaqueHash::default(),
            auth_trace: Vec::new(),
            segment_root_lookup: Vec::new(),
            results: Vec::new(),
            auth_gas_used: 0,
        }
    }
}
impl Default for Ticket {
    fn default() -> Self {
        Ticket {
            attempt: 0,
            signature: [0u8; std::mem::size_of::<BandersnatchRingVrfSignature>()],
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

impl Default for ReportedWorkPackage {
    fn default() -> Self {
        Self {
            hash: OpaqueHash::default(),
            exports_root: OpaqueHash::default(),
        }
    }
}
impl Default for ReportedPackage {
    fn default() -> Self {
        Self {
            work_package_hash: OpaqueHash::default(),
            segment_tree_root: OpaqueHash::default(),
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

impl Default for RefineLoad {
    fn default() -> Self {
        RefineLoad {
            gas_used: 0,
            imports: 0,
            extrinsic_count: 0,
            extrinsic_size: 0,
            exports: 0,
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
            refine_load: RefineLoad::default(),
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

impl Default for ImportSpec {
    fn default() -> Self {
        Self {
            tree_root: OpaqueHash::default(),
            index: 0,
        }
    }
}

impl Default for ExtrinsicSpec {
    fn default() -> Self {
        Self {
            hash: OpaqueHash::default(),
            len: 0,
        }
    }
}

/*impl Default for AuthQueues {
    fn default() -> Self {
        AuthQueues(Box::new(std::array::from_fn(|_| {
            Box::new([AuthorizerHash::default(); MAX_ITEMS_AUTHORIZATION_QUEUE])
        })))
    }
}*/
impl Default for SerializedState {
    fn default() -> Self {
        SerializedState {
            map: HashMap::new(),
        }
    }
}

impl Default for KeyValue {
    fn default() -> Self {
        Self {
            key: [0u8; 31],
            value: Vec::new(),
        }
    }
}

impl Default for AvailabilityAssignments {
    fn default() -> Self {
        AvailabilityAssignments {
            list: Box::new(std::array::from_fn(|_| None)),
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

impl Default for AccumulationPartialState {
    fn default() -> Self {
        AccumulationPartialState {
            service_accounts: ServiceAccounts::default(),
            next_validators: ValidatorsData::default(),
            queues_auth: AuthQueues::default(),
            manager: ServiceId::default(),
            assign: Box::new(from_fn(|_| ServiceId::default())),
            designate: ServiceId::default(),
            always_acc: HashMap::new(),
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
impl Default for DeferredTransfer {
    fn default() -> Self {
        DeferredTransfer {
            from: 0,
            to: 0,
            amount: 0,
            memo: Vec::new(),
            gas_limit: 0,
        }
    }
}
use std::sync::Arc;
/*impl Default for AccumulationContext<'a> {
    fn default() -> Self {
        AccumulationContext {
            service_id: 0,
            partial_state: &'a mut AccumulationPartialState::default(),
            index: 0,
            deferred_transfers: Vec::new(),
            y: None,
            preimages: Vec::new(),
        }
    }
}*/
// ----------------------------------------------------------------------------------------------------------
// Authorization
// ----------------------------------------------------------------------------------------------------------

impl Default for AuthPools {
    fn default() -> Self {
        AuthPools(Box::new(std::array::from_fn(|_| AuthPool::default())))
    }
}

impl Default for AuthQueues {
    fn default() -> Self {
        AuthQueues(Box::new(std::array::from_fn(|_| {
            Box::new([AuthorizerHash::default(); MAX_ITEMS_AUTHORIZATION_QUEUE])
        })))
    }
}
impl Default for CodeAuthorizer {
    fn default() -> Self {
        CodeAuthorizer {
            core: 0,
            auth_hash: OpaqueHash::default(),
        }
    }
}
impl Default for CodeAuthorizers {
    fn default() -> Self {
        CodeAuthorizers {
            authorizers: Vec::new(),
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
impl Default for Judgement {
    fn default() -> Self {
        Self {
            vote: false,
            index: 0,
            signature: [0u8; std::mem::size_of::<Ed25519Signature>()],
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
            manager: 0,
            assign: Box::new(from_fn(|_| ServiceId::default())),
            designate: 0,
            always_acc: HashMap::new(),
        }
    }
}
// ----------------------------------------------------------------------------------------------------------
// Block History
// ----------------------------------------------------------------------------------------------------------
impl Default for Mmr {
    fn default() -> Self {
        Mmr {
            peaks: Vec::new(),
        }
    }
}
impl Default for RecentAccOutputs {
    fn default() -> Self {
        RecentAccOutputs { pairs: Vec::new() }
    }
}
impl Default for RecentBlocks {
    fn default() -> Self {
        RecentBlocks {
            history: VecDeque::with_capacity(RECENT_HISTORY_SIZE),
            mmr: Mmr::default(),
        }
    }
}
// ----------------------------------------------------------------------------------------------------------
// Assignments
// ----------------------------------------------------------------------------------------------------------

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
impl Default for EpochMark {
    fn default() -> Self {
        Self {
            entropy: Entropy::default(),
            tickets_entropy: Entropy::default(),
            validators: Box::new([(
                BandersnatchPublic::default(),
                Ed25519Public::default(),
            ); VALIDATORS_COUNT]),
        }
    }
}

impl Default for BandersnatchEpoch {
    fn default() -> Self {
        Self {
            epoch: Box::new([BandersnatchPublic::default(); EPOCH_LENGTH]),
        }
    }
}

// ----------------------------------------------------------------------------------------------------------
// Service Accounts
// ----------------------------------------------------------------------------------------------------------

impl Default for Account {
    fn default() -> Self {
        Account {
            storage: HashMap::new(),
            code_hash: OpaqueHash::default(),
            balance: 0,
            acc_min_gas: 0,
            xfer_min_gas: 0,
            gratis_storage_offset: 0,
            created_at: 0,
            last_acc: 0,
            parent_service: 0,
            items: 0,
            octets: 0,
        }
    }
}

impl Default for ServiceInfo {
    fn default() -> Self {
        Self {
            code_hash: OpaqueHash::default(),
            balance: 0,
            acc_min_gas: 0,
            xfer_min_gas: 0,
            octets: 0,
            gratis_storage_offset: 0,
            items: 0,
            created_at: 0,
            last_acc: 0,
            parent_service: 0,
        }
    }
}

impl Default for ServiceItem {
    fn default() -> Self {
        Self {
            id: ServiceId::default(),
            info: ServiceInfo::default(),
        }
    }
}

impl Default for Preimage {
    fn default() -> Self {
        Self {
            requester: ServiceId::default(),
            blob: Vec::new(),
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
impl Default for ValidatorStatistics {
    fn default() -> Self {
        Self {
            records: Box::new([ActivityRecord::default(); VALIDATORS_COUNT]),
        }
    }
}
impl Default for CoreActivityRecord {
    fn default() -> Self {
        Self {
            gas_used: 0,
            imports: 0,
            extrinsic_count: 0,
            extrinsic_size: 0,
            exports: 0,
            bundle_size: 0,
            da_load: 0,
            popularity: 0,
        }
    }
}
impl Default for CoresStatistics {
    fn default() -> Self {
        Self {
            records: Box::new(std::array::from_fn(|_| CoreActivityRecord::default())),
        }
    }
}
impl Default for SeviceActivityRecord {
    fn default() -> Self {
        Self {
            provided_count: 0,
            provided_size: 0,
            refinement_count: 0,
            refinement_gas_used: 0,
            imports: 0,
            extrinsic_count: 0,
            extrinsic_size: 0,
            exports: 0,
            accumulate_count: 0,
            accumulate_gas_used: 0,
            on_transfers_count: 0,
            on_transfers_gas_used: 0,
        }
    }
}
impl Default for ServicesStatisticsMapEntry {
    fn default() -> Self {
        Self {
            id: 0,
            record: SeviceActivityRecord::default(),
        }
    }
}
impl Default for ServicesStatistics {
    fn default() -> Self {
        Self {
            records: HashMap::new(),
        }
    }
}
impl Default for Statistics {
    fn default() -> Self {
        Self {
            curr: ValidatorStatistics::default(),
            prev: ValidatorStatistics::default(),
            cores: CoresStatistics::default(),
            services: ServicesStatistics::default(),
        }
    }
}
// ----------------------------------------------------------------------------------------------------------
// Validators
// ----------------------------------------------------------------------------------------------------------
impl Default for ValidatorData {
    fn default() -> Self {
        ValidatorData { 
            bandersnatch: [0u8; std::mem::size_of::<BandersnatchPublic>()], 
            ed25519: [0u8; std::mem::size_of::<Ed25519Public>()], 
            bls: [0u8; std::mem::size_of::<BlsPublic>()], 
            metadata: [0u8; std::mem::size_of::<Metadata>()] 
        }
    }
}

impl Default for ValidatorsData {
    fn default() -> Self {
        Self {
            list: Box::new([ValidatorData::default(); VALIDATORS_COUNT]),
        }
    }
}
// ----------------------------------------------------------------------------------------------------------
// Assurances
// ----------------------------------------------------------------------------------------------------------

impl Default for Assurance {
    fn default() -> Self {
        Assurance { 
            anchor: OpaqueHash::default(), 
            bitfield: [0u8; AVAIL_BITFIELD_BYTES], 
            validator_index: 0, 
            signature: [0u8; std::mem::size_of::<Ed25519Signature>()] 
        }
    }
}


// ----------------------------------------------------------------------------------------------------------
// Polkadot Virtual Machine
// ----------------------------------------------------------------------------------------------------------
/*impl Default for PageTable {
    fn default() -> Self {
        PageTable {
            pages: HashMap::new(),
        }
    }
}*/

impl Default for PageMap {
    fn default() -> Self {
        PageMap {
            address: 0,
            length: 0,
            is_writable: false,
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