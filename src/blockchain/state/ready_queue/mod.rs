use crate::constants::EPOCH_LENGTH;
use crate::types::{
    OpaqueHash, ReadyQueue, ReadyRecord, RefineContext, SegmentRootLookup, SegmentRootLookupItem, WorkPackageSpec, WorkReport, WorkResult
};













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