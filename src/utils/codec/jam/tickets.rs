use crate::types::{
    BandersnatchPublic, BandersnatchRingVrfSignature, TicketAttempt, TicketBody, TicketEnvelope, TicketsExtrinsic, TicketsMark, TicketsOrKeys
};
use crate::constants::EPOCH_LENGTH;
use crate::utils::codec::{Encode, Decode, BytesReader, ReadError};
use crate::utils::codec::generic::{encode_unsigned, decode_unsigned};

// Tickets Extrinsic is a sequence of proofs of valid tickets; a ticket implies an entry in our epochal “contest” 
// to determine which validators are privileged to author a block for each timeslot in the following epoch. 
// Tickets specify an entry index together with a proof of ticket’s validity. The proof implies a ticket identifier, 
// a high-entropy unbiasable 32-octet sequence, which is used both as a score in the aforementioned contest and as 
// input to the on-chain vrf. 
// Towards the end of the epoch (i.e. Y slots from the start) this contest is closed implying successive blocks 
// within the same epoch must have an empty tickets extrinsic. At this point, the following epoch’s seal key sequence 
// becomes fixed. 
// We define the extrinsic as a sequence of proofs of valid tickets, each of which is a tuple of an entry index 
// (a natural number less than N) and a proof of ticket validity.

impl Decode for TicketsExtrinsic {
    
    fn decode(ticket_blob: &mut BytesReader) -> Result<Self, ReadError> {
    
        let num_tickets = decode_unsigned(ticket_blob)?;
        let mut ticket_envelop = Vec::with_capacity(num_tickets);
    
        for _ in 0..num_tickets {
            let attempt = TicketAttempt::decode(ticket_blob)?;
            let signature = BandersnatchRingVrfSignature::decode(ticket_blob)?;
            ticket_envelop.push(TicketEnvelope{ attempt, signature });
        }      
    
        Ok(TicketsExtrinsic { tickets: ticket_envelop })
    }
}

impl Encode for TicketsExtrinsic {
    
    fn encode(&self) -> Vec<u8> {
        
        let mut ticket_blob: Vec<u8> = Vec::new();
        self.encode_len().encode_to(&mut ticket_blob);
        
        return ticket_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode()); 
    }
}

impl TicketsExtrinsic {

    fn encode_len(&self) -> Vec<u8> {

        let num_tickets = self.tickets.len();
        let mut ticket_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<TicketEnvelope>() * num_tickets);
        
        encode_unsigned(num_tickets).encode_to(&mut ticket_blob);
        
        for ticket in &self.tickets {
            ticket.attempt.encode_to(&mut ticket_blob);
            ticket.signature.encode_to(&mut ticket_blob);
        }
        
        return ticket_blob;
    }

    pub fn len(&self) -> usize {
        self.tickets.len()  
    }
}

impl Decode for TicketsOrKeys {
    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        
        let marker = blob.read_byte()?;  

        match marker {
            1 => {
                let mut keys = Box::new(std::array::from_fn(|_| BandersnatchPublic::default()));
                for key in keys.iter_mut() {
                    *key = BandersnatchPublic::decode(blob)?;
                }
        
                Ok(TicketsOrKeys::Keys(keys))
            }
            0 => {        
                let mut tickets = TicketsMark{ tickets_mark: Box::new(std::array::from_fn(|_| TicketBody::default())) };
                for ticket in tickets.tickets_mark.iter_mut() {
                    *ticket = TicketBody::decode(blob)?;
                }
                
                Ok(TicketsOrKeys::Tickets(tickets)) 
            }
            _ => {
                Err(ReadError::InvalidData)
            }
        }
    }
}

impl Encode for TicketsOrKeys {

    fn encode(&self) -> Vec<u8> {

        let mut encoded = Vec::new();

        match self {
            TicketsOrKeys::Keys(keys) => {
                encoded.push(1); // Keys marker
                for key in keys.iter() {
                    key.encode_to(&mut encoded);
                }
            }
            TicketsOrKeys::Tickets(tickets) => {
                encoded.push(0); // Tickets marker
                for ticket in tickets.tickets_mark.iter() {
                    ticket.encode_to(&mut encoded);
                }
            }
            _ => {},
        }

        return encoded;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}
