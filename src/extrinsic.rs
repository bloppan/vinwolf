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
    pub fn decode(dispute_blob: &[u8]) -> Self {
        let num_verdicts: usize = dispute_blob[0] as usize;
        let mut verdicts: Vec<Verdict> = Vec::with_capacity(num_verdicts);
        let mut i = 1;
        for _ in 0..num_verdicts {
            let mut votes: Vec<Judgement> = Vec::with_capacity(VALIDATORS_SUPER_MAJORITY);
            let mut target = [0u8; 32];
            target.copy_from_slice(&dispute_blob[i..i + 32]);
            i += 32;
            let age = decode_trivial(&dispute_blob[i..i + 4][..]) as u32;
            i += 4;
            for _ in 0..VALIDATORS_SUPER_MAJORITY {
                let vote: bool = dispute_blob[i] != 0;
                i += 1;
                let index = decode_trivial(&dispute_blob[i..i + 2][..]) as u16;
                i += 2;
                let mut signature = [0u8; 64];
                signature.copy_from_slice(&dispute_blob[i..i + 64]);
                i += 64;
                let v = Judgement {vote, index, signature};
                votes.push(v);
            }
            let verdict = Verdict {target, age, votes};
            verdicts.push(verdict);
        }
        let num_culprits: usize = dispute_blob[i] as usize;
        i += 1;
        let mut culprits: Vec<Culprit> = Vec::with_capacity(num_culprits);
        for _ in 0..num_culprits {
            let mut target = [0u8; 32];
            target.copy_from_slice(&dispute_blob[i..i + 32]);
            i += 32;
            let mut key = [0u8; 32];
            key.copy_from_slice(&dispute_blob[i..i + 32]);
            i += 32;
            let mut signature = [0u8; 64];
            signature.copy_from_slice(&dispute_blob[i..i + 64]);
            i += 64;
            let culprit = Culprit {target, key, signature};
            culprits.push(culprit);
        }
        let num_faults: usize = dispute_blob[i] as usize;
        i += 1;
        let mut faults: Vec<Fault> = Vec::with_capacity(num_faults);
        for _ in 0..num_faults {
            let mut target = [0u8; 32];
            target.copy_from_slice(&dispute_blob[i..i + 32]);
            i += 32;
            let vote: bool = dispute_blob[i] != 0;
            i += 1;
            let mut key = [0u8; 32];
            key.copy_from_slice(&dispute_blob[i..i + 32]);
            i += 32;
            let mut signature = [0u8; 64];
            signature.copy_from_slice(&dispute_blob[i..i + 64]);
            i += 64;
            let fault = Fault {target, vote, key, signature};
            faults.push(fault);
        }

        DisputesExtrinsic {
            verdicts,
            culprits,
            faults,
        }
    }
    pub fn encode(dispute: &DisputesExtrinsic) -> Vec<u8> {
        let mut dispute_blob: Vec<u8> = Vec::new();
        let num_verdicts = dispute.verdicts.len() as u8;
        dispute_blob.push(num_verdicts);
        for i in 0..num_verdicts {
            dispute_blob.extend_from_slice(&dispute.verdicts[i as usize].target);
            dispute_blob.extend_from_slice(&encode_trivial(dispute.verdicts[i as usize].age as usize, 4));
            for j in 0..VALIDATORS_SUPER_MAJORITY {
                dispute_blob.push(dispute.verdicts[i as usize].votes[j as usize].vote as u8);
                dispute_blob.extend_from_slice(&encode_trivial(dispute.verdicts[i as usize].votes[j as usize].index as usize, 2));
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
        dispute_blob
    }
}

pub struct TicketEnvelope {
    attempt: TicketAttempt,
    signature: BandersnatchRingSignature,
}

impl TicketEnvelope {
    pub fn decode(ticket_blob: &[u8]) -> Vec<Self> {
        let num_tickets: usize = ticket_blob[0] as usize;
        let mut index = 1;
        let mut ticket_envelop = Vec::with_capacity(num_tickets);
        for _ in 0..num_tickets {
            let attempt = ticket_blob[index];
            index += 1;
            let signature = ticket_blob[index..index + 784]
                                        .try_into()
                                        .expect("slice with incorrect length for signature");
            index += 784;
            ticket_envelop.push(TicketEnvelope{attempt, signature});
        }
        
        ticket_envelop
    }
    pub fn encode(ticket: &[TicketEnvelope]) -> Vec<u8> {
        let num_tickets = ticket.len();
        let mut ticket_blob: Vec<u8> = vec![];
        ticket_blob.push(num_tickets as u8);
        for i in 0..num_tickets {
            ticket_blob.push(ticket[i].attempt);
            ticket_blob.extend_from_slice(&ticket[i].signature);
        }

        ticket_blob
    }
}
