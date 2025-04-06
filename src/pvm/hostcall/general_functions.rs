use crate::types::{
    Account, ExitReason, Gas, PageTable, RamAddress, RamMemory, Registers, ServiceAccounts, ServiceId, ServiceInfo 
};
use crate::constants::PAGE_SIZE;
use crate::constants::{NONE, WHAT, OOB, WHO, FULL, CORE, CASH, LOW, HUH, OK};
use crate::utils::codec::Encode;
use crate::pvm::hostcall::is_writable;

pub fn info(gas: &mut Gas,
            reg: &mut Registers,
            ram: &mut RamMemory,
            service_id: &ServiceId, 
            accounts: &ServiceAccounts
) -> ExitReason {

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