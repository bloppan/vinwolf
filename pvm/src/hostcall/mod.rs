use std::collections::HashMap;

use jam_types::{Account, AccumulationContext, DataSegment,WorkExecResult, WorkExecError};
use utils::log;
use crate::pvm_types::{Program, ExitReason, HostCallFn, RamAddress, RamMemory, RegSize, Registers, RefineMemory, Gas};
use codec::{BytesReader, Decode};
use crate::{pvmi::invoke_pvm, mm::program_init::init_std_program};

pub mod accumulate; pub mod refine; pub mod on_transfer; pub mod is_authorized; pub mod general_fn;

/// An extended version of the pvm invocation which is able to progress an inner host-call
/// state-machine in the case of a host-call halt condition is defined as:
fn hostcall<F>(
    program_code: &Program, 
    pc: &mut RegSize, 
    gas: &mut Gas, 
    reg: &mut Registers, 
    ram: &mut RamMemory, 
    dispatch_hostcall: F, 
    ctx: &mut HostCallContext

) -> (ExitReason, Gas)
where 
    F: for<'m, 'c, 'r, 'g> Fn(HostCallFn, &'g mut Gas, &'r mut Registers, &'m mut RamMemory, &'c mut HostCallContext) -> ExitReason
{
    log::debug!("Execute hostcall. gas: {gas}");

    loop {
        // On exit, the instruction counter references the instruction which caused the exit. Should the machine be invoked
        // again using this instruction counter and code, then the same instruction which caused the exit would be executed. This
        // is sensible when the instruction is one which necessarily needs re-executing such as in the case of an out-of-gas or page
        // fault reason.
        let exit_reason = invoke_pvm(program_code, pc, gas, ram, reg);
        
        if exit_reason == ExitReason::Halt 
            || exit_reason == ExitReason::Panic 
            || exit_reason == ExitReason::OutOfGas  
            || matches!(exit_reason, ExitReason::PageFault(_page)) {
            
            log::error!("Exit reason: {:?}", exit_reason);
            //return (exit_reason, pvm_ctx.pc, pvm_ctx.gas, pvm_ctx.reg, pvm_ctx.ram);
            return (exit_reason, *gas);
        } 
        
        if let ExitReason::HostCall(n) = exit_reason {
            // However, when the exit reason to hostcall is a hostcall, then the resultant instruction-counter has a value of the hostcall 
            // instruction and resuming with this state would immediately exit with the same result. Re-invoking would therefore require 
            // both the post-host-call machine state and the instruction counter value for the instruction following the one which resulted 
            // in the host-call exit reason. This is always one greater plus the relevant argument skip distance. Resuming the machine with 
            // this instruction counter will continue beyond the host-call instruction.
            // We use both values of instruction-counter for the definition of ΨH since if the host-call results in a page fault we need
            // to allow the outer environment to resolve the fault and re-try the host-call. Conversely, if we successfully transition state
            // according to the host-call, then on resumption we wish to begin with the instruction directly following the host-call.
            let hostcall_exit_reason = dispatch_hostcall(n, gas, reg, ram, ctx);

            match hostcall_exit_reason {
                ExitReason::PageFault(hostcall_page_fault) => {
                    log::error!("Page fault: {:?}", hostcall_page_fault);
                    //return (ExitReason::PageFault(hostcall_page_fault), pvm_ctx.pc, pvm_ctx.gas, pvm_ctx.reg, pvm_ctx.ram);
                    return (ExitReason::PageFault(hostcall_page_fault), *gas);
                },
                ExitReason::Continue => {
                    continue;
                },
                ExitReason::Panic 
                | ExitReason::Halt
                | ExitReason::OutOfGas => {
                    log::error!("Hostcall exit reason: {:?}", hostcall_exit_reason);
                    //return (hostcall_exit_reason, pvm_ctx.pc, pvm_ctx.gas, pvm_ctx.reg, pvm_ctx.ram);
                    return (hostcall_exit_reason, *gas);
                },
                _ => {
                    log::error!("Incorrect hostcall exit reason: {:?}", hostcall_exit_reason);
                    //return (hostcall_exit_reason, pvm_ctx.pc, pvm_ctx.gas, pvm_ctx.reg, pvm_ctx.ram);
                    return (hostcall_exit_reason, *gas);
                }
            }
        }

        log::debug!("Hostcall exit: {:?}", exit_reason);   
        //return (exit_reason, pvm_ctx.pc, pvm_ctx.gas, pvm_ctx.reg, pvm_ctx.ram);
        return (exit_reason, *gas);
    }
}

