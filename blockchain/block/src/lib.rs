pub mod extrinsic;
pub mod header;

use bandersnatch_vrf_spec::{Verifier, Prover};
use codec::Encode;
use constants::node::{EPOCH_LENGTH, TICKET_SUBMISSION_ENDS};
use jam_types::*;


pub fn build(
    state: &GlobalState, 
    slot: TimeSlot, 
    verifier: Verifier,
    prover: &Prover,
) -> Block {

    let mut block = Block::default();

    if (slot % EPOCH_LENGTH as TimeSlot) < TICKET_SUBMISSION_ENDS as TimeSlot {
        block.extrinsic.tickets = extrinsic::tickets::select_for_block_inclusion(slot, verifier, &state.safrole, &state.entropy);
    }

    block.header.unsigned.author_index = prover.prover_idx as ValidatorIndex;
    block.header.unsigned.extrinsic_hash = header::encode_extrinsic(&block);
    block.header.unsigned.parent = header::get_parent_header();
    block.header.unsigned.parent_state_root = state_handler::get_state_root().lock().unwrap().clone();
    block.header.unsigned.slot = slot;

    // Seal
    match &state.safrole.seal {
        Seal::Tickets(tickets) => {
            // The context is "jam_fallback_seal" + entropy[3] + ticket_attempt
            let slot_index = (block.header.unsigned.slot % EPOCH_LENGTH as TimeSlot) as usize;
            let c = [&b"jam_ticket_seal"[..], &state.entropy.buf[3].encode(), &tickets.tickets_mark[slot_index].attempt.encode()].concat();
            let vrf_output = prover.vrf_output(&c);
            // Step 2
            let context = [&b"jam_entropy"[..], &vrf_output.encode()].concat();
            block.header.unsigned.entropy_source = prover.ietf_vrf_sign(&context, &[]).try_into().unwrap();
            // Step 3
            block.header.seal = prover.ietf_vrf_sign(&c, &block.header.unsigned.encode()).try_into().unwrap();
        },
        Seal::Keys(_keys) => {
            // Step 1
            let c = [&b"jam_fallback_seal"[..], &state.entropy.buf[3].encode()].concat();
            let vrf_output = prover.vrf_output(&c);
            // Step 2
            let context = [&b"jam_entropy"[..], &vrf_output.encode()].concat();
            block.header.unsigned.entropy_source = prover.ietf_vrf_sign(&context, &[]).try_into().unwrap();
            // Step 3
            block.header.seal = prover.ietf_vrf_sign(&c, &block.header.unsigned.encode()).try_into().unwrap();
        },
        Seal::None => {
            { };
        },
    }

    return block;
}
