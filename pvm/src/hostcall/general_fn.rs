use jam_types::{
    Account, AccumulationOperand, DataSegments, DeferredTransfer, Gas, OpaqueHash, 
    ServiceAccounts, ServiceId, StateKeyType, WorkPackage, WorkExecResult, WorkItem};
use crate::pvm_types::{ExitReason, RamAddress, RamMemory, RegSize, Registers};
use constants::pvm::*;
use utils::{hex, log};
use constants::node::{
    CORES_COUNT, EPOCH_LENGTH, VALIDATORS_COUNT, MIN_BALANCE_PER_ITEM, MIN_BALANCE_PER_OCTET, MIN_BALANCE, MAX_TIMESLOTS_AFTER_UNREFEREND_PREIMAGE,
    WORK_REPORT_GAS_LIMIT, WORK_PACKAGE_GAS_LIMIT, WORK_PACKAGE_REFINE_GAS, TOTAL_GAS_ALLOCATED, RECENT_HISTORY_SIZE, MAX_WORK_ITEMS, MAX_DEPENDENCY_ITEMS, MAX_AGE_LOOKUP_ANCHOR,
    MAX_ITEMS_AUTHORIZATION_POOL, SLOT_PERIOD, MAX_ITEMS_AUTHORIZATION_QUEUE, ROTATION_PERIOD, MAX_EXTRINSICS_IN_WP, REPORTED_WORK_REPLACEMENT_PERIOD,
    MAX_IS_AUTHORIZED_SIZE, MAX_ENCODED_WORK_PACKAGE_SIZE, MAX_SERVICE_CODE_SIZE, PIECE_SIZE, MAX_WORK_PACKAGE_IMPORTS, SEGMENT_PIECES, MAX_WORK_REPORT_TOTAL_SIZE,
    TRANSFER_MEMO_SIZE, MAX_WORK_PACKAGE_EXPORTS, TICKET_SUBMISSION_ENDS, MAX_TICKETS_PER_EXTRINSIC, TICKET_ENTRIES_PER_VALIDATOR,
};
use codec::{Encode, EncodeSize, EncodeLen};
use utils::serialization::{construct_storage_key, construct_preimage_key, StateKeyTrait};

use super::HostCallContext;

pub fn gas(gas: &mut Gas, reg: &mut Registers) -> ExitReason
{
    *gas -= 10;

    if *gas < 0 {
        log::error!("Out of gas!");
        return ExitReason::OutOfGas;
    } 
 
    reg[7] = *gas as RegSize;

    log::debug!("gas: {gas}");
    return ExitReason::Continue;
}

pub fn fetch(gas: &mut Gas, 
             reg: &mut Registers, 
             ram: &mut RamMemory, 
             _pkg: Option<WorkPackage>,
             n: Option<OpaqueHash>,
             _result: Option<WorkExecResult>,
             _segments: Option<Vec<DataSegments>>,
             _work_items: Option<Vec<WorkItem>>,
             operands: Option<Vec<AccumulationOperand>>,
             transfers: Option<Vec<DeferredTransfer>>,
             ctx: &mut HostCallContext) 

-> ExitReason 
{
    *gas -= 10;

    if *gas < 0 {
        log::debug!("Out of gas!");
        return ExitReason::OutOfGas;
    } 
    //println!("reg_10: {:?}", reg[10]);
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

    log::debug!("reg 10: {:?}", reg[10]);
    let value: Option<Vec<u8>> = if reg[10] == 0 {
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
        log::debug!("value: {}", hex::encode(&value.as_ref().unwrap()));
        value.as_ref().unwrap().len()
    } else {
        0
    };

    let start_address = reg[7] as RamAddress;
    let f = std::cmp::min(reg[8] as usize, value_len);
    let l = std::cmp::min(reg[9] as usize, value_len - f);
    log::debug!("value_len: {:?}, start_address: {start_address}, f: {f}, l: {l}", value_len);

    if let Err(_) = ram.is_writable(start_address, l as RamAddress) {
        log::error!("Panic: The RAM is not readable from address: {start_address} num_bytes: {l}");
        return ExitReason::panic;
    }

    if value.is_none() {
        reg[7] = NONE;
        log::debug!("Exit: NONE");
        return ExitReason::Continue;
    }

    reg[7] = value_len as RegSize;
    ram.write(start_address, &value.unwrap()[f as usize..(f + l) as usize]);
    
    log::debug!("Exit: OK");
    return ExitReason::Continue;
}


pub fn lookup(gas: &mut Gas, reg: &mut Registers, ram: &mut RamMemory, account: &Option<Account>, service_id: ServiceId, services: &ServiceAccounts) 

