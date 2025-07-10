use crate::types::{
    Account, AccumulationOperand, DataSegments, DeferredTransfer, ExitReason, Gas, OpaqueHash, RamAddress, RamMemory, RegSize, 
    Registers, ServiceAccounts, ServiceId, StateKeyType, WorkPackage, WorkExecResult, WorkItem};
use crate::constants::{
    CORES_COUNT, EPOCH_LENGTH, FULL, NONE, OK, VALIDATORS_COUNT, MIN_BALANCE_PER_ITEM, MIN_BALANCE_PER_OCTET, MIN_BALANCE, MAX_TIMESLOTS_AFTER_UNREFEREND_PREIMAGE,
    WORK_REPORT_GAS_LIMIT, WORK_PACKAGE_GAS_LIMIT, WORK_PACKAGE_REFINE_GAS, TOTAL_GAS_ALLOCATED, RECENT_HISTORY_SIZE, MAX_WORK_ITEMS, MAX_DEPENDENCY_ITEMS, MAX_AGE_LOOKUP_ANCHOR,
    MAX_ITEMS_AUTHORIZATION_POOL, SLOT_PERIOD, MAX_ITEMS_AUTHORIZATION_QUEUE, ROTATION_PERIOD, MAX_ENTRIES_IN_ACC_QUEUE, MAX_EXTRINSICS_IN_WP, REPORTED_WORK_REPLACEMENT_PERIOD,
    MAX_IS_AUTHORIZED_SIZE, MAX_ENCODED_WORK_PACKAGE_SIZE, MAX_SERVICE_CODE_SIZE, PIECE_SIZE, SEGMENT_SIZE, MAX_WORK_PACKAGE_IMPORTS, SEGMENT_PIECES, MAX_WORK_REPORT_TOTAL_SIZE,
    TRANSFER_MEMO_SIZE, MAX_WORK_PACKAGE_EXPORTS, TICKET_SUBMISSION_ENDS, MAX_TICKETS_PER_EXTRINSIC, TICKET_ENTRIES_PER_VALIDATOR
};
use crate::utils::codec::{Encode, EncodeSize, EncodeLen};
use crate::utils::codec::generic::encode_unsigned;
use crate::utils::serialization::{construct_storage_key, StateKeyTrait};

use super::HostCallContext;

pub fn gas(mut gas: Gas, mut reg: Registers, ram: RamMemory, ctx: HostCallContext)

-> (ExitReason, Gas, Registers, RamMemory, HostCallContext)
{
    gas -= 10;

    if gas < 0 {
        return (ExitReason::OutOfGas, gas, reg, ram, ctx);
    } 
 
    reg[7] = gas as RegSize;

    return (ExitReason::Continue, gas, reg, ram, ctx);
}

pub fn fetch(mut gas: Gas, 
             mut reg: Registers, 
             mut ram: RamMemory, 
             pkg: Option<WorkPackage>,
             n: Option<OpaqueHash>,
             result: Option<WorkExecResult>,
             segments: Option<Vec<DataSegments>>,
             work_items: Option<Vec<WorkItem>>,
             operands: Option<Vec<AccumulationOperand>>,
             transfers: Option<Vec<DeferredTransfer>>,
             ctx: HostCallContext) 

-> (ExitReason, Gas, Registers, RamMemory, HostCallContext) 
{
    gas -= 10;

    if gas < 0 {
        return (ExitReason::OutOfGas, gas, reg, ram, ctx);
    } 
    //println!("bernux reg_10: {:?}", reg[10]);
    /*println!("BEFORE");
    println!();
    for i in 0..(4096 / 32) {
        print!("{:08}:\t", (i * 32) + 4096 * 50);
        for j in 0..32 {
            let index = i * 32 + j;
            print!("{:02x?} ", ram.pages[50].as_ref().unwrap().data[index as usize]);
        }
        println!();
    }
    println!();*/

    let value: Option<_> = if reg[10] == 0 {
        Some([
            MIN_BALANCE_PER_ITEM.encode_size(8), 
            MIN_BALANCE_PER_OCTET.encode_size(8), 
            MIN_BALANCE.encode_size(8), 
            CORES_COUNT.encode_size(2),
            MAX_TIMESLOTS_AFTER_UNREFEREND_PREIMAGE.encode_size(4), 
            EPOCH_LENGTH.encode_size(4), 
            WORK_REPORT_GAS_LIMIT.encode_size(8),
            WORK_PACKAGE_GAS_LIMIT.encode_size(8), 
            WORK_PACKAGE_REFINE_GAS.encode_size(8), 
            TOTAL_GAS_ALLOCATED.encode_size(8), 
            RECENT_HISTORY_SIZE.encode_size(2),
            MAX_WORK_ITEMS.encode_size(2), 
            MAX_DEPENDENCY_ITEMS.encode_size(2), 
            MAX_TICKETS_PER_EXTRINSIC.encode_size(2), 
            MAX_AGE_LOOKUP_ANCHOR.encode_size(4), 
            TICKET_ENTRIES_PER_VALIDATOR.encode_size(2), 
            MAX_ITEMS_AUTHORIZATION_POOL.encode_size(2), 
            SLOT_PERIOD.encode_size(2), 
            MAX_ITEMS_AUTHORIZATION_QUEUE.encode_size(2), 
            ROTATION_PERIOD.encode_size(2),
            MAX_EXTRINSICS_IN_WP.encode_size(2), 
            REPORTED_WORK_REPLACEMENT_PERIOD.encode_size(2), 
            VALIDATORS_COUNT.encode_size(2),
            MAX_IS_AUTHORIZED_SIZE.encode_size(4),
            MAX_ENCODED_WORK_PACKAGE_SIZE.encode_size(4),
            MAX_SERVICE_CODE_SIZE.encode_size(4), 
            PIECE_SIZE.encode_size(4), 
            MAX_WORK_PACKAGE_IMPORTS.encode_size(4), 
            SEGMENT_PIECES.encode_size(4), 
            MAX_WORK_REPORT_TOTAL_SIZE.encode_size(4), 
            TRANSFER_MEMO_SIZE.encode_size(4),
            MAX_WORK_PACKAGE_EXPORTS.encode_size(4), 
            TICKET_SUBMISSION_ENDS.encode_size(4)
        ].concat())
    } else if n.is_some() && reg[10] == 1 {
        Some(n.unwrap().encode())
    } else if operands.is_some() && reg[10] == 14 {
        Some(operands.unwrap().encode_len())
    } else if operands.is_some() && reg[10] == 15 && (reg[11] as usize) < operands.as_ref().unwrap().len() {
        Some(operands.as_ref().unwrap()[reg[11] as usize].encode())
    } else if transfers.is_some() && reg[10] == 16 {
        Some(transfers.unwrap().encode_len())
    } else if transfers.is_some() && reg[10] == 17 && (reg[11] as usize) < transfers.as_ref().unwrap().len() {
        Some(transfers.as_ref().unwrap()[reg[11] as usize].encode())
    } else {
        None
    };

    let value_len = if value.is_some() {
        value.as_ref().unwrap().len()
    } else {
        0
    };

    let start_address = reg[7] as RamAddress;
    let f = std::cmp::min(reg[8] as usize, value_len);
    let l = std::cmp::min(reg[9] as usize, value_len - f);

    /*println!("start_address: {:?}", start_address);
    println!("f: {:?} l: {:?}", f, l);
    println!("value_len: {value_len}");
    println!("value: {:x?}", value);*/

    if !ram.is_writable(start_address, l as RamAddress) {
        //println!("fetch: panic!!!");
        return (ExitReason::panic, gas, reg, ram, ctx);
    }

    if value.is_none() {
        //println!("fetch: NONE");
        reg[7] = NONE;
        return (ExitReason::Continue, gas, reg, ram, ctx);
    }

    reg[7] = value_len as RegSize;
    ram.write(start_address, value.unwrap()[f as usize..(f + l) as usize].to_vec());
    
    println!();
    /*println!("AFTER");
    for i in 0..(4096 / 32) {
        print!("{:08}:\t", (i * 32) as u64 + (4096 as u64 * 1044447 as u64) as u64);
        for j in 0..32 {
            let index = i * 32 + j;
            print!("{:02x?} ", ram.pages[1044447].as_ref().unwrap().data[index as usize]);
        }
        println!();
    }
    println!();*/

    //println!("fetch: OK");
    return (ExitReason::Continue, gas, reg, ram, ctx);
}


pub fn lookup(mut gas: Gas, mut reg: Registers, mut ram: RamMemory, account: Account, service_id: ServiceId, services: ServiceAccounts) 

-> (ExitReason, Gas, Registers, RamMemory, Account)
{
    gas -= 10;

    if gas < 0 {
        return (ExitReason::OutOfGas, gas, reg, ram, account);
    }  

    let a_account: Option<Account> = if reg[7] as ServiceId == service_id || reg[7] == u64::MAX {
        Some(account.clone())
    } else if services.contains_key(&(reg[7] as ServiceId)) {
        services.get(&(reg[7] as ServiceId)).cloned()
    } else {
        None    
    };

    let read_start_address = reg[8] as RamAddress;
    let write_start_address = reg[9] as RamAddress;

    if !ram.is_readable(read_start_address, 32) {
        return (ExitReason::panic, gas, reg, ram, account);
    }

    let hash: OpaqueHash = ram.read(read_start_address, 32).try_into().unwrap();

    let preimage_blob: Option<Vec<u8>> = if a_account.is_none() {
        None
    } else if !a_account.as_ref().unwrap().preimages.contains_key(&hash) {
        None
    } else {
        a_account.unwrap().preimages.get(&hash).cloned()
    };

    let preimage_len = preimage_blob.as_ref().map(|v| v.len()).unwrap_or(0) as RegSize;
    
    let f = std::cmp::min(reg[10],  preimage_len);
    let l = std::cmp::min(reg[11], preimage_len - f);

    if !ram.is_writable(write_start_address, 32) {
        return (ExitReason::panic, gas, reg, ram, account);
    }

    if preimage_blob.is_none() {
        //println!("NONE");
        reg[7] = NONE;
        return (ExitReason::Continue, gas, reg, ram, account);
    }

    reg[7] = preimage_len;

    if let Some(blob) = preimage_blob {
        ram.write(write_start_address, blob[f as usize..(f + l) as usize].to_vec());
    }
    //println!("OK");
    return (ExitReason::Continue, gas, reg, ram, account);
}

pub fn read(mut gas: Gas, mut reg: Registers, mut ram: RamMemory, account: Account, service_id: ServiceId, services: ServiceAccounts) 

-> (ExitReason, Gas, Registers, RamMemory, Account)
{
    gas -= 10;

    if gas < 0 {
        return (ExitReason::OutOfGas, gas, reg, ram, account);
    }   

    let id = if reg[7] == u64::MAX {
        service_id
    } else {
        reg[7] as ServiceId
    };
    //println!("READ service id: {}", id);
    let target_account: Option<Account> = if service_id == id {
        Some(account.clone())
    } else if services.contains_key(&id) {
        services.get(&id).cloned()
    } else {
        None
    };
    //println!("target_account: {:?}", target_account.is_some());
    let start_read_address = reg[8] as RamAddress;
    let bytes_to_read = reg[9] as RamAddress;
    let start_write_address = reg[10] as RamAddress;

    if !ram.is_readable(start_read_address, bytes_to_read) {
        //println!("READ PANIC");
        return (ExitReason::panic, gas, reg, ram, account);
    }

    let key= sp_core::blake2_256(&[id.encode(), ram.read(start_read_address, bytes_to_read).encode()].concat())[..31].try_into().unwrap();
    //println!("raw key: {:x?}", key);
    let key = StateKeyType::Account(id, construct_storage_key(&key).to_vec()).construct();
    //println!("service: {:?} storage key: {:x?}", id, key);
    
    /*for item in target_account.as_ref().unwrap().storage.iter() {
        println!("find key: {:x?}", item.0);
    }*/
    
    let value: Vec<u8> = if target_account.is_some() && target_account.as_ref().unwrap().storage.contains_key(&key) {
        //println!("key found: {:x?}", key);
        target_account.unwrap().storage.get(&key).unwrap().clone()
    } else {
        reg[7] = NONE;
        /*println!("READ NONE");
        println!("key: {:x?} not found", key);
        println!("service_id: {} storage: {:x?}", id, target_account.unwrap().storage);*/
        return (ExitReason::Continue, gas, reg, ram, account);
    };
    //println!("value: {:x?}", value);
    // TODO revisar el orden de los return, no estoy seguro de si estan
    let f = std::cmp::min(reg[11], value.len() as RegSize); 
    let l = std::cmp::min(reg[12], value.len() as RegSize - f);
    /*println!("start address: {start_write_address}");
    println!("f: {f}, l: {l}");*/
    if !ram.is_writable(start_write_address, l as RamAddress) {
        //println!("READ PANIC");
        return (ExitReason::panic, gas, reg, ram, account);
    }
    reg[7] = value.len() as RegSize;
    ram.write(start_write_address, value[f as usize..(f + l) as usize].to_vec());
    /*println!("READ OK. l: {l}, f: {f}");
    println!("key: {:x?}, value: {:x?}", key, value[f as usize..(f + l) as usize].to_vec());*/
    return (ExitReason::Continue, gas, reg, ram, account);
}

pub fn write(mut gas: Gas, mut reg: Registers, ram: RamMemory, account: Account, service_id: ServiceId) 

-> (ExitReason, Gas, Registers, RamMemory, Account)
{
    gas -= 10;

    if gas < 0 {
        return (ExitReason::OutOfGas, gas, reg, ram, account);
    }   

    let key_start_address = reg[7];
    let key_size = reg[8];
    let value_start_address = reg[9];
    let value_size = reg[10];

    if !ram.is_readable(key_start_address as RamAddress, key_size as RamAddress) {
        //println!("panic: not readable");
        return (ExitReason::panic, gas, reg, ram, account);
    }
    
    //println!("service_id: {}", service_id);

    let key = sp_core::blake2_256(&[service_id.encode(), ram.read(key_start_address as RamAddress, key_size as RamAddress)].concat())[..31].try_into().unwrap();
    //println!("raw key: {:x?}", key);
    let key = StateKeyType::Account(service_id, construct_storage_key(&key).to_vec()).construct();
    //println!("service: {:?} storage key: {:x?}", service_id, key);

    let mut s_account = account.clone();

    let modified_account = if value_size == 0 {
        //println!("remove key: {:x?}", key);
        s_account.storage.remove(&key);
        s_account
    } else if ram.is_readable(value_start_address as RamAddress, value_size as RamAddress) {
        let storage_data = ram.read(value_start_address as RamAddress, value_size as RamAddress);
        //println!("insert key: {:x?} | value = {:x?}", key, storage_data);
        s_account.storage.insert(key, storage_data);
        s_account
    } else {
        //println!("WRITE panic");
        return (ExitReason::panic, gas, reg, ram, account);
    };

    let l: RegSize = if let Some(storage_data) = account.storage.get(&key) {
        //println!("WRITE OK");
        storage_data.len() as RegSize
    } else {
        //println!("WRITE NONE");
        NONE as RegSize
    };

    let threshold = modified_account.get_footprint_and_threshold().2;

    if threshold > modified_account.balance {
        //println!("WRITE FULL");
        reg[7] = FULL as RegSize;
        return (ExitReason::Continue, gas, reg, ram, account);
    }
    reg[7] = l;
    return (ExitReason::Continue, gas, reg, ram, modified_account);
    
}

pub fn info(mut gas: Gas, mut reg: Registers, mut ram: RamMemory, service_id: ServiceId, accounts: ServiceAccounts)

-> (ExitReason, Gas, Registers, RamMemory, Account) {

    gas -= 10;

    if gas < 0 {
        return (ExitReason::OutOfGas, gas, reg, ram, Account::default());
    }   

    let account: Option<Account> = if reg[7] == u64::MAX {
        if let Some(account) = accounts.get(&service_id).cloned() {
            Some(account)
        } else {
            None
        }
    } else {
        if let Some(account) = accounts.get(&(reg[7] as ServiceId)).cloned() {
            Some(account)
        } else {
            None
        }
    };

    let metadata = if let Some(account) = account.as_ref() {

        [account.code_hash.encode(), 
         encode_unsigned(account.balance as usize),
         encode_unsigned(account.get_footprint_and_threshold().2 as usize),
         encode_unsigned(account.acc_min_gas as usize),
         encode_unsigned(account.xfer_min_gas as usize),
         encode_unsigned(account.get_footprint_and_threshold().1 as usize),
         encode_unsigned(account.get_footprint_and_threshold().0 as usize),
         ].concat()
    } else {
        //println!("Info: NONE");
        reg[7] = NONE;
        return (ExitReason::Continue, gas, reg, ram, Account::default());
    };

    let start_address = reg[8] as RamAddress;

    if !ram.is_writable(start_address, metadata.len() as RamAddress) {
        //println!("Info: Panic");
        return (ExitReason::panic, gas, reg, ram, Account::default());
    }

    /*println!("code_hash: {:x?}", account.as_ref().unwrap().code_hash);
    println!("balance: {:?}", account.as_ref().unwrap().balance);
    println!("balance footprint: {:?}", account.as_ref().unwrap().get_footprint_and_threshold().2);
    println!("acc gas: {:?}", account.as_ref().unwrap().acc_min_gas);
    println!("xfer gas: {:?}", account.as_ref().unwrap().xfer_min_gas);
    println!("items: {:?}", account.as_ref().unwrap().get_footprint_and_threshold().0);
    println!("octets: {:?}", account.as_ref().unwrap().get_footprint_and_threshold().1);
    

    println!("Info: OK");
    println!("m: {:x?}", metadata);*/

    ram.write(start_address, metadata);

    reg[7] = OK;

    return (ExitReason::Continue, gas, reg, ram, account.unwrap());
}

pub fn log(reg: &Registers, ram: &RamMemory, service_id: &ServiceId) {

    //println!("Log hostcall");

    let level = reg[7];
    
    let target_start_address = reg[8] as RamAddress;
    let target_size = reg[9] as RamAddress;
    
    let msg_start_address = reg[10] as RamAddress;
    let msg_size = reg[11] as RamAddress;

    if !ram.is_readable(target_start_address, target_size) {
        println!("Ram memory is not readable for target");
    }

    if !ram.is_readable(msg_start_address, msg_size) {
        println!("Ram memory is not readable for message");
    }

    let target = ram.read(target_start_address, target_size);
    let msg = ram.read(msg_start_address, msg_size);
    let msg_str = String::from_utf8_lossy(&msg);
    
    println!(" \nLevel: {level} target: {:?} service: {service_id} msg: {:?}\n", target, msg_str);
}