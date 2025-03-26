use std::collections::HashMap;

use crate::types::{
    Account, AccumulationContext, Context, DataSegment, ExitReason, Gas, PageTable, ProgramFormat, RamAccess, RamAddress, 
    RefineMemory, RegSize, Registers, HostCallType
};

use crate::constants::PAGE_SIZE;
use crate::pvm::invoke_pvm;
use crate::utils::codec::{Decode, BytesReader};

mod accumulate;
mod refine;
mod on_transfer;
mod is_authorized;
pub mod general_functions;
mod program_init;

use accumulate::dispatch_accumulate;

pub fn is_writable(page_table: &PageTable, start_page: &RamAddress, end_page: &RamAddress) -> Result<bool, ExitReason> {
    
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

pub fn is_readable(page_table: &PageTable, start_page: &RamAddress, end_page: &RamAddress) -> Result<bool, ExitReason> {
    
    for i in *start_page..=*end_page {

        if let Some(page) = page_table.pages.get(&(i as u32)) {
            if page.flags.access != RamAccess::Read {
                return Err(ExitReason::panic);
            }
        } else {
            return Err(ExitReason::PageFault(i));
        }
    }

    return Ok(true);
}

#[derive(Debug, Clone, PartialEq)]
pub enum HostCallContext {
    Accumulate(AccumulationContext, AccumulationContext),
    Refine(HashMap<usize, RefineMemory>, Vec<DataSegment>),
    OnTransfer(Account),
    IsAuthorized(),
}

pub type HostCallArgs = (HostCallType, Gas, Registers, PageTable, HostCallContext);

pub enum HostCallResult {
    Ok(ExitReason, Gas, Registers, PageTable, HostCallContext),
    Err(usize)
}

fn hostcall(program_blob: &[u8], pc: RegSize, args: HostCallArgs) -> (ExitReason, RegSize, Gas, Registers, PageTable, HostCallContext) {

    let (n, mut gas, mut reg, mut ram, mut ctx) = args;

    let mut pvm_ctx = Context::default();
    pvm_ctx.pc = pc;
    pvm_ctx.gas = gas;
    pvm_ctx.reg = reg;
    pvm_ctx.page_table = ram;

    let exit_reason = invoke_pvm(&mut pvm_ctx, program_blob);

    if exit_reason == ExitReason::Halt 
        || exit_reason == ExitReason::panic 
        || exit_reason == ExitReason::OutOfGas  
        || matches!(exit_reason, ExitReason::PageFault(page)) {

        return (exit_reason, pvm_ctx.pc, pvm_ctx.gas, pvm_ctx.reg, pvm_ctx.page_table, ctx);
    } 
    
    if exit_reason == ExitReason::HostCall(n.clone()) {

        let host_call_result = match ctx {
            HostCallContext::Accumulate(_, _) => {
                dispatch_accumulate((n.clone(), pvm_ctx.gas, pvm_ctx.reg, pvm_ctx.page_table.clone(), ctx.clone()))
            },
            _ => {
                HostCallResult::Err(0) 
            }
        };

        if let HostCallResult::Ok(hostcall_exit_reason, gas, reg, ram, ctx) = host_call_result {
            
            match hostcall_exit_reason {
                ExitReason::PageFault(hc_page_fault) => {
                    return (ExitReason::PageFault(hc_page_fault), pvm_ctx.pc, pvm_ctx.gas, pvm_ctx.reg, pvm_ctx.page_table, ctx);
                },
                ExitReason::OutOfGas => {
                    return (ExitReason::OutOfGas, pvm_ctx.pc, pvm_ctx.gas, pvm_ctx.reg, pvm_ctx.page_table, ctx);
                },
                ExitReason::Halt => {
                    return (ExitReason::Halt, pvm_ctx.pc, pvm_ctx.gas, pvm_ctx.reg, pvm_ctx.page_table, ctx);
                },
                ExitReason::panic => {
                    return (ExitReason::panic, pvm_ctx.pc, pvm_ctx.gas, pvm_ctx.reg, pvm_ctx.page_table, ctx);
                },
                ExitReason::Continue => {
                    return hostcall(program_blob, pvm_ctx.pc, (n, gas, reg, ram, ctx));
                },
                _ => {
                    return (hostcall_exit_reason, pvm_ctx.pc, pvm_ctx.gas, pvm_ctx.reg, pvm_ctx.page_table, ctx);
                }
            }
        }
    }

    return (exit_reason, pvm_ctx.pc, pvm_ctx.gas, pvm_ctx.reg, pvm_ctx.page_table, ctx);
}


fn hostcall_argument(std_program: &[u8], pc: RegSize, gas: Gas, a: &[u8], n: HostCallType, ctx: HostCallContext)
-> (Gas, Result<Vec<u8>, ExitReason>, HostCallContext) 
{
    let mut std_program_blob = BytesReader::new(std_program);
    let  decoded_std_program= ProgramFormat::decode(&mut std_program_blob);
    let std_program = match decoded_std_program {
        Ok(program) => program,
        Err(_) => return (gas, Err(ExitReason::panic), ctx),
    };

    //TODO
    let args = (n, gas, Registers::default(), PageTable::default(), ctx);
    R(gas, hostcall(std_program.c.as_slice(), pc, args))
}

#[allow(non_snake_case)]
fn R(gas: Gas, 
     hostcall_result: (ExitReason, RegSize, Gas, Registers, PageTable, HostCallContext)
) -> (Gas, Result<Vec<u8>, ExitReason>, HostCallContext) {

    let (exit_reason, _post_pc, post_gas, post_reg, post_ram, post_ctx) = hostcall_result;
    let gas_consumed = gas - std::cmp::max(post_gas, 0);

    if exit_reason == ExitReason::OutOfGas {
        return (gas_consumed, Err(ExitReason::OutOfGas), post_ctx);
    }

    let start_page = post_reg[7] as RamAddress / PAGE_SIZE;
    let end_page = (post_reg[7] + post_reg[8]) as RamAddress / PAGE_SIZE;

    if exit_reason == ExitReason::Halt {
        match is_readable(&post_ram, &start_page, &end_page) {
            Ok(readable) => {
                if readable {
                    let mut data = vec![];
                    for page_number in start_page..=end_page {
                        let page = post_ram.pages.get(&page_number).unwrap();
                        data.extend_from_slice(page.data.as_slice());
                    }
                    return (gas_consumed, Ok(data), post_ctx);
                } 

                return (gas_consumed, Ok(vec![]), post_ctx);
            },
            Err(error) => {
                //TODO que hago aqui?
                println!("run_hostcall Error: {:?}", error);
                return (gas_consumed, Ok(vec![]), post_ctx);
            }
        }
    }   

    return (gas_consumed, Err(ExitReason::panic), post_ctx);
}
