use crate::utils::codec::{Encode, EncodeSize, Decode, BytesReader, ReadError};
use crate::utils::codec::generic::{encode_unsigned, decode_unsigned};
use crate::types::{
    TimeSlot, WorkReport, ValidatorIndex, Ed25519Signature, ReportGuarantee, ValidatorSignature, GuaranteesExtrinsic
};

impl Encode for GuaranteesExtrinsic {

    fn encode(&self) -> Vec<u8> {

        let mut guarantees_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<Self>() * self.report_guarantee.len());
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