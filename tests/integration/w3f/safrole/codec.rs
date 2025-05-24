use vinwolf::types::{
    Entropy, Ed25519Public, EntropyPool, TimeSlot, BandersnatchRingCommitment, ValidatorsData, TicketsExtrinsic, TicketBody, TicketsOrKeys
};
use vinwolf::utils::codec::{Encode, EncodeLen, Decode, DecodeLen, BytesReader, ReadError};

#[derive(Debug)]
pub struct InputSafrole {
    pub slot: TimeSlot,
    pub entropy: Entropy,
    pub tickets_extrinsic: TicketsExtrinsic,
}

impl Encode for InputSafrole {

    fn encode(&self) -> Vec<u8> {

        let mut input_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<InputSafrole>());
        self.slot.encode_to(&mut input_blob);
        self.entropy.encode_to(&mut input_blob);
        self.tickets_extrinsic.encode_to(&mut input_blob);

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
            entropy: Entropy::decode(input_blob)?,
            tickets_extrinsic: TicketsExtrinsic::decode(input_blob)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SafroleState {
    pub tau: TimeSlot,
    pub eta: EntropyPool,
    pub lambda: ValidatorsData,
    pub kappa: ValidatorsData,
    pub gamma_k: ValidatorsData,
    pub iota: ValidatorsData,
    pub gamma_a: Vec<TicketBody>,
    pub gamma_s: TicketsOrKeys,
    pub gamma_z: BandersnatchRingCommitment,
    pub post_offenders: Vec<Ed25519Public>,
}

impl Encode for SafroleState {

    fn encode(&self) -> Vec<u8> {

        let mut state_encoded = Vec::new();

        self.tau.encode_to(&mut state_encoded);
        self.eta.encode_to(&mut state_encoded);
        self.lambda.encode_to(&mut state_encoded);
        self.kappa.encode_to(&mut state_encoded);
        self.gamma_k.encode_to(&mut state_encoded);
        self.iota.encode_to(&mut state_encoded);
        self.gamma_a.encode_to(&mut state_encoded);
        self.gamma_s.encode_to(&mut state_encoded);
        self.gamma_z.encode_to(&mut state_encoded);
        self.post_offenders.encode_len().encode_to(&mut state_encoded);

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
            eta: EntropyPool::decode(state_blob)?,
            lambda: ValidatorsData::decode(state_blob)?,
            kappa: ValidatorsData::decode(state_blob)?,
            gamma_k: ValidatorsData::decode(state_blob)?,
            iota: ValidatorsData::decode(state_blob)?,
            gamma_a: Vec::<TicketBody>::decode_len(state_blob)?,
            gamma_s: TicketsOrKeys::decode(state_blob)?,
            gamma_z: BandersnatchRingCommitment::decode(state_blob)?,  
            post_offenders: Vec::<Ed25519Public>::decode_len(state_blob)?,
        })
    }
}
