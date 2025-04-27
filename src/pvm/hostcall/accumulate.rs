use std::collections::HashSet;
use sp_core::blake2_256;

use crate::blockchain::state::time::get_current_block_slot;
use crate::pvm;
use crate::types::{
    Account, AccumulationContext, AccumulationOperand, AccumulationPartialState, DeferredTransfer, ExitReason, Gas, HostCallFn, OpaqueHash, 
    RamAddress, RamMemory, RegSize, Registers, ServiceId, TimeSlot, WorkExecResult, CoreIndex
};
use crate::constants::{CASH, CORES_COUNT, FULL, HUH, LOW, MAX_ITEMS_AUTHORIZATION_QUEUE, NONE, OK, TRANSFER_MEMO_SIZE, WHAT, WHO, CORE};
use crate::blockchain::state::{services::decode_preimage, entropy, time};
use crate::pvm::hostcall::{hostcall_argument, HostCallContext};
use crate::utils::codec::{Encode, DecodeSize, BytesReader};
use super::general_fn::{write, gas, info, read};

pub fn invoke_accumulation(
    partial_state: &AccumulationPartialState,
    slot: &TimeSlot,
    service_id: &ServiceId,
    gas: Gas,
    operands: &[AccumulationOperand]
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

    let args = [slot.encode(), service_id.encode(), operands.encode()].concat();

    let preimage_data = decode_preimage(&preimage_code).unwrap(); // TODO handle error

    let hostcall_arg_result = hostcall_argument(
                                &preimage_data.code, 
                                5, 
                                gas, 
                                &args, 
                                accumulation_dispatcher, 
                                HostCallContext::Accumulate(I(partial_state, service_id), I(partial_state, service_id)));
    
    let (gas, exec_result, ctx) = hostcall_arg_result;

    collapse(gas, exec_result, ctx)
}

pub fn accumulation_dispatcher(n: HostCallFn, mut gas: Gas, mut reg: Registers, ram: RamMemory, ctx: HostCallContext) 

-> (ExitReason, Gas, Registers, RamMemory, HostCallContext) {

    //println!("Dispatch accumulate: n: {:?}, gas: {:?}, reg: {:?}", n, gas, reg);
    
    match n {
        HostCallFn::Gas         => pvm::hostcall::general_fn::gas(gas, reg, ram, ctx),
        HostCallFn::Assign      => assign(gas, reg, ram, ctx),
        HostCallFn::Checkpoint  => checkpoint(gas, reg, ram, ctx),
        HostCallFn::New         => new(gas, reg, ram, ctx),
        HostCallFn::Transfer    => transfer(gas, reg, ram, ctx),
        HostCallFn::Solicit     => solicit(gas, reg, ram, ctx, get_current_block_slot()),
        HostCallFn::Write => {
            let (ctx_x, ctx_y) = ctx.to_acc_ctx();
            let account = get_accumulating_service_account(&ctx_x.partial_state, &ctx_x.service_id).unwrap();
            general_fn(write(gas, reg, ram, account, ctx_x.service_id), (ctx_x, ctx_y))
        }
        HostCallFn::Info => {
            let (ctx_x, ctx_y) = ctx.to_acc_ctx();
            general_fn(info(gas, reg, ram, ctx_x.service_id, ctx_x.partial_state.services_accounts.clone()), (ctx_x, ctx_y))
        }
        HostCallFn::Read => {
            let (ctx_x, ctx_y) = ctx.to_acc_ctx();
            let account = get_accumulating_service_account(&ctx_x.partial_state, &ctx_x.service_id).unwrap();
            general_fn(read(gas, reg, ram, account, ctx_x.service_id, ctx_x.partial_state.services_accounts.clone()), (ctx_x, ctx_y))
        }
        /*HostCallFn::Info => {
            let exit_reason = info(&mut gas, &mut reg, &mut ram, &ctx_x.service_id, &ctx_x.partial_state.services_accounts);
            let account = get_accumulating_service_account(&ctx_x.partial_state, &ctx_x.service_id).unwrap();
            general_fn(account, &mut ctx_x);
            return HostCallResult::Ok(exit_reason, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y)); 
        }*/
        HostCallFn::Log      => { println!("ACCUMULATE: Log hostcall!"); (ExitReason::Continue, gas, reg, ram, ctx)},
        _ => {
            gas -= 10;
            reg[7] = WHAT;
            return (ExitReason::Continue, gas, reg, ram, ctx);
        }
    }
}

impl HostCallContext {
    fn to_acc_ctx(self) -> (AccumulationContext, AccumulationContext)
    {
        let HostCallContext::Accumulate(ctx_x, ctx_y) = self else {
            unreachable!("Dispatch accumulate: We should never be here! Invalid context");
        };

        return (ctx_x, ctx_y);
    }
}

