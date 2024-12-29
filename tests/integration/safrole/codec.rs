use std::array::from_fn;
use crate::types::{
    BandersnatchPublic, Ed25519Public, BlsPublic, Metadata, OpaqueHash, TimeSlot, BandersnatchRingCommitment,
    ValidatorData, ValidatorsData, TicketsExtrinsic, TicketBody, TicketsOrKeys, EpochMark};
use crate::constants::{VALIDATORS_COUNT, EPOCH_LENGTH};
use crate::utils::codec::{Encode, Decode, DecodeLen, BytesReader, ReadError};
use crate::utils::codec::generic::encode_unsigned;

#[derive(Debug)]
pub struct InputSafrole {
    pub slot: TimeSlot,
    pub entropy: OpaqueHash,
    pub extrinsic: TicketsExtrinsic,
    pub post_offenders: Vec<Ed25519Public>,
}

impl Encode for InputSafrole {

    fn encode(&self) -> Vec<u8> {

        let mut input_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<InputSafrole>());
        self.slot.encode_to(&mut input_blob);
        self.entropy.encode_to(&mut input_blob);
        self.extrinsic.encode_to(&mut input_blob);
        encode_unsigned(self.post_offenders.len()).encode_to(&mut input_blob);
        self.post_offenders.encode_to(&mut input_blob);

        return input_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for InputSafrole {

    fn decode(input_blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(InputSafrole {
            slot: TimeSlot::decode(input_blob)?,
            entropy: OpaqueHash::decode(input_blob)?,
            extrinsic: TicketsExtrinsic::decode(input_blob)?,
            post_offenders: Vec::<Ed25519Public>::decode_len(input_blob)?,
        })
    }
}

/*
    @gamma_k:   validators's pending set
    @gamma_a:   ticket accumulator. A series of highestscoring ticket identifiers to be used for the next epoch
    @gamma_s:   current epoch's slot-sealer series
    @gamma_z:   epoch's root, a Bandersnatch ring root composed with the one Bandersnatch key of each of the next
                epoch’s validators
    @iota:      validator's staging set
    @kappa:     validator's active set
    @lambda:    validator's active set in the prior epoch
*/
#[derive(Debug, Clone, PartialEq)]
pub struct SafroleState {
    pub tau: TimeSlot,
    pub eta: Box<[OpaqueHash; 4]>,
    pub lambda: Vec<ValidatorData>,
    pub kappa: Vec<ValidatorData>,
    pub gamma_k: Vec<ValidatorData>,
    pub iota: Vec<ValidatorData>,
    pub gamma_a: Vec<TicketBody>,
    pub gamma_s: TicketsOrKeys,
    pub gamma_z: BandersnatchRingCommitment,
}

impl Encode for SafroleState {

    fn encode(&self) -> Vec<u8> {

        let mut state_encoded = Vec::new();

        self.tau.encode_to(&mut state_encoded);
        self.eta.encode_to(&mut state_encoded);
        ValidatorData::encode_all(&self.lambda).encode_to(&mut state_encoded);
        ValidatorData::encode_all(&self.kappa).encode_to(&mut state_encoded);
        ValidatorData::encode_all(&self.gamma_k).encode_to(&mut state_encoded);
        ValidatorData::encode_all(&self.iota).encode_to(&mut state_encoded);
        TicketBody::encode_len(&self.gamma_a).encode_to(&mut state_encoded);
        self.gamma_s.encode_to(&mut state_encoded);
        self.gamma_z.encode_to(&mut state_encoded);

        return state_encoded;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for SafroleState {

    fn decode(state_blob: &mut BytesReader) -> Result<Self, ReadError> {
        
        Ok(SafroleState {
            tau: TimeSlot::decode(state_blob)?, 
            eta: Box::new(<[OpaqueHash; 4]>::decode(state_blob)?),
            lambda: ValidatorData::decode_all(state_blob)?,
            kappa: ValidatorData::decode_all(state_blob)?,
            gamma_k: ValidatorData::decode_all(state_blob)?,
            iota: ValidatorData::decode_all(state_blob)?,
            gamma_a: TicketBody::decode_len(state_blob)?,
            gamma_s: TicketsOrKeys::decode(state_blob)?,
            gamma_z: BandersnatchRingCommitment::decode(state_blob)?,  
        })
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq)]
pub enum OutputSafrole {
    ok(OutputMarks),
    err(ErrorType),
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ErrorType {
    bad_slot = 0,           // Timeslot value must be strictly monotonic.
    unexpected_ticket = 1,  // Received a ticket while in epoch's tail.
    bad_ticket_order = 2,   // Tickets must be sorted.
    bad_ticket_proof = 3,   // Invalid ticket ring proof.
    bad_ticket_attempt = 4, // Invalid ticket attempt value.
    reserved = 5,           // Reserved.
    duplicate_ticket = 6,   // Found a ticket duplicate.
}

impl Encode for OutputSafrole {

    fn encode(&self) -> Vec<u8> {

        let mut output_blob = Vec::new();

        match self {
            OutputSafrole::ok(output_marks) => {
                output_blob.push(0); // OK = 0
                output_marks.encode_to(&mut output_blob);
            }
            OutputSafrole::err(error_type) => {
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
            Ok(OutputSafrole::ok(OutputMarks::decode(blob)?))           
        } else {
            let error_type = blob.read_byte()?;
            let error = match error_type {
                0 => ErrorType::bad_slot,
                1 => ErrorType::unexpected_ticket,
                2 => ErrorType::bad_ticket_order,
                3 => ErrorType::bad_ticket_proof,
                4 => ErrorType::bad_ticket_attempt,
                5 => ErrorType::reserved,
                6 => ErrorType::duplicate_ticket,
                _ => return Err(ReadError::InvalidData),
            };
            Ok(OutputSafrole::err(error))
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct OutputMarks {
    pub epoch_mark: Option<EpochMark>,
    pub tickets_mark: Option<Vec<TicketBody>>,
}

impl Encode for OutputMarks {

    fn encode(&self) -> Vec<u8> {

        let mut output_mark_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<OutputMarks>());
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

impl Decode for OutputMarks {
    
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

        Ok(OutputMarks {
            epoch_mark,
            tickets_mark,
        })
    }
}