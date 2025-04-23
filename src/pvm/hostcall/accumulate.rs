use std::collections::{HashMap, HashSet};

use sp_core::blake2_256;
use sp_core::ecdsa::PUBLIC_KEY_SERIALIZED_SIZE;

use crate::blockchain::state::entropy::get_recent_entropy;
use crate::blockchain::state::{get_entropy, get_time};
use crate::types::{
    Account, AccumulationContext, AccumulationOperand, AccumulationPartialState, DeferredTransfer, ExitReason, Gas, Hash, 
    HostCallFn, OpaqueHash, RamAddress, RamMemory, RegSize, Registers, ServiceId, TimeSlot, WorkExecResult, 
};
use crate::constants::{CASH, CORE, FULL, HUH, LOW, NONE, OK, OOB, PAGE_SIZE, VALIDATORS_COUNT, WHAT, WHO, TRANSFER_MEMO_SIZE};
use crate::utils::codec::{Encode, DecodeSize, BytesReader};
use crate::pvm::hostcall::{hostcall_argument, is_readable, HostCallContext};
use crate::pvm::hostcall::general_functions::info;
use crate::blockchain::state::services::{decode_preimage, historical_preimage_lookup};
use crate::utils::common;

pub fn invoke_accumulation(
    partial_state: &AccumulationPartialState,
    slot: &TimeSlot,
    service_id: &ServiceId,
    gas: Gas,
    operand: &[AccumulationOperand]
) -> (AccumulationPartialState, Vec<DeferredTransfer>, Option<OpaqueHash>, Gas) {
    println!("Invoke accumulation: slot: {:?}, service_id: {:?}", slot, service_id);
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
    operand.encode_to(&mut a);
    println!("\nInvoke accumulation: a: {:?}, len: {:?}", a, a.len());

    let preimage_data = decode_preimage(&preimage_code).unwrap(); // TODO handle error

    let hostcall_arg_result = hostcall_argument(&preimage_data.code, 5, gas, &a, accumulation_dispatcher, HostCallContext::Accumulate(I(partial_state, service_id), I(partial_state, service_id)));
    
    let (gas, exec_result, ctx) = hostcall_arg_result;

    collapse(gas, exec_result, ctx)
}

// F: Fn(&[u8], RegSize, Gas, Registers, RamMemory, HostCallContext) -> (ExitReason, RegSize, Gas, Registers, RamMemory, HostCallContext)
// F: Fn(HostCallFn, Gas, Registers, RamMemory, HostCallContext) -> (ExitReason, RegSize, Gas, Registers, RamMemory, HostCallContext)
pub fn accumulation_dispatcher(n: HostCallFn, mut gas: Gas, mut reg: Registers, mut ram: RamMemory, mut ctx: HostCallContext) 

-> (ExitReason, Gas, Registers, RamMemory, HostCallContext) {

    //println!("Dispatch accumulate: n: {:?}, gas: {:?}, reg: {:?}", n, gas, reg);
    
    match n {
        HostCallFn::New => {
            let o = reg[7];
            let l = reg[8];
            let g = reg[9];
            let m = reg[10];

            let HostCallContext::Accumulate(mut ctx_x, mut ctx_y) = ctx else {
                unreachable!("Dispatch accumulate: Invalid context");
            };

            let from_address = o as RamAddress;
            let to_address = o as RamAddress + 32;
            
            if ram.is_readable(from_address, to_address) && l < (1 << 32) {
                let c = ram.read(from_address, 32);
                let mut new_account = Account::default();
                new_account.code_hash.copy_from_slice(&c);
                new_account.lookup.insert((new_account.code_hash, l as u32), vec![]);
                new_account.gas = g as Gas;
                new_account.min_gas = m as Gas;
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
        HostCallFn::Transfer => {
            transfer(gas, reg, ram, ctx)
        }
        HostCallFn::Write => {
            
            let HostCallContext::Accumulate(mut ctx_x, mut ctx_y) = ctx else {
                unreachable!("Dispatch accumulate: Invalid context");
            };
            let account = get_accumulating_service_account(&ctx_x.partial_state, &ctx_x.service_id).unwrap();
            let service_id = ctx_x.service_id;
            general_hostcall(write(gas, reg, ram, account, service_id), (ctx_x, ctx_y))
        }
        /*HostCallFn::Info => {
            let exit_reason = info(&mut gas, &mut reg, &mut ram, &ctx_x.service_id, &ctx_x.partial_state.services_accounts);
            let account = get_accumulating_service_account(&ctx_x.partial_state, &ctx_x.service_id).unwrap();
            general_hostcall(account, &mut ctx_x);
            return HostCallResult::Ok(exit_reason, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y)); 
        }*/
        _ => {
            let HostCallContext::Accumulate(mut ctx_x, mut ctx_y) = ctx else {
                unreachable!("Dispatch accumulate: Invalid context");
            };
            gas -= 10;
            reg[7] = WHAT;
            return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
        }
    }
}

fn transfer(
    gas: Gas, 
    mut reg: Registers, 
    ram: RamMemory, 
    ctx: HostCallContext)

-> (ExitReason, Gas, Registers, RamMemory, HostCallContext)
{
    let HostCallContext::Accumulate(mut ctx_x, ctx_y) = ctx else {
        unreachable!("Dispatch transfer: Invalid context");
    };

    let d = reg[7];
    let a = reg[8];
    let l = reg[9];
    let o = reg[10];

    if !ram.is_readable(o as RamAddress, o as RamAddress + TRANSFER_MEMO_SIZE as RamAddress) {
        return (ExitReason::panic, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
    }

    let transfer = DeferredTransfer {
        from: ctx_x.service_id,
        to: d as ServiceId,
        amount: a as u64,
        memo: ram.read(o as RamAddress, TRANSFER_MEMO_SIZE as RamAddress),
        gas_limit: l as Gas,
    };
    
    println!("TRANSFER from: {:?}", transfer.from);
    println!("TRANSFER to: {:?}", transfer.to);
    println!("TRANSFER gas_limit: {:?}", transfer.gas_limit);
    println!("TRANSFER ammount: {:?}", transfer.amount);

    let context_services = ctx_x.partial_state.services_accounts.clone();
    let b = if let Some(account) = context_services.service_accounts.get(&ctx_x.service_id) {
        account.balance.saturating_sub(a as u64)
    } else {
        return (ExitReason::panic, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y)); // TODO esto es panic?
    };

    if let Some(account) = context_services.service_accounts.get(&(d as ServiceId)) {
        if l < account.min_gas as u64 {
            reg[7] = LOW;
            return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
        }
        if b < account.get_footprint_and_threshold().2 {
            reg[7] = CASH;
            return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
        }
        reg[7] = OK;
        println!("reg = {:?}", reg);
        ctx_x.deferred_transfers.push(transfer);
        println!("transfer service_id: {:?}, balance: {b}", ctx_x.service_id);
        ctx_x.partial_state.services_accounts.service_accounts.get_mut(&ctx_x.service_id).unwrap().balance = b;
        println!("ctx_x.balance: {:?}", ctx_x.partial_state.services_accounts.service_accounts.get(&ctx_x.service_id).unwrap().balance);
        return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
    } 

    reg[7] = WHO;
    return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
}

fn write(
    gas: Gas, 
    mut reg: Registers, 
    ram: RamMemory, 
    account: Account, 
    service_id: ServiceId) 

-> (ExitReason, Gas, Registers, RamMemory, Account)
{
    let k_o = reg[7];
    let k_z = reg[8];
    let v_o = reg[9];
    let v_z = reg[10];

    if !ram.is_readable(k_o as RamAddress, k_o as RamAddress + k_z as RamAddress) {
        return (ExitReason::panic, gas, reg, ram, account);
    }

    let k = sp_core::blake2_256(&[service_id.encode(), ram.read(k_o as RamAddress, k_z as RamAddress)].concat());
    let mut s_account = account.clone();

    let modified_account = if v_z == 0 {    
        let mut key_set = HashSet::new();
        key_set.insert(k);
        let storage = common::dict_subtract(&s_account.storage, &key_set);
        s_account.storage = storage;
        s_account
    } else if ram.is_readable(v_o as RamAddress, v_o as RamAddress + v_z as RamAddress) {
        let storage_data = ram.read(v_o as RamAddress, v_z as RamAddress);
        s_account.storage.insert(k, storage_data);
        s_account
    } else {
        return (ExitReason::panic, gas, reg, ram, account);
    };

    let l: RegSize = if let Some(storage_data) = modified_account.storage.get(&k) {
        storage_data.len() as RegSize
    } else {
        NONE as RegSize
    };

    let threshold = modified_account.get_footprint_and_threshold().2;

    if threshold > modified_account.balance {
        reg[7] = FULL as RegSize;
        return (ExitReason::Continue, gas, reg, ram, account);
    }

    reg[7] = l;
    return (ExitReason::Continue, gas, reg, ram, modified_account);
    
}

fn general_hostcall(
    results: (ExitReason, Gas, Registers, RamMemory, Account), 
    ctx: (AccumulationContext, AccumulationContext))

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

fn collapse(gas: Gas, output: WorkExecResult, context: HostCallContext) 

-> (AccumulationPartialState, Vec<DeferredTransfer>, Option<OpaqueHash>, Gas) 
{
    let (ctx_x, ctx_y) = match context {
        HostCallContext::Accumulate(ctx_x, ctx_y) => (ctx_x, ctx_y),
        _ => {
            println!("We should never be here! Collapse: Invalid context");
            return (AccumulationPartialState::default(), vec![], None, 0);
        }
    };

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