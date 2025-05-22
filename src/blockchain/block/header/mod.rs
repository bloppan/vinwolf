
// The header comprises a parent hash and prior state root, an extrinsic hash, a time-slot index, the epoch, 
// winning-tickets and offenders markers, and, a Bandersnatch block author index and two Bandersnatch signatures; 
// the entropy-yielding, vrf signature, and a block seal. Excepting the Genesis header, all block headers H have
// an associated parent header, whose hash is Hp.

// The epoch and winning-tickets markers are information placed in the header in order to minimize 
// data transfer necessary to determine the validator keys associated with any given epoch. They 
// are particularly useful to nodes which do not synchronize the entire state for any given block 
// since they facilitate the secure tracking of changes to the validator key sets using only the 
// chain of headers.


// The epoch marker specifies key and entropy relevant to the following epoch in case the ticket 
// contest does not complete adequately (a very much unexpected eventuality).The epoch marker is
// either empty or, if the block is the first in a new epoch, then a tuple of the epoch randomness 
// and a sequence of Bandersnatch keys defining the Bandersnatch validator keys (kb) beginning in 
// the next epoch.
use ark_vrf::suites::bandersnatch::Public;
use crate::blockchain::state::safrole::bandersnatch::Verifier;

use crate::constants::EPOCH_LENGTH;
use crate::types::{EntropyPool, Header, OpaqueHash, Safrole, SafroleErrorCode, TicketsOrKeys, TimeSlot, ValidatorsData, ProcessError};
use crate::utils::codec::Encode;

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
                // The context is "jam_fallback_seal" + entropy[3] + ticket_attempt
                let mut context = Vec::from(b"jam_ticket_seal");
                entropy.buf[3].encode_to(&mut context);
                tickets.tickets_mark[i as usize].attempt.encode_to(&mut context);
                // Verify the seal
                let seal_vrf_output = verifier.ietf_vrf_verify(
                                                        &context,
                                                        &unsigned_header,
                                                        &self.seal,
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
                                                            &self.seal,
                                                            block_author,
                ).map_err(|_| ProcessError::SafroleError(SafroleErrorCode::InvalidKeySeal))?;
                
                if keys.epoch[i as usize] != current_validators.list[block_author].bandersnatch {
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
                                                                            &self.unsigned.entropy_source,
                                                                            block_author);

        let entropy_source_vrf_output = match entropy_source_vrf_result {
            Ok(_) => entropy_source_vrf_result.unwrap(),
            Err(_) => { return Err(ProcessError::SafroleError(SafroleErrorCode::InvalidEntropySource)) },
        };

        Ok(entropy_source_vrf_output)
    }

}
