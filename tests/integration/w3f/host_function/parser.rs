use std::collections::HashMap;

extern crate vinwolf;
use vinwolf::constants::{NUM_REG, PAGE_SIZE};
use vinwolf::types::{Account, Context, OpaqueHash, Page, PageFlags, RamAccess, RamMemory, ServiceAccounts, StorageKey};
use vinwolf::utils::serialization::construct_lookup_key;

use super::{DeltaEntry, InitialMemory, HostCallTestFile};

pub fn parse_account(json_data: &DeltaEntry) -> Account {
    
    let mut account = Account::default();

    let mut hash: OpaqueHash = [0u8; 32];
    let mut code_hash: Vec<u8> = Vec::new();

    if json_data.code_hash.len() > 2 {
        code_hash = hex::decode(&json_data.code_hash[2..]).unwrap();
    }

    for (i, byte) in code_hash.iter().enumerate() {
        hash[i] = *byte;
    }
    
    account.code_hash = hash;
    account.balance = json_data.balance;
    account.acc_min_gas = json_data.g;
    account.xfer_min_gas = json_data.m;
    
    for item in json_data.s_map.iter() {
        let mut value: Vec<u8> = Vec::new();
        for i in 0..item.1.len() {
            value.push(item.1[i]);
        }
        let hash = hex::decode(&item.0[2..]).unwrap();
        let mut hash_storage: StorageKey = [0u8; 31];
        for (i, byte) in hash.iter().enumerate() {
            hash_storage[i] = *byte;
        }
        account.storage.insert(hash_storage, value);
    }
    for item in json_data.l_map.iter() {
        let hash = hex::decode(&item.0[2..]).unwrap();
        let mut hash_lookup: OpaqueHash = [0u8; 32];
        for (i, byte) in hash.iter().enumerate() {
            hash_lookup[i] = *byte;
        }
        let mut timeslots: Vec<u32> = Vec::new();
        for slot in item.1.t.iter() {
            timeslots.push(*slot);
        }
        let length = item.1.l;
        account.lookup.insert(construct_lookup_key(&hash_lookup, length), timeslots);
    }
    for item in json_data.p_map.iter() {
        let mut value: Vec<u8> = Vec::new();
        for i in 0..item.1.len() {
            value.push(item.1[i]);
        }
        let hash = hex::decode(&item.0[2..]).unwrap();
        let mut hash_storage: OpaqueHash = [0u8; 32];
        for (i, byte) in hash.iter().enumerate() {
            hash_storage[i] = *byte;
        }

        //account.preimages.insert(hash_storage, value);
    }

    return account;
}

pub fn parse_service_accounts(json_data: &HashMap<String, DeltaEntry>) -> ServiceAccounts {

    let mut service_accounts: ServiceAccounts = ServiceAccounts::default();

    for service in json_data.iter() {
        let account = parse_account(service.1);
        service_accounts.insert(service.0.parse::<u32>().unwrap(), account);
    }

    return service_accounts;
}

pub fn parse_regs(json_data: &HashMap<String, u64>) -> [u64; NUM_REG] {
    let mut regs: [u64; NUM_REG] = [0; NUM_REG];
    for reg in json_data.iter() {
        regs[reg.0.parse::<usize>().unwrap()] = *reg.1;
    }
    return regs;
}
// TODO arreglar esto
pub fn parse_memory(json_data: &InitialMemory) -> RamMemory {
    let ram: RamMemory = RamMemory::default();
    for page in json_data.pages.iter() {
        let mut flags = PageFlags::default();
        if page.1.access.writable {
            flags.access.insert(RamAccess::Write);
        } else  {
            flags.access.insert(RamAccess::Read);
        }
        let _new_page = Page {
            flags,
            data: {
                let mut data = Box::new([0; PAGE_SIZE as usize]);
                for i in 0..page.1.value.len() {
                    data[i] = page.1.value[i];
                }
                data
            }   
        };
        //page_table.pages.insert(page.0.parse::<u32>().unwrap(), new_page);
    }
    return ram;
}

pub enum TestPart {
    Initial,
    Expected,
}

pub fn parse_context(json_data: &HostCallTestFile, part: TestPart) -> Context {
    let mut context = Context::default();
    match part {
        TestPart::Initial => {
            context.gas = json_data.initial_gas;
            context.reg = parse_regs(&json_data.initial_regs);
            context.ram = parse_memory(&json_data.initial_memory);
        },
        TestPart::Expected => {
            context.gas = json_data.expected_gas;
            context.reg = parse_regs(&json_data.expected_regs);
            context.ram = parse_memory(&json_data.expected_memory);
        }
    }
    return context;
}