/// The four instances where the pvm is utilized each expect to be able to pass argument data in and receive some return data back. 
/// We thus define the common pvm program-argument invocation function
fn hostcall_argument<F>(program_code: &[u8], mut pc: RegSize, mut gas: Gas, arg: &[u8], f: F, ctx: &mut HostCallContext) 

-> (Gas, WorkExecResult) 
where 
    F: for<'m, 'c, 'r, 'g> Fn(HostCallFn, &'g mut Gas, &'r mut Registers, &'m mut RamMemory, &'c mut HostCallContext) -> ExitReason
{
    log::debug!("Execute hostcall argument function");
    let mut std_program_decoded = match init_std_program(&program_code, &arg) {
        Ok(program) => {
            if program.is_none() {
                log::error!("Panic: Program decoded is none");
                return (gas, WorkExecResult::Error(WorkExecError::Panic));
            }
            program.unwrap()
        }
        Err(_) => {
            log::error!("Panic: Failed to decode the program");
            return (gas, WorkExecResult::Error(WorkExecError::Panic));
        },
    };

    let program = match Program::decode(&mut BytesReader::new(&std_program_decoded.code)) {
        Ok(program) => { program },
        Err(_) => { 
            log::error!("Panic: Decoding code program");
            return (gas, WorkExecResult::Error(WorkExecError::Panic)); 
        }
    };

    let gas_init = gas;
    log::info!("gas init: {gas_init}");

    R(gas_init, hostcall(&program, &mut pc, &mut gas, &mut std_program_decoded.reg, &mut std_program_decoded.ram, f, ctx), &std_program_decoded.reg, &std_program_decoded.ram)
}

#[allow(non_snake_case)]
fn R(gas_init: Gas, hostcall_result: (ExitReason, Gas), reg: &Registers, ram: &RamMemory) -> (Gas, WorkExecResult) {

    //let (exit_reason, _post_pc, post_gas, post_reg, post_ram) = hostcall_result;
    let (exit_reason, post_gas) = hostcall_result;
    log::info!("post_gas: {post_gas}, gas_init: {gas_init}");
    let gas_consumed = gas_init - std::cmp::max(post_gas, 0);
    
    if exit_reason == ExitReason::OutOfGas {
        log::error!("R: Out of gas!");
        return (gas_consumed, WorkExecResult::Error(WorkExecError::OutOfGas));
    }

    let start_address = reg[7] as RamAddress;
    let bytes_to_read = reg[8] as RamAddress;

    if exit_reason == ExitReason::Halt {
        if ram.is_readable(start_address, bytes_to_read).is_ok() {
            let data = ram.read(start_address, reg[8] as u32);
            log::debug!("The ram is readable after halt");
            return (gas_consumed, WorkExecResult::Ok(data));
        } else {
            log::error!("The ram is not readable after halt");
            return (gas_consumed, WorkExecResult::Ok(vec![]));
        }
    }   

    log::error!("R: Work exec result panic for exit reason {:?}", exit_reason);
    return (gas_consumed, WorkExecResult::Error(WorkExecError::Panic));
}

impl HostCallContext {

    pub fn to_acc_ctx(&mut self) -> (&mut AccumulationContext, &mut AccumulationContext) {
        match self {
            HostCallContext::Accumulate(ref mut x, ref mut y) => (x, y),
            _ => unreachable!("to_acc_ctx: invalid acc context"),
        }
    }

    pub fn to_acc_ctx_ro(&self) -> (&AccumulationContext, &AccumulationContext) {
        match self {
            HostCallContext::Accumulate(ref x, ref y) => (x, y),
            _ => unreachable!("to_acc_ctx_ro: invalid acc context"),
        }
    }

    pub fn to_xfer_ctx(&mut self) -> &mut Option<Account> {
        match self {
            HostCallContext::OnTransfer(ref mut account) => account,
            _ => unreachable!("to_acc_ctx: invalid xfer context"),
        }
    }

    pub fn to_xfer_ctx_ro(&self) -> &Option<Account> {
        match self {
            HostCallContext::OnTransfer(account) => account,
            _ => unreachable!("to_acc_ctx: invalid xfer context"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum HostCallContext {
    Accumulate(AccumulationContext, AccumulationContext),
    Refine(HashMap<usize, RefineMemory>, Vec<DataSegment>),
    OnTransfer(Option<Account>),
    IsAuthorized(),
}
