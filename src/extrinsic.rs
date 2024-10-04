use crate::types::*;
use crate::globals::*;

use crate::codec::*;

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
            guarantee.report.encode_to(&mut guarantees_blob);
            guarantee.slot.encode_size(4).encode_to(&mut guarantees_blob);
            guarantees_blob.push(guarantee.signatures.len() as u8);
            for signature in &guarantee.signatures {
                signature.validator_index.encode_to(&mut guarantees_blob);
                signature.signature.encode_to(&mut guarantees_blob);
            }
        }

        Ok(guarantees_blob)
    }

    pub fn encode_to(&self, into: &mut Vec<u8>) -> Result<(), ReadError {
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

    pub fn encode_to(&self, into: &mut Vec<u8>) -> Result<(), ReadError {
        into.extend_from_slice(&self.encode()?); 
        Ok(())
    }
}

struct Preimage {
    requester: ServiceId,
    blob: Vec<u8>,
}

pub struct PreimagesExtrinsic {
    pub preimages: Vec<Preimage>,
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

    pub fn encode_to(&self, into: &mut Vec<u8>) -> Result<(), ReadError {
        into.extend_from_slice(&self.encode()?); 
        Ok(())
    }
}

struct Judgement {
    vote: bool,
    index: ValidatorIndex,
    signature: Ed25519Signature,
}

struct Verdict {
    target: OpaqueHash,
    age: u32,
    votes: Vec<Judgement>,
}

struct Culprit {
    target: OpaqueHash,
    key: Ed25519Key,
    signature: Ed25519Signature,
}

struct Fault {
    target: OpaqueHash,
    vote: bool,
    key: Ed25519Key,
    signature: Ed25519Signature,
}

pub struct DisputesExtrinsic {
    verdicts: Vec<Verdict>,
    culprits: Vec<Culprit>,
    faults: Vec<Fault>,
}

impl DisputesExtrinsic {
    pub fn decode(dispute_blob: &mut BytesReader) -> Result<Self, ReadError> {

        let num_verdicts = dispute_blob.read_byte()? as usize;
        let mut verdicts: Vec<Verdict> = Vec::with_capacity(num_verdicts);
        for _ in 0..num_verdicts {
            let mut votes: Vec<Judgement> = Vec::with_capacity(VALIDATORS_SUPER_MAJORITY);
            let mut target = OpaqueHash::decode(dispute_blob)?;
            let age = u32::decode(dispute_blob)?;
            for _ in 0..VALIDATORS_SUPER_MAJORITY {
                let vote: bool = dispute_blob.read_byte()? != 0;
                let index = ValidatorIndex::decode(dispute_blob)?;
                let signature = Ed25519Signature::decode(dispute_blob)?; 
                let judgement = Judgement {vote, index, signature};
                votes.push(judgement);
            }
            let verdict = Verdict {target, age, votes};
            verdicts.push(verdict);
        }
        let num_culprits = dispute_blob.read_byte()? as usize; 
        let mut culprits: Vec<Culprit> = Vec::with_capacity(num_culprits);
        for _ in 0..num_culprits {
            let target = OpaqueHash::decode(dispute_blob)?;
            let key = Ed25519Key::decode(dispute_blob)?; 
            let signature = Ed25519Signature::decode(dispute_blob)?;
            let culprit = Culprit {target, key, signature};
            culprits.push(culprit);
        }
        let num_faults = dispute_blob.read_byte()? as usize;
        let mut faults: Vec<Fault> = Vec::with_capacity(num_faults);
        for _ in 0..num_faults {
            let target = OpaqueHash::decode(dispute_blob)?; 
            let vote: bool = dispute_blob.read_byte()? != 0;
            let key = OpaqueHash::decode(dispute_blob)?;
            let signature = Ed25519Signature::decode(dispute_blob)?;
            let fault = Fault {target, vote, key, signature};
            faults.push(fault);
        }

        Ok(DisputesExtrinsic {
            verdicts,
            culprits,
            faults,
        })
    }

