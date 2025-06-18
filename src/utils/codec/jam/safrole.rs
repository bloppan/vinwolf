use crate::types::{
    TicketsOrKeys ,EpochMark, OutputDataSafrole, OutputSafrole, Safrole, SafroleErrorCode, TicketsMark, ValidatorsData, BandersnatchRingCommitment,
    TicketBody
};
use crate::utils::codec::{Encode, EncodeLen, Decode, DecodeLen, BytesReader, ReadError};

impl Encode for Safrole {

    fn encode(&self) -> Vec<u8> {
        
        let mut blob = Vec::new();

        self.pending_validators.encode_to(&mut blob);
        self.epoch_root.encode_to(&mut blob);
        self.seal.encode_to(&mut blob);
        self.ticket_accumulator.encode_len().encode_to(&mut blob);

        return blob;
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for Safrole {
    
    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(Safrole {
            pending_validators: ValidatorsData::decode(blob)?,
            epoch_root: BandersnatchRingCommitment::decode(blob)?,
            seal: TicketsOrKeys::decode(blob)?,
            ticket_accumulator: Vec::<TicketBody>::decode_len(blob)?,
        })
    }
}

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
                7 => SafroleErrorCode::TooManyTickets,
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
            tickets_mark.encode_to(&mut output_mark_blob);
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
            Some(TicketsMark::decode(blob)?)
        } else {
            None
        };

        Ok(OutputDataSafrole {
            epoch_mark,
            tickets_mark,
        })
    }
}