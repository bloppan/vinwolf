use crate::types::{Account, ExitReason, Gas, RamAddress, RamMemory, Registers, ServiceAccounts, ServiceId, OpaqueHash, RegSize};
use crate::constants::{NONE, OK, FULL};
use crate::utils::codec::Encode;

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
        println!("NONE");
        reg[7] = NONE;
        return (ExitReason::Continue, gas, reg, ram, account);
    }

    reg[7] = preimage_len;

    if let Some(blob) = preimage_blob {
        ram.write(write_start_address, blob[f as usize..(f + l) as usize].to_vec());
    }
    println!("OK");
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
    println!("READ service id: {}", id);
    let target_account: Option<Account> = if service_id == id {
        Some(account.clone())
    } else if services.contains_key(&id) {
        services.get(&id).cloned()
    } else {
        None
    };
    println!("target_account: {:?}", target_account.is_some());
    let start_read_address = reg[8] as RamAddress;
    let bytes_to_read = reg[9] as RamAddress;
    let start_write_address = reg[10] as RamAddress;

    if !ram.is_readable(start_read_address, bytes_to_read) {
        println!("READ PANIC");
        return (ExitReason::panic, gas, reg, ram, account);
    }

    let key = sp_core::blake2_256(&[id.encode(), ram.read(start_read_address, bytes_to_read).encode()].concat());

    let value: Vec<u8> = if target_account.is_some() && target_account.as_ref().unwrap().storage.contains_key(&key) {
        target_account.unwrap().storage.get(&key).unwrap().clone()
    } else {
        reg[7] = NONE;
        println!("READ NONE");
        println!("key: {:x?} not found", key);
        println!("service_id: {} storage: {:x?}", id, target_account.unwrap().storage);
        return (ExitReason::Continue, gas, reg, ram, account);
    };
    // TODO revisar el orden de los return, no estoy seguro de si estan
    let f = std::cmp::min(reg[11], value.len() as RegSize); 
    let l = std::cmp::min(reg[12], value.len() as RegSize - f);
    if !ram.is_writable(start_write_address, l as RamAddress) {
        println!("READ PANIC");
        return (ExitReason::panic, gas, reg, ram, account);
    }
    reg[7] = value.len() as RegSize;
    ram.write(start_write_address, value[f as usize..(f + l) as usize].to_vec());
    println!("READ OK. l: {l}, f: {f}");
    println!("key: {:x?}, value: {:x?}", key, value[f as usize..(f + l) as usize].to_vec());
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
        println!("panic: not readable");
        return (ExitReason::panic, gas, reg, ram, account);
    }
    println!("service_id: {}", service_id);
    let key = sp_core::blake2_256(&[service_id.encode(), ram.read(key_start_address as RamAddress, key_size as RamAddress)].concat());
    let mut s_account = account.clone();

    let modified_account = if value_size == 0 {
        println!("remove key: {:x?}", key);
        s_account.storage.remove(&key);
        s_account
    } else if ram.is_readable(value_start_address as RamAddress, value_size as RamAddress) {
        let storage_data = ram.read(value_start_address as RamAddress, value_size as RamAddress);
        println!("insert key: {:x?} | value = {:x?}", key, storage_data);
        s_account.storage.insert(key, storage_data);
        s_account
    } else {
        println!("WRITE panic");
        return (ExitReason::panic, gas, reg, ram, account);
    };

    let l: RegSize = if let Some(storage_data) = account.storage.get(&key) {
        println!("WRITE OK");
        storage_data.len() as RegSize
    } else {
        println!("WRITE NONE");
        NONE as RegSize
    };

    let threshold = modified_account.get_footprint_and_threshold().2;

    if threshold > modified_account.balance {
        println!("WRITE FULL");
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
         account.balance.encode(),
         account.get_footprint_and_threshold().2.encode(),
         account.acc_min_gas.encode(),
         account.xfer_min_gas.encode(),
         account.get_footprint_and_threshold().1.encode(),
         account.get_footprint_and_threshold().0.encode()].concat()
    } else {
        reg[7] = NONE;
        return (ExitReason::Continue, gas, reg, ram, Account::default());
    };

    let start_address = reg[8] as RamAddress;

    if !ram.is_writable(start_address, metadata.len() as RamAddress) {
        return (ExitReason::panic, gas, reg, ram, Account::default());
    }

    ram.write(start_address, metadata);
    reg[7] = OK;

    return (ExitReason::Continue, gas, reg, ram, account.unwrap());
}