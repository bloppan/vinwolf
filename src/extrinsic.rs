use crate::types::*;
use crate::globals::*;

use crate::codec::*;

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
    pub fn decode(dispute_blob: &[u8]) -> Result<Self, ReadError> {

        let mut blob = SliceReader::new(dispute_blob);
        let num_verdicts: usize = blob.read_next_byte()? as usize;
        let mut verdicts: Vec<Verdict> = Vec::with_capacity(num_verdicts);
        for _ in 0..num_verdicts {
            let mut votes: Vec<Judgement> = Vec::with_capacity(VALIDATORS_SUPER_MAJORITY);
            let mut target = blob.read_32bytes()?;
            let age = blob.read_u32()?; 
            for _ in 0..VALIDATORS_SUPER_MAJORITY {
                let vote: bool = blob.read_next_byte()? != 0;
                let index = blob.read_u16()?; 
                let  signature = blob.read_64bytes()?; 
                let v = Judgement {vote, index, signature};
                votes.push(v);
            }
            let verdict = Verdict {target, age, votes};
            verdicts.push(verdict);
        }
        let num_culprits: usize = blob.read_next_byte()? as usize; 
        let mut culprits: Vec<Culprit> = Vec::with_capacity(num_culprits);
        for _ in 0..num_culprits {
            let target = blob.read_32bytes()?;
            let key = blob.read_32bytes()?; 
            let signature = blob.read_64bytes()?;
            let culprit = Culprit {target, key, signature};
            culprits.push(culprit);
        }
        let num_faults: usize = blob.read_next_byte()? as usize;
        let mut faults: Vec<Fault> = Vec::with_capacity(num_faults);
        for _ in 0..num_faults {
            let target = blob.read_32bytes()?; 
            let vote: bool = blob.read_next_byte()? != 0;
            let key = blob.read_32bytes()?;
            let signature = blob.read_64bytes()?;
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

        let mut dispute_blob: Vec<u8> = Vec::new();
        let num_verdicts = dispute.verdicts.len() as u8;
        dispute_blob.push(num_verdicts);
        for i in 0..num_verdicts {
            dispute_blob.extend_from_slice(&dispute.verdicts[i as usize].target);
            dispute_blob.extend_from_slice(&dispute.verdicts[i as usize].age.to_le_bytes());
            for j in 0..VALIDATORS_SUPER_MAJORITY {
                dispute_blob.push(dispute.verdicts[i as usize].votes[j as usize].vote as u8);
                dispute_blob.extend_from_slice(&dispute.verdicts[i as usize].votes[j as usize].index.to_le_bytes());
                dispute_blob.extend_from_slice(&dispute.verdicts[i as usize].votes[j as usize].signature[..]);
            }
        }
        let num_culprits = dispute.culprits.len() as u8;
        dispute_blob.push(num_culprits);
        for i in 0..num_culprits {
            dispute_blob.extend_from_slice(&dispute.culprits[i as usize].target);
            dispute_blob.extend_from_slice(&dispute.culprits[i as usize].key);
            dispute_blob.extend_from_slice(&dispute.culprits[i as usize].signature);
        }
        let num_faults = dispute.faults.len() as u8;
        dispute_blob.push(num_faults);
        for i in 0..num_faults {
            dispute_blob.extend_from_slice(&dispute.faults[i as usize].target);
            dispute_blob.push(dispute.faults[i as usize].vote as u8);
            dispute_blob.extend_from_slice(&dispute.faults[i as usize].key);
            dispute_blob.extend_from_slice(&dispute.faults[i as usize].signature);
        }
        
        Ok(dispute_blob)
    }
}

pub struct TicketEnvelope {
    attempt: TicketAttempt,
    signature: BandersnatchRingSignature,
}

impl TicketEnvelope {
    pub fn decode(ticket_blob: &[u8]) -> Result<Vec<Self>, ReadError> {

        let mut blob = SliceReader::new(ticket_blob);

        let num_tickets: usize = blob.read_next_byte()? as usize;
        let mut ticket_envelop = Vec::with_capacity(num_tickets);
        for _ in 0..num_tickets {
            let attempt = blob.read_next_byte()?; //ticket_blob[index];
            let signature = blob.read_784bytes()?;
            ticket_envelop.push(TicketEnvelope{attempt, signature});
        }
        
        Ok(ticket_envelop)
    }

    pub fn encode(ticket: &[TicketEnvelope]) -> Result<Vec<u8>, ReadError> {
        let num_tickets = ticket.len();
        let mut ticket_blob: Vec<u8> = vec![];
        ticket_blob.push(num_tickets as u8);
        for i in 0..num_tickets {
            ticket_blob.push(ticket[i].attempt);
            ticket_blob.extend_from_slice(&ticket[i].signature);
        }

        Ok(ticket_blob)
    }
}
