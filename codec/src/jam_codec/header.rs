use jam_types::{OpaqueHash, TimeSlot, Ed25519Public, ValidatorIndex, BandersnatchVrfSignature, BandersnatchPublic, TicketBody, TicketsMark, EpochMark, Entropy};
use constants::node::{EPOCH_LENGTH, VALIDATORS_COUNT};
use crate::{Encode, EncodeLen, EncodeSize, Decode, BytesReader, ReadError};
use crate::generic_codec::decode_unsigned;



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

impl Encode for EpochMark {
    
    fn encode(&self) -> Vec<u8> {

        let mut blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<EpochMark>() + (std::mem::size_of::<BandersnatchPublic>() * VALIDATORS_COUNT));
        
        self.entropy.encode_to(&mut blob);
        self.tickets_entropy.encode_to(&mut blob);

        for validator in self.validators.iter() {
            validator.0.encode_to(&mut blob);
            validator.1.encode_to(&mut blob);
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
            entropy: Entropy::decode(blob)?,
            tickets_entropy: Entropy::decode(blob)?,
            validators: {
                let mut validators = Box::new(std::array::from_fn(|_| (BandersnatchPublic::default(), Ed25519Public::default())));
                for validator in validators.iter_mut() {
                    *validator = (BandersnatchPublic::decode(blob)?, Ed25519Public::decode(blob)?);
                }
                validators
            },
        })  
    }
}

// The Tickets Marker provides the series of EPOCH_LENGTH (600) slot sealing “tickets” for the next epoch. Is either 
// empty or, if the block is the first after the end of the submission period for tickets and if the ticket accumulator 
// is saturated, then the final sequence of ticket identifiers.

impl Encode for TicketsMark {

    fn encode(&self) -> Vec<u8> {

        let mut tickets_mark_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<TicketBody>() * EPOCH_LENGTH);
        
        for ticket in self.tickets_mark.iter() {
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

        let mut tickets = Box::new(std::array::from_fn(|_| TicketBody::default()));

        for ticket in tickets.iter_mut() {
            *ticket = TicketBody::decode(tickets_mark_blob)?;
        }

        Ok(TicketsMark {
            tickets_mark: tickets,
        })
    }
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
