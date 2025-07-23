
// The header comprises a parent hash and prior state root, an extrinsic hash, a time-slot index, the epoch, winning-tickets and
// offenders markers, and, a Bandersnatch block author index and two Bandersnatch signatures; the entropy-yielding, vrf signature,
// and a block seal. Excepting the Genesis header, all block headers H have an associated parent header, whose hash is Hp.

// The epoch and winning-tickets markers are information placed in the header in order to minimize data transfer necessary to
// determine the validator keys associated with any given epoch. They are particularly useful to nodes which do not synchronize
// the entire state for any given block since they facilitate the secure tracking of changes to the validator key sets using only the
// chain of headers.

// The epoch marker specifies key and entropy relevant to the following epoch in case the ticket contest does not complete adequately
// (a very much unexpected eventuality).The epoch marker is either empty or, if the block is the first in a new epoch, then a tuple of
// the epoch randomness and a sequence of Bandersnatch keys defining the Bandersnatch validator keys (kb) beginning in the next epoch.

use ark_vrf::suites::bandersnatch::Public;
use handler::get_state_root;
use utils::bandersnatch::Verifier;

use constants::node::{EPOCH_LENGTH, VALIDATORS_COUNT};
use crate::{Header, UnsignedHeader, Extrinsic};
use jam_types::{
    EntropyPool, OpaqueHash, ProcessError, HeaderErrorCode, Safrole, SafroleErrorCode, TicketsOrKeys, TimeSlot, ValidatorsData, ReadError, EpochMark,
    TicketsMark, Ed25519Public, ValidatorIndex, BandersnatchVrfSignature
};
use codec::{Encode, EncodeLen, EncodeSize, BytesReader, Decode, DecodeLen};
use codec::generic_codec::{encode_unsigned, decode_unsigned};

impl Default for Header {
    fn default() -> Self {
        Self {
            unsigned: UnsignedHeader::default(),
            seal: [0u8; 96],
        }
    }
}

impl Default for UnsignedHeader {
    fn default() -> Self {
        Self {
            parent: OpaqueHash::default(),
            parent_state_root: OpaqueHash::default(),
            extrinsic_hash: OpaqueHash::default(),
            slot: 0,
            epoch_mark: None,
            tickets_mark: None,
            offenders_mark: Vec::new(),
            author_index: 0,
            entropy_source: [0u8; 96],
        }
    }
}

impl Header {
    // Sealing using the ticket is of greater security, and we utilize this knowledge when determining a candidate block
    // on which to extend the chain.
    pub fn seal_verify(&self,
            safrole: &Safrole,
            entropy: &EntropyPool,
            current_validators: &ValidatorsData,
            ring_set: Vec<Public>,
    ) -> Result<OpaqueHash, ProcessError> {
        // The header must contain a valid seal and valid vrf output. These are two signatures both using the current slot’s 
        // seal key; the message data of the former is the header’s serialization omitting the seal component Hs, whereas the 
        // latter is used as a bias-resistant entropy source and thus its message must already have been fixed: we use the entropy
        // stemming from the vrf of the seal signature. 
        let unsigned_header = self.unsigned.encode();
        // Create the verifier object
        let verifier = Verifier::new(ring_set.clone());
        // Get the block author
        let block_author = self.unsigned.author_index as usize;
        let i = self.unsigned.slot % EPOCH_LENGTH as TimeSlot;

        let seal_vrf_output = match &safrole.seal {
            TicketsOrKeys::Tickets(tickets) => {
                log::debug!("Verify tickets seal");
                // The context is "jam_fallback_seal" + entropy[3] + ticket_attempt
                let context = [&b"jam_ticket_seal"[..], &entropy.buf[3].encode(), &tickets.tickets_mark[i as usize].attempt.encode()].concat();
                // Verify the seal
                let seal_vrf_output_result = verifier.ietf_vrf_verify(
                                                        &context,
                                                        &unsigned_header,
                                                        &self.seal,
                                                        block_author,
                );

                let seal_vrf_output = match seal_vrf_output_result {
                    Ok(vrf_output) => vrf_output,
                    Err(_) => {
                        log::error!("Invalid tickets seal");
                        return Err(ProcessError::SafroleError(SafroleErrorCode::InvalidTicketSeal));
                    }
                };

                if tickets.tickets_mark[i as usize].id != seal_vrf_output {
                    log::error!("Ticket not match");
                    return Err(ProcessError::SafroleError(SafroleErrorCode::TicketNotMatch));
                }
                log::debug!("Seal tickets verified successfully");
                seal_vrf_output
            },
            TicketsOrKeys::Keys(keys) => {
                log::debug!("Verify keys seal");
                // The context is "jam_fallback_seal" + entropy[3]
                let context = [&b"jam_fallback_seal"[..], &entropy.buf[3].encode()].concat();
                // Verify the seal
                let seal_vrf_output_result = verifier.ietf_vrf_verify(
                                                            &context,
                                                            &unsigned_header,
                                                            &self.seal,
                                                            block_author,
                );

                let seal_vrf_output = match seal_vrf_output_result {
                    Ok(vrf_output) => vrf_output,
                    Err(_) => {
                        log::error!("Invalid key seal");
                        return Err(ProcessError::SafroleError(SafroleErrorCode::InvalidTicketSeal));
                    }
                };
                
                if keys.epoch[i as usize] != current_validators.list[block_author].bandersnatch {
                    log::error!("Key not match: Seal key {:02x?} != bandersnatch key author {block_author} {:02x?}", utils::print_hash!(keys.epoch[i as usize]), utils::print_hash!(current_validators.list[block_author].bandersnatch));
                    return Err(ProcessError::SafroleError(SafroleErrorCode::KeyNotMatch));
                }

                log::debug!("Seal keys verified successfully");
                seal_vrf_output
            },
            TicketsOrKeys::None => {
                log::error!("None tickets or keys");
                return Err(ProcessError::SafroleError(SafroleErrorCode::TicketsOrKeysNone));
            },
        };
        
        // Verify the entropy source
        let context = [&b"jam_entropy"[..], &seal_vrf_output.encode()].concat();
        let entropy_source_vrf_result = verifier.ietf_vrf_verify(
                                                                                &context,
                                                                                &[],
                                                                                &self.unsigned.entropy_source,
                                                                                block_author);

        let entropy_source_vrf_output = match entropy_source_vrf_result {
            Ok(_) => entropy_source_vrf_result.unwrap(),
            Err(_) => { 
                log::error!("Invalid entropy source");
                return Err(ProcessError::SafroleError(SafroleErrorCode::InvalidEntropySource)) 
            },
        };

        log::debug!("Seal header verified successfully. vrf output: 0x{}", utils::print_hash!(entropy_source_vrf_output));
        Ok(entropy_source_vrf_output)
    }