    pub fn encode(dispute: &DisputesExtrinsic) -> Result<Vec<u8>, ReadError> {

        let mut dispute_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<DisputesExtrinsic>());
        let num_verdicts = dispute.verdicts.len() as u8;
        dispute_blob.push(num_verdicts);
        for i in 0..num_verdicts {
            dispute.verdicts[i as usize].target.encode_to(&mut dispute_blob);
            dispute.verdicts[i as usize].age.encode_size(4).encode_to(&mut dispute_blob);
            
            for j in 0..VALIDATORS_SUPER_MAJORITY {
                dispute_blob.push(dispute.verdicts[i as usize].votes[j as usize].vote as u8);
                dispute.verdicts[i as usize].votes[j as usize].index.encode_size(2).encode_to(&mut dispute_blob);
                dispute.verdicts[i as usize].votes[j as usize].signature.encode_to(&mut dispute_blob);
            }
        }
        let num_culprits = dispute.culprits.len() as u8;
        dispute_blob.push(num_culprits);
        for i in 0..num_culprits {
            dispute.culprits[i as usize].target.encode_to(&mut dispute_blob);
            dispute.culprits[i as usize].key.encode_to(&mut dispute_blob);
            dispute.culprits[i as usize].signature.encode_to(&mut dispute_blob);
        }
        let num_faults = dispute.faults.len() as u8;
        dispute_blob.push(num_faults);
        for i in 0..num_faults {
            dispute.faults[i as usize].target.encode_to(&mut dispute_blob);
            dispute_blob.push(dispute.faults[i as usize].vote as u8);
            dispute.faults[i as usize].key.encode_to(&mut dispute_blob);
            dispute.faults[i as usize].signature.encode_to(&mut dispute_blob);
        }
        
        Ok(dispute_blob)
    }

    pub fn encode_to(&self, into: &mut Vec<u8>) -> Result<(), ReadError {
        into.extend_from_slice(&self.encode()?); 
        Ok(())
    }
}

pub struct TicketsExtrinsic { 
    tickets: Vec<TicketEnvelope>,
}

impl TicketsExtrinsic {
    pub fn decode(tickets_blob: &mut BytesReader) -> Result<Self, ReadError> {
        let tickets = TicketEnvelope::decode(tickets_blob)?; // Decodificar la lista de TicketEnvelope
        Ok(TicketsExtrinsic { tickets })
    }

    // Método encode para TicketsExtrinsic
    pub fn encode(&self) -> Result<Vec<u8>, ReadError> {
        let mut tickets_blob: Vec<u8> = Vec::new();
        // Codificar la lista de tickets
        tickets_blob.extend_from_slice(&TicketEnvelope::encode(&self.tickets)?);
        Ok(tickets_blob)
    }

    // Método encode_to para TicketsExtrinsic
    pub fn encode_to(&self, into: &mut Vec<u8>) -> Result<(), ReadError> {
        into.extend_from_slice(&self.encode()?); // Llama al método encode y extiende
        Ok(())
    }
}

pub struct TicketEnvelope {
    attempt: TicketAttempt,
    signature: BandersnatchRingSignature,
}

impl TicketEnvelope {
    pub fn decode(ticket_blob: &mut BytesReader) -> Result<Vec<Self>, ReadError> {
        let num_tickets = ticket_blob.read_byte()? as usize;
        let mut ticket_envelop = Vec::with_capacity(num_tickets);
        for _ in 0..num_tickets {
            let attempt = TicketAttempt::decode(ticket_blob)?;
            let signature = BandersnatchRingSignature::decode(ticket_blob)?;
            ticket_envelop.push(TicketEnvelope{ attempt, signature });
        }      
        Ok(ticket_envelop)
    }

    pub fn encode(tickets: &[TicketEnvelope]) -> Result<Vec<u8>, ReadError> {
        let num_tickets = tickets.len();
        let mut ticket_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<TicketEnvelope>() * num_tickets);
        ticket_blob.push(num_tickets as u8);
        for i in 0..num_tickets {
            tickets[i].attempt.encode_to(&mut ticket_blob);
            tickets[i].signature.encode_to(&mut ticket_blob);
        }
        Ok(ticket_blob)
    }
}