-> ExitReason
{
    *gas -= 10;

    if *gas < 0 {
        log::debug!("Out of gas!");
        return ExitReason::OutOfGas;
    }  

    let a_account: &Option<Account> = if reg[7] as ServiceId == service_id || reg[7] == u64::MAX {
        account
    } else if let Some(acc) = services.get(&(reg[7] as ServiceId)) {
        &Some(acc.clone())
    } else {
        log::debug!("The account is none");
        &None    
    };

    
    let read_start_address = reg[8] as RamAddress;
    let write_start_address = reg[9] as RamAddress;

    if let Err(_) = ram.is_readable(read_start_address, 32) {
        log::debug!("Panic: The RAM is not readable from address: {read_start_address} num_bytes: 32");
        return ExitReason::panic;
    }

    let hash: OpaqueHash = ram.read(read_start_address, 32).try_into().unwrap();
    log::debug!("hash: 0x{}", hex::encode(hash));
    let preimage_key = StateKeyType::Account(service_id, construct_preimage_key(&hash)).construct();  
    log::debug!("preimage_key: 0x{}", hex::encode(preimage_key));

    /*let preimage_blob: Option<Vec<u8>> = if a_account.is_none() {
        None
    } else if !a_account.as_ref().unwrap().storage.contains_key(&preimage_key) {
        None
    } else {
        a_account.unwrap().storage.get(&preimage_key).cloned()
    };*/

    let preimage_blob: Option<&Vec<u8>> = a_account.as_ref().and_then(|acc| acc.storage.get(&preimage_key));

    let preimage_len = if preimage_blob.is_none() {
        0 as RegSize
    } else {
        preimage_blob.as_ref().unwrap().len() as RegSize
    };
    
    let f = std::cmp::min(reg[10],  preimage_len);
    let l = std::cmp::min(reg[11], preimage_len - f);

    if let Err(_) = ram.is_writable(write_start_address, l as RamAddress) {
        log::error!("Panic: The RAM is not writable from address: {write_start_address} num_bytes: {l}");
        return ExitReason::panic;
    }

    if preimage_blob.is_none() {
        reg[7] = NONE;
        log::debug!("Exit: NONE");
        return ExitReason::Continue;
    }

    log::debug!("preimage len: {preimage_len}");
    reg[7] = preimage_len;
    ram.write(write_start_address, &preimage_blob.unwrap()[f as usize..(f + l) as usize]);
    
    log::debug!("Exit: OK");
    return ExitReason::Continue;
}

pub fn read(gas: &mut Gas, reg: &mut Registers, ram: &mut RamMemory, account: &Option<Account>, service_id: ServiceId, services: &ServiceAccounts) 

-> ExitReason
{
    *gas -= 10;

    if *gas < 0 {
        log::error!("Out of gas!");
        return ExitReason::OutOfGas;
    }   

    let star_service = if reg[7] == u64::MAX {
        service_id
    } else {
        reg[7] as ServiceId
    };
    
    log::debug!("service id: {star_service}");

    let target_account: &Option<Account> = if service_id == star_service {
        account
    } else if services.contains_key(&star_service) {
        &services.get(&star_service).cloned()
    } else {
        log::debug!("target account is none");
        &None
    };
    
    let start_read_address = reg[8] as RamAddress;
    let bytes_to_read = reg[9] as RamAddress;
    let start_write_address = reg[10] as RamAddress;

    if let Err(_) = ram.is_readable(start_read_address, bytes_to_read) {
        log::error!("Panic: The RAM is not readable from address: {start_read_address} num_bytes: {bytes_to_read}");
        return ExitReason::panic;
    }

    let storage_raw_key= ram.read(start_read_address, bytes_to_read);
    log::debug!("storage raw key: 0x{}", hex::encode(&storage_raw_key));

    let storage_key = StateKeyType::Account(star_service, construct_storage_key(&storage_raw_key)).construct();
    log::debug!("service: {:?} storage key: 0x{}", star_service, hex::encode(&storage_key));

    let value: Option<&Vec<u8>> = if target_account.is_some() && target_account.as_ref().unwrap().storage.contains_key(&storage_key) {
        Some(target_account.as_ref().unwrap().storage.get(&storage_key).unwrap())
    } else {
        None
    };
    
    let value_len = if value.is_none() {
        log::debug!("The value is None");
        0
    } else {
        value.as_ref().unwrap().len()
    };

    let f = std::cmp::min(reg[11], value_len as RegSize); 
    let l = std::cmp::min(reg[12], (value_len as RegSize).saturating_sub(f));
    log::debug!("f: {f}, l: {l}");
    
    if let Err(_) = ram.is_writable(start_write_address, l as RamAddress) {
        log::error!("Panic: The RAM is not writable from address: {start_write_address} num_bytes: {l}");
        return ExitReason::panic;
    }

    if value.is_none() {
        log::debug!("Exit: NONE");
        reg[7] = NONE;
        return ExitReason::Continue;
    }

    reg[7] = value_len as RegSize;
    ram.write(start_write_address, &value.unwrap()[f as usize..(f + l) as usize]);

    log::debug!("Exit: OK");
    return ExitReason::Continue;
}

