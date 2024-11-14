use crate::types::{
                    OpaqueHash, TimeSlot, Ed25519Key, ValidatorIndex, 
                    BandersnatchVrfSignature, BandersnatchKey, TicketAttempt
};
use crate::constants::{EPOCH_LENGTH, VALIDATORS_COUNT};
use crate::codec::{Encode, EncodeSize, Decode, BytesReader, ReadError};
use crate::codec::{encode_unsigned, decode_unsigned};
//use crate::codec::safrole::{EpochMark};

// The epoch and winning-tickets markers are information placed in the header in order to minimize 
// data transfer necessary to determine the validator keys associated with any given epoch. They 
// are particularly useful to nodes which do not synchronize the entire state for any given block 
// since they facilitate the secure tracking of changes to the validator key sets using only the 
// chain of headers.


// The epoch marker specifies key and entropy relevant to the following epoch in case the ticket 
// contest does not complete adequately (a very much unexpected eventuality).The epoch marker is
// either empty or, if the block is the first in a new epoch, then a tuple of the epoch randomness 
// and a sequence of Bandersnatch keys defining the Bandersnatch validator keys (kb) beginning in 
// the next epoch.

#[derive(Debug, PartialEq)]
pub struct EpochMark {
    pub entropy: OpaqueHash,
    pub validators: Vec<BandersnatchKey>,
}

impl Encode for EpochMark {
    
    fn encode(&self) -> Vec<u8> {

        let mut blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<EpochMark>());
        
        self.entropy.encode_to(&mut blob);
        
        for validator in self.validators.iter() {
            validator.encode_to(&mut blob);
        }

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for EpochMark {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {      

        Ok(EpochMark {
            entropy: OpaqueHash::decode(blob)?,
            validators: {
                let mut validators_vec: Vec<BandersnatchKey> = Vec::with_capacity(VALIDATORS_COUNT);
                for _ in 0..VALIDATORS_COUNT {
                    validators_vec.push(BandersnatchKey::decode(blob)?);
                }
                validators_vec
            },
        })  
    }
}

// The Tickets Marker provides the series of EPOCH_LENGTH (600) slot sealing “tickets” for the next epoch. Is either 
// empty or, if the block is the first after the end of the submission period for tickets and if the ticket accumulator 
// is saturated, then the final sequence of ticket identifiers.
#[derive(Debug, PartialEq, Clone)]
pub struct TicketsMark {
    pub tickets_mark: Vec<TicketBody>,
}

impl Encode for TicketsMark {

    fn encode(&self) -> Vec<u8> {

        let len = self.tickets_mark.len();
        let mut tickets_mark_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<TicketBody>() * len);
//        encode_unsigned(len).encode_to(&mut tickets_mark_blob);

        for ticket in &self.tickets_mark {
            ticket.encode_to(&mut tickets_mark_blob);
        }
        
        return tickets_mark_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for TicketsMark {

    fn decode(tickets_mark_blob: &mut BytesReader) -> Result<Self, ReadError> {

        //let len = decode_unsigned(tickets_mark_blob)?;
        let mut tickets = Vec::with_capacity(EPOCH_LENGTH);

        for _ in 0..EPOCH_LENGTH {
            tickets.push(TicketBody::decode(tickets_mark_blob)?);
        }

        Ok(TicketsMark {
            tickets_mark: tickets,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Ord, PartialOrd, Eq)]
pub struct TicketBody {
    pub id: OpaqueHash,
    pub attempt: u8,
}

impl Decode for TicketBody {
    
    fn decode(body_blob: &mut BytesReader) -> Result<Self, ReadError> {
        
        Ok( TicketBody {
            id: OpaqueHash::decode(body_blob)?,
            attempt: u8::decode(body_blob)?,
        })
    }
}

impl Encode for TicketBody {

    fn encode(&self) -> Vec<u8> {

        let mut body_blob = Vec::with_capacity(std::mem::size_of::<TicketBody>());

        self.id.encode_to(&mut body_blob);
        self.attempt.encode_to(&mut body_blob);

        return body_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl TicketBody {
   
    pub fn decode_len(blob: &mut BytesReader) -> Result<Vec<Self>, ReadError> {
        
        let len = decode_unsigned(blob)?;
        let mut tickets_mark: Vec<TicketBody> = Vec::with_capacity(std::mem::size_of::<TicketBody>() * len);

        for _ in 0..len {
            tickets_mark.push(TicketBody::decode(blob)?);
        }

        return Ok(tickets_mark);
    }

    pub fn encode_len(tickets_body: &Vec<TicketBody>, len: usize) -> Vec<u8> {
        
        let mut body_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<TicketBody>() * len);
        encode_unsigned(len).encode_to(&mut body_blob);

        for ticket in tickets_body {
            ticket.encode_to(&mut body_blob);
        }

        return body_blob;
    }
}

// The header comprises a parent hash and prior state root, an extrinsic hash, a time-slot index, the epoch, 
// winning-tickets and offenders markers, and, a Bandersnatch block author index and two Bandersnatch signatures; 
// the entropy-yielding, vrf signature, and a block seal. Excepting the Genesis header, all block headers H have
// an associated parent header, whose hash is Hp.

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

impl Decode for Header {

    fn decode(header_blob: &mut BytesReader) -> Result<Self, ReadError> {
        
        let parent = OpaqueHash::decode(header_blob)?;
        let parent_state_root = OpaqueHash::decode(header_blob)?;
        let extrinsic_hash = OpaqueHash::decode(header_blob)?;
        let slot = TimeSlot::decode(header_blob)?;
        
        let epoch_mark = if header_blob.read_byte()? != 0 {
                Some(EpochMark::decode(header_blob)?)
            } else {
                None
        };

        let tickets_mark = if header_blob.read_byte()? != 0 {
                Some(TicketsMark::decode(header_blob)?)
            } else {
                None
        };

        let num_offenders = decode_unsigned(header_blob)?;
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
}

impl Encode for Header {

    fn encode(&self) -> Vec<u8> {

        let mut header_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<Header>());
        self.parent.encode_to(&mut header_blob);
        self.parent_state_root.encode_to(&mut header_blob);
        self.extrinsic_hash.encode_to(&mut header_blob);
        self.slot.encode_size(4).encode_to(&mut header_blob);
  
        if let Some(epoch_mark) = &self.epoch_mark {
            (1u8).encode_to(&mut header_blob); // 1 = Mark there is epoch 
            epoch_mark.encode_to(&mut header_blob);
        } else {
            (0u8).encode_to(&mut header_blob); // 0 = Mark there isn't epoch
        }

        if let Some(tickets_mark) = &self.tickets_mark {
            (1u8).encode_to(&mut header_blob); // 1 = Mark there are tickets 
            tickets_mark.encode_to(&mut header_blob);
        } else {
            (0u8).encode_to(&mut header_blob); // 0 = Mark there aren't tickets
        }
        
        encode_unsigned(self.offenders_mark.len()).encode_to(&mut header_blob);
        for mark in &self.offenders_mark {
            mark.encode_to(&mut header_blob);
        }

        self.author_index.encode_size(2).encode_to(&mut header_blob);
        self.entropy_source.encode_to(&mut header_blob);
        self.seal.encode_to(&mut header_blob);

        return header_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode()); 
    }
}