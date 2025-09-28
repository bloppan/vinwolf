
use jam_types::{
    AvailabilityAssignment, AvailabilityAssignments, AvailabilityAssignmentsItem, WorkReport, Hash, RefineContext, TimeSlot,
    ReportedWorkPackage, ServiceId, WorkItem, WorkPackage, OpaqueHash, Gas, ImportSpec, ExtrinsicSpec, WorkResult, WorkExecResult, WorkExecError, RefineLoad,
    Ed25519Public, WorkPackageHash, SegmentRootLookupItem, WorkPackageSpec, ReportedPackage, OutputDataReports, 
};
use constants::node::CORES_COUNT;
use crate::{Encode, Decode, BytesReader, EncodeSize, EncodeLen, DecodeSize, DecodeLen, ReadError};
use crate::generic_codec::{encode_unsigned, decode_unsigned};

impl Encode for WorkReport {

    fn encode(&self) -> Vec<u8> {

        let mut work_report_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<WorkReport>());
        
        self.package_spec.hash.encode_to(&mut work_report_blob);
        self.package_spec.length.encode_to(&mut work_report_blob);
        self.package_spec.erasure_root.encode_to(&mut work_report_blob);
        self.package_spec.exports_root.encode_to(&mut work_report_blob);
        self.package_spec.exports_count.encode_to(&mut work_report_blob);
        self.context.encode_to(&mut work_report_blob);
        encode_unsigned(self.core_index as usize).encode_to(&mut work_report_blob);
        self.authorizer_hash.encode_to(&mut work_report_blob);
        encode_unsigned(self.auth_gas_used as usize).encode_to(&mut work_report_blob);
        self.auth_trace.encode_len().encode_to(&mut work_report_blob);
        self.segment_root_lookup.encode_len().encode_to(&mut work_report_blob);
        self.results.encode_len().encode_to(&mut work_report_blob);
        
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
            core_index: decode_unsigned(work_report)? as u16,
            authorizer_hash: OpaqueHash::decode(work_report)?,
            auth_gas_used: decode_unsigned(work_report)? as Gas,
            auth_trace: Vec::<u8>::decode_len(work_report)?,
            segment_root_lookup: Vec::<SegmentRootLookupItem>::decode_len(work_report)?,
            results: Vec::<WorkResult>::decode_len(work_report)?,
            
        })
    }
}

impl Encode for RefineContext {

    fn encode(&self) -> Vec<u8> {

        let mut refine_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<RefineContext>());  
        
        self.anchor.encode_to(&mut refine_blob);
        self.state_root.encode_to(&mut refine_blob);
        self.beefy_root.encode_to(&mut refine_blob);
        self.lookup_anchor.encode_to(&mut refine_blob);
        self.lookup_anchor_slot.encode_size(4).encode_to(&mut refine_blob);
        self.prerequisites.encode_len().encode_to(&mut refine_blob);
   