pub fn write(gas: &mut Gas, reg: &mut Registers, ram: &mut RamMemory, account: &mut Option<Account>, service_id: &ServiceId) 

-> ExitReason
{
    *gas -= 10;

    if *gas < 0 {
        log::debug!("Out of gas!");
        return ExitReason::OutOfGas;
    }

    let key_start_address = reg[7];
    let key_size = reg[8];
    let value_start_address = reg[9];
    let value_size = reg[10];

    if let Err(_) = ram.is_readable(key_start_address as RamAddress, key_size as RamAddress) {
        log::error!("Panic: The RAM is not readable from address: {key_start_address} num_bytes: {key_size}");
        return ExitReason::panic;
    }
    
    if account.is_none() {
        log::error!("Account not found for service: {:?}", service_id);
        return ExitReason::panic;
    }

    let raw_storage_key = ram.read(key_start_address as RamAddress, key_size as RamAddress);
    log::debug!("raw key: 0x{}", hex::encode(&raw_storage_key));

    let storage_key = StateKeyType::Account(*service_id, construct_storage_key(&raw_storage_key)).construct();
    log::debug!("service: {:?} storage key: 0x{}", service_id, hex::encode(&storage_key));

    let mut s_account = account.as_ref().unwrap().clone();

    let old_storage_data_len: Option<usize> = if let Some (old_storage_data) = s_account.storage.get(&storage_key) {
        Some(old_storage_data.len())
    } else {
        None
    };

    let modified_account = if value_size == 0 {
        log::debug!("remove key: 0x{}", hex::encode(&storage_key));
        s_account.storage.remove(&storage_key);
                
        if old_storage_data_len.is_some() {
            log::debug!("Substract {:?} to octets storage", old_storage_data_len.unwrap() as u64 + raw_storage_key.len() as u64 + 34);
            s_account.octets -= old_storage_data_len.unwrap() as u64 + raw_storage_key.len() as u64 + 34;
            s_account.items -= 1;
        }

        s_account
    } else if ram.is_readable(value_start_address as RamAddress, value_size as RamAddress).is_ok() {
        let storage_data = ram.read(value_start_address as RamAddress, value_size as RamAddress);
        log::debug!("insert key: 0x{}, value = 0x{}", hex::encode(&storage_key), hex::encode(&storage_data));

        if old_storage_data_len.is_some() {

            let diff_storage: i64 = (storage_data.len() as i64 - old_storage_data_len.unwrap() as i64) as i64;

            if diff_storage.is_positive() {
                log::debug!("Sum {:?} to octets storage", diff_storage);
                s_account.octets += diff_storage as u64;
            } else if diff_storage.is_negative() {
                log::debug!("Substract {:?} to octets storage", diff_storage);
                s_account.octets -= diff_storage.abs() as u64;
            }

            if old_storage_data_len.unwrap() == 0 {
                s_account.items += 1;
            }
        } else {
            log::debug!("Sum 1 to items storage");
            s_account.items += 1;
            log::debug!("Sum {:?} to octets storage", storage_data.len() as u64 + raw_storage_key.len() as u64 + 34);
            s_account.octets += storage_data.len() as u64 + raw_storage_key.len() as u64 + 34;
        }
        
        s_account.storage.insert(storage_key, storage_data);
        s_account
    } else {
        log::error!("Panic: The RAM is not readable from address: {value_start_address}, num_bytes: {value_size}");
        return ExitReason::panic;
    };

    let l: RegSize = if let Some(storage_data) = account.as_ref().unwrap().storage.get(&storage_key) {
        storage_data.len() as RegSize
    } else {
        NONE as RegSize
    };

    let threshold = utils::common::get_threshold(&modified_account);
    log::debug!("l: {l}, threshold: {threshold}");

    if threshold > modified_account.balance {
        reg[7] = FULL as RegSize;
        log::debug!("Exit: FULL");
        return ExitReason::Continue;
    }

    reg[7] = l;
    *account = Some(modified_account);
    log::debug!("Exit OK, l: {:?}", l);

    return ExitReason::Continue;
}

pub fn info(gas: &mut Gas, reg: &mut Registers, ram: &mut RamMemory, service_id: ServiceId, accounts: &ServiceAccounts, s_account: &Option<Account>)

