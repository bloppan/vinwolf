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
use ark_ec_vrfs::suites::bandersnatch::edwards as bandersnatch_ark_ec_vrfs;
use ark_ec_vrfs::prelude::ark_serialize;
use ark_serialize::CanonicalDeserialize;
use bandersnatch_ark_ec_vrfs::Public;
use crate::blockchain::state::safrole::bandersnatch::Verifier;

use crate::constants::{EPOCH_LENGTH, TICKET_SUBMISSION_ENDS, MAX_TICKETS_PER_EXTRINSIC, TICKET_ENTRIES_PER_VALIDATOR};
use crate::types::{
    EntropyPool, Header, OpaqueHash, Safrole, SafroleErrorCode, TicketsOrKeys, TicketBody, TicketsExtrinsic, 
    TimeSlot, ValidatorsData, ProcessError,
};
use crate::utils::codec::Encode;
use crate::utils::common::{has_duplicates, bad_order};

// Sealing using the ticket is of greater security, and we utilize this knowledge when determining a candidate block
// on which to extend the chain.
pub fn verify_seal(
        safrole: &Safrole,
        entropy: &EntropyPool,
        current_validators: &ValidatorsData,
        ring_set: Vec<Public>,
        header: &Header
) -> Result<[u8; 32], ProcessError> {
    // The header must contain a valid seal and valid vrf output. These are two signatures both using the current slot’s 
    // seal key; the message data of the former is the header’s serialization omitting the seal component Hs, whereas the 
    // latter is used as a bias-resistant entropy source and thus its message must already have been fixed: we use the entropy
    // stemming from the vrf of the seal signature. 
    let unsigned_header = header.unsigned.encode();
    // Create the verifier object
    let verifier = Verifier::new(ring_set.clone());
    // Get the block author
    let block_author = header.unsigned.author_index as usize;
    let i = header.unsigned.slot % EPOCH_LENGTH as TimeSlot;

    let seal_vrf_output = match &safrole.seal {
        TicketsOrKeys::Tickets(tickets) => {
            // The context is "jam_fallback_seal" + entropy[3] + ticket_attempt
            let mut context = Vec::from(b"jam_ticket_seal");
            entropy.buf[3].encode_to(&mut context);
            tickets.tickets_mark[i as usize].attempt.encode_to(&mut context);
            // Verify the seal
            let seal_vrf_output = verifier.ietf_vrf_verify(
                                                    &context,
                                                    &unsigned_header,
                                                    &header.seal,
                                                    block_author,
            ).map_err(|_| ProcessError::SafroleError(SafroleErrorCode::InvalidTicketSeal))?;

            if tickets.tickets_mark[i as usize].id != seal_vrf_output {
                return Err(ProcessError::SafroleError(SafroleErrorCode::TicketNotMatch));
            }

            seal_vrf_output
        },
        TicketsOrKeys::Keys(keys) => {
            // The context is "jam_fallback_seal" + entropy[3]
            let mut context = Vec::from(b"jam_fallback_seal");
            entropy.buf[3].encode_to(&mut context);
            // Verify the seal
            let seal_vrf_output = verifier.ietf_vrf_verify(
                                                        &context,
                                                        &unsigned_header,
                                                        &header.seal,
                                                        block_author,
            ).map_err(|_| ProcessError::SafroleError(SafroleErrorCode::InvalidKeySeal))?;
            
            if keys.0[i as usize] != current_validators.0[block_author].bandersnatch {
                return Err(ProcessError::SafroleError(SafroleErrorCode::KeyNotMatch));
            }

            seal_vrf_output
        },
        TicketsOrKeys::None => {
            return Err(ProcessError::SafroleError(SafroleErrorCode::TicketsOrKeysNone));
        },
    };
    
    // Verify the entropy source
    let mut context = Vec::from(b"jam_entropy");
    seal_vrf_output.encode_to(&mut context);
    let entropy_source_vrf_result = verifier.ietf_vrf_verify(
                                                                        &context,
                                                                        &[],
                                                                        &header.unsigned.entropy_source,
                                                                        block_author);

    let entropy_source_vrf_output = match entropy_source_vrf_result {
        Ok(_) => entropy_source_vrf_result.unwrap(),
        Err(_) => { return Err(ProcessError::SafroleError(SafroleErrorCode::InvalidEntropySource)) },
    };

    Ok(entropy_source_vrf_output)
}


