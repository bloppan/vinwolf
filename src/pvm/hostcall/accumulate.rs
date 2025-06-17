use log::debug;
use std::collections::HashMap;
use sp_core::blake2_256;

use crate::blockchain::state::time::get_current_block_slot;
use crate::pvm;
use crate::types::{
    Account, AccumulationContext, AccumulationOperand, AccumulationPartialState, CoreIndex, DeferredTransfer, ExitReason, Gas, HostCallFn, OpaqueHash, 
    RamAddress, RamMemory, RegSize, Registers, ServiceId, TimeSlot, ValidatorsData, WorkExecResult
};
use crate::constants::{
    CASH, CORE, CORES_COUNT, FULL, HUH, LOW, MAX_ITEMS_AUTHORIZATION_QUEUE, MAX_TIMESLOTS_AFTER_UNREFEREND_PREIMAGE, NONE, OK, TRANSFER_MEMO_SIZE, 
    VALIDATORS_COUNT, WHAT, WHO
};
use crate::blockchain::state::{services::decode_preimage, entropy, time};
use crate::pvm::hostcall::{hostcall_argument, HostCallContext};
use crate::utils::codec::{Encode, EncodeLen, EncodeSize, DecodeSize, BytesReader};
use crate::utils::codec::generic::{encode_unsigned, decode};
use super::general_fn::{write, info, read, lookup, log};

pub fn invoke_accumulation(
    partial_state: AccumulationPartialState,
    slot: &TimeSlot,
    service_id: &ServiceId,
    gas: Gas,
    operands: &[AccumulationOperand]
) -> (AccumulationPartialState, Vec<DeferredTransfer>, Option<OpaqueHash>, Gas, Vec<(ServiceId, Vec<u8>)>) {
    
    let preimage_code = if let Some(code) = partial_state
        .service_accounts
        .get(service_id)
        .and_then(|account| account.preimages.get(&account.code_hash).cloned())
    {
        code
    } else {
        return (partial_state.clone(), vec![], None, 0, vec![]);
    };
    println!("Wrangled results: {:x?}", operands);

    //let args = [slot.encode(), service_id.encode(), operands.encode_len()].concat();
    println!("slot: {:?}, service_id: {:?}", *slot, *service_id);
    let args = [encode_unsigned(*slot as usize), encode_unsigned(*service_id as usize), operands.encode_len()].concat();
    println!("args: {:x?}", args);
    let preimage_data = decode_preimage(&preimage_code).unwrap(); // TODO handle error

    let hostcall_arg_result = hostcall_argument(
                                &preimage_data.code, 
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

    //println!("Dispatch accumulate: n: {:?}, gas: {:?}, reg: {:?}", n, gas, reg);
    
    match n {
        HostCallFn::Gas         => pvm::hostcall::general_fn::gas(gas, reg, ram, ctx),
        HostCallFn::Bless       => bless(gas, reg, ram, ctx),
        HostCallFn::Assign      => assign(gas, reg, ram, ctx),
        HostCallFn::Designate   => designate(gas, reg, ram, ctx),
        HostCallFn::Checkpoint  => checkpoint(gas, reg, ram, ctx),
        HostCallFn::New         => new(gas, reg, ram, ctx),
        HostCallFn::Upgrade     => upgrade(gas, reg, ram, ctx),
        HostCallFn::Transfer    => transfer(gas, reg, ram, ctx),
        HostCallFn::Eject       => eject(gas, reg, ram, ctx, get_current_block_slot()),
        HostCallFn::Query       => query(gas, reg, ram, ctx),
        HostCallFn::Solicit     => solicit(gas, reg, ram, ctx, get_current_block_slot()),
        HostCallFn::Forget      => forget(gas, reg, ram, ctx, get_current_block_slot()),
        HostCallFn::Yield       => yield_(gas, reg, ram, ctx),
        HostCallFn::Provide     => {
                                    let (ctx_x, _ctx_y) = ctx.clone().to_acc_ctx();
                                    provide(gas, reg, ram, ctx, ctx_x.service_id)
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
        println!("WorkExecResult::Error: {:?}", output);
        return (ctx_y.partial_state, ctx_y.deferred_transfers, ctx_y.y, gas, ctx_y.preimages);
    }

    if let WorkExecResult::Ok(payload) = output {
        if payload.len() == std::mem::size_of::<OpaqueHash>() {
            println!("WorkExecResult::Ok: {:?}", payload);
            return (ctx_x.partial_state, ctx_x.deferred_transfers, Some(payload.try_into().unwrap()), gas, ctx_x.preimages);
        }
    }

    println!("Service HASH: {:x?}", ctx_x.y);
    return (ctx_x.partial_state, ctx_x.deferred_transfers, ctx_x.y, gas, ctx_x.preimages);
}

#[allow(non_snake_case)]
fn I(partial_state: &AccumulationPartialState, service_id: &ServiceId) -> AccumulationContext {

    //let encoded = [service_id.encode(), entropy::get_recent_entropy().encode(), time::get_current_block_slot().encode()].concat();
    let encoded = [encode_unsigned(*service_id as usize), entropy::get_recent_entropy().encode(), encode_unsigned(time::get_current_block_slot() as usize)].concat();
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
        return (ExitReason::OutOfGas, gas, reg, ram, ctx);
    }

    let dest = reg[7];
    let amount = reg[8];
    let limit = reg[9];
    let start_address = reg[10];
   
    if !ram.is_readable(start_address as RamAddress, TRANSFER_MEMO_SIZE as RamAddress) {
        println!("TRANSFER panic");
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
            println!("TRANSFER LOW");
            reg[7] = LOW;
            return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
        }
        if balance < account.get_footprint_and_threshold().2 {
            println!("TRANSFER CASH");
            reg[7] = CASH;
            return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
        }
        println!("TRANSFER OK");
        reg[7] = OK;
        ctx_x.deferred_transfers.push(transfer);
        ctx_x.partial_state.service_accounts.get_mut(&ctx_x.service_id).unwrap().balance = balance;
        return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
    } 
    
    println!("TRANSFER WHO");
    reg[7] = WHO;
    return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
}

