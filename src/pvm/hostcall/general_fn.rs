use std::collections::HashSet;
use crate::types::{Account, ExitReason, Gas, RamAddress, RamMemory, Registers, ServiceAccounts, ServiceId, ServiceInfo, RegSize};
use crate::constants::{PAGE_SIZE, NONE, OK, FULL};
use crate::utils::{codec::Encode, common};
use crate::pvm::hostcall::is_writable;

pub fn write(mut gas: Gas, mut reg: Registers, ram: RamMemory, account: Account, service_id: ServiceId) 

-> (ExitReason, Gas, Registers, RamMemory, Account)
{
    gas -= 10;

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

pub fn info(gas: &mut Gas, reg: &mut Registers, ram: &mut RamMemory, service_id: &ServiceId, accounts: &ServiceAccounts)

-> ExitReason {

    if *gas < 10 {
        return ExitReason::OutOfGas;
    }

    *gas -= 10;

    let t: Option<&Account> = if reg[7] == u64::MAX {
        if let Some(account) = accounts.service_accounts.get(&service_id) {
            Some(account)
        } else {
            None
        }
    } else {
        if let Some(account) = accounts.service_accounts.get(&(reg[7] as ServiceId)) {
            Some(account)
        } else {
            None
        }
    };

    let o = reg[8].clone();

    let m = if t.is_some() {
        let account = t.unwrap();
        let mut num_bytes: u64 = 0;
        let mut num_items: u32 = 0;
        for item in account.storage.iter() {
            num_bytes += item.1.len() as u64;
            num_items += 1;
        }
        let service_info = ServiceInfo {
            code_hash: account.code_hash,
            balance: account.balance,
            min_item_gas: account.gas,
            min_memo_gas: account.min_gas,
            bytes: num_bytes,
            items: num_items,
        };
        Some(service_info.encode())
    } else {
        None
    };

    if m.is_none() {
        reg[7] = NONE;
        return ExitReason::Continue;
    }

    let start_page = o as RamAddress / PAGE_SIZE;
    let end_page = (o + m.as_ref().unwrap().len() as u64) as RamAddress / PAGE_SIZE;
    let mut offset = (o % PAGE_SIZE as u64) as usize;

    if let Err(error) = is_writable(&ram, &start_page, &end_page) {
        return error;
    }

    for page_number in start_page..=end_page {
        let page = ram.pages[page_number as usize].as_mut().unwrap();
        let mut i = 0;
        if page_number != start_page {
            offset = 0;
        }
        while i < (PAGE_SIZE as usize - offset) && i < m.as_ref().unwrap().len() {
            page.data[i + offset] = m.as_ref().unwrap()[i];
            i += 1;
        }
    }

    reg[7] = OK;
    ExitReason::Continue
}