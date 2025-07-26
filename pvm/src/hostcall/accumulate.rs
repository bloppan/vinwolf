use std::collections::HashMap;
use sp_core::blake2_256;
use {once_cell::sync::Lazy, std::sync::Mutex};

use jam_types::{
    Account, AccumulationContext, AccumulationOperand, AccumulationPartialState, CoreIndex, DeferredTransfer, Gas, OpaqueHash, 
    ServiceId, TimeSlot, ValidatorsData, WorkExecResult, StateKeyType,
};
use crate::pvm_types::{RamAddress, RamMemory, RegSize, Registers, ExitReason, HostCallFn};
use constants::pvm::*;

use constants::node::{
    CORES_COUNT, MAX_ITEMS_AUTHORIZATION_QUEUE, MAX_TIMESLOTS_AFTER_UNREFEREND_PREIMAGE, TRANSFER_MEMO_SIZE, VALIDATORS_COUNT, MAX_SERVICE_CODE_SIZE
};
use utils::common::parse_preimage;
use crate::hostcall::{hostcall_argument, HostCallContext};
use codec::{Encode, EncodeSize, DecodeSize, BytesReader};
use codec::generic_codec::{encode_unsigned, decode};
use utils::serialization::{StateKeyTrait, construct_lookup_key, construct_preimage_key};
use crate::hostcall::general_fn::{fetch, write, info, read, lookup, log};

static OPERANDS: Lazy<Mutex<Vec<AccumulationOperand>>> = Lazy::new(|| {
    Mutex::new(Vec::new())
});

fn set_operands(operands: &[AccumulationOperand]) {
    let mut lock = OPERANDS.lock().unwrap();
    lock.clear();
    lock.extend_from_slice(operands);
}

fn get_operands() -> std::sync::MutexGuard<'static, Vec<AccumulationOperand>> {
    OPERANDS.lock().unwrap()
}

pub fn invoke_accumulation(
    partial_state: AccumulationPartialState,
    slot: &TimeSlot,
    service_id: &ServiceId,
    gas: Gas,
    operands: &[AccumulationOperand]
) -> (AccumulationPartialState, Vec<DeferredTransfer>, Option<OpaqueHash>, Gas, Vec<(ServiceId, Vec<u8>)>) {
    
    log::debug!("Invoke accumulation for service {:?} gas {:?} slot {:?}", *service_id, gas, *slot);

    let preimage = match parse_preimage(&partial_state.service_accounts, service_id) {
        Ok(preimage) => {
            if preimage.is_none() {
                log::error!("The preimage is none");
                return (partial_state.clone(), vec![], None, 0, vec![]);
            }
            preimage.unwrap()
        },
        Err(_) => { 
            log::error!("Failed to decode preimage");
            return (partial_state.clone(), vec![], None, 0, vec![]); 
        },
    };

    if preimage.code.len() > MAX_SERVICE_CODE_SIZE {
        log::error!("The preimage code len is greater than the max service code size allowed");
        return (partial_state.clone(), vec![], None, 0, vec![]);
    }

    let args = [encode_unsigned(*slot as usize), encode_unsigned(*service_id as usize), encode_unsigned(operands.len())].concat();
    log::debug!("Hostcall args: {}", hex::encode(&args));

    set_operands(operands);

    let hostcall_arg_result: (i64, WorkExecResult, HostCallContext) = hostcall_argument(
                                &preimage.code, 
                                5, 
                                gas, 
                                &args, 
                                dispatch_acc, 
                                HostCallContext::Accumulate(I(&partial_state, service_id), I(&partial_state, service_id)));
    
    let (gas, exec_result, ctx) = hostcall_arg_result;

    collapse(gas, exec_result, ctx)
}

fn dispatch_acc(n: HostCallFn, mut gas: Gas, mut reg: Registers, ram: RamMemory, ctx: HostCallContext) 

