
use serde::Deserialize;
use crate::codec::{Encode, EncodeLen, Decode, DecodeLen, BytesReader, ReadError};
use crate::codec::extrinsic::{TicketsExtrinsic, TicketEnvelope};
use crate::types::*;
use crate::constants::{VALIDATORS_COUNT, EPOCH_LENGTH};

#[derive(Debug, Clone, PartialEq)]
pub struct ValidatorData {
    pub bandersnatch: BandersnatchKey,
    pub ed25519: Ed25519Key,
    pub bls: BlsKey,
    pub metadata: Metadata,
}

impl ValidatorData {
    pub fn decode(data_blob: &mut BytesReader) -> Result<Self, ReadError> {
    
        let bandersnatch = BandersnatchKey::decode(data_blob)?;
        let ed25519 = Ed25519Key::decode(data_blob)?;
        let bls = BlsKey::decode(data_blob)?;
        let metadata = Metadata::decode(data_blob)?;

        Ok(ValidatorData {
            bandersnatch,
            ed25519,
            bls,
            metadata,
        })
    }

    pub fn encode(&self) -> Vec<u8> {

        let mut validator_data: Vec<u8> = Vec::with_capacity(std::mem::size_of::<ValidatorData>());
        
        self.bandersnatch.encode_to(&mut validator_data);
        self.ed25519.encode_to(&mut validator_data);
        self.bls.encode_to(&mut validator_data);
        self.metadata.encode_to(&mut validator_data);

        return validator_data;
    }

    pub fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
pub enum TicketsOrKeys {
    keys(Box<[BandersnatchKey; EPOCH_LENGTH]>),
    tickets(Vec<TicketBody>),
}

impl Decode for TicketsOrKeys {
    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        let length = blob.read_byte()?;  // Leer el indicador de variante

        match length {
            /*0 => {
                // Retornar una representación de "Null" para `TicketsOrKeys`
                Ok(TicketsOrKeys::keys(Box::new([Default::default(); E as usize])))
            }*/
            1 => {
                // Decodificar `keys`
                let keys = <[BandersnatchKey; EPOCH_LENGTH]>::decode(blob)?;
                Ok(TicketsOrKeys::keys(Box::new(keys)))
            }
            0 => {
                // Decodificar `tickets` con `E` como la longitud del vector
                let mut tickets: Vec<TicketBody> = Vec::with_capacity(std::mem::size_of::<TicketBody>() * EPOCH_LENGTH);
                for _ in 0..EPOCH_LENGTH {
                    let ticket = TicketBody::decode(blob)?;
                    tickets.push(ticket);
                }
                Ok(TicketsOrKeys::tickets(tickets)) // Devolver `tickets` como `TicketsOrKeys::tickets`
            }
            _ => Err(ReadError::InvalidData),
        }
    }
}

impl Encode for TicketsOrKeys {
    fn encode(&self) -> Vec<u8> {
        let mut encoded = Vec::new();

        match self {
            TicketsOrKeys::keys(keys_array) => {
                // Añadir un indicador para la variante `keys`
                encoded.push(1);
                
                // Codificar cada elemento en `keys_array`
                for key in keys_array.iter() {
                    encoded.extend(key.encode());
                }
            }
            TicketsOrKeys::tickets(tickets_vec) => {
                // Añadir un indicador para la variante `tickets`
                encoded.push(0);

                // Codificar la longitud de `tickets_vec` como `u8`
                //encoded.extend((tickets_vec.len() as u8).encode());

                // Codificar cada `TicketBody` en `tickets_vec`
                for ticket in tickets_vec {
                    encoded.extend(ticket.encode());
                }
            }
        }

        encoded
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq, Ord, PartialOrd, Eq)]
pub struct TicketBody {
    pub id: OpaqueHash,
    pub attempt: u8,
}

impl TicketBody {
    pub fn decode(body_blob: &mut BytesReader) -> Result<Self, ReadError> {
        let id = OpaqueHash::decode(body_blob)?;
        let attempt = u8::decode(body_blob)?;

        Ok( TicketBody {
            id,
            attempt,
        })
    }
    
    pub fn decode_len(blob: &mut BytesReader) -> Result<Vec<Self>, ReadError> {
        let len = blob.read_byte()?;
        let mut tickets_mark: Vec<TicketBody> = Vec::with_capacity(std::mem::size_of::<TicketBody>() * len as usize);
        for _ in 0..len {
            let ticket_mark: TicketBody = TicketBody::decode(blob)?;
            tickets_mark.push(ticket_mark);
        }
        return Ok(tickets_mark);
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut body_blob = Vec::with_capacity(std::mem::size_of::<TicketBody>());
        self.id.encode_to(&mut body_blob);
        self.attempt.encode_to(&mut body_blob);
        return body_blob;
    }

