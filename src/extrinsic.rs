use crate::types::{
    ValidatorIndex, Ed25519Key, Ed25519Signature, TimeSlot, 
    OpaqueHash, ServiceId, BandersnatchRingSignature, TicketAttempt
};
use crate::globals::{VALIDATORS_SUPER_MAJORITY, AVAIL_BITFIELD_BYTES};
use crate::codec::{Encode, EncodeSize, EncodeLen, Decode, DecodeLen, ReadError, BytesReader};
use crate::work::package::WorkReport;

struct ValidatorSignature {
    validator_index: ValidatorIndex,
    signature: Ed25519Signature,
}

struct ReportGuarantee {
    report: WorkReport,
    slot: TimeSlot,
    signatures: Vec<ValidatorSignature>,
}

pub struct GuaranteesExtrinsic {
    report_guarantee: Vec<ReportGuarantee>,
}

impl GuaranteesExtrinsic {

    pub fn decode(guarantees_blob: &mut BytesReader) -> Result<Self, ReadError> {
        
        let num_guarantees = guarantees_blob.read_byte()? as usize;
        let mut report_guarantee: Vec<ReportGuarantee> = Vec::with_capacity(num_guarantees);
        for _ in 0..num_guarantees {
            let report = WorkReport::decode(guarantees_blob)?;
            let slot = TimeSlot::decode(guarantees_blob)?;
            let num_signatures = guarantees_blob.read_byte()? as usize;
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

    pub fn encode(&self) -> Result<Vec<u8>, ReadError> {

        let mut guarantees_blob: Vec<u8> = Vec::new();
        guarantees_blob.push(self.report_guarantee.len() as u8);

        for guarantee in &self.report_guarantee {
            let _ = guarantee.report.encode_to(&mut guarantees_blob);
            guarantee.slot.encode_size(4).encode_to(&mut guarantees_blob);
            guarantees_blob.push(guarantee.signatures.len() as u8);
            for signature in &guarantee.signatures {
                signature.validator_index.encode_to(&mut guarantees_blob);
                signature.signature.encode_to(&mut guarantees_blob);
            }
        }

        Ok(guarantees_blob)
    }

    pub fn encode_to(&self, into: &mut Vec<u8>) -> Result<(), ReadError> {
        into.extend_from_slice(&self.encode()?); 
        Ok(())
    }
}

struct AvailAssurance {
    anchor: OpaqueHash,
    bitfield: [u8; AVAIL_BITFIELD_BYTES],
    validator_index: ValidatorIndex,
    signature: Ed25519Signature,
}

pub struct AssurancesExtrinsic {
    assurances: Vec<AvailAssurance>, 
}

impl AssurancesExtrinsic {

    pub fn decode(assurances_blob: &mut BytesReader) -> Result<Self, ReadError> {
        
        let num_assurances = assurances_blob.read_byte()? as usize;
        let mut assurances = Vec::with_capacity(num_assurances);  

        for _ in 0..num_assurances {
            let anchor = OpaqueHash::decode(assurances_blob)?;
            let bitfield = <[u8; AVAIL_BITFIELD_BYTES]>::decode(assurances_blob)?;
            let validator_index = ValidatorIndex::decode(assurances_blob)?;
            let signature = Ed25519Signature::decode(assurances_blob)?;
            
            assurances.push(AvailAssurance {
                anchor,
                bitfield,
                validator_index,
                signature,
            });
        }

        Ok(AssurancesExtrinsic { assurances })
    }

    pub fn encode(&self) -> Result<Vec<u8>, ReadError> {

        let mut assurances_blob: Vec<u8> = Vec::new();
        assurances_blob.push(self.assurances.len() as u8);

        for assurance in &self.assurances {
            assurance.anchor.encode_to(&mut assurances_blob);
            assurance.bitfield.encode_to(&mut assurances_blob);
            assurance.validator_index.encode_size(2).encode_to(&mut assurances_blob);
            assurance.signature.encode_to(&mut assurances_blob);
        }

        Ok(assurances_blob)
    }

    pub fn encode_to(&self, into: &mut Vec<u8>) -> Result<(), ReadError> {
        into.extend_from_slice(&self.encode()?); 
        Ok(())
    }
}

struct Preimage {
    requester: ServiceId,
    blob: Vec<u8>,
}

pub struct PreimagesExtrinsic {
    preimages: Vec<Preimage>,
}

impl PreimagesExtrinsic {

    pub fn decode(preimage_blob: &mut BytesReader) -> Result<Self, ReadError> {
        let num_preimages = preimage_blob.read_byte()? as usize;
        let mut preimg_extrinsic: Vec<Preimage> = Vec::with_capacity(num_preimages);
        for _ in 0..num_preimages {
            let requester = ServiceId::decode(preimage_blob)?;
            let blob = Vec::<u8>::decode_len(preimage_blob)?;
            preimg_extrinsic.push(Preimage { requester, blob });
        }
        Ok(PreimagesExtrinsic { preimages: preimg_extrinsic })
    }

    pub fn encode(&self) -> Result<Vec<u8>, ReadError> {
        let mut preimg_encoded = Vec::with_capacity(std::mem::size_of::<PreimagesExtrinsic>());
        preimg_encoded.push(self.preimages.len() as u8);
        for preimage in &self.preimages {
            preimage.requester.encode_to(&mut preimg_encoded);
            preimage.blob.as_slice().encode_len().encode_to(&mut preimg_encoded);
        }
        Ok(preimg_encoded)
    }

    pub fn encode_to(&self, into: &mut Vec<u8>) -> Result<(), ReadError> {
        into.extend_from_slice(&self.encode()?); 
        Ok(())
    }
}

struct Judgement {
    vote: bool,
    index: ValidatorIndex,
    signature: Ed25519Signature,
}

impl Judgement {
    fn decode_len(judgement_blob: &mut BytesReader) -> Result<Vec<Self>, ReadError> {
        let mut votes: Vec<Judgement> = Vec::with_capacity(VALIDATORS_SUPER_MAJORITY);
        for _ in 0..VALIDATORS_SUPER_MAJORITY {
            let vote: bool = judgement_blob.read_byte()? != 0;
            let index = ValidatorIndex::decode(judgement_blob)?;
            let signature = Ed25519Signature::decode(judgement_blob)?; 
            let judgement = Judgement {vote, index, signature};
            votes.push(judgement);
        }
        Ok(votes)
    }
}

pub struct DisputesExtrinsic {
    verdicts: Vec<Verdict>,
    culprits: Vec<Culprit>,
    faults: Vec<Fault>,
}

struct Culprit {
    target: OpaqueHash,
    key: Ed25519Key,
    signature: Ed25519Signature,
}

impl Culprit {
    fn decode_len(culprit_blob: &mut BytesReader) -> Result<Vec<Self>, ReadError> {
        let num_culprits = culprit_blob.read_byte()? as usize; 
        let mut culprits: Vec<Culprit> = Vec::with_capacity(num_culprits);
        for _ in 0..num_culprits {
            let target = OpaqueHash::decode(culprit_blob)?;
            let key = Ed25519Key::decode(culprit_blob)?; 
            let signature = Ed25519Signature::decode(culprit_blob)?;
            let culprit = Culprit {target, key, signature};
            culprits.push(culprit);
        }
        Ok(culprits)
    }
}

struct Verdict {
    target: OpaqueHash,
    age: u32,
    votes: Vec<Judgement>,
}

impl Verdict {
    fn decode_len(verdict_blob: &mut BytesReader) -> Result<Vec<Self>, ReadError> {
        let num_verdicts = verdict_blob.read_byte()? as usize;
        let mut verdicts: Vec<Verdict> = Vec::with_capacity(num_verdicts);
        for _ in 0..num_verdicts {
            let target = OpaqueHash::decode(verdict_blob)?;
            let age = u32::decode(verdict_blob)?;
            let votes = Judgement::decode_len(verdict_blob)?;
            verdicts.push(Verdict {target, age, votes});
        }
        Ok(verdicts)
    }
}

struct Fault {
    target: OpaqueHash,
    vote: bool,
    key: Ed25519Key,
    signature: Ed25519Signature,
}

impl Fault {
    fn decode_len(faults_blob: &mut BytesReader) -> Result<Vec<Self>, ReadError> {
        let num_faults = faults_blob.read_byte()? as usize;
        let mut faults: Vec<Fault> = Vec::with_capacity(num_faults);
        for _ in 0..num_faults {
            let target = OpaqueHash::decode(faults_blob)?; 
            let vote: bool = faults_blob.read_byte()? != 0;
            let key = OpaqueHash::decode(faults_blob)?;
            let signature = Ed25519Signature::decode(faults_blob)?;
            let fault = Fault {target, vote, key, signature};
            faults.push(fault);
        }
        Ok(faults)
    }
}

impl DisputesExtrinsic {
    pub fn decode(dispute_blob: &mut BytesReader) -> Result<Self, ReadError> {

        let verdicts = Verdict::decode_len(dispute_blob)?;
        let culprits = Culprit::decode_len(dispute_blob)?;
        let faults = Fault::decode_len(dispute_blob)?;

        Ok(DisputesExtrinsic {
            verdicts,
            culprits,
            faults,
        })
    }

    pub fn encode(&self) -> Result<Vec<u8>, ReadError> {

        let mut dispute_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<DisputesExtrinsic>());
        dispute_blob.push(self.verdicts.len() as u8);
        for verdict in &self.verdicts {
            verdict.target.encode_to(&mut dispute_blob);
            verdict.age.encode_size(4).encode_to(&mut dispute_blob);
            for j in 0..VALIDATORS_SUPER_MAJORITY {
                dispute_blob.push(verdict.votes[j as usize].vote as u8);
                verdict.votes[j as usize].index.encode_size(2).encode_to(&mut dispute_blob);
                verdict.votes[j as usize].signature.encode_to(&mut dispute_blob);
            }
        }
        dispute_blob.push(self.culprits.len() as u8);
        for culprit in &self.culprits {
            culprit.target.encode_to(&mut dispute_blob);
            culprit.key.encode_to(&mut dispute_blob);
            culprit.signature.encode_to(&mut dispute_blob);
        }
        dispute_blob.push(self.faults.len() as u8);
        for fault in &self.faults {
            fault.target.encode_to(&mut dispute_blob);
            dispute_blob.push(fault.vote as u8);
            fault.key.encode_to(&mut dispute_blob);
            fault.signature.encode_to(&mut dispute_blob);
        }
        
        Ok(dispute_blob)
    }

    pub fn encode_to(&self, into: &mut Vec<u8>) -> Result<(), ReadError> {
        into.extend_from_slice(&self.encode()?); 
        Ok(())
    }
}

pub struct TicketsExtrinsic { 
    tickets: Vec<TicketEnvelope>,
}

pub struct TicketEnvelope {
    pub attempt: TicketAttempt,
    pub signature: BandersnatchRingSignature,
}

impl TicketsExtrinsic {
    pub fn decode(ticket_blob: &mut BytesReader) -> Result<Self, ReadError> {
        let num_tickets = ticket_blob.read_byte()? as usize;
        let mut ticket_envelop = Vec::with_capacity(num_tickets);
        for _ in 0..num_tickets {
            let attempt = TicketAttempt::decode(ticket_blob)?;
            let signature = BandersnatchRingSignature::decode(ticket_blob)?;
            ticket_envelop.push(TicketEnvelope{ attempt, signature });
        }      
        Ok(TicketsExtrinsic { tickets: ticket_envelop })
    }

    pub fn encode(&self) -> Result<Vec<u8>, ReadError> {
        let mut ticket_blob: Vec<u8> = Vec::new();
        self.encode_len()?.encode_to(&mut ticket_blob);
        Ok(ticket_blob)
    }

    fn encode_len(&self) -> Result<Vec<u8>, ReadError> {
        let num_tickets = self.tickets.len();
        let mut ticket_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<TicketEnvelope>() * num_tickets);
        ticket_blob.push(num_tickets as u8);
        for ticket in &self.tickets {
            ticket.attempt.encode_to(&mut ticket_blob);
            ticket.signature.encode_to(&mut ticket_blob);
        }
        Ok(ticket_blob)
    }

    pub fn encode_to(&self, into: &mut Vec<u8>) -> Result<(), ReadError> {
        into.extend_from_slice(&self.encode()?); 
        Ok(())
    }
}
