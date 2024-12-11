use crate::types::{TimeSlot, ValidatorIndex, Ed25519Signature};
use crate::codec::{Encode, EncodeSize, Decode, BytesReader, ReadError};
use crate::codec::work_report::WorkReport;
use crate::codec::{encode_unsigned, decode_unsigned};

// The guarantees extrinsic is a series of guarantees, at most one for each core, each of which is 
// a tuple of a work-report, a credential and its corresponding timeslot. The core index of each 
// guarantee must be unique and guarantees must be in ascending order of this.
// They are reports of newly completed workloads whose accuracy is guaranteed by specific validators. 
// A work-package, which comprises several work items, is transformed by validators acting as guarantors 
// into its corresponding workreport, which similarly comprises several work outputs and then presented 
// on-chain within the guarantees extrinsic.

#[derive(Debug, Clone, PartialEq)]
pub struct GuaranteesExtrinsic {
    pub report_guarantee: Vec<ReportGuarantee>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReportGuarantee {
    pub report: WorkReport,
    pub slot: TimeSlot,
    pub signatures: Vec<ValidatorSignature>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValidatorSignature {
    pub validator_index: ValidatorIndex,
    pub signature: Ed25519Signature,
}

impl Encode for GuaranteesExtrinsic {

    fn encode(&self) -> Vec<u8> {

        let mut guarantees_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<GuaranteesExtrinsic>());
        encode_unsigned(self.report_guarantee.len()).encode_to(&mut guarantees_blob);

        for guarantee in &self.report_guarantee {

            guarantee.report.encode_to(&mut guarantees_blob);
            guarantee.slot.encode_size(4).encode_to(&mut guarantees_blob);
            encode_unsigned(guarantee.signatures.len()).encode_to(&mut guarantees_blob);

            for signature in &guarantee.signatures {
                signature.validator_index.encode_to(&mut guarantees_blob);
                signature.signature.encode_to(&mut guarantees_blob);
            }
        }

        return guarantees_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode()); 
    }
}

impl Decode for GuaranteesExtrinsic {

    fn decode(guarantees_blob: &mut BytesReader) -> Result<Self, ReadError> {
        
        let num_guarantees = decode_unsigned(guarantees_blob)?;
        let mut report_guarantee: Vec<ReportGuarantee> = Vec::with_capacity(num_guarantees);

        for _ in 0..num_guarantees {

            let report = WorkReport::decode(guarantees_blob)?;
            let slot = TimeSlot::decode(guarantees_blob)?;
            let num_signatures = decode_unsigned(guarantees_blob)?;
            let mut signatures: Vec<ValidatorSignature> = Vec::with_capacity(num_signatures);

            for _ in 0..num_signatures {
                let validator_index = ValidatorIndex::decode(guarantees_blob)?;
                let signature = Ed25519Signature::decode(guarantees_blob)?;
                signatures.push(ValidatorSignature{validator_index, signature});
            }

            report_guarantee.push(ReportGuarantee{report, slot, signatures});
        }

        Ok(GuaranteesExtrinsic{ report_guarantee })
    }
}

