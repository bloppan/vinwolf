use jam_types::{
    Block, Header, UnsignedHeader, Extrinsic, ReadError, BandersnatchVrfSignature, Ticket, Guarantee, Preimage, DisputesExtrinsic, OpaqueHash, EpochMark,
    ValidatorIndex, Ed25519Public, TimeSlot, TicketsMark, Assurance, Verdict, Culprit, Fault, BandersnatchPublic, TicketBody,  Entropy
};
use constants::node::{EPOCH_LENGTH, VALIDATORS_COUNT};
use crate::{Encode, EncodeLen, EncodeSize, Decode, DecodeLen, BytesReader};
use crate::generic_codec::decode_unsigned;

impl Encode for Block {

    fn encode(&self) -> Vec<u8> {

        let mut block_blob: Vec<u8> = Vec::new();

        self.header.encode_to(&mut block_blob);
        self.extrinsic.encode_to(&mut block_blob);

        return block_blob;
    }
    
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    } 
}

impl Decode for Block {

    fn decode(block_blob: &mut BytesReader) -> Result<Self, ReadError> {
        
        let header = Header::decode(block_blob)?;
        let extrinsic = Extrinsic::decode(block_blob)?;

        Ok(Block { header, extrinsic })
    }
}

impl Encode for Extrinsic {
    fn encode(&self) -> Vec<u8> {
        let mut extrinsic_blob: Vec<u8> = Vec::new();

        self.tickets.encode_len().encode_to(&mut extrinsic_blob);
        self.preimages.encode_len().encode_to(&mut extrinsic_blob);
        self.guarantees.encode_len().encode_to(&mut extrinsic_blob);
        self.assurances.encode_len().encode_to(&mut extrinsic_blob);
        self.disputes.encode_to(&mut extrinsic_blob);      

        return extrinsic_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode()); 
    }
}

impl Decode for Extrinsic {
    fn decode(extrinsic_blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(Extrinsic {
            tickets: Vec::<Ticket>::decode_len(extrinsic_blob)?,
            preimages: Vec::<Preimage>::decode_len(extrinsic_blob)?,
            guarantees: Vec::<Guarantee>::decode_len(extrinsic_blob)?,
            assurances: Vec::<Assurance>::decode_len(extrinsic_blob)?,
            disputes: DisputesExtrinsic::decode(extrinsic_blob)?,
        })
    }
}

impl Encode for DisputesExtrinsic {

    fn encode(&self) -> Vec<u8> {

        let mut dispute_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<DisputesExtrinsic>());
        
        self.verdicts.encode_len().encode_to(&mut dispute_blob);
        self.culprits.encode_len().encode_to(&mut dispute_blob);
        self.faults.encode_len().encode_to(&mut dispute_blob);

        return dispute_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode()); 
    }
}

impl Decode for DisputesExtrinsic {

    fn decode(dispute_blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(DisputesExtrinsic {
            verdicts : Vec::<Verdict>::decode_len(dispute_blob)?,
            culprits : Vec::<Culprit>::decode_len(dispute_blob)?,
            faults : Vec::<Fault>::decode_len(dispute_blob)?,
        })
    }
}

impl Encode for UnsignedHeader {
    fn encode(&self) -> Vec<u8> {

        let mut header_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<UnsignedHeader>());
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
        
        self.author_index.encode_size(2).encode_to(&mut header_blob);
        self.entropy_source.encode_to(&mut header_blob);
        self.offenders_mark.encode_len().encode_to(&mut header_blob);
        
        return header_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode()); 
    }
}

impl Decode for UnsignedHeader {
    fn decode(header_blob: &mut BytesReader) -> Result<Self, ReadError> {
        
        Ok(UnsignedHeader {
            parent: OpaqueHash::decode(header_blob)?,
            parent_state_root: OpaqueHash::decode(header_blob)?,
            extrinsic_hash: OpaqueHash::decode(header_blob)?,
            slot: TimeSlot::decode(header_blob)?,
            epoch_mark: if header_blob.read_byte()? != 0 {
                Some(EpochMark::decode(header_blob)?)
            } else {
                None
            },
            tickets_mark: if header_blob.read_byte()? != 0 {
                Some(TicketsMark::decode(header_blob)?)
            } else {
                None
            },
            author_index: ValidatorIndex::decode(header_blob)?,
            entropy_source: BandersnatchVrfSignature::decode(header_blob)?,
            offenders_mark: {
                let num_offenders = decode_unsigned(header_blob)?;
                let mut offenders_mark: Vec<Ed25519Public> = Vec::with_capacity(num_offenders);
                for _ in 0..num_offenders {
                    offenders_mark.push(Ed25519Public::decode(header_blob)?);
                }
                offenders_mark
            },
        })
    }
}

// The header comprises a parent hash and prior state root, an extrinsic hash, a time-slot index, the epoch, 
// winning-tickets and offenders markers, and, a Bandersnatch block author index and two Bandersnatch signatures; 
// the entropy-yielding, vrf signature, and a block seal. Excepting the Genesis header, all block headers H have
// an associated parent header, whose hash is Hp.

impl Encode for Header {

    fn encode(&self) -> Vec<u8> {

        let mut header_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<Header>());

        self.unsigned.encode_to(&mut header_blob);
        self.seal.encode_to(&mut header_blob);

        return header_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode()); 
    }
}

impl Decode for Header {

    fn decode(header_blob: &mut BytesReader) -> Result<Self, ReadError> {
        
        Ok(Header {
            unsigned: UnsignedHeader::decode(header_blob)?,
            seal: BandersnatchVrfSignature::decode(header_blob)?,
        })
    }
}

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
