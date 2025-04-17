use sp_core::blake2_256;

use crate::blockchain::state::entropy::get_recent_entropy;
use crate::blockchain::state::{get_entropy, get_time};
use crate::types::{
    Account, AccumulationContext, AccumulationOperand, AccumulationPartialState, DeferredTransfer, ExitReason, Gas, HostCallFn, 
    OpaqueHash, ServiceId, TimeSlot, WorkExecResult, Registers, RamMemory,
};
use crate::constants::{NONE, WHAT, OOB, WHO, FULL, CORE, CASH, LOW, HUH, OK};
use crate::utils::codec::{Encode, DecodeSize, BytesReader};
use crate::pvm::hostcall::{hostcall_argument, HostCallContext};
use crate::pvm::hostcall::general_functions::info;
use crate::blockchain::state::services::{decode_preimage, historical_preimage_lookup};


pub fn invoke_accumulation(
    partial_state: &AccumulationPartialState,
    slot: &TimeSlot,
    service_id: &ServiceId,
    gas: Gas,
    operand: &[AccumulationOperand]
) -> (AccumulationPartialState, Vec<DeferredTransfer>, Option<OpaqueHash>, Gas) {

    let preimage_code = if let Some(code) = partial_state
        .services_accounts
        .service_accounts
        .get(service_id)
        .and_then(|account| account.preimages.get(&account.code_hash).cloned())
    {
        code
    } else {
        return (partial_state.clone(), vec![], None, 0);
    };

    let mut a: Vec<u8> = Vec::new();
    slot.encode_to(&mut a);
    service_id.encode_to(&mut a);
    operand.len().encode_to(&mut a);

    let preimage_data = decode_preimage(&preimage_code).unwrap(); // TODO handle error

    let hostcall_arg_result = hostcall_argument(&preimage_data.code, 5, gas, &a, accumulation_dispatcher, HostCallContext::Accumulate(I(partial_state, service_id), I(partial_state, service_id)));
    
    let (gas, exec_result, ctx) = hostcall_arg_result;

    collapse(gas, exec_result, ctx)
}

// F: Fn(&[u8], RegSize, Gas, Registers, RamMemory, HostCallContext) -> (ExitReason, RegSize, Gas, Registers, RamMemory, HostCallContext)
// F: Fn(HostCallFn, Gas, Registers, RamMemory, HostCallContext) -> (ExitReason, RegSize, Gas, Registers, RamMemory, HostCallContext)
pub fn accumulation_dispatcher(n: HostCallFn, mut gas: Gas, mut reg: Registers, mut ram: RamMemory, mut ctx: HostCallContext) 

-> (ExitReason, Gas, Registers, RamMemory, HostCallContext) {

    let HostCallContext::Accumulate(mut ctx_x, mut ctx_y) = ctx else {
        unreachable!("Dispatch accumulate: Invalid context");
    };

    match n {

        /*HostCallType::Info => {
            let exit_reason = info(&mut gas, &mut reg, &mut ram, &ctx_x.service_id, &ctx_x.partial_state.services_accounts);
            let account = get_accumulating_service_account(&ctx_x.partial_state, &ctx_x.service_id).unwrap();
            general_hostcall(account, &mut ctx_x);
            return HostCallResult::Ok(exit_reason, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y)); 
        }*/
        _ => {
            gas -= 10;
            reg[7] = WHAT;
            return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
        }
    }
}

fn general_hostcall(account: Account, ctx_x: &mut AccumulationContext)
{
    ctx_x.partial_state.services_accounts.service_accounts.insert(ctx_x.service_id, account);
}

fn get_accumulating_service_account(partial_state: &AccumulationPartialState, service_id: &ServiceId) -> Option<Account> {

    if let Some(account) = partial_state.services_accounts.service_accounts.get(service_id) {
        return Some(account.clone());
    }

    return None;
}

fn collapse(gas: Gas, output: WorkExecResult, context: HostCallContext

) -> (AccumulationPartialState, Vec<DeferredTransfer>, Option<OpaqueHash>, Gas) {

    let (ctx_x, ctx_y) = match context {
        HostCallContext::Accumulate(ctx_x, ctx_y) => (ctx_x, ctx_y),
        _ => {
            println!("We should never be here! Collapse: Invalid context");
            return (AccumulationPartialState::default(), vec![], None, 0);
        }
    };

    if let WorkExecResult::Error(_) = output {
        return (ctx_y.partial_state, ctx_y.deferred_transfers, ctx_y.y, gas);
    }

    if let WorkExecResult::Ok(payload) = output {
        if payload.len() == std::mem::size_of::<OpaqueHash>() {
            return (ctx_x.partial_state, ctx_x.deferred_transfers, Some(payload.try_into().unwrap()), gas);
        }
    }

    return (ctx_x.partial_state, ctx_x.deferred_transfers, ctx_x.y, gas);
}


fn I(partial_state: &AccumulationPartialState, service_id: &ServiceId) -> AccumulationContext {

    let mut encoded = Vec::from(service_id.encode());
    crate::blockchain::state::entropy::get_recent_entropy().encode_to(&mut encoded);
    crate::blockchain::state::time::get_current_slot().encode_to(&mut encoded);

    let payload = ((OpaqueHash::decode_size(&mut BytesReader::new(&blake2_256(&encoded)), 4).unwrap() % ((1 << 32) - (1 << 9))) + (1 << 8)) as ServiceId;
    
    let i = check(&partial_state, &payload);

    return AccumulationContext {
        service_id: *service_id,
        partial_state: partial_state.clone(),
        index: i,
        deferred_transfers: vec![],
        y: None,
    };
}


fn check(partial_state: &AccumulationPartialState, i: &ServiceId) -> ServiceId {

    if partial_state.services_accounts.service_accounts.get(i).is_none() {
        return *i;
    }
    let index = ((((((*i - (1 << 8) + 1) as i64) % (1i64 << 32)) - (1 << 9))) + (1 << 8)) as ServiceId;
    return check(partial_state, &index);
}