    pub fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }

    pub fn encode_len(&self, len: u8) -> Vec<u8> {
        let mut body_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<TicketBody>() * len as usize);
        body_blob.push(len as u8);
        for _i in 0..len {
            self.encode_to(&mut body_blob);
        }
        return body_blob;
    }

}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct Keys {
    keys: Vec<String>,
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
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
pub struct SafroleState {
    pub tau: TimeSlot,
    pub eta: Box<[OpaqueHash; 4]>,
    pub lambda: Box<[ValidatorData; VALIDATORS_COUNT]>,
    pub kappa: Box<[ValidatorData; VALIDATORS_COUNT]>,
    pub gamma_k: Box<[ValidatorData; VALIDATORS_COUNT]>,
    pub iota: Box<[ValidatorData; VALIDATORS_COUNT]>,
    pub gamma_a: Vec<TicketBody>,
    pub gamma_s: TicketsOrKeys,
    pub gamma_z: BandersnatchRingCommitment,
}

impl Decode for SafroleState {
    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(SafroleState {
            tau: TimeSlot::decode(reader)?,  // Decodifica tau
            eta: {
                let eta_vec: Vec<OpaqueHash> = (0..4).map(|_| OpaqueHash::decode(reader)).collect::<Result<_, _>>()?;
                eta_vec.into_boxed_slice().try_into().map_err(|_| ReadError::NotEnoughData)?
            },
            lambda: {
                let lambda_vec: Vec<ValidatorData> = (0..VALIDATORS_COUNT).map(|_| ValidatorData::decode(reader)).collect::<Result<_, _>>()?;
                lambda_vec.into_boxed_slice().try_into().map_err(|_| ReadError::NotEnoughData)?
            },
            kappa: {
                let kappa_vec: Vec<ValidatorData> = (0..VALIDATORS_COUNT).map(|_| ValidatorData::decode(reader)).collect::<Result<_, _>>()?;
                kappa_vec.into_boxed_slice().try_into().map_err(|_| ReadError::NotEnoughData)?
            },
            gamma_k: {
                let gamma_k_vec: Vec<ValidatorData> = (0..VALIDATORS_COUNT).map(|_| ValidatorData::decode(reader)).collect::<Result<_, _>>()?;
                gamma_k_vec.into_boxed_slice().try_into().map_err(|_| ReadError::NotEnoughData)?
            },
            iota: {
                let iota_vec: Vec<ValidatorData> = (0..VALIDATORS_COUNT).map(|_| ValidatorData::decode(reader)).collect::<Result<_, _>>()?;
                iota_vec.into_boxed_slice().try_into().map_err(|_| ReadError::NotEnoughData)?
            },
            /*gamma_a: {
                let gamma_a_len = u8::decode(reader)? as usize;  
                (0..gamma_a_len).map(|_| TicketBody::decode(reader)).collect::<Result<_, _>>()?
            },*/
            gamma_a: TicketBody::decode_len(reader)?,
            gamma_s: TicketsOrKeys::decode(reader)?,
            gamma_z: BandersnatchRingCommitment::decode(reader)?,  
        })
    }
}

