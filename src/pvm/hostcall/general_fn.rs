use std::collections::HashSet;
use crate::types::{Account, ExitReason, Gas, RamAddress, RamMemory, Registers, ServiceAccounts, ServiceId, ServiceInfo, RegSize};
use crate::constants::{PAGE_SIZE, NONE, OK, FULL};
use crate::utils::{codec::Encode, common};

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

    let target_account: Option<Account> = if service_id == id {
        Some(account.clone())
    } else if services.service_accounts.contains_key(&id) {
        services.service_accounts.get(&id).cloned()
    } else {
        None
    };

    let start_read_address = reg[8] as RamAddress;
    let bytes_to_read = reg[9] as RamAddress;
    let start_write_address = reg[10] as RamAddress;

    if !ram.is_readable(start_read_address, start_read_address + bytes_to_read) {
        return (ExitReason::panic, gas, reg, ram, account);
    }

    let key = sp_core::blake2_256(&[id.encode(), ram.read(start_read_address, bytes_to_read).encode()].concat());

    let value: Vec<u8> = if target_account.is_some() && target_account.as_ref().unwrap().storage.contains_key(&key) {
        target_account.unwrap().storage.get(&key).unwrap().clone()
    } else {
        reg[7] = NONE;
        return (ExitReason::Continue, gas, reg, ram, account);
    };
    // TODO revisar el orden de los return, no estoy seguro de si estan
    let f = std::cmp::min(reg[11], value.len() as RegSize);
    let l = std::cmp::min(reg[12], value.len() as RegSize - f);

    if !ram.is_writable(start_write_address, start_write_address + l as RamAddress) {
        return (ExitReason::panic, gas, reg, ram, account);
    }

    reg[7] = value.len() as RegSize;
    ram.write(start_write_address, value[f as usize..(f + l) as usize].to_vec());

    return (ExitReason::Continue, gas, reg, ram, account);
}

pub fn write(mut gas: Gas, mut reg: Registers, ram: RamMemory, account: Account, service_id: ServiceId) 

-> (ExitReason, Gas, Registers, RamMemory, Account)
{
    gas -= 10;

    if gas < 0 {
        return (ExitReason::OutOfGas, gas, reg, ram, account);
    }   

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

pub fn info(mut gas: Gas, mut reg: Registers, mut ram: RamMemory, service_id: ServiceId, accounts: ServiceAccounts)

-> (ExitReason, Gas, Registers, RamMemory, Account) {

    gas -= 10;

    if gas < 0 {
        return (ExitReason::OutOfGas, gas, reg, ram, Account::default());
    }   

    let account: Option<Account> = if reg[7] == u64::MAX {
        if let Some(account) = accounts.service_accounts.get(&service_id).cloned() {
            Some(account)
        } else {
            None
        }
    } else {
        if let Some(account) = accounts.service_accounts.get(&(reg[7] as ServiceId)).cloned() {
            Some(account)
        } else {
            None
        }
    };

    let metadata = if let Some(account) = account.as_ref() {

        [account.code_hash.encode(), 
         account.balance.encode(),
         account.get_footprint_and_threshold().2.encode(),
         account.gas.encode(),
         account.min_gas.encode(),
         account.get_footprint_and_threshold().1.encode(),
         account.get_footprint_and_threshold().0.encode()].concat()
    } else {
        reg[7] = NONE;
        return (ExitReason::Continue, gas, reg, ram, Account::default());
    };

    let start_address = reg[8] as RamAddress;

    if !ram.is_writable(start_address, start_address + metadata.len() as RamAddress) {
        return (ExitReason::panic, gas, reg, ram, Account::default());
    }

    ram.write(start_address, metadata);
    reg[7] = OK;

    return (ExitReason::Continue, gas, reg, ram, account.unwrap());
}