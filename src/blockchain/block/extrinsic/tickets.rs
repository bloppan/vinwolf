/*
    Tickets Extrinsic is a sequence of proofs of valid tickets; a ticket implies an entry in our epochal “contest” to determine 
    which validators are privileged to author a block for each timeslot in the following epoch. Tickets specify an entry index 
    together with a proof of ticket’s validity. The proof implies a ticket identifier, a high-entropy unbiasable 32-octet sequence, 
    which is used both as a score in the aforementioned contest and as input to the on-chain vrf. Towards the end of the epoch 
    (i.e. Y slots from the start) this contest is closed implying successive blocks within the same epoch must have an empty tickets 
    extrinsic. At this point, the following epoch’s seal key sequence becomes fixed. We define the extrinsic as a sequence of proofs 
    of valid tickets, each of which is a tuple of an entry index (a natural number less than N) and a proof of ticket validity.
*/

use ark_vrf::{reexports::ark_serialize::CanonicalDeserialize, suites::bandersnatch::Public};

use crate::blockchain::state::safrole::bandersnatch::Verifier;

use crate::constants::{EPOCH_LENGTH, TICKET_SUBMISSION_ENDS, MAX_TICKETS_PER_EXTRINSIC, TICKET_ENTRIES_PER_VALIDATOR};
use crate::types::{
    EntropyPool, Header, OpaqueHash, Safrole, SafroleErrorCode, TicketsOrKeys, TicketBody, TicketsExtrinsic, 
    TimeSlot, ValidatorsData, ProcessError,
};
use crate::utils::codec::Encode;
use crate::utils::common::{has_duplicates, bad_order};

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
        let ring_keys: Vec<_> = safrole_state.pending_validators
                                            .list
                                            .iter()
                                            .map(|validator| validator.bandersnatch.clone())
                                            .collect();
    
        // Create a bandersnatch ring set 
        let ring_set: Vec<Public> = ring_keys
                                            .iter()
                                            .map(|key| {
                                                let point = Public::deserialize_compressed(&key[..])
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
