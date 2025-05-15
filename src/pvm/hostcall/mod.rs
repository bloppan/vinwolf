use std::collections::HashMap;

use crate::types::{
    Account, AccumulationContext, Context, DataSegment, ExitReason, Gas, HostCallFn, RamAddress, RamMemory, RefineMemory, 
    RegSize, Registers, WorkExecResult, WorkExecError
};

use crate::pvm::{invoke_pvm, hostcall::program_init::init_std_program};
use crate::utils::codec::ReadError;

pub mod accumulate; pub mod refine; pub mod on_transfer; pub mod is_authorized; pub mod general_fn; pub mod program_init;

/// An extended version of the pvm invocation which is able to progress an inner host-call
/// state-machine in the case of a host-call halt condition is defined as:
fn hostcall<F>(program_code: &[u8], pc: RegSize, gas: Gas, reg: Registers, ram: RamMemory, f: F, ctx: HostCallContext) 

-> (ExitReason, RegSize, Gas, Registers, RamMemory, HostCallContext)
where 
    F: Fn(HostCallFn, Gas, Registers, RamMemory, HostCallContext) -> (ExitReason, Gas, Registers, RamMemory, HostCallContext)
{
    println!("Hostcall");
    let mut pvm_ctx = Context::default();
    pvm_ctx.pc = pc;
    pvm_ctx.gas = gas;
    pvm_ctx.reg = reg;
    pvm_ctx.ram = ram;

    // On exit, the instruction counter references the instruction which caused the exit. Should the machine be invoked
    // again using this instruction counter and code, then the same instruction which caused the exit would be executed. This
    // is sensible when the instruction is one which necessarily needs re-executing such as in the case of an out-of-gas or page
    // fault reason.
    let exit_reason = invoke_pvm(&mut pvm_ctx, program_code);
    
    if exit_reason == ExitReason::Halt 
        || exit_reason == ExitReason::panic 
        || exit_reason == ExitReason::OutOfGas  
        || matches!(exit_reason, ExitReason::PageFault(_page)) {

        return (exit_reason, pvm_ctx.pc, pvm_ctx.gas, pvm_ctx.reg, pvm_ctx.ram, ctx);
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
        let (hostcall_exit_reason, 
             post_gas, 
             post_reg, 
             post_ram, 
             post_ctx) = f(n, pvm_ctx.gas.clone(), pvm_ctx.reg.clone(), pvm_ctx.ram.clone(), ctx.clone());

        match hostcall_exit_reason {
            ExitReason::PageFault(hostcall_page_fault) => {
                return (ExitReason::PageFault(hostcall_page_fault), pvm_ctx.pc, pvm_ctx.gas, pvm_ctx.reg, pvm_ctx.ram, ctx);
            },
            ExitReason::Continue => {
                return hostcall(program_code, pvm_ctx.pc, post_gas, post_reg, post_ram, f, post_ctx);
            },
            ExitReason::panic 
            | ExitReason::Halt
            | ExitReason::halt
            | ExitReason::OutOfGas => {
                println!("Hostcall exit reason: {:?}", hostcall_exit_reason);
                return (hostcall_exit_reason, pvm_ctx.pc, post_gas, post_reg, post_ram, post_ctx);
            },
            _ => {
                println!("Incorrect Hostcall exit reason: {:?}", hostcall_exit_reason);
                return (hostcall_exit_reason, pvm_ctx.pc, post_gas, post_reg, post_ram, post_ctx);
            }
        }
    }

    println!("Hostcall exit: {:?}", exit_reason);
    
    return (exit_reason, pvm_ctx.pc, pvm_ctx.gas, pvm_ctx.reg, pvm_ctx.ram, ctx);
}

/// The four instances where the pvm is utilized each expect to be able to pass argument data in and receive some return data back. 
/// We thus define the common pvm program-argument invocation function
fn hostcall_argument<F>(program_code: &[u8], pc: RegSize, gas: Gas, arg: &[u8], f: F, ctx: HostCallContext) 