fn general_fn(results: (ExitReason, Gas, Registers, RamMemory, Account), ctx: (AccumulationContext, AccumulationContext))

-> (ExitReason, Gas, Registers, RamMemory, HostCallContext) 
{
    let (exit_reason, gas, reg, ram, account) = results;
    let (mut ctx_x, ctx_y) = ctx;

    ctx_x.partial_state.services_accounts.service_accounts.insert(ctx_x.service_id, account);

    return (exit_reason, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
}

fn get_accumulating_service_account(partial_state: &AccumulationPartialState, service_id: &ServiceId) -> Option<Account> {

    if let Some(account) = partial_state.services_accounts.service_accounts.get(service_id) {
        return Some(account.clone());
    }

    return None;
}

fn collapse(gas: Gas, output: WorkExecResult, ctx: HostCallContext) 

-> (AccumulationPartialState, Vec<DeferredTransfer>, Option<OpaqueHash>, Gas) 
{
    let (ctx_x, ctx_y) = ctx.to_acc_ctx();

    if let WorkExecResult::Error(_) = output {
        println!("WorkExecResult::Error: {:?}", output);
        return (ctx_y.partial_state, ctx_y.deferred_transfers, ctx_y.y, gas);
    }

    if let WorkExecResult::Ok(payload) = output {
        if payload.len() == std::mem::size_of::<OpaqueHash>() {
            println!("WorkExecResult::Ok: {:?}", payload);
            return (ctx_x.partial_state, ctx_x.deferred_transfers, Some(payload.try_into().unwrap()), gas);
        }
    }

    println!("Service HASH: {:x?}", ctx_x.y);
    return (ctx_x.partial_state, ctx_x.deferred_transfers, ctx_x.y, gas);
}

#[allow(non_snake_case)]
fn I(partial_state: &AccumulationPartialState, service_id: &ServiceId) -> AccumulationContext {

    let encoded = [service_id.encode(), entropy::get_recent_entropy().encode(), time::get_current_block_slot().encode()].concat();
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

fn transfer(mut gas: Gas, mut reg: Registers, ram: RamMemory, ctx: HostCallContext)

-> (ExitReason, Gas, Registers, RamMemory, HostCallContext)
{
    gas = gas - 10 - reg[9] as Gas;

    if gas < 0 {
        return (ExitReason::OutOfGas, gas, reg, ram, ctx);
    }

    let dest = reg[7];
    let amount = reg[8];
    let limit = reg[9];
    let start_address = reg[10];
   
    if !ram.is_readable(start_address as RamAddress, start_address as RamAddress + TRANSFER_MEMO_SIZE as RamAddress) {
        return (ExitReason::panic, gas, reg, ram, ctx);
    }

    let (mut ctx_x, ctx_y) = ctx.to_acc_ctx();    

    let transfer = DeferredTransfer {
        from: ctx_x.service_id,
        to: dest as ServiceId,
        amount: amount as u64,
        memo: ram.read(start_address as RamAddress, TRANSFER_MEMO_SIZE as RamAddress),
        gas_limit: limit as Gas,
    };

    let context_services = ctx_x.partial_state.services_accounts.clone();
    let balance = context_services.service_accounts.get(&ctx_x.service_id).unwrap().balance.saturating_sub(amount as u64);

    if let Some(account) = context_services.service_accounts.get(&(dest as ServiceId)) {
        
        if limit < account.min_gas as u64 {
            reg[7] = LOW;
            return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
        }
        if balance < account.get_footprint_and_threshold().2 {
            reg[7] = CASH;
            return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
        }

        reg[7] = OK;
        ctx_x.deferred_transfers.push(transfer);
        ctx_x.partial_state.services_accounts.service_accounts.get_mut(&ctx_x.service_id).unwrap().balance = balance;
        return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
    } 

    reg[7] = WHO;
    return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
}

fn new(mut gas: Gas, mut reg: Registers, ram: RamMemory, ctx: HostCallContext)

-> (ExitReason, Gas, Registers, RamMemory, HostCallContext)
{
    gas -= 10;

    if gas < 0 {
        return (ExitReason::OutOfGas, gas, reg, ram, ctx);
    }

    let start_address = reg[7];
    let limit = reg[8];
    let new_account_gas = reg[9];
    let new_account_min_gas = reg[10];

    let HostCallContext::Accumulate(mut ctx_x, ctx_y) = ctx else {
        unreachable!("Dispatch accumulate: Invalid context");
    };

    let from_address = start_address as RamAddress;
    let to_address = start_address as RamAddress + 32;

    if ram.is_readable(from_address, to_address) && limit < (1 << 32) {
        let c = ram.read(from_address, 32);
        let mut new_account = Account::default();
        new_account.code_hash.copy_from_slice(&c);
        new_account.lookup.insert((new_account.code_hash, limit as u32), vec![]);
        new_account.gas = new_account_gas as Gas;
        new_account.min_gas = new_account_min_gas as Gas;
        let threshold_account = new_account.get_footprint_and_threshold().2;
        new_account.balance = threshold_account;

        let mut service_account = ctx_x.partial_state.services_accounts.service_accounts.get(&ctx_x.service_id).unwrap().clone(); // TODO handle error
        service_account.balance = service_account.balance.saturating_sub(threshold_account);

        if service_account.balance < ctx_x.partial_state.services_accounts.service_accounts.get(&ctx_x.service_id).unwrap().get_footprint_and_threshold().2 {
            reg[7] = CASH;
            return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
        }

        let i = (1u64 << 8) + (ctx_x.index as u64 - (1u64 << 8) + 42) % ((1u64 << 32) - (1u64 << 9));
        
        reg[7] = ctx_x.index as RegSize;
        println!("berservice: {:?} | new account: {:?}", ctx_x.index, new_account);
        ctx_x.partial_state.services_accounts.service_accounts.insert(ctx_x.index, new_account);
        ctx_x.partial_state.services_accounts.service_accounts.insert(ctx_x.service_id, service_account);
        ctx_x.index = check(&ctx_x.partial_state, &(i as ServiceId));

        return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
    }
        
    return (ExitReason::panic, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
}

fn solicit(mut gas: Gas, mut reg: Registers, ram: RamMemory, ctx: HostCallContext, slot: TimeSlot)

-> (ExitReason, Gas, Registers, RamMemory, HostCallContext)
{
    gas -= 10;

    if gas < 0 {
        return (ExitReason::OutOfGas, gas, reg, ram, ctx);
    }

    let start_address = reg[7] as RamAddress;
    let preimage_size = reg[8] as u32;
    
    let (mut ctx_x, ctx_y) = ctx.to_acc_ctx();

    if !ram.is_readable(start_address, start_address + 32){
        return (ExitReason::panic, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
    }

    let hash: OpaqueHash = ram.read(start_address,  32).try_into().unwrap();

    let mut account = ctx_x.partial_state.services_accounts.service_accounts.get(&ctx_x.service_id).unwrap().clone();

    if account.lookup.contains_key(&(hash, preimage_size)) {
        
        let mut timeslots = account.lookup.get(&(hash, preimage_size)).unwrap().clone();

        if timeslots.len() != 2 {
            reg[7] = HUH;
            return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
        }

        timeslots.push(slot);

        account.lookup.insert((hash, preimage_size), timeslots);
    } else {
        account.lookup.insert((hash, preimage_size), vec![]);
    }

    if account.balance < account.get_footprint_and_threshold().2 {
        reg[7] = FULL;
        return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
    }

    reg[7] = OK;
    ctx_x.partial_state.services_accounts.service_accounts.insert(ctx_x.service_id, account);

    return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
}

fn assign(mut gas: Gas, mut reg: Registers, ram: RamMemory, ctx: HostCallContext)

-> (ExitReason, Gas, Registers, RamMemory, HostCallContext)
{
    gas -= 10;

    if gas < 0 {
        return (ExitReason::OutOfGas, gas, reg, ram, ctx);
    }

    let core_index = reg[7] as CoreIndex;
    
    if core_index >= CORES_COUNT as CoreIndex {
        reg[7] = CORE;
        return (ExitReason::Continue, gas, reg, ram, ctx);
    }

    let start_address = reg[8] as RamAddress;
    let (mut ctx_x, ctx_y) = ctx.to_acc_ctx();

    if !ram.is_readable(start_address, start_address + 32 * MAX_ITEMS_AUTHORIZATION_QUEUE as RamAddress) {
        return (ExitReason::panic, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
    }

    for i in 0..MAX_ITEMS_AUTHORIZATION_QUEUE {
        ctx_x.partial_state.queues_auth.auth_queues[core_index as usize].auth_queue[i] = ram.read(start_address + 32 * i as u32, 32).try_into().unwrap();
    }

    reg[7] = OK;

    return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
}

fn checkpoint(mut gas: Gas, mut reg: Registers, ram: RamMemory, ctx: HostCallContext)

-> (ExitReason, Gas, Registers, RamMemory, HostCallContext)
{
    gas -= 10;

    if gas < 0 {
        return (ExitReason::OutOfGas, gas, reg, ram, ctx);
    }

    let (ctx_x, mut ctx_y) = ctx.to_acc_ctx();
    
    ctx_y = ctx_x.clone();
    reg[7] = gas as RegSize;
    
    return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
}