-> (ExitReason, Gas, Registers, RamMemory, HostCallContext) {

    log::debug!("Dispatch accumulate: {:?} hostcall", n);
    
    match n {
        HostCallFn::Gas         => crate::hostcall::general_fn::gas(gas, reg, ram, ctx),
        HostCallFn::Bless       => bless(gas, reg, ram, ctx),
        HostCallFn::Assign      => assign(gas, reg, ram, ctx),
        HostCallFn::Designate   => designate(gas, reg, ram, ctx),
        HostCallFn::Checkpoint  => checkpoint(gas, reg, ram, ctx),
        HostCallFn::New         => new(gas, reg, ram, ctx),
        HostCallFn::Upgrade     => upgrade(gas, reg, ram, ctx),
        HostCallFn::Transfer    => transfer(gas, reg, ram, ctx),
        HostCallFn::Eject       => eject(gas, reg, ram, ctx, state_handler::time::get_current()),
        HostCallFn::Query       => query(gas, reg, ram, ctx),
        HostCallFn::Solicit     => solicit(gas, reg, ram, ctx, state_handler::time::get_current()),
        HostCallFn::Forget      => forget(gas, reg, ram, ctx, state_handler::time::get_current()),
        HostCallFn::Yield       => yield_(gas, reg, ram, ctx),
        HostCallFn::Provide     => {
            let (ctx_x, _ctx_y) = ctx.clone().to_acc_ctx();
            provide(gas, reg, ram, ctx, ctx_x.service_id)
        }
        HostCallFn::Fetch => {
            let operands: Vec<AccumulationOperand> = get_operands().clone();
            fetch(gas, reg, ram, None, Some(state_handler::entropy::get_recent().entropy), None, None, None, Some(operands), None, ctx)
        }
        HostCallFn::Write => {
            let (ctx_x, ctx_y) = ctx.to_acc_ctx();
            let account = get_accumulating_service_account(&ctx_x.partial_state, &ctx_x.service_id).unwrap();
            general_fn(write(gas, reg, ram, account, ctx_x.service_id), (ctx_x, ctx_y))
        }
        HostCallFn::Info => {
            let (ctx_x, ctx_y) = ctx.to_acc_ctx();
            general_fn(info(gas, reg, ram, ctx_x.service_id, ctx_x.partial_state.service_accounts.clone()), (ctx_x, ctx_y))
        }
        HostCallFn::Read => {
            let (ctx_x, ctx_y) = ctx.to_acc_ctx();
            let account = get_accumulating_service_account(&ctx_x.partial_state, &ctx_x.service_id).unwrap();
            general_fn(read(gas, reg, ram, account, ctx_x.service_id, ctx_x.partial_state.service_accounts.clone()), (ctx_x, ctx_y))
        }
        HostCallFn::Lookup => {
            let (ctx_x, ctx_y) = ctx.to_acc_ctx();
            let account = get_accumulating_service_account(&ctx_x.partial_state, &ctx_x.service_id).unwrap();
            general_fn(lookup(gas, reg, ram, account, ctx_x.service_id, ctx_x.partial_state.service_accounts.clone()), (ctx_x, ctx_y))
        }
        HostCallFn::Log      => { 
            let (ctx_x, _ctx_y) = ctx.clone().to_acc_ctx(); 
            log(&reg, &ram, &ctx_x.service_id); 
            (ExitReason::Continue, gas, reg, ram, ctx)
        },
        _ => {
            log::error!("Unknown hostcall function: {:?}", n);
            gas -= 10;
            reg[7] = WHAT;
            return (ExitReason::Continue, gas, reg, ram, ctx);
        }
    }
}

fn general_fn(results: (ExitReason, Gas, Registers, RamMemory, Account), ctx: (AccumulationContext, AccumulationContext))

-> (ExitReason, Gas, Registers, RamMemory, HostCallContext) 
{
    let (exit_reason, gas, reg, ram, account) = results;
    let (mut ctx_x, ctx_y) = ctx;

    ctx_x.partial_state.service_accounts.insert(ctx_x.service_id, account);

    return (exit_reason, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
}

fn get_accumulating_service_account(partial_state: &AccumulationPartialState, service_id: &ServiceId) -> Option<Account> {

    if let Some(account) = partial_state.service_accounts.get(service_id) {
        return Some(account.clone());
    }

    return None;
}

fn collapse(gas: Gas, output: WorkExecResult, ctx: HostCallContext) 

-> (AccumulationPartialState, Vec<DeferredTransfer>, Option<OpaqueHash>, Gas, Vec<(ServiceId, Vec<u8>)>) 
{
    let (ctx_x, ctx_y) = ctx.to_acc_ctx();

    if let WorkExecResult::Error(_) = output {
        log::error!("WorkExecResult::Error: {:?}", output);
        return (ctx_y.partial_state, ctx_y.deferred_transfers, ctx_y.y, gas, ctx_y.preimages);
    }

    if let WorkExecResult::Ok(payload) = output {
        if payload.len() == std::mem::size_of::<OpaqueHash>() {
            log::debug!("WorkExecResult::Ok: {:?}", payload);
            return (ctx_x.partial_state, ctx_x.deferred_transfers, Some(payload.try_into().unwrap()), gas, ctx_x.preimages);
        }
    }

    //log::debug!("Service HASH: {:x?}", ctx_x.y);
    return (ctx_x.partial_state, ctx_x.deferred_transfers, ctx_x.y, gas, ctx_x.preimages);
}

#[allow(non_snake_case)]
fn I(partial_state: &AccumulationPartialState, service_id: &ServiceId) -> AccumulationContext {

    //let encoded = [service_id.encode(), entropy::get_recent_entropy().encode(), time::get_current_block_slot().encode()].concat();
    let encoded = [encode_unsigned(*service_id as usize), state_handler::entropy::get_recent().encode(), encode_unsigned(state_handler::time::get_current() as usize)].concat();
    let payload = ((OpaqueHash::decode_size(&mut BytesReader::new(&blake2_256(&encoded)), 4).unwrap() % ((1 << 32) - (1 << 9))) + (1 << 8)) as ServiceId;
    let i = check(&partial_state, &payload);

    return AccumulationContext {
        service_id: *service_id,
        partial_state: partial_state.clone(),
        index: i,
        deferred_transfers: vec![],
        y: None,
        preimages: Vec::new(),
    };
}

fn check(partial_state: &AccumulationPartialState, i: &ServiceId) -> ServiceId {

    if partial_state.service_accounts.get(i).is_none() {
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
        log::error!("Out of gas!");
        return (ExitReason::OutOfGas, gas, reg, ram, ctx);
    }

    let dest = reg[7];
    let amount = reg[8];
    let limit = reg[9];
    let start_address = reg[10];
    
    log::debug!("Dest: {:?} Amount: {:?} Limit: {:?}", dest, amount, limit);

    if !ram.is_readable(start_address as RamAddress, TRANSFER_MEMO_SIZE as RamAddress) {
        log::error!("Panic: RAM is not readable from address: {:?} num_bytes: {:?}", start_address, TRANSFER_MEMO_SIZE);
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

    let context_services = ctx_x.partial_state.service_accounts.clone();
    let balance = context_services.get(&ctx_x.service_id).unwrap().balance.saturating_sub(amount as u64);

    if let Some(account) = context_services.get(&(dest as ServiceId)) {
        
        if limit < account.acc_min_gas as u64 {
            log::debug!("Exit: LOW");
            reg[7] = LOW;
            return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
        }
        let (_items, _octets, threshold) = utils::common::get_footprint_and_threshold(account);
        if balance < threshold {
            log::debug!("Exit: CASH");
            reg[7] = CASH;
            return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
        }
        
        reg[7] = OK;
        ctx_x.deferred_transfers.push(transfer);
        ctx_x.partial_state.service_accounts.get_mut(&ctx_x.service_id).unwrap().balance = balance;
        
        log::debug!("Exit: OK");
        return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
    } 
    
    reg[7] = WHO;
    log::debug!("Exit: WHO");
    return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
}

fn eject(mut gas: Gas, mut reg: Registers, ram: RamMemory, ctx: HostCallContext, slot: TimeSlot)

-> (ExitReason, Gas, Registers, RamMemory, HostCallContext)
{
    gas -= 10;

    if gas < 0 {
        log::error!("Out of gas!");
        return (ExitReason::OutOfGas, gas, reg, ram, ctx);
    }

    let service_id = reg[7] as ServiceId;
    let start_address = reg[8] as RamAddress;

    log::debug!("Service id: {:?}", service_id);

    if !ram.is_readable(start_address, 32) {
        log::error!("Panic: The RAM is not readable from address: {:?} num_bytes: 32", start_address);
        return (ExitReason::panic, gas, reg, ram, ctx);
    }

    let hash: OpaqueHash = ram.read(start_address, 32).try_into().unwrap();
    let (mut ctx_x, ctx_y) = ctx.to_acc_ctx();
    
    if service_id != ctx_x.service_id && ctx_x.partial_state.service_accounts.contains_key(&service_id) {

        let d_account = ctx_x.partial_state.service_accounts.get(&service_id).unwrap().clone();
        let (items, octets, _threshold) = utils::common::get_footprint_and_threshold(&d_account);
        let length = (std::cmp::max(81, octets) - 81) as u32;

        let mut s_account = ctx_x.partial_state.service_accounts.get(&ctx_x.service_id).unwrap().clone();
        s_account.balance += d_account.balance;

        let xs_encoded: OpaqueHash = ctx_x.service_id.encode_size(32).try_into().unwrap();
        log::debug!("xs_encoded: 0x{}", hex::encode(xs_encoded));

        if d_account.code_hash != xs_encoded {
            log::debug!("Exit: WHO");
            reg[7] = WHO;
            return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
        }

        let lookup_key = StateKeyType::Account(ctx_x.service_id, construct_lookup_key(&hash, length).to_vec()).construct();
        
        if items != 2 || !d_account.lookup.contains_key(&lookup_key) {
            log::debug!("Exit: HUH");
            reg[7] = HUH;
            return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
        }

        let timeslots = d_account.lookup.get(&lookup_key).unwrap().clone();

        if timeslots.len() == 2 && (timeslots[1] < slot.saturating_sub(MAX_TIMESLOTS_AFTER_UNREFEREND_PREIMAGE)) {

            ctx_x.partial_state.service_accounts.remove(&service_id);
            ctx_x.partial_state.service_accounts.insert(ctx_x.service_id, s_account);
            reg[7] = OK;
            
            log::debug!("Exit: OK");
            return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
        }

        reg[7] = HUH;

        log::debug!("Exit: HUH");
        return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
    }

    reg[7] = WHO;

    log::debug!("Exit: WHO");
    return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
}

fn query(mut gas: Gas, mut reg: Registers, ram: RamMemory, ctx: HostCallContext)

-> (ExitReason, Gas, Registers, RamMemory, HostCallContext)
{
    gas -= 10;

    if gas < 0 {
        log::error!("Out of gas!");
        return (ExitReason::OutOfGas, gas, reg, ram, ctx);
    }

    let start_address = reg[7] as RamAddress;
    let length = reg[8] as u32;

    if !ram.is_readable(start_address, 32) {
        log::error!("Panic: The RAM is not readable from address: {:?} num_bytes: 32", start_address);
        return (ExitReason::panic, gas, reg, ram, ctx);
    }

    let (ctx_x, ctx_y) = ctx.to_acc_ctx();

    let hash: OpaqueHash = ram.read(start_address, 32).try_into().unwrap();
    let lookup_key = StateKeyType::Account(ctx_x.service_id, construct_lookup_key(&hash, length).to_vec()).construct();

    log::debug!("length: {:?}, hash: 0x{}, lookup_key: 0x{}", length, hex::encode(hash), hex::encode(lookup_key));
    
    if !ctx_x.partial_state.service_accounts.get(&ctx_x.service_id).unwrap().lookup.contains_key(&lookup_key) {
        reg[7] = NONE;
        log::debug!("Exit: NONE");
        return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
    }

    let timeslots = ctx_x.partial_state.service_accounts.get(&ctx_x.service_id).unwrap().lookup.get(&lookup_key).unwrap().clone();
    let timeslots_len = timeslots.len();
    log::debug!("timeslots: {:?}", timeslots);

    if timeslots_len == 0 {
        reg[7] = 0;
        reg[8] = 0;
    } else if timeslots_len == 1 {
        reg[7] = (1 + ((1_u64 << 32) as u64) * timeslots[0] as u64) as RegSize;
        reg[8] = 0;
    } else if timeslots_len == 2 {
        reg[7] = (2 + ((1_u64 << 32) as u64) * timeslots[0] as u64) as RegSize;
        reg[8] = timeslots[1] as RegSize;
    } else if timeslots_len == 3 {
        reg[7] = (3 + ((1_u64 << 32) as u64) * timeslots[0] as u64) as RegSize;
        reg[8] = timeslots[1] as RegSize + (1_u64 << 32) as u64 * timeslots[2] as RegSize;
    }

    log::debug!("reg_7: {:?}, reg_8: {:?}", reg[7], reg[8]);
    log::debug!("Exit: OK");
    return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
}

fn new(mut gas: Gas, mut reg: Registers, ram: RamMemory, ctx: HostCallContext)

-> (ExitReason, Gas, Registers, RamMemory, HostCallContext)
{
    gas -= 10;

    if gas < 0 {
        log::error!("Out of gas!");
        return (ExitReason::OutOfGas, gas, reg, ram, ctx);
    }

    let start_address = reg[7] as RamAddress;
    let length = reg[8];
    let new_account_gas = reg[9];
    let new_account_min_gas = reg[10];

    log::debug!("start_address: {:?}, length: {:?}, gas: {:?}, min_gas: {:?}", start_address, length, new_account_gas, new_account_min_gas);

    let HostCallContext::Accumulate(mut ctx_x, ctx_y) = ctx else {
        unreachable!("Dispatch accumulate: Invalid context");
    };

    if ram.is_readable(start_address, 32) && length < (1 << 32) {
        let c = ram.read(start_address, 32);
        let mut new_account = Account::default();
        new_account.code_hash.copy_from_slice(&c);
        let lookup_key = StateKeyType::Account(ctx_x.service_id, construct_lookup_key(&new_account.code_hash, length as u32).to_vec()).construct();
        new_account.lookup.insert(lookup_key, vec![]);
        new_account.acc_min_gas = new_account_gas as Gas;
        new_account.xfer_min_gas = new_account_min_gas as Gas;
        let (_items, _octets, threshold) = utils::common::get_footprint_and_threshold(&new_account);
        new_account.balance = threshold;

        let mut service_account = ctx_x.partial_state.service_accounts.get(&ctx_x.service_id).unwrap().clone(); // TODO handle error
        service_account.balance = service_account.balance.saturating_sub(threshold);
        let (_items, _octets, service_account_threshold) = utils::common::get_footprint_and_threshold(&service_account);
        if service_account.balance < service_account_threshold {
            reg[7] = CASH;
            log::debug!("Exit: CASH");
            return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
        }

        let i = (1u64 << 8) + (ctx_x.index as u64 - (1u64 << 8) + 42) % ((1u64 << 32) - (1u64 << 9));
        
        reg[7] = ctx_x.index as RegSize;
        ctx_x.partial_state.service_accounts.insert(ctx_x.index, new_account);
        ctx_x.partial_state.service_accounts.insert(ctx_x.service_id, service_account);
        ctx_x.index = check(&ctx_x.partial_state, &(i as ServiceId));

        log::debug!("Exit: OK");
        return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
    }
    
    log::error!("Panic: The RAM is not readable from address: {:?} num_bytes: 32", start_address);
    return (ExitReason::panic, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
}

fn upgrade(mut gas: Gas, mut reg: Registers, ram: RamMemory, ctx: HostCallContext)

-> (ExitReason, Gas, Registers, RamMemory, HostCallContext)
{
    gas -= 10;

    if gas < 0 {
        log::error!("Out of gas!");
        return (ExitReason::OutOfGas, gas, reg, ram, ctx);
    }

    let start_address = reg[7] as RamAddress;
    let new_gas = reg[8] as Gas;
    let new_min_gas = reg[9] as Gas;

    if !ram.is_readable(start_address, 32) {
        log::error!("Panic: The RAM is not readable from address: {:?} num_bytes: 32", start_address);
        return (ExitReason::panic, gas, reg, ram, ctx);
    }

    let code_hash: OpaqueHash = ram.read(start_address, 32).try_into().unwrap();
    log::debug!("gas: {:?}, min_gas: {:?}, code_hash: 0x{}", new_gas, new_min_gas, hex::encode(code_hash));
    let (mut ctx_x, ctx_y) = ctx.to_acc_ctx();

    ctx_x.partial_state.service_accounts.get_mut(&ctx_x.service_id).unwrap().code_hash = code_hash;
    ctx_x.partial_state.service_accounts.get_mut(&ctx_x.service_id).unwrap().acc_min_gas = new_gas;
    ctx_x.partial_state.service_accounts.get_mut(&ctx_x.service_id).unwrap().xfer_min_gas = new_min_gas;

    reg[7] = OK;

    log::debug!("Exit: OK");
    return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
}

fn solicit(mut gas: Gas, mut reg: Registers, ram: RamMemory, ctx: HostCallContext, slot: TimeSlot)

-> (ExitReason, Gas, Registers, RamMemory, HostCallContext)
{
    gas -= 10;

    if gas < 0 {
        log::error!("Out of gas!");
        return (ExitReason::OutOfGas, gas, reg, ram, ctx);
    }

    let start_address = reg[7] as RamAddress;
    let preimage_size = reg[8] as u32;
    
    let (mut ctx_x, ctx_y) = ctx.to_acc_ctx();

    if !ram.is_readable(start_address, 32){
        log::error!("Panic: The RAM is not readable from address: {:?} num_bytes: 32", start_address);
        return (ExitReason::panic, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
    }

    let hash: OpaqueHash = ram.read(start_address,  32).try_into().unwrap();
    let lookup_key = StateKeyType::Account(ctx_x.service_id, construct_lookup_key(&hash, preimage_size).to_vec()).construct();
    log::debug!("preimage_size: {:?}, hash: 0x{}, lookup_key: 0x{}", preimage_size, hex::encode(hash), hex::encode(lookup_key));

    let mut account = ctx_x.partial_state.service_accounts.get(&ctx_x.service_id).unwrap().clone();

    if !account.lookup.contains_key(&lookup_key) {
        log::debug!("Insert key 0x{} value: ( )", hex::encode(lookup_key));
        account.lookup.insert(lookup_key, vec![]);
    } else if account.lookup.get(&lookup_key).unwrap().len() == 2 {
        let mut timeslots = account.lookup.get(&lookup_key).unwrap().clone();
        timeslots.push(slot);
        let key = StateKeyType::Account(ctx_x.service_id, lookup_key.to_vec()).construct();
        log::debug!("Insert key 0x{} value: {:?}", hex::encode(key), timeslots);
        account.lookup.insert(key, timeslots);
    } else {
        reg[7] = HUH;
        log::debug!("Exit: HUH");
        return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
    };

    let (_items, _octets, threshold) = utils::common::get_footprint_and_threshold(&account);
    
    if account.balance < threshold {
        reg[7] = FULL;
        log::debug!("Exit: FULL");
        return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
    }

    reg[7] = OK;
    ctx_x.partial_state.service_accounts.insert(ctx_x.service_id, account);

    log::debug!("Exit: OK");
    return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
}

fn bless(mut gas: Gas, mut reg: Registers, ram: RamMemory, ctx: HostCallContext)

-> (ExitReason, Gas, Registers, RamMemory, HostCallContext)
{
    gas -= 10;

    if gas < 0 {
        log::error!("Out of gas!");
        return (ExitReason::OutOfGas, gas, reg, ram, ctx);
    }

    let manager = reg[7] as ServiceId;
    let assign = reg[8] as ServiceId;
    let validator = reg[9] as ServiceId;
    let start_address = reg[10] as RamAddress;
    let n_pairs = reg[11] as RamAddress;

    if !ram.is_readable(start_address, 12 * n_pairs) {
        log::error!("Panic: The RAM is not readable from address: {:?} num_bytes: {:?}", start_address, 12 * n_pairs);
        return (ExitReason::panic, gas, reg, ram, ctx);    
    }

    log::debug!("manager: {:?}, assign: {:?}, validator: {:?}, n_pairs: {:?}, start_address: {:?}", manager, assign, validator, n_pairs, start_address);

    let mut service_gas_pairs: HashMap<ServiceId, Gas> = HashMap::new();

    for i in 0..n_pairs {
        let pair = ram.read(start_address + 12 * i, 12);
        let service = decode::<ServiceId>(&pair, std::mem::size_of::<ServiceId>());
        let gas = decode::<Gas>(&pair[std::mem::size_of::<ServiceId>()..], std::mem::size_of::<Gas>());
        log::debug!("service: {service}, gas: {gas}");
        service_gas_pairs.insert(service, gas);
    }

    let (mut ctx_x, ctx_y) = ctx.to_acc_ctx();

    if !ctx_x.partial_state.service_accounts.contains_key(&manager)
    || !ctx_x.partial_state.service_accounts.contains_key(&assign)
    || !ctx_x.partial_state.service_accounts.contains_key(&validator) {

        reg[7] = WHO;
        log::debug!("Exit: WHO");
        return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
    }

    ctx_x.partial_state.privileges.bless = manager;
    ctx_x.partial_state.privileges.assign = assign;
    ctx_x.partial_state.privileges.designate = validator;

    log::debug!("Exit: OK");
    return (ExitReason::OutOfGas, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
}

fn designate(mut gas: Gas, mut reg: Registers, ram: RamMemory, ctx: HostCallContext)

-> (ExitReason, Gas, Registers, RamMemory, HostCallContext)
{
    gas -= 10;

    if gas < 0 {
        log::error!("Out of gas!");
        return (ExitReason::OutOfGas, gas, reg, ram, ctx);
    }

    let start_address = reg[7] as RamAddress;

    if !ram.is_readable(start_address, 336 * VALIDATORS_COUNT as RamAddress) {
        log::error!("Panic: The RAM is not readable from address: {:?} num_bytes: {:?}", start_address, 336 * VALIDATORS_COUNT);
        return (ExitReason::panic, gas, reg, ram, ctx);    
    }

    let mut validators: ValidatorsData = ValidatorsData::default();

    for i in 0..VALIDATORS_COUNT {
        let validators_data = ram.read(start_address + 336 * i as RamAddress, 336);
        validators.list[i].bandersnatch = validators_data[..32].try_into().unwrap();
        validators.list[i].ed25519 = validators_data[32..64].try_into().unwrap();
        validators.list[i].bls = validators_data[64..208].try_into().unwrap();
        validators.list[i].metadata = validators_data[208..].try_into().unwrap();
    }

    let (mut ctx_x, ctx_y) = ctx.to_acc_ctx();

    ctx_x.partial_state.next_validators = validators;
    reg[7] = OK;

    log::debug!("Exit: OK");
    return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
}

fn assign(mut gas: Gas, mut reg: Registers, ram: RamMemory, ctx: HostCallContext)

-> (ExitReason, Gas, Registers, RamMemory, HostCallContext)
{
    gas -= 10;

    if gas < 0 {
        log::error!("Out of gas!");
        return (ExitReason::OutOfGas, gas, reg, ram, ctx);
    }

    let core_index = reg[7] as CoreIndex;
    
    if core_index >= CORES_COUNT as CoreIndex {
        log::debug!("core_index {:?} >= CORES_COUNT {:?}", core_index, CORES_COUNT);
        reg[7] = CORE;
        log::debug!("Exit: CORE");
        return (ExitReason::Continue, gas, reg, ram, ctx);
    }

    let start_address = reg[8] as RamAddress;
    let (mut ctx_x, ctx_y) = ctx.to_acc_ctx();

    if !ram.is_readable(start_address, 32 * MAX_ITEMS_AUTHORIZATION_QUEUE as RamAddress) {
        log::error!("Panic: The RAM is not readable from address: {:?} num_bytes: {:?}", start_address, 32 * MAX_ITEMS_AUTHORIZATION_QUEUE);
        return (ExitReason::panic, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
    }

    for i in 0..MAX_ITEMS_AUTHORIZATION_QUEUE {
        ctx_x.partial_state.queues_auth.0[core_index as usize][i] = ram.read(start_address + 32 * i as u32, 32).try_into().unwrap();
    }

    reg[7] = OK;

    log::debug!("Exit: OK");
    return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
}

fn checkpoint(mut gas: Gas, mut reg: Registers, ram: RamMemory, ctx: HostCallContext)

-> (ExitReason, Gas, Registers, RamMemory, HostCallContext)
{
    gas -= 10;

    if gas < 0 {
        log::error!("Out of gas!");
        return (ExitReason::OutOfGas, gas, reg, ram, ctx);
    }

    let (ctx_x, _ctx_y) = ctx.to_acc_ctx();
    
    let ctx_y = ctx_x.clone();
    reg[7] = gas as RegSize;
    
    log::debug!("gas: {gas}");
    return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
}

fn forget(mut gas: Gas, mut reg: Registers, ram: RamMemory, ctx: HostCallContext, slot: TimeSlot)

-> (ExitReason, Gas, Registers, RamMemory, HostCallContext)
{
    gas -= 10;

    if gas < 0 {
        log::error!("Out of gas!");
        return (ExitReason::OutOfGas, gas, reg, ram, ctx);
    }

    let start_address = reg[7] as RamAddress;
    let length = reg[8] as RamAddress;

    if !ram.is_readable(start_address, 32) {
        log::error!("Panic: The RAM is not readable from address: {:?} num_bytes: {:?}", start_address, 32);
        return (ExitReason::panic, gas, reg, ram, ctx);
    }

    let hash = ram.read(start_address, 32).try_into().unwrap();
    let (mut ctx_x, ctx_y) = ctx.to_acc_ctx();
    let lookup_key = StateKeyType::Account(ctx_x.service_id, construct_lookup_key(&hash, length).to_vec()).construct();
    log::debug!("length: {length}, hash: 0x{}, lookup_key: 0x{}", hex::encode(hash), hex::encode(lookup_key));

    if !ctx_x.partial_state.service_accounts.contains_key(&ctx_x.service_id) {
        reg[7] = HUH;
        log::debug!("Exit: HUH");
        return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
    }

    let mut account = ctx_x.partial_state.service_accounts.get(&ctx_x.service_id).unwrap().clone();

    if let Some(mut timeslot) = account.lookup.get(&lookup_key).cloned() {
        log::debug!("slot: {slot}, timeslots: {:?}", timeslot);
        if timeslot.len() == 0 || (timeslot.len() == 2 && (timeslot[1] < slot.saturating_sub(MAX_TIMESLOTS_AFTER_UNREFEREND_PREIMAGE))) {
            account.lookup.remove(&lookup_key);
            account.preimages.remove(&StateKeyType::Account(ctx_x.service_id, construct_preimage_key(&hash).to_vec()).construct());
            log::debug!("remove lookup: 0x{}, remove preimage: 0x{}", hex::encode(lookup_key), hex::encode(hash));
        } else if timeslot.len() == 1 {
            log::debug!("Insert to lookup key 0x{} slot: {:?}", hex::encode(lookup_key), slot);
            timeslot.push(slot);
            account.lookup.insert(lookup_key, timeslot);
        } else if timeslot.len() == 3 && (timeslot[1] < slot.saturating_sub(MAX_TIMESLOTS_AFTER_UNREFEREND_PREIMAGE)) {
            log::debug!("Inserted to lookup key 0x{} timeslots: {:?}", hex::encode(lookup_key), timeslot);
            account.lookup.insert(lookup_key, vec![timeslot[2], slot]);
        } else {
            reg[7] = HUH;
            log::debug!("Exit: HUH");
            return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
        }
    } else {
        reg[7] = HUH;
        log::debug!("Exit: HUH");
        return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
    }
    
    reg[7] = OK;
    log::debug!("Exit: OK");
    ctx_x.partial_state.service_accounts.insert(ctx_x.service_id, account);

    return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
}



fn yield_(mut gas: Gas, mut reg: Registers, ram: RamMemory, ctx: HostCallContext)

-> (ExitReason, Gas, Registers, RamMemory, HostCallContext)
{
    gas -= 10;

    if gas < 0 {
        log::error!("Out of gas!");
        return (ExitReason::OutOfGas, gas, reg, ram, ctx);
    }

    let start_address = reg[7] as RamAddress;

    if !ram.is_readable(start_address, 32) {
        log::error!("Panic: The RAM is not readable from address: {start_address} num_bytes: 32");
        return (ExitReason::panic, gas, reg, ram, ctx);
    }

    let (mut ctx_x, ctx_y) = ctx.to_acc_ctx();

    ctx_x.y = Some(ram.read(start_address, 32).try_into().unwrap());
    reg[7] = OK;

    log::debug!("Exit: OK");
    return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
}

fn provide(mut gas: Gas, mut reg: Registers, ram: RamMemory, ctx: HostCallContext, service_id: ServiceId)

-> (ExitReason, Gas, Registers, RamMemory, HostCallContext)
{
    gas -= 10;

    if gas < 0 {
        log::error!("Out of gas!");
        return (ExitReason::OutOfGas, gas, reg, ram, ctx);
    }

    let start_address = reg[8] as RamAddress;
    let size = reg[9] as RamAddress;

    let id = if reg[7] == u64::MAX {
        service_id
    } else {
        reg[7] as ServiceId
    };

    if !ram.is_readable(start_address, size) {
        log::error!("Panic: The RAM is not readable from address: {start_address} num_bytes: {size}");
        return (ExitReason::panic, gas, reg, ram, ctx);
    }

    let (mut ctx_x, ctx_y) = ctx.to_acc_ctx();

    let account: Option<Account> = if ctx_x.partial_state.service_accounts.contains_key(&id) {
        ctx_x.partial_state.service_accounts.get(&id).cloned()
    } else {
        None
    };

    if account.is_none() {
        reg[7] = WHO;
        log::debug!("Exit: WHO");
        return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
    }

    let item = ram.read(start_address, size);
    let lookup_key = StateKeyType::Account(ctx_x.service_id, construct_lookup_key(&sp_core::blake2_256(&item), size).to_vec()).construct();
    log::debug!("lookup key: 0x{}", hex::encode(lookup_key));

    if account.unwrap().lookup.contains_key(&lookup_key) {
        reg[7] = HUH;
        log::debug!("lookup already contains the lookup key");
        log::debug!("Exit: HUH");
        return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
    }
    
    if ctx_x.preimages.contains(&(id, item.clone())) {
        log::debug!("preimages already contains id: {id} and item: {:?}", item);
        log::debug!("Exit: HUH");
        reg[7] = HUH;
        return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
    }

    ctx_x.preimages.push((id, item));
    reg[7] = OK;

    log::debug!("Exit: OK");
    return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
}