-> (Gas, WorkExecResult, HostCallContext) 
where 
    F: Fn(HostCallFn, Gas, Registers, RamMemory, HostCallContext) -> (ExitReason, Gas, Registers, RamMemory, HostCallContext)
{
    println!("Hostcall argument");
    let std_program_decoded = match init_std_program(&program_code, &arg) {
        Ok(program) => {
            if program.is_none() {
                return (gas, WorkExecResult::Error(WorkExecError::Panic), ctx);
            }
            program.unwrap()
        }
        Err(_) => return (gas, WorkExecResult::Error(WorkExecError::Panic), ctx),
    };

    R(gas, hostcall(&std_program_decoded.code, pc, gas, std_program_decoded.reg, std_program_decoded.ram, f, ctx))
}

#[allow(non_snake_case)]
fn R(gas: Gas, hostcall_result: (ExitReason, RegSize, Gas, Registers, RamMemory, HostCallContext)

) -> (Gas, WorkExecResult, HostCallContext) {

    let (exit_reason, _post_pc, post_gas, post_reg, post_ram, post_ctx) = hostcall_result;
    let gas_consumed = gas - std::cmp::max(post_gas, 0);

    if exit_reason == ExitReason::OutOfGas {
        println!("R: Out of gas!!!");
        return (gas_consumed, WorkExecResult::Error(WorkExecError::OutOfGas), post_ctx);
    }

    let start_address = post_reg[7] as RamAddress;
    //let end_address = (post_reg[7] + post_reg[8]) as RamAddress;
    let bytes_to_read = post_reg[8] as RamAddress;

    if exit_reason == ExitReason::Halt || exit_reason == ExitReason::halt { // TODO cambiar esto
        if post_ram.is_readable(start_address, bytes_to_read) {
            let data = post_ram.read(start_address, post_reg[8] as u32);
            return (gas_consumed, WorkExecResult::Ok(data), post_ctx);
        } else {
            return (gas_consumed, WorkExecResult::Ok(vec![]), post_ctx);
        }
    }   

    return (gas_consumed, WorkExecResult::Error(WorkExecError::Panic), post_ctx);
}

use std::convert::TryFrom;
impl TryFrom<u8> for HostCallFn {
    type Error = ReadError;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0  => Ok(HostCallFn::Gas),
            1  => Ok(HostCallFn::Lookup),
            2  => Ok(HostCallFn::Read),
            3  => Ok(HostCallFn::Write),
            4  => Ok(HostCallFn::Info),
            5  => Ok(HostCallFn::Bless),
            6  => Ok(HostCallFn::Assign),
            7  => Ok(HostCallFn::Designate),
            8  => Ok(HostCallFn::Checkpoint),
            9  => Ok(HostCallFn::New),
            10 => Ok(HostCallFn::Upgrade),
            11 => Ok(HostCallFn::Transfer),
            12 => Ok(HostCallFn::Eject),
            13 => Ok(HostCallFn::Query),
            14 => Ok(HostCallFn::Solicit),
            15 => Ok(HostCallFn::Forget),
            16 => Ok(HostCallFn::Yield),
            17 => Ok(HostCallFn::HistoricalLookup),
            18 => Ok(HostCallFn::Fetch),
            19 => Ok(HostCallFn::Export),
            20 => Ok(HostCallFn::Machine),
            21 => Ok(HostCallFn::Peek),
            22 => Ok(HostCallFn::Poke),
            23 => Ok(HostCallFn::Zero),
            24 => Ok(HostCallFn::Void),
            25 => Ok(HostCallFn::Invoke),
            26 => Ok(HostCallFn::Expugne),
            27 => Ok(HostCallFn::Provide),
            100 => Ok(HostCallFn::Log),
            _  => Err(ReadError::InvalidData),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum HostCallContext {
    Accumulate(AccumulationContext, AccumulationContext),
    Refine(HashMap<usize, RefineMemory>, Vec<DataSegment>),
    OnTransfer(Account),
    IsAuthorized(),
}

/*pub fn is_writable(ram: &RamMemory, start_page: &RamAddress, end_page: &RamAddress) -> Result<bool, ExitReason> {
    
    for i in *start_page..=*end_page {

        if let Some(page) = ram.pages[i as usize].as_ref() {
            if page.flags.access.get(&RamAccess::Write).is_none() {
                return Err(ExitReason::panic);
            }
        } else {
            return Err(ExitReason::PageFault(i));
        }
    }

    return Ok(true);
}*/

mod tests {
    
    #[test]
    fn init_program_test() {

        use super::*;
        use crate::types::*;
        use crate::blockchain::state::services::decode_preimage;
        
        let program_blob_str = "0x09626f6f747374726170000000000000020000200032040000050283ae7900300194012f027802280d00000028ae00000028ab029511e07b10187b15107b1608648664783309043307000001ac967c9566fc5106769587047d7833050159083a8489ff003305025329c0002d3305035329e000253305045329f0001d3305055329f800153305065329fc000d8898fe009a850801ac564564587b175010029c026478e45607c95707d88709e48707c98707887720d479098217c8750594983307000001da95072805330801821018821510821608951120320000951150ff7b10a8007b15a0007b169800330908ac98e7003309fcaa97e5015107e101958af8957508510a457d583306015908408489ff003306025329c0002d3306035329e000253306045329f0001d3306055329f800153306065329fc000d8898fe009a860801ae6a092890003306017b166457646864a6501004e501821a51077be4a607c9a70753176072c85a089576a09587607b1751064c7d783305015908378489ff003305025329c0002d3305035329e000253305045329f0001d3305055329f800153305065329fc000d8898fe009a850801ac562a016458501006810128073305330701e45608c95808e47808c97808330921ae981d33083307000001018210a8008215a000821698009511b00032008219c89505c857077c792051090933083307286e958adf957521510a547d573306015907378477ff003306025327c0002d3306035327e000253306045327f0001d3306055327f800153306065327fc000d8877fe009a7608017b1aac6a920064576468501008e6006478821a28073306330801c86507e46a09c96909e6890801c878088088fc330964330a640a0964757b1708481114951714330804951908330a040a0395171833098000330850100a4a330820a107330964951a1864570a0b81180833070000023b080000029889183b090300029889103b090200029888083b080100023308202806ff0000003307000001330832008d7a84aa07c8a70b510a0e647c0178c895cc01acbcfbc9a903843cf8c8cb0a580c1d8482ff0014090101010101010101ca920c017bbc95bb08acabfb843907520905280ec8a9090178a895aa01ac9afb320051089b0064797c77510791005127ff0090006c7a570a09330a330828735527c0000d330a01330b80284a5527e0000e330a02330b40ff283c5527f0000e330a03330b20ff282e5527f8000e330a04330b10ff28205527fc000e330a05330b08ff2812887afe00330b04ff93ab02ff85aa0701ae8a2b3308c8b70764ab01c8b90c7ccc97880895bbffd4c808520bf28aa903cf9707c88707320032000000002124492a21494a22212121212132154a9224a5909a248d88482422494924242424244426ad0a258924a524121212121222a3504d92a43022a292a44a52120909090909914585aa26c924a924494a1421a984909090903c54495a92241140962465495111942a24854421514814124544a6342549923a";
        let program_blob = hex::decode(&program_blob_str[2..]).unwrap();

        let preimage_data: PreimageData = decode_preimage(&program_blob).unwrap();
        let std_program = init_std_program(&preimage_data.code, &[]);

        match std_program {
            Ok(Some(program)) => {
                println!("Program initialized successfully");
                println!("RAM: \n");
                for i in 0..program.ram.pages.len() {
                    if let Some(page) = &program.ram.pages[i] {
                        println!("Page {}: {:?}", i, page.data);
                    }
                }
                println!("\nRegisters: {:?}", program.reg);
                //println!("Code: {:?}", program.code);
            }
            Ok(None) => {
                println!("Program initialization failed");
            }
            Err(e) => {
                println!("Error initializing program: {:?}", e);
            }
        }

    }
}