fn eject(mut gas: Gas, mut reg: Registers, ram: RamMemory, ctx: HostCallContext, slot: TimeSlot)

-> (ExitReason, Gas, Registers, RamMemory, HostCallContext)
{
    gas -= 10;

    if gas < 0 {
        return (ExitReason::OutOfGas, gas, reg, ram, ctx);
    }

    let service_id = reg[7] as ServiceId;
    let start_address = reg[8] as RamAddress;

    if !ram.is_readable(start_address, 32) {
        return (ExitReason::panic, gas, reg, ram, ctx);
    }

    let hash: OpaqueHash = ram.read(start_address, 32).try_into().unwrap();

    let (mut ctx_x, ctx_y) = ctx.to_acc_ctx();
    
    if service_id != ctx_x.service_id && ctx_x.partial_state.service_accounts.contains_key(&service_id) {

        let d_account = ctx_x.partial_state.service_accounts.get(&service_id).unwrap().clone();
        let length = (std::cmp::max(81, d_account.get_footprint_and_threshold().1) - 81) as u32;

        let mut s_account = ctx_x.partial_state.service_accounts.get(&ctx_x.service_id).unwrap().clone();
        s_account.balance += d_account.balance;

        let xs_encoded: OpaqueHash = ctx_x.service_id.encode_size(32).try_into().unwrap();
        println!("xs_encoded: {:x?}", xs_encoded);

        if d_account.code_hash != xs_encoded {
            println!("WHO: code_hash: {:x?}",d_account.code_hash);
            reg[7] = WHO;
            return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
        }

        if d_account.get_footprint_and_threshold().0 != 2 || !d_account.lookup.contains_key(&(hash, length)) {
            println!("HUH");
            reg[7] = HUH;
            return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
        }

        let timeslots = d_account.lookup.get(&(hash, length)).unwrap().clone();

        if timeslots.len() == 2 && (slot - timeslots[1] < MAX_TIMESLOTS_AFTER_UNREFEREND_PREIMAGE) {

            ctx_x.partial_state.service_accounts.remove(&service_id);
            ctx_x.partial_state.service_accounts.insert(ctx_x.service_id, s_account);
            reg[7] = OK;
            println!("OK");
            return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
        }

        println!("HUH");
        reg[7] = HUH;
        return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
    }

    println!("WHO");
    reg[7] = WHO;
    return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
}

fn query(mut gas: Gas, mut reg: Registers, ram: RamMemory, ctx: HostCallContext)