    pub fn verify(&self, extrinsic: &Extrinsic) -> Result<(), ProcessError> {

        self.extrinsic_verify(extrinsic)?;
        self.validator_index_verify()?;
        self.offenders_verify(extrinsic)?;
        // TODO verify parent state root
        log::debug!("Header verified successfully");
        return Ok(());
    }

    pub fn offenders_verify(&self, extrinsic: &Extrinsic) -> Result<(), ProcessError> {
        
        //The offenders markers must contain exactly the keys of all new offenders, respectively
        if self.unsigned.offenders_mark.encode() != [extrinsic.disputes.culprits.encode(), extrinsic.disputes.faults.encode()].concat() {
            log::error!("Bad offenders");
            return Err(ProcessError::HeaderError(HeaderErrorCode::BadOffenders));
        }

        return Ok(());
    }

    pub fn state_root_verify(&self) -> Result<(), ProcessError> {

        let parent_state_root = get_state_root().lock().unwrap();

        if self.unsigned.parent_state_root != *parent_state_root {
            log::error!("Bad parent state root");
            return Err(ProcessError::HeaderError(HeaderErrorCode::BadParentStateRoot));
        }

        return Ok(());
    }

    pub fn validator_index_verify(&self) -> Result<(), ProcessError> { 

        if self.unsigned.author_index >= VALIDATORS_COUNT as u16 {
            log::error!("Bad validator index: {:?}", self.unsigned.author_index);
            return Err(ProcessError::HeaderError(HeaderErrorCode::BadValidatorIndex));
        }
    
        return Ok(());
    }
    

    pub fn extrinsic_verify(&self, extrinsic: &Extrinsic) -> Result<(), ProcessError> {

        let mut guarantees_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<Self>() * extrinsic.guarantees.report_guarantee.len());
        encode_unsigned(extrinsic.guarantees.report_guarantee.len()).encode_to(&mut guarantees_blob);

        for guarantee in extrinsic.guarantees.report_guarantee.iter() {

            sp_core::blake2_256(&guarantee.report.encode()).encode_to(&mut guarantees_blob);
            guarantee.slot.encode_size(4).encode_to(&mut guarantees_blob);
            encode_unsigned(guarantee.signatures.len()).encode_to(&mut guarantees_blob);

            for signature in &guarantee.signatures {
                signature.validator_index.encode_to(&mut guarantees_blob);
                signature.signature.encode_to(&mut guarantees_blob);
            }
        }

        let a = [sp_core::blake2_256(&extrinsic.tickets.encode()),
                               sp_core::blake2_256(&extrinsic.preimages.encode()),
                               sp_core::blake2_256(&guarantees_blob),
                               sp_core::blake2_256(&extrinsic.assurances.encode()),
                               sp_core::blake2_256(&extrinsic.disputes.encode())].concat();
    
        if self.unsigned.extrinsic_hash != sp_core::blake2_256(&a) {
            log::error!("Bad extrinsic hash");
            return Err(ProcessError::HeaderError(HeaderErrorCode::BadExtrinsicHash));
        }

        log::trace!("Header extrinsic expected: {:x?}", self.unsigned.extrinsic_hash );
        log::trace!("Eeader extrinsic   result: {:x?}\n", sp_core::blake2_256(&a) );
        
        return Ok(());
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
        
        self.offenders_mark.encode_len().encode_to(&mut header_blob);
        self.author_index.encode_size(2).encode_to(&mut header_blob);
        self.entropy_source.encode_to(&mut header_blob);
        
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
            offenders_mark: {
                let num_offenders = decode_unsigned(header_blob)?;
                let mut offenders_mark: Vec<Ed25519Public> = Vec::with_capacity(num_offenders);
                for _ in 0..num_offenders {
                    offenders_mark.push(Ed25519Public::decode(header_blob)?);
                }
                offenders_mark
            },
            author_index: ValidatorIndex::decode(header_blob)?,
            entropy_source: BandersnatchVrfSignature::decode(header_blob)?,
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