impl Encode for SafroleState {
    fn encode(&self) -> Vec<u8> {
        let mut state_encoded = Vec::new();

        self.tau.encode_to(&mut state_encoded);

        for eta in self.eta.iter() {
            eta.encode_to(&mut state_encoded);
        }

        for validator in self.lambda.iter() {
            validator.encode_to(&mut state_encoded);
        }

        for validator in self.kappa.iter() {
            validator.encode_to(&mut state_encoded);
        }

        for validator in self.gamma_k.iter() {
            validator.encode_to(&mut state_encoded);
        }

        for validator in self.iota.iter() {
            validator.encode_to(&mut state_encoded);
        }

        state_encoded.extend((self.gamma_a.len() as u8).encode()); // Encode length as u8
        for ticket in &self.gamma_a {
            state_encoded.extend(ticket.encode());
        }
     
        match self.gamma_s {
            TicketsOrKeys::tickets(ref tickets) if tickets.is_empty() => {
                state_encoded.push(0);
            }
            TicketsOrKeys::tickets(ref tickets) => {
                state_encoded.push(0);
                //state_encoded.push(E as u8);
                for ticket in tickets {
                    ticket.encode_to(&mut state_encoded);
                }
            }
            TicketsOrKeys::keys(ref keys) => {
                state_encoded.push(1);
                let keys_array = keys.as_ref();
                for key in keys_array {
                    key.encode_to(&mut state_encoded);
                }
            }
        }
        

        // Encode `gamma_z` (BandersnatchRingCommitment)
        state_encoded.extend(self.gamma_z.encode());

        state_encoded
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

#[allow(non_camel_case_types)]
pub enum KeySet {
    gamma_k,
    kappa,
}

pub struct Safrole {
    pub pre_state: SafroleState,
    pub post_state: SafroleState,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ErrorType {
    bad_slot = 0, // Timeslot value must be strictly monotonic.
    unexpected_ticket = 1, // Received a ticket while in epoch's tail.
    bad_ticket_order = 2, // Tickets must be sorted.
    bad_ticket_proof = 3, // Invalid ticket ring proof.
    bad_ticket_attempt = 4, // Invalid ticket attempt value.
    reserved = 5, // Reserved.
    duplicate_ticket = 6, // Found a ticket duplicate.
}

#[derive(Debug, PartialEq)]
pub struct EpochMark {
    pub entropy: OpaqueHash,
    pub validators: Box<[BandersnatchKey; VALIDATORS_COUNT]>,
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

#[derive(Debug, PartialEq)]
pub struct OutputMarks {
    pub epoch_mark: Option<EpochMark>,
    pub tickets_mark: Option<Vec<TicketBody>>,
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq)]
pub enum Output {
    ok(OutputMarks),
    err(ErrorType),
}

impl Decode for Output {
    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        let err = blob.read_byte()?;
        
        if err == 0 {
            // Decodificar Output::ok
            let e_mark = blob.read_byte()?; 
            let epoch_mark = if e_mark == 1 {
                // Decodificar `epoch_mark` si `mark` es 1
                Some(EpochMark {
                    entropy: OpaqueHash::decode(blob)?,
                    validators: {
                        // Decodificamos los validadores en un `Vec`
                        let validators_vec: Vec<BandersnatchKey> = (0..VALIDATORS_COUNT)
                            .map(|_| BandersnatchKey::decode(blob))
                            .collect::<Result<_, _>>()?;
                        
                        // Convertimos el `Vec` a un array de tamaño fijo
                        let validators_array: [BandersnatchKey; VALIDATORS_COUNT] = validators_vec
                            .try_into()
                            .map_err(|_| ReadError::NotEnoughData)?;
                        
                        // Envolvemos el array en un `Box`
                        Box::new(validators_array)
                    },
                })
            } else {
                None // Si `mark` no es 1, `epoch_mark` será `None`
            };

            let t_mark = blob.read_byte()?; 

            let tickets_mark = if t_mark == 1 {
                Some((0..EPOCH_LENGTH)
                    .map(|_| TicketBody::decode(blob))
                    .collect::<Result<Vec<_>, _>>()?)
            } else {
                None
            };
            Ok(Output::ok(OutputMarks {
                epoch_mark,
                tickets_mark,
            }))
        } else {
            // Decodificar Output::err con el valor de `ErrorType`
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

impl Encode for Output {
    fn encode(&self) -> Vec<u8> {
        let mut output_blob = Vec::new();

        match self {
            Output::ok(output_marks) => {
                // Escribir `0` para indicar la variante `ok`
                output_blob.push(0);

                // Codificar el contenido de `OutputMarks`
                if let Some(epoch_mark) = &output_marks.epoch_mark {
                    output_blob.push(1); // Indicador de `Some` para `epoch_mark`
                    output_blob.extend(epoch_mark.encode());
                } else {
                    output_blob.push(0); // Indicador de `None` para `epoch_mark`
                }

                if let Some(tickets) = &output_marks.tickets_mark {
                    output_blob.push(1); // Indicador de `Some` para `tickets_mark`
                    //output_blob.extend((tickets.len() as u8).encode());
                    for ticket in tickets.iter() {
                        output_blob.extend(ticket.encode()); // Codifica cada `TicketBody` individualmente
                    }
                    
                } else {
                    output_blob.push(0); // Indicador de `None` para `tickets_mark`
                }
            }
            Output::err(error_type) => {
                // Escribir `1` para indicar la variante `err`
                output_blob.push(1);

                // Codificar el `ErrorType`
                output_blob.push(*error_type as u8);
            }
        }

        output_blob
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct Input {
    pub slot: TimeSlot,
    pub entropy: OpaqueHash,
    pub extrinsic: TicketsExtrinsic,
    pub post_offenders: Vec<Ed25519Key>,
}

impl Input {

    pub fn decode(input_blob: &mut BytesReader) -> Result<Self, ReadError> {

        let slot = TimeSlot::decode(input_blob)?;
        let entropy = OpaqueHash::decode(input_blob)?;
        let extrinsic = TicketsExtrinsic::decode(input_blob)?;
        let post_offenders: Vec<Ed25519Key> = Vec::<Ed25519Key>::decode_len(input_blob)?;

        Ok(Input {
            slot,
            entropy,
            extrinsic,
            post_offenders,
        })
    }

    pub fn encode(&self) -> Vec<u8> {

        let mut input_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<Input>());
        self.slot.encode_to(&mut input_blob);
        self.entropy.encode_to(&mut input_blob);
        self.extrinsic.encode_to(&mut input_blob);
        input_blob.push(self.post_offenders.len() as u8);
        self.post_offenders.encode_to(&mut input_blob);

        return input_blob;
    }

}