-> (ExitReason, Gas, Registers, RamMemory, HostCallContext)
{
    gas -= 10;

    if gas < 0 {
        return (ExitReason::OutOfGas, gas, reg, ram, ctx);
    }

    let start_address = reg[7] as RamAddress;
    let length = reg[8] as u32;

    if !ram.is_readable(start_address, 32) {
        return (ExitReason::panic, gas, reg, ram, ctx);
    }

    let hash: OpaqueHash = ram.read(start_address, 32).try_into().unwrap();
    println!("hash: {:x?}", hash);
    println!("length: {:?}", hash);
    let (ctx_x, ctx_y) = ctx.to_acc_ctx();

    if !ctx_x.partial_state.service_accounts.get(&ctx_x.service_id).unwrap().lookup.contains_key(&(hash, length)) {
        println!("NONE");
        reg[7] = NONE;
        return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
    }

    let timeslots = ctx_x.partial_state.service_accounts.get(&ctx_x.service_id).unwrap().lookup.get(&(hash, length)).unwrap().clone();
    let timeslots_len = timeslots.len();
    println!("timestlots_len = {timeslots_len}");
    println!("timeslots: {:?}", timeslots);

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
    println!("reg_7: {:?}", reg[7]);
    println!("reg_8: {:?}", reg[8]);
    println!("OK");
    return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
}

fn new(mut gas: Gas, mut reg: Registers, ram: RamMemory, ctx: HostCallContext)