impl TicketsExtrinsic {

    pub fn process(
        &self,
        safrole_state: &mut Safrole,
        entropy_state: &mut EntropyPool,
        post_tau: &TimeSlot,
    ) -> Result<(), ProcessError> {
        
        if self.tickets.is_empty() {
            return Ok(());
        }
        // Towards the end of the epoch, ticket submission ends implying successive blocks within the same epoch
        // must have an empty tickets extrinsic
        if (*post_tau % EPOCH_LENGTH as TimeSlot) >= TICKET_SUBMISSION_ENDS as TimeSlot {
            return Err(ProcessError::SafroleError(SafroleErrorCode::UnexpectedTicket));
        }

        if self.tickets.len() > MAX_TICKETS_PER_EXTRINSIC {
            return Err(ProcessError::SafroleError(SafroleErrorCode::TooManyTickets));
        }

        // We define the extrinsic as a sequence of proofs of valid tickets, each of which is a tuple of an entry index (a
        // natural number less than TICKET_ENTRIES_PER_VALIDATOR) and a proof of ticket validity.
        for i in 0..self.tickets.len() {
            if self.tickets[i].attempt >= TICKET_ENTRIES_PER_VALIDATOR {
                return Err(ProcessError::SafroleError(SafroleErrorCode::BadTicketAttempt));
            }
        }
    
        // Create a bandersnatch ring keys
        let ring_keys: Vec<_> = safrole_state.pending_validators.0
                                            .iter()
                                            .map(|validator| validator.bandersnatch.clone())
                                            .collect();
    
        // Create a bandersnatch ring set 
        let ring_set: Vec<Public> = ring_keys
                                            .iter()
                                            .map(|key| {
                                                let point = bandersnatch_ark_ec_vrfs::Public::deserialize_compressed(&key[..])
                                                .expect("Deserialization failed");
                                                point
                                            })
                                            .collect();
        
        let verifier = Verifier::new(ring_set);
        let mut new_ticket_ids: Vec<OpaqueHash> = vec![];
        // Verify each ticket
        for i in 0..self.tickets.len() {
            let mut vrf_input_data = Vec::from(b"jam_ticket_seal");
            entropy_state.buf[2].encode_to(&mut vrf_input_data);
            self.tickets[i].attempt.encode_to(&mut vrf_input_data);
            let aux_data = vec![];
            // Verify ticket validity
            let res = verifier.ring_vrf_verify(&vrf_input_data, &aux_data, &self.tickets[i].signature);
            match res {
                Ok(result) => {
                    new_ticket_ids.push(result);
                    safrole_state.ticket_accumulator.push(TicketBody {
                        id: result,
                        attempt: self.tickets[i].attempt,
                    });
                },
                Err(_) => { println!("Bad ticket {}", i); return Err(ProcessError::SafroleError(SafroleErrorCode::BadTicketProof)); }
            }
        }
        // Check tickets order
        if bad_order(&new_ticket_ids) {
            return Err(ProcessError::SafroleError(SafroleErrorCode::BadTicketOrder));
        }
        // Check if there are duplicate tickets
        let ids: Vec<OpaqueHash> = safrole_state.ticket_accumulator.iter().map(|ticket| ticket.id.clone()).collect();
        if has_duplicates(&ids) {
            return Err(ProcessError::SafroleError(SafroleErrorCode::DuplicateTicket));
        }
        // Sort tickets
        safrole_state.ticket_accumulator.sort();

        // Remove old tickets to make space for new ones
        if safrole_state.ticket_accumulator.len() > EPOCH_LENGTH {
            safrole_state.ticket_accumulator.drain(EPOCH_LENGTH..);
        }

        // Return ok
        Ok(())
    }
}