        return refine_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for RefineContext {

    fn decode(refine_blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(RefineContext {
            anchor: OpaqueHash::decode(refine_blob)?,
            state_root: OpaqueHash::decode(refine_blob)?,
            beefy_root: OpaqueHash::decode(refine_blob)?,
            lookup_anchor: OpaqueHash::decode(refine_blob)?,
            lookup_anchor_slot: TimeSlot::decode(refine_blob)?,
            prerequisites: Vec::<OpaqueHash>::decode_len(refine_blob)?,
        })
    }
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

impl Encode for OutputDataReports {

    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::with_capacity(std::mem::size_of::<ReportedPackage>() * self.reported.len() + std::mem::size_of::<Ed25519Public>() * self.reporters.len());

        self.reported.encode_len().encode_to(&mut blob);
        self.reporters.encode_len().encode_to(&mut blob);
        
        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for OutputDataReports {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(OutputDataReports{
            reported: Vec::<ReportedPackage>::decode_len(blob)?,
            reporters: Vec::<Ed25519Public>::decode_len(blob)?,
        })
    }
}

impl Encode for SegmentRootLookupItem {

    fn encode(&self) -> Vec<u8> {
        
        let mut item = Vec::with_capacity(std::mem::size_of::<Self>());

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

impl Encode for WorkResult {

    fn encode(&self) -> Vec<u8> {

        let mut blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<WorkResult>());
        self.service.encode_size(4).encode_to(&mut blob);
        self.code_hash.encode_to(&mut blob);
        self.payload_hash.encode_to(&mut blob);
        self.gas.encode_size(8).encode_to(&mut blob);

        self.result[0].encode_to(&mut blob);

        if self.result[0] == 0 {
            let result_len = encode_unsigned(self.result.len() - 1);
            result_len.encode_to(&mut blob);
            self.result[1..].encode_to(&mut blob);
        } 
        
        self.refine_load.encode_to(&mut blob);
        
        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for WorkResult {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(WorkResult {
            service: ServiceId::decode(blob)?,
            code_hash: OpaqueHash::decode(blob)?,
            payload_hash: OpaqueHash::decode(blob)?,
            gas: Gas::decode_size(blob, 8)? as Gas,
            result: {
                let mut result: Vec<u8> = vec![];
                let exec_result = blob.read_byte()?;
                exec_result.encode_to(&mut result);
                
                match exec_result {
                    0 => {
                        let len = decode_unsigned(blob)?;
                        result.extend_from_slice(&blob.read_bytes(len)?);
                        WorkExecResult::Ok(result.clone())
                    },
                    1 => WorkExecResult::Error(WorkExecError::OutOfGas),
                    2 => WorkExecResult::Error(WorkExecError::Panic),
                    3 => WorkExecResult::Error(WorkExecError::BadNumberExports),
                    4 => WorkExecResult::Error(WorkExecError::ServiceCodeNotAvailableForLookup),
                    5 => WorkExecResult::Error(WorkExecError::BadCode),
                    6 => WorkExecResult::Error(WorkExecError::CodeOversize),
                    _ => { 
                        return Err(ReadError::InvalidData);
                    }
                };
                result
            },
            refine_load: RefineLoad::decode(blob)?,
        })
    }  
}

impl Encode for RefineLoad {

    fn encode(&self) -> Vec<u8> {
        let mut blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<Self>());
        
        encode_unsigned(self.gas_used as usize).encode_to(&mut blob);
        encode_unsigned(self.imports as usize).encode_to(&mut blob);
        encode_unsigned(self.extrinsic_count as usize).encode_to(&mut blob);
        encode_unsigned(self.extrinsic_size as usize).encode_to(&mut blob);
        encode_unsigned(self.exports as usize).encode_to(&mut blob);

        return blob;
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for RefineLoad {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(RefineLoad {
            gas_used: decode_unsigned(blob)? as u64,
            imports: decode_unsigned(blob)? as u16,
            extrinsic_count: decode_unsigned(blob)? as u16,
            extrinsic_size: decode_unsigned(blob)? as u32,
            exports: decode_unsigned(blob)? as u16,
        })
    }
}


impl Encode for WorkItem {
    
    fn encode(&self) -> Vec<u8> {
        
        let mut work_item_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<WorkItem>());
        
        self.service.encode_size(4).encode_to(&mut work_item_blob);
        self.code_hash.encode_to(&mut work_item_blob);
        self.refine_gas_limit.encode_size(8).encode_to(&mut work_item_blob);
        self.acc_gas_limit.encode_size(8).encode_to(&mut work_item_blob);
        self.export_count.encode_size(2).encode_to(&mut work_item_blob);
        self.payload.as_slice().encode_len().encode_to(&mut work_item_blob);       
        self.import_segments.encode_len().encode_to(&mut work_item_blob);
        self.extrinsic.encode_len().encode_to(&mut work_item_blob);
        
        return work_item_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for WorkItem {

    fn decode(work_item_blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(WorkItem {
            service: ServiceId::decode(work_item_blob)?,
            code_hash: OpaqueHash::decode(work_item_blob)?,
            refine_gas_limit: Gas::decode_size(work_item_blob, 8)? as Gas,
            acc_gas_limit: Gas::decode_size(work_item_blob, 8)? as Gas,
            export_count: u16::decode(work_item_blob)?,
            payload: Vec::<u8>::decode_len(work_item_blob)?,
            import_segments: Vec::<ImportSpec>::decode_len(work_item_blob)?,
            extrinsic: Vec::<ExtrinsicSpec>::decode_len(work_item_blob)?,
        })
    }
}

impl Encode for ImportSpec {

    fn encode(&self) -> Vec<u8> {

        let mut import_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<Self>());

        self.tree_root.encode_to(&mut import_blob);
        self.index.encode_to(&mut import_blob);

        return import_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode()); 
    }
}

impl Decode for ImportSpec {

    fn decode(spec_blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(ImportSpec {
            tree_root : OpaqueHash::decode(spec_blob)?,
            index : u16::decode(spec_blob)?,        
        })
    }
}

impl Decode for ExtrinsicSpec {
    fn decode(ext_blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(ExtrinsicSpec {
            hash : OpaqueHash::decode(ext_blob)?,
            len : u32::decode(ext_blob)?,
        })
    }
}

impl Encode for ExtrinsicSpec {

    fn encode(&self) -> Vec<u8> {

        let mut ext_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<ExtrinsicSpec>());

        self.hash.encode_to(&mut ext_blob);
        self.len.encode_size(4).encode_to(&mut ext_blob);

        return ext_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Encode for WorkPackage {

    fn encode(&self) -> Vec<u8> {

        let mut work_pkg_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<WorkPackage>());

        self.auth_code_host.encode_size(4).encode_to(&mut work_pkg_blob);
        self.auth_code_hash.encode_to(&mut work_pkg_blob);
        self.context.encode_to(&mut work_pkg_blob);
        self.authorization.as_slice().encode_len().encode_to(&mut work_pkg_blob);
        self.configuration_blob.encode_len().encode_to(&mut work_pkg_blob); 
        self.items.encode_len().encode_to(&mut work_pkg_blob);
        
        return work_pkg_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for WorkPackage {

    fn decode(work_pkg_blob: &mut BytesReader) -> Result<Self, ReadError> {
        
        Ok(WorkPackage {
            auth_code_host: ServiceId::decode(work_pkg_blob)?,
            auth_code_hash: OpaqueHash::decode(work_pkg_blob)?,
            context : RefineContext::decode(work_pkg_blob)?,
            authorization : Vec::<u8>::decode_len(work_pkg_blob)?,
            configuration_blob: Vec::<u8>::decode_len(work_pkg_blob)?,         
            items : Vec::<WorkItem>::decode_len(work_pkg_blob)?,
        })
    }
}

impl Encode for ReportedWorkPackage {

    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::with_capacity(std::mem::size_of::<Self>());
        
        self.hash.encode_to(&mut blob);
        self.exports_root.encode_to(&mut blob);
        
        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for ReportedWorkPackage {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(ReportedWorkPackage{
            hash: Hash::decode(blob)?,
            exports_root: Hash::decode(blob)?,
        })
    }
}

impl Encode for AvailabilityAssignment {

    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::with_capacity(std::mem::size_of::<Self>());

        self.report.encode_to(&mut blob);
        self.timeout.encode_to(&mut blob);

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for AvailabilityAssignment {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(AvailabilityAssignment {
            report: WorkReport::decode(blob)?,
            timeout: u32::decode(blob)?,
        })
    }
}

impl Encode for AvailabilityAssignments {

    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::with_capacity(std::mem::size_of::<Self>() * CORES_COUNT);

        for assigment in self.list.iter() {
            assigment.encode_to(&mut blob);
        }

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for AvailabilityAssignments {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        let mut assignments: AvailabilityAssignments = AvailabilityAssignments::default();
        
        for assignment in assignments.list.iter_mut() {
            *assignment = AvailabilityAssignmentsItem::decode(blob)?;
        }

        Ok(assignments)
    }
}

