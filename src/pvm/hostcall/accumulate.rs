use sp_core::blake2_256;

use crate::blockchain::state::{get_entropy, get_time};
use crate::types::{
    Account, AccumulationContext, AccumulationOperand, AccumulationPartialState, DeferredTransfer, ExitReason, Gas, OpaqueHash, 
    ServiceId, TimeSlot, WorkExecResult, HostCallType
};
use crate::pvm::hostcall::{HostCallContext, HostCallArgs, HostCallResult};
use crate::constants::{NONE, WHAT, OOB, WHO, FULL, CORE, CASH, LOW, HUH, OK};
use crate::utils::codec::{Encode, DecodeSize, BytesReader};
use crate::pvm::hostcall::general_functions::info;

pub fn invoke_accumulation(
    partial_state: AccumulationPartialState,
    slot: &TimeSlot,
    service_id: &ServiceId,
    gas: Gas,
    operand: &[AccumulationOperand]
) -> (AccumulationPartialState, Vec<DeferredTransfer>, Option<OpaqueHash>, Gas) {

    if partial_state.services_accounts.service_accounts.get(service_id).is_none() {
        return (I(partial_state, service_id).partial_state, vec![], None, 0);
    }

    let mut a: Vec<u8> = Vec::new();
    slot.encode_to(&mut a);
    service_id.encode_to(&mut a);
    operand.len().encode_to(&mut a);

    return (I(partial_state, service_id).partial_state, vec![], None, 0);
}

pub fn dispatch_accumulate(args: HostCallArgs) -> HostCallResult {

    let (n, mut gas, mut reg, mut ram, ctx) = args;
    
    let HostCallContext::Accumulate(mut ctx_x, mut ctx_y) = ctx else {
        println!("Dispatch accumulate: Invalid context");
        return HostCallResult::Err(0);
    };

    match n {

        HostCallType::Info => {
            let exit_reason = info(&mut gas, &mut reg, &mut ram, &ctx_x.service_id, &ctx_x.partial_state.services_accounts);
            let account = get_accumulating_service_account(&ctx_x.partial_state, &ctx_x.service_id).unwrap();
            general_hostcall(account, &mut ctx_x);
            return HostCallResult::Ok(exit_reason, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y)); 
        }
        _ => {
            gas -= 10;
            reg[7] = WHAT;
            return HostCallResult::Ok(ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
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

fn collapse(gas: Gas, 
            output: WorkExecResult, 
            context: (AccumulationContext, AccumulationContext)
) -> (AccumulationPartialState, Vec<DeferredTransfer>, Option<OpaqueHash>, Gas) {

    let (ctx_x, ctx_y) = context;

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


fn I(partial_state: AccumulationPartialState, service_id: &ServiceId) -> AccumulationContext {

    let entropy_pool = get_entropy();
    let post_tau = get_time();

    let mut encoded = Vec::from(service_id.encode());
    entropy_pool.buf[0].encode_to(&mut encoded);
    post_tau.encode_to(&mut encoded);

    let payload = ((OpaqueHash::decode_size(&mut BytesReader::new(&blake2_256(&encoded)), 4).unwrap() % ((1 << 32) - (1 << 9))) + (1 << 8)) as ServiceId;
    
    let i = check(&partial_state, &payload);

    return AccumulationContext {
        service_id: *service_id,
        partial_state,
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