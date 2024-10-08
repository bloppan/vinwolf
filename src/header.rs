use crate::types::*;
use crate::globals::*;

use crate::codec::*;

#[derive(Debug)]
struct EpochMark {
    entropy: OpaqueHash,
    validators: Vec<BandersnatchKey>,
}

impl EpochMark {
    pub fn encode(&self) -> Vec<u8> {
        let mut blob: Vec<u8> = Vec::new();
        blob.extend_from_slice(&self.entropy);
        for validator in &self.validators {
            blob.extend_from_slice(validator);
        }
        blob
    }
}

#[derive(Debug)]
struct TicketBody {
    id: OpaqueHash,
    attempt: TicketAttempt,
}
#[derive(Debug)]
struct TicketsMark {
    tickets_mark: Vec<TicketBody>,
}

impl TicketsMark {
    pub fn encode(&self) -> Vec<u8> {
        let mut blob: Vec<u8> = Vec::new();
        for ticket in &self.tickets_mark {
            blob.extend_from_slice(&ticket.id);   
            blob.push(ticket.attempt);
        }
        blob
    }
}

#[derive(Debug)]
pub struct Header {
    parent: OpaqueHash,
    parent_state_root: OpaqueHash,
    extrinsic_hash: OpaqueHash,
    slot: TimeSlot,
    epoch_mark: Option<EpochMark>,
    tickets_mark: Option<TicketsMark>,
    offenders_mark: Vec<Ed25519Key>,
    author_index: ValidatorIndex,
    entropy_source: BandersnatchVrfSignature,
    seal: BandersnatchVrfSignature,
}

impl Header {

    pub fn decode(header_blob: &mut BytesReader) -> Result<Self, ReadError> {
        
        let parent = OpaqueHash::decode(header_blob)?;
        let parent_state_root = OpaqueHash::decode(header_blob)?;
        let extrinsic_hash = OpaqueHash::decode(header_blob)?;
        let slot = TimeSlot::decode(header_blob)?;
        
        let epoch_mark = if header_blob.read_byte()? != 0 {
            let entropy = OpaqueHash::decode(header_blob)?;
            let mut validators: Vec<BandersnatchKey> = Vec::with_capacity(NUM_VALIDATORS);
            for _ in 0..NUM_VALIDATORS {
                validators.push(BandersnatchKey::decode(header_blob)?);
            }
            Some(EpochMark { entropy, validators })
            } else {
                None
        };

        let tickets_mark = if header_blob.read_byte()? != 0 {
            let mut tickets: Vec<TicketBody> = Vec::new(); 
            for _ in 0..EPOCH_LENGTH {
                let id = OpaqueHash::decode(header_blob)?;
                let attempt = TicketAttempt::decode(header_blob)?;
                tickets.push(TicketBody { id, attempt });
            }
            Some(TicketsMark { tickets_mark: tickets })
            } else {
                None
        };

        let num_offenders = header_blob.read_byte()? as usize;
        let mut offenders_mark: Vec<Ed25519Key> = Vec::with_capacity(num_offenders);
        for _ in 0..num_offenders {
            offenders_mark.push(Ed25519Key::decode(header_blob)?);
        }
        let author_index = ValidatorIndex::decode(header_blob)?;
        let entropy_source = BandersnatchVrfSignature::decode(header_blob)?;
        let seal = BandersnatchVrfSignature::decode(header_blob)?;
        
        Ok(Header {
            parent,
            parent_state_root,
            extrinsic_hash,
            slot,
            epoch_mark,
            tickets_mark,
            offenders_mark,
            author_index,
            entropy_source,
            seal
        })
    }

    pub fn encode(&self) -> Result<Vec<u8>, ReadError> {

        let mut header_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<Header>());
        self.parent.encode_to(&mut header_blob);
        self.parent_state_root.encode_to(&mut header_blob);
        self.extrinsic_hash.encode_to(&mut header_blob);
        self.slot.encode_size(4).encode_to(&mut header_blob);
  
        if let Some(epoch_mark) = &self.epoch_mark {
            (1u8).encode_to(&mut header_blob);
            header_blob.extend_from_slice(&epoch_mark.encode());
        } else {
            (0u8).encode_to(&mut header_blob);
        }

        if let Some(tickets_mark) = &self.tickets_mark {
            (1u8).encode_to(&mut header_blob);
            header_blob.extend_from_slice(&tickets_mark.encode()); 
        } else {
            (0u8).encode_to(&mut header_blob);
        }
        
        header_blob.push(self.offenders_mark.len() as u8);
        for mark in &self.offenders_mark {
            mark.encode_to(&mut header_blob);
        }
        self.author_index.encode_size(2).encode_to(&mut header_blob);
        self.entropy_source.encode_to(&mut header_blob);
        self.seal.encode_to(&mut header_blob);

        Ok(header_blob)
    }

    pub fn encode_to(&self, into: &mut Vec<u8>) -> Result<(), ReadError> {
        into.extend_from_slice(&self.encode()?); 
        Ok(())
    }
}