-> (ExitReason, Gas, Registers, RamMemory, HostCallContext)
{
    gas -= 10;

    if gas < 0 {
        return (ExitReason::OutOfGas, gas, reg, ram, ctx);
    }

    let start_address = reg[7] as RamAddress;
    let limit = reg[8];
    let new_account_gas = reg[9];
    let new_account_min_gas = reg[10];

    let HostCallContext::Accumulate(mut ctx_x, ctx_y) = ctx else {
        unreachable!("Dispatch accumulate: Invalid context");
    };

    if ram.is_readable(start_address, 32) && limit < (1 << 32) {
        let c = ram.read(start_address, 32);
        let mut new_account = Account::default();
        new_account.code_hash.copy_from_slice(&c);
        new_account.lookup.insert((new_account.code_hash, limit as u32), vec![]);
        new_account.acc_min_gas = new_account_gas as Gas;
        new_account.xfer_min_gas = new_account_min_gas as Gas;
        let threshold_account = new_account.get_footprint_and_threshold().2;
        new_account.balance = threshold_account;

        let mut service_account = ctx_x.partial_state.service_accounts.get(&ctx_x.service_id).unwrap().clone(); // TODO handle error
        service_account.balance = service_account.balance.saturating_sub(threshold_account);

        if service_account.balance < ctx_x.partial_state.service_accounts.get(&ctx_x.service_id).unwrap().get_footprint_and_threshold().2 {
            reg[7] = CASH;
            return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
        }

        let i = (1u64 << 8) + (ctx_x.index as u64 - (1u64 << 8) + 42) % ((1u64 << 32) - (1u64 << 9));
        
        reg[7] = ctx_x.index as RegSize;
        println!("berservice: {:?} | new account: {:?}", ctx_x.index, new_account);
        ctx_x.partial_state.service_accounts.insert(ctx_x.index, new_account);
        ctx_x.partial_state.service_accounts.insert(ctx_x.service_id, service_account);
        ctx_x.index = check(&ctx_x.partial_state, &(i as ServiceId));

        return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
    }
        
    return (ExitReason::panic, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
}

fn upgrade(mut gas: Gas, mut reg: Registers, ram: RamMemory, ctx: HostCallContext)

-> (ExitReason, Gas, Registers, RamMemory, HostCallContext)
{
    gas -= 10;

    if gas < 0 {
        return (ExitReason::OutOfGas, gas, reg, ram, ctx);
    }

    let start_address = reg[7] as RamAddress;
    let new_gas = reg[8] as Gas;
    let new_min_gas = reg[9] as Gas;

    if !ram.is_readable(start_address, 32) {
        return (ExitReason::panic, gas, reg, ram, ctx);
    }

    let code_hash: OpaqueHash = ram.read(start_address, 32).try_into().unwrap();

    let (mut ctx_x, ctx_y) = ctx.to_acc_ctx();

    ctx_x.partial_state.service_accounts.get_mut(&ctx_x.service_id).unwrap().code_hash = code_hash;
    ctx_x.partial_state.service_accounts.get_mut(&ctx_x.service_id).unwrap().acc_min_gas = new_gas;
    ctx_x.partial_state.service_accounts.get_mut(&ctx_x.service_id).unwrap().xfer_min_gas = new_min_gas;

    reg[7] = OK;

    return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
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

    if !ram.is_readable(start_address, 32){
        return (ExitReason::panic, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
    }

    let hash: OpaqueHash = ram.read(start_address,  32).try_into().unwrap();

    let mut account = ctx_x.partial_state.service_accounts.get(&ctx_x.service_id).unwrap().clone();

    if !account.lookup.contains_key(&(hash, preimage_size)) {
        account.lookup.insert((hash, preimage_size), vec![]);
    } else if account.lookup.get(&(hash, preimage_size)).unwrap().len() == 2 {
        let mut timeslots = account.lookup.get(&(hash, preimage_size)).unwrap().clone();
        timeslots.push(slot);
        account.lookup.insert((hash, preimage_size), timeslots);
    } else {
        reg[7] = HUH;
        return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
    };


    /*if account.lookup.contains_key(&(hash, preimage_size)) {
        
        let mut timeslots = account.lookup.get(&(hash, preimage_size)).unwrap().clone();

        if timeslots.len() != 2 {
            println!("HUH");
            reg[7] = HUH;
            return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
        }

        timeslots.push(slot);

        account.lookup.insert((hash, preimage_size), timeslots);
    } else {
        account.lookup.insert((hash, preimage_size), vec![]);
    }*/

    if account.balance < account.get_footprint_and_threshold().2 {
        println!("FULL");
        reg[7] = FULL;
        return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
    }

    println!("hash: {:x?}", hash);
    println!("timeslots: {:?}", account.lookup.get(&(hash, preimage_size)).unwrap());
    println!("size: {:?}", preimage_size);
    println!("OK");
    reg[7] = OK;
    ctx_x.partial_state.service_accounts.insert(ctx_x.service_id, account);

    return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
}

fn bless(mut gas: Gas, mut reg: Registers, ram: RamMemory, ctx: HostCallContext)

-> (ExitReason, Gas, Registers, RamMemory, HostCallContext)
{
    gas -= 10;

    if gas < 0 {
        return (ExitReason::OutOfGas, gas, reg, ram, ctx);
    }

    let manager = reg[7] as ServiceId;
    let assign = reg[8] as ServiceId;
    let validator = reg[9] as ServiceId;
    let start_address = reg[10] as RamAddress;
    let n_pairs = reg[11] as RamAddress;

    if !ram.is_readable(start_address, 12 * n_pairs) {
        return (ExitReason::panic, gas, reg, ram, ctx);    
    }

    let mut service_gas_pairs: HashMap<ServiceId, Gas> = HashMap::new();

    for i in 0..n_pairs {
        let pair = ram.read(start_address + 12 * i, 12);
        let service = decode::<ServiceId>(&pair, std::mem::size_of::<ServiceId>());
        let gas = decode::<Gas>(&pair[std::mem::size_of::<ServiceId>()..], std::mem::size_of::<Gas>());
        println!("Service: {service}, Gas: {gas}");
        service_gas_pairs.insert(service, gas);
    }

    let (mut ctx_x, ctx_y) = ctx.to_acc_ctx();

    if !ctx_x.partial_state.service_accounts.contains_key(&manager)
    || !ctx_x.partial_state.service_accounts.contains_key(&assign)
    || !ctx_x.partial_state.service_accounts.contains_key(&validator) {
        println!("WHO");
        reg[7] = WHO;
        return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
    }

    println!("OK");
    ctx_x.partial_state.privileges.bless = manager;
    ctx_x.partial_state.privileges.assign = assign;
    ctx_x.partial_state.privileges.designate = validator;

    return (ExitReason::OutOfGas, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
}

fn designate(mut gas: Gas, mut reg: Registers, ram: RamMemory, ctx: HostCallContext)

-> (ExitReason, Gas, Registers, RamMemory, HostCallContext)
{
    gas -= 10;

    if gas < 0 {
        return (ExitReason::OutOfGas, gas, reg, ram, ctx);
    }

    let start_address = reg[7] as RamAddress;

    if !ram.is_readable(start_address, 336 * VALIDATORS_COUNT as RamAddress) {
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

    if !ram.is_readable(start_address, 32 * MAX_ITEMS_AUTHORIZATION_QUEUE as RamAddress) {
        return (ExitReason::panic, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
    }

    println!("core_index = {core_index}");
    for i in 0..MAX_ITEMS_AUTHORIZATION_QUEUE {
        ctx_x.partial_state.queues_auth.0[core_index as usize][i] = ram.read(start_address + 32 * i as u32, 32).try_into().unwrap();
    }
    println!("OK");
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

    let (ctx_x, _ctx_y) = ctx.to_acc_ctx();
    
    let ctx_y = ctx_x.clone();
    reg[7] = gas as RegSize;
    
    return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
}

fn forget(mut gas: Gas, mut reg: Registers, ram: RamMemory, ctx: HostCallContext, slot: TimeSlot)

-> (ExitReason, Gas, Registers, RamMemory, HostCallContext)
{
    gas -= 10;

    if gas < 0 {
        return (ExitReason::OutOfGas, gas, reg, ram, ctx);
    }

    let start_address = reg[7] as RamAddress;
    let length = reg[8] as RamAddress;

    if !ram.is_readable(start_address, 32) {
        return (ExitReason::panic, gas, reg, ram, ctx);
    }

    let hash = ram.read(start_address, 32).try_into().unwrap();
    let (mut ctx_x, ctx_y) = ctx.to_acc_ctx();

    if !ctx_x.partial_state.service_accounts.contains_key(&ctx_x.service_id) {
        println!("HUH");
        reg[7] = HUH;
        return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
    }

    let mut account = ctx_x.partial_state.service_accounts.get(&ctx_x.service_id).unwrap().clone();

    if let Some(mut timeslot) = account.lookup.get(&(hash, length)).cloned() {
        println!("slot: {slot}");
        println!("timeslot len: {:?}, timeslot: {:?}", timeslot.len(), timeslot);
        println!("account timeslot before: {:?}", account.lookup.get(&(hash, length)).unwrap().clone());
        if timeslot.len() == 0 || (timeslot.len() == 2 && (slot - timeslot[1] < MAX_TIMESLOTS_AFTER_UNREFEREND_PREIMAGE)) {
            account.lookup.remove(&(hash, length));
            account.preimages.remove(&hash);
        } else if timeslot.len() == 1 {
            timeslot.push(slot);
            account.lookup.insert((hash, length), timeslot);
        } else if timeslot.len() == 3 && (slot - timeslot[1] < MAX_TIMESLOTS_AFTER_UNREFEREND_PREIMAGE) {
            account.lookup.insert((hash, length), vec![timeslot[2], slot]);
        } else {
            println!("HUH");
            reg[7] = HUH;
            return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
        }
    } else {
        println!("HUH");
        reg[7] = HUH;
        return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
    }
    println!("account timeslot after: {:?}", account.lookup.get(&(hash, length)).unwrap().clone());
    println!("OK");
    reg[7] = OK;
    ctx_x.partial_state.service_accounts.insert(ctx_x.service_id, account);

    return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
}



fn yield_(mut gas: Gas, mut reg: Registers, ram: RamMemory, ctx: HostCallContext)

-> (ExitReason, Gas, Registers, RamMemory, HostCallContext)
{
    gas -= 10;

    if gas < 0 {
        return (ExitReason::OutOfGas, gas, reg, ram, ctx);
    }

    let start_address = reg[7] as RamAddress;

    if !ram.is_readable(start_address, 32) {
        return (ExitReason::panic, gas, reg, ram, ctx);
    }

    let (mut ctx_x, ctx_y) = ctx.to_acc_ctx();

    ctx_x.y = Some(ram.read(start_address, 32).try_into().unwrap());
    reg[7] = OK;

    return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
}

fn provide(mut gas: Gas, mut reg: Registers, ram: RamMemory, ctx: HostCallContext, service_id: ServiceId)

-> (ExitReason, Gas, Registers, RamMemory, HostCallContext)
{
    gas -= 10;

    if gas < 0 {
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
        return (ExitReason::panic, gas, reg, ram, ctx);
    }

    let (mut ctx_x, ctx_y) = ctx.to_acc_ctx();

    let account: Option<Account> = if ctx_x.partial_state.service_accounts.contains_key(&id) {
        ctx_x.partial_state.service_accounts.get(&id).cloned()
    } else {
        None
    };

    if account.is_none() {
        debug!("provide: WHO");
        reg[7] = WHO;
        return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
    }

    let item = ram.read(start_address, size);

    if account.unwrap().lookup.contains_key(&(sp_core::blake2_256(&item), size)) {
        debug!("provide: HUH");
        reg[7] = HUH;
        return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
    }
    
    if ctx_x.preimages.contains(&(id, item.clone())) {
        debug!("provide: HUH. preimages already contains id and item");
        reg[7] = HUH;
        return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
    }

    ctx_x.preimages.push((id, item));
    reg[7] = OK;
    debug!("provide: OK");

    return (ExitReason::Continue, gas, reg, ram, HostCallContext::Accumulate(ctx_x, ctx_y));
}