-> ExitReason {

    *gas -= 10;

    if *gas < 0 {
        log::error!("Out of gas!");
        return ExitReason::OutOfGas;
    }

    log::debug!("Info hostcall, service id context: {:?}", service_id);

    let account: Option<Account> = if reg[7] == u64::MAX {
        s_account.clone()
        /*if let Some(account) = accounts.get(&service_id).cloned() {
            log::debug!("Selected account for service: {:?}", service_id);
            Some(account)
        } else {
            log::error!("Account not found for service {:?}", service_id);
            return ExitReason::panic;
        }*/
    } else {
        if let Some(account) = accounts.get(&(reg[7] as ServiceId)).cloned() {
            log::debug!("Selected account for service: {:?}", reg[7]);
            Some(account)
        } else {
            log::error!("Account not found for service {:?}", reg[7]);
            return ExitReason::panic;
        }
    };

    let metadata: Option<Vec<u8>> = if let Some(account) = account.as_ref() {
        let threshold = utils::common::get_threshold(account);
        Some([
            account.code_hash.encode(),
            account.balance.encode_size(8),
            threshold.encode_size(8),
            account.acc_min_gas.encode_size(8),
            account.xfer_min_gas.encode_size(8),
            account.octets.encode_size(8),
            account.items.encode_size(4),
            account.gratis_storage_offset.encode_size(8),
            account.created_at.encode_size(4),
            account.last_acc.encode_size(4),
            account.parent_service.encode_size(4)
        ].concat())
    } else {
        None
    };

    let metadata_len = if metadata.is_none() {
        0
    } else {
        metadata.as_ref().unwrap().len()
    };

    let f = std::cmp::min(reg[9], metadata_len as RegSize);
    let l = std::cmp::min(reg[10], (metadata_len as RegSize).saturating_sub(f));

    let start_address = reg[8] as RamAddress;

    if let Err(_) = ram.is_writable(start_address, l as RamAddress) {
        log::debug!("Panic: The RAM is not writable from address: {start_address} num_bytes: {:?}", l);
        return ExitReason::panic;
    }

    if metadata.is_none() {
        reg[7] = NONE as RegSize;
        log::debug!("Exit: NONE");
        return ExitReason::Continue;
    }

    log::debug!("code_hash: 0x{}", hex::encode(account.as_ref().unwrap().code_hash));
    let threshold = utils::common::get_threshold(account.as_ref().unwrap());
    log::debug!("balance: {:?}, threshold: {:?}, acc gas: {:?}, xfer gas: {:?}, items: {:?}, octets: {:?}, gratis_offset: {:?}, created_at: {:?}, last_acc: {:?}, parent_service: {:?}", 
                account.as_ref().unwrap().balance, threshold, account.as_ref().unwrap().acc_min_gas,
                account.as_ref().unwrap().xfer_min_gas, account.as_ref().unwrap().items, account.as_ref().unwrap().octets,
                account.as_ref().unwrap().gratis_storage_offset, account.as_ref().unwrap().created_at, account.as_ref().unwrap().last_acc,
                account.as_ref().unwrap().parent_service);

    ram.write(start_address, &metadata.unwrap()[f as usize ..(f + l) as usize]);
    reg[7] = metadata_len as RegSize;

    log::debug!("reg_7: {:?} Exit: OK", metadata_len);
    return ExitReason::Continue;
}

pub fn log(reg: &Registers, ram: &RamMemory, service_id: &ServiceId) -> ExitReason {

    let level = reg[7];
    
    let target_start_address = reg[8] as RamAddress;
    let target_size = reg[9] as RamAddress;
    
    let msg_start_address = reg[10] as RamAddress;
    let msg_size = reg[11] as RamAddress;

    if let Err(_) = ram.is_readable(target_start_address, target_size) {
        log::error!("The RAM memory is not readable from address: {target_start_address} num_bytes: {target_size}");
    }

    if let Err(_) = ram.is_readable(msg_start_address, msg_size) {
        log::error!("The RAM memory is not readable from address: {msg_start_address} num_bytes: {msg_size}");
    }

    let target = ram.read(target_start_address, target_size);
    let msg = ram.read(msg_start_address, msg_size);
    let msg_str = String::from_utf8_lossy(&msg);
    
    match level {
        0 => { log::error!("LOG target: {:?} service: {service_id} msg: {:?}", target, msg_str); },
        1 => { log::warn!("LOG target: {:?} service: {service_id} msg: {:?}", target, msg_str); },
        2 => { log::info!("LOG target: {:?} service: {service_id} msg: {:?}", target, msg_str); },
        3 => { log::debug!("LOG target: {:?} service: {service_id} msg: {:?}", target, msg_str); },
        4 => { log::debug!("LOG target: {:?} service: {service_id} msg: {:?}", target, msg_str); },
        _ => { log::debug!("LOG unknown level: {:?}", level); },
    }

    return ExitReason::Continue;
}