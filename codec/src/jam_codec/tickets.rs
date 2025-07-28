use jam_types::{BandersnatchPublic, BandersnatchRingVrfSignature, TicketAttempt, TicketBody, Ticket, TicketsMark, TicketsOrKeys, BandersnatchEpoch};
use crate::{Encode, Decode, BytesReader, ReadError};

impl Encode for Ticket {

    fn encode(&self) -> Vec<u8> {
        
        let mut blob = Vec::new();

        self.attempt.encode_to(&mut blob);
        self.signature.encode_to(&mut blob);

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode()); 
    }
}

impl Decode for Ticket {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
            
        Ok(Ticket{
            attempt: TicketAttempt::decode(blob)?,
            signature: BandersnatchRingVrfSignature::decode(blob)?,
        })
    }
}

impl Decode for TicketsOrKeys {
    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        
        let marker = blob.read_byte()?;  

        match marker {
            1 => {
                    let mut keys: BandersnatchEpoch = BandersnatchEpoch::default();

                    for key in keys.epoch.iter_mut() {
                        *key = BandersnatchPublic::decode(blob)?;
                    }

                    Ok(TicketsOrKeys::Keys(keys))
            }
            0 => {        
                    let mut tickets = TicketsMark::default();

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
                for key in keys.epoch.iter() {
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
