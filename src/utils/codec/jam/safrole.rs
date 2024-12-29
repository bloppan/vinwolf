use crate::types::{TicketBody, EpochMark, OutputSafrole, OutputDataSafrole, SafroleErrorCode};
use crate::constants::EPOCH_LENGTH;
use crate::utils::codec::{Encode, Decode, BytesReader, ReadError};

impl Encode for OutputSafrole {

    fn encode(&self) -> Vec<u8> {

        let mut output_blob = Vec::new();

        match self {
            OutputSafrole::Ok(output_marks) => {
                output_blob.push(0); // OK = 0
                output_marks.encode_to(&mut output_blob);
            }
            OutputSafrole::Err(error_type) => {
                output_blob.push(1); // ERROR = 1
                output_blob.push(*error_type as u8);
            }
        }

        return output_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for OutputSafrole {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        
        if blob.read_byte()? == 0 { // OK = 0
            Ok(OutputSafrole::Ok(OutputDataSafrole::decode(blob)?))           
        } else {
            let error_type = blob.read_byte()?;
            let error = match error_type {
                0 => SafroleErrorCode::BadSlot,
                1 => SafroleErrorCode::UnexpectedTicket,
                2 => SafroleErrorCode::BadTicketOrder,
                3 => SafroleErrorCode::BadTicketProof,
                4 => SafroleErrorCode::BadTicketAttempt,
                5 => SafroleErrorCode::Reserved,
                6 => SafroleErrorCode::DuplicateTicket,
                _ => return Err(ReadError::InvalidData),
            };
            Ok(OutputSafrole::Err(error))
        }
    }
}

impl Encode for OutputDataSafrole {

    fn encode(&self) -> Vec<u8> {

        let mut output_mark_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<OutputDataSafrole>());
        // Encode Epoch Marks
        if let Some(epoch_mark) = &self.epoch_mark {
            output_mark_blob.push(1); 
            epoch_mark.encode_to(&mut output_mark_blob);
        } else {
            output_mark_blob.push(0); 
        }
        // Encode Tickets Marks
        if let Some(tickets_mark) = &self.tickets_mark {
            output_mark_blob.push(1); 
            for ticket in tickets_mark {
                output_mark_blob.extend(ticket.encode());
            }
        } else {
            output_mark_blob.push(0); 
        }

        return output_mark_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for OutputDataSafrole {
    
    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        // Epoch Mark
        let epoch_mark = if blob.read_byte()? == 1 {
            Some(EpochMark::decode(blob)?)
        } else {
            None
        };
        // Tickets Mark
        let tickets_mark = if blob.read_byte()? == 1 {
            let mut tickets: Vec<TicketBody> = Vec::with_capacity(std::mem::size_of::<TicketBody>() * EPOCH_LENGTH);
            for _ in 0..EPOCH_LENGTH {
                tickets.push(TicketBody::decode(blob)?);
            }
            Some(tickets)
        } else {
            None
        };

        Ok(OutputDataSafrole {
            epoch_mark,
            tickets_mark,
        })
    }
}