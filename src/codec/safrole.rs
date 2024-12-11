use crate::types::{BandersnatchKey, Ed25519Key, BlsKey, Metadata, OpaqueHash, TimeSlot, BandersnatchRingCommitment};
use crate::constants::{VALIDATORS_COUNT, EPOCH_LENGTH};
use crate::codec::{Encode, Decode, DecodeLen, BytesReader, ReadError};
use crate::codec::tickets_extrinsic::{TicketsExtrinsic};
use crate::codec::header::{EpochMark, TicketBody};
use crate::codec::{encode_unsigned};

#[derive(Debug)]
pub struct Input {
    pub slot: TimeSlot,
    pub entropy: OpaqueHash,
    pub extrinsic: TicketsExtrinsic,
    pub post_offenders: Vec<Ed25519Key>,
}

impl Encode for Input {

    fn encode(&self) -> Vec<u8> {

        let mut input_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<Input>());
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

impl Decode for Input {

    fn decode(input_blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(Input {
            slot: TimeSlot::decode(input_blob)?,
            entropy: OpaqueHash::decode(input_blob)?,
            extrinsic: TicketsExtrinsic::decode(input_blob)?,
            post_offenders: Vec::<Ed25519Key>::decode_len(input_blob)?,
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
pub enum Output {
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

impl Encode for Output {

    fn encode(&self) -> Vec<u8> {

        let mut output_blob = Vec::new();

        match self {
            Output::ok(output_marks) => {
                output_blob.push(0); // OK = 0
                output_marks.encode_to(&mut output_blob);
            }
            Output::err(error_type) => {
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

impl Decode for Output {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        
        if blob.read_byte()? == 0 { // OK = 0
            Ok(Output::ok(OutputMarks::decode(blob)?))           
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
            Ok(Output::err(error))
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

/// This is a combination of a set of cryptographic public keys and metadata which is an opaque octet sequence, 
/// but utilized to specify practical identifiers for the validator, not least a hardware address. The set of 
/// validator keys itself is equivalent to the set of 336-octet sequences. However, for clarity, we divide the
/// sequence into four easily denoted components. For any validator key k, the Bandersnatch key is is equivalent 
/// to the first 32-octets; the Ed25519 key, ke, is the second 32 octets; the bls key denoted bls is equivalent 
/// to the following 144 octets, and finally the metadata km is the last 128 octets.
#[derive(Debug, Clone, PartialEq)]
pub struct ValidatorData {
    pub bandersnatch: BandersnatchKey,
    pub ed25519: Ed25519Key,
    pub bls: BlsKey,
    pub metadata: Metadata,
}

impl Encode for ValidatorData {
    
    fn encode(&self) -> Vec<u8> {

        let mut validator_data: Vec<u8> = Vec::with_capacity(std::mem::size_of::<ValidatorData>());
        
        self.bandersnatch.encode_to(&mut validator_data);
        self.ed25519.encode_to(&mut validator_data);
        self.bls.encode_to(&mut validator_data);
        self.metadata.encode_to(&mut validator_data);

        return validator_data;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for ValidatorData {

    fn decode(data_blob: &mut BytesReader) -> Result<Self, ReadError> {
    
        Ok(ValidatorData {
            bandersnatch: BandersnatchKey::decode(data_blob)?,
            ed25519: Ed25519Key::decode(data_blob)?,
            bls: BlsKey::decode(data_blob)?,
            metadata: Metadata::decode(data_blob)?,
        })
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct ValidatorsData {
    pub validators: Vec<ValidatorData>,
}

impl Encode for ValidatorsData {

    fn encode(&self) -> Vec<u8> {

        let mut validators_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<ValidatorData>() * VALIDATORS_COUNT);

        for validator in &self.validators {
            validator.encode_to(&mut validators_blob);
        }

        return validators_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for ValidatorsData {

    fn decode(validators_blob: &mut BytesReader) -> Result<Self, ReadError> {

        let mut all_validators: ValidatorsData = ValidatorsData{ validators: Vec::with_capacity(std::mem::size_of::<ValidatorData>() * VALIDATORS_COUNT) };
            
        for _ in 0..VALIDATORS_COUNT {
            all_validators
                .validators.push(ValidatorData::decode(validators_blob)?);
        }

        Ok(all_validators)
    }
}

impl ValidatorData {

    pub fn encode_all(all_validators: &Vec<ValidatorData>) -> Vec<u8> {
        
        let mut data_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<ValidatorData>() * VALIDATORS_COUNT);

        for validator in all_validators {
            validator.encode_to(&mut data_blob);
        }

        return data_blob;
    }

    pub fn decode_all(data_blob: &mut BytesReader) -> Result<Vec<Self>, ReadError> {
        
        let mut all_validators: Vec<ValidatorData> = Vec::with_capacity(std::mem::size_of::<ValidatorData>() * VALIDATORS_COUNT);
        
        for _ in 0..VALIDATORS_COUNT {
            all_validators.push(ValidatorData::decode(data_blob)?);
        }

        Ok(all_validators)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TicketsOrKeys {
    Keys(Vec<BandersnatchKey>),
    Tickets(Vec<TicketBody>),
}

impl Decode for TicketsOrKeys {
    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        
        let marker = blob.read_byte()?;  

        match marker {
            1 => {
                let mut keys: Vec<BandersnatchKey> = Vec::with_capacity(std::mem::size_of::<BandersnatchKey>() * EPOCH_LENGTH);
                for _ in 0..EPOCH_LENGTH {
                    keys.push(BandersnatchKey::decode(blob)?);
                }
                Ok(TicketsOrKeys::Keys(keys))
            }
            0 => {
                let mut tickets: Vec<TicketBody> = Vec::with_capacity(std::mem::size_of::<TicketBody>() * EPOCH_LENGTH);
                for _ in 0..EPOCH_LENGTH {
                    tickets.push(TicketBody::decode(blob)?);
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
            TicketsOrKeys::Keys(keys_array) => {
                encoded.push(1); // Keys marker
                for key in keys_array.iter() {
                    key.encode_to(&mut encoded);
                }
            }
            TicketsOrKeys::Tickets(tickets_vec) => {
                encoded.push(0); // Tickets marker
                for ticket in tickets_vec {
                    ticket.encode_to(&mut encoded);
                }
            }
        }

        return encoded;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}
