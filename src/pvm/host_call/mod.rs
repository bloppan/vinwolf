use crate::types::{Page, PageTable, ProgramFormat, RamAccess, RamAddress, RamMemory, ServiceAccounts, ServiceId, StandardProgram};
use crate::constants::{NUM_REG, PAGE_SIZE, PVM_INIT_INPUT_DATA_SIZE, PVM_INIT_ZONE_SIZE, RAM_SIZE, Zz, Zi};
use crate::constants::{NONE, WHAT, OOB, WHO, FULL, CORE, CASH, LOW, HUH, OK};
use crate::utils::codec::{Encode, Decode, BytesReader, ReadError};

use crate::types::{Account, Context, ExitReason, Gas, ServiceInfo};

fn Page(x: usize) -> u64 {
    x.div_ceil(PAGE_SIZE as usize) as u64 * PAGE_SIZE as u64
}

fn Zone(x: usize) -> u64 {
    x.div_ceil(PVM_INIT_ZONE_SIZE as usize) as u64 * PVM_INIT_ZONE_SIZE as u64
}

fn ram_initialization(params: &ProgramFormat, arg: &[u8]) -> RamMemory {

    let mut ram = RamMemory::default();

    for i in 0..RAM_SIZE {

        let page = i / PAGE_SIZE as u64;
        let offset = i % PAGE_SIZE as u64;

        if Zz <= i && i < Zz + params.o.len() as u64 {
            if ram.pages[page as usize].is_none() {
                ram.pages[page as usize] = Some(Page::default());
            }
            ram.pages[page as usize].as_mut().unwrap().data[offset as usize] = params.o[i as usize - Zz as usize];
        } else if Zz + params.o.len() as u64 <= i && i < Zz + Page(params.o.len()) as u64 {
            if ram.pages[page as usize].is_none() {
                ram.pages[page as usize] = Some(Page::default());
            }
            ram.pages[page as usize].as_mut().unwrap().data[offset as usize] = 0;
        } else if 2 * Zz + Zone(params.o.len()) as u64 <= i && i < 2 * Zz + Zone(params.o.len()) as u64 + params.w.len() as u64 {
            if ram.pages[page as usize].is_none() {
                ram.pages[page as usize] = Some(Page::default());
            }
            ram.pages[page as usize].as_mut().unwrap().flags.access = RamAccess::Write;
            ram.pages[page as usize].as_mut().unwrap().data[offset as usize] = params.w[i as usize - (2 * Zz + Zone(params.o.len()) as u64) as usize];
        } else if 2 * Zz + Zone(params.o.len()) as u64 + params.w.len() as u64 <= i && i < 2 * Zz + Zone(params.o.len()) as u64 + Page(params.w.len()) as u64 + params.z as u64 * PAGE_SIZE as u64 {
            if ram.pages[page as usize].is_none() {
                ram.pages[page as usize] = Some(Page::default());
            }
            ram.pages[page as usize].as_mut().unwrap().flags.access = RamAccess::Write;
        } else if (1 << 32) - 2 * Zz - Zi - Page(params.s as usize) as u64 <= i && i < (1 << 32) - 2 * Zz - Zi {
            if ram.pages[page as usize].is_none() {
                ram.pages[page as usize] = Some(Page::default());
            }
            ram.pages[page as usize].as_mut().unwrap().flags.access = RamAccess::Write;
        } else if (1 << 32) - Zz - Zi <= i && i < (1 << 32) - Zz - Zi + arg.len() as u64{
            if ram.pages[page as usize].is_none() {
                ram.pages[page as usize] = Some(Page::default());
            }
            ram.pages[page as usize].as_mut().unwrap().data[offset as usize] = arg[i as usize - ((1 << 32) - Zz - Zi) as usize]; 
        } else if (1 << 32) - Zz - Zi + arg.len() as u64 <= i && i < (1 << 32) - Zz - Zi + Page(arg.len() as usize) as u64 {
            if ram.pages[page as usize].is_none() {
                ram.pages[page as usize] = Some(Page::default());
            }
        }
    }

    return ram;

}

fn reg_initialization(params: &ProgramFormat, arg: &[u8]) -> [u64; NUM_REG] {

    let mut reg = [0; NUM_REG];

    for i in 0..NUM_REG {

        if i == 0 {
            reg[i] = 0xFFFF0000;
        } else if i == 1 {
            reg[i] = (1 << 32) - 2 * Zz - Zi;
        } else if i == 7 {
            reg[i] = (1 << 32) - Zz - Zi;
        } else if i == 8 {
            reg[i] = arg.len() as u64;
        } else {
            reg[i] = 0;
        }
    }

    return reg;
}

fn standard_program_initialization(program: &[u8], arg: &[u8]) -> Result<Option<StandardProgram>, ReadError> {

    let mut blob = BytesReader::new(program);
    let params = ProgramFormat::decode(&mut blob)?;

    if 5 * Zz + Zone(params.o.len()) + Zone(params.w.len() + params.z as usize * PAGE_SIZE as usize) + Zone(params.s as usize) + Zi > (1 << 32) {
        return Ok(None);
    }

    return Ok(Some(StandardProgram {
        ram: ram_initialization(&params, arg),
        reg: reg_initialization(&params, arg),
        code: params.c,
    }));
}

fn is_writable(page_table: &PageTable, start_page: &RamAddress, end_page: &RamAddress) -> Result<bool, ExitReason> {
    
    for i in *start_page..=*end_page {

        if let Some(page) = page_table.pages.get(&(i as u32)) {
            if page.flags.access != RamAccess::Write {
                return Err(ExitReason::panic);
            }
        } else {
            return Err(ExitReason::PageFault(i));
        }
    }

    return Ok(true);
}

pub fn info(ctx: &mut Context, service_id: &ServiceId, accounts: &ServiceAccounts) -> ExitReason {

    let t: Option<Account> = if ctx.reg[7] == u64::MAX {
        if let Some(account) = accounts.service_accounts.get(&service_id) {
            Some(account.clone())
        } else {
            None
        }
    } else {
        if let Some(account) = accounts.service_accounts.get(&(ctx.reg[7] as ServiceId)) {
            Some(account.clone())
        } else {
            None
        }
    };

    let o = ctx.reg[8];

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
        ctx.reg[7] = NONE;
        return ExitReason::Continue;
    }

    /*let start_address = o as RamAddress;
    let end_address = (o + m.as_ref().unwrap().len() as u64) as RamAddress;*/
    let start_page = o as RamAddress / PAGE_SIZE;
    let end_page = (o + m.as_ref().unwrap().len() as u64) as RamAddress / PAGE_SIZE;
    let mut offset = (o % PAGE_SIZE as u64) as usize;

    if let Err(error) = is_writable(&ctx.page_table, &start_page, &end_page) {
        return error;
    }

    for page_number in start_page..=end_page {
        let page = ctx.page_table.pages.get_mut(&page_number).unwrap();
        let mut i = 0;
        if page_number != start_page {
            offset = 0;
        }
        while i < (PAGE_SIZE as usize - offset) && i < m.as_ref().unwrap().len() {
            page.data[i + offset] = m.as_ref().unwrap()[i];
            i += 1;
        }
    }

    ctx.reg[7] = OK;
    ExitReason::Continue
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_division() {
        assert_eq!(1, 1_u32.div_ceil(2));
        assert_eq!(1, 2_u32.div_ceil(5));
        assert_eq!(3, 5_u32.div_ceil(2));
    }
}