use std::collections::HashMap;

use crate::types::{
    Account, AccumulationContext, Context, DataSegment, ExitReason, Gas, HostCallType, PageTable, ProgramFormat, RamAccess, RamAddress, RamMemory, RefineMemory, RegSize, Registers
};

use crate::constants::PAGE_SIZE;
use crate::pvm::invoke_pvm;
use crate::utils::codec::{Decode, BytesReader};
use crate::utils::codec::generic::decode_unsigned;
use crate::pvm::hostcall::program_init::init_std_program;

mod accumulate;
mod refine;
mod on_transfer;
mod is_authorized;
pub mod general_functions;
mod program_init;

use accumulate::dispatch_accumulate;

pub fn is_writable(ram: &RamMemory, start_page: &RamAddress, end_page: &RamAddress) -> Result<bool, ExitReason> {
    
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
}

pub fn is_readable(ram: &RamMemory, start_page: &RamAddress, end_page: &RamAddress) -> Result<bool, ExitReason> {
    
    for i in *start_page..=*end_page {

        if let Some(page) = ram.pages[i as usize].as_ref() {
            if page.flags.access.get(&RamAccess::Read).is_none() {
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

pub type HostCallArgs = (HostCallType, Gas, Registers, RamMemory, HostCallContext);

pub enum HostCallResult {
    Ok(ExitReason, Gas, Registers, RamMemory, HostCallContext),
    Err(usize)
}

fn hostcall(program_blob: &[u8], pc: RegSize, args: HostCallArgs) -> (ExitReason, RegSize, Gas, Registers, RamMemory, HostCallContext) {

    let (n, mut gas, mut reg, mut ram, mut ctx) = args;

    let mut pvm_ctx = Context::default();
    pvm_ctx.pc = pc;
    pvm_ctx.gas = gas;
    pvm_ctx.reg = reg;
    pvm_ctx.ram = ram;

    let exit_reason = invoke_pvm(&mut pvm_ctx, program_blob);

    if exit_reason == ExitReason::Halt 
        || exit_reason == ExitReason::panic 
        || exit_reason == ExitReason::OutOfGas  
        || matches!(exit_reason, ExitReason::PageFault(page)) {

        return (exit_reason, pvm_ctx.pc, pvm_ctx.gas, pvm_ctx.reg, pvm_ctx.ram, ctx);
    } 
    
    if exit_reason == ExitReason::HostCall(n.clone()) {

        let host_call_result = match ctx {
            HostCallContext::Accumulate(_, _) => {
                dispatch_accumulate((n.clone(), pvm_ctx.gas, pvm_ctx.reg, pvm_ctx.ram.clone(), ctx.clone()))
            },
            _ => {
                HostCallResult::Err(0) 
            }
        };

        if let HostCallResult::Ok(hostcall_exit_reason, gas, reg, ram, ctx) = host_call_result {
            
            match hostcall_exit_reason {
                ExitReason::PageFault(hc_page_fault) => {
                    return (ExitReason::PageFault(hc_page_fault), pvm_ctx.pc, pvm_ctx.gas, pvm_ctx.reg, pvm_ctx.ram, ctx);
                },
                ExitReason::OutOfGas => {
                    return (ExitReason::OutOfGas, pvm_ctx.pc, pvm_ctx.gas, pvm_ctx.reg, pvm_ctx.ram, ctx);
                },
                ExitReason::Halt => {
                    return (ExitReason::Halt, pvm_ctx.pc, pvm_ctx.gas, pvm_ctx.reg, pvm_ctx.ram, ctx);
                },
                ExitReason::panic => {
                    return (ExitReason::panic, pvm_ctx.pc, pvm_ctx.gas, pvm_ctx.reg, pvm_ctx.ram, ctx);
                },
                ExitReason::Continue => {
                    return hostcall(program_blob, pvm_ctx.pc, (n, gas, reg, ram, ctx));
                },
                _ => {
                    return (hostcall_exit_reason, pvm_ctx.pc, pvm_ctx.gas, pvm_ctx.reg, pvm_ctx.ram, ctx);
                }
            }
        }
    }

    return (exit_reason, pvm_ctx.pc, pvm_ctx.gas, pvm_ctx.reg, pvm_ctx.ram, ctx);
}


fn hostcall_argument(program_blob: &[u8], pc: RegSize, gas: Gas, arg: &[u8], n: HostCallType, ctx: HostCallContext)
-> (Gas, Result<Vec<u8>, ExitReason>, HostCallContext) 
{
    struct AccountCode {
        metadata: Vec<u8>,
        code: Vec<u8>,
    }

    let mut reader = BytesReader::new(&program_blob);
    let metadata_len = decode_unsigned(&mut reader).unwrap();
    let _metadata = reader.read_bytes(metadata_len as usize).unwrap();
    let code = reader.read_bytes(reader.data.len() - metadata_len as usize - 1).unwrap();
    
    let std_program = init_std_program(&code, &arg);

    let program_decoded = match std_program {
        Ok(program) => {
            if program.is_none() {
                return (gas, Err(ExitReason::panic), ctx);
            }
            program.unwrap()
        }
        Err(_) => return (gas, Err(ExitReason::panic), ctx),
    };

    let args = (n, gas, program_decoded.reg, program_decoded.ram, ctx);
    R(gas, hostcall(&program_decoded.code, pc, args))
}

#[allow(non_snake_case)]
fn R(gas: Gas, 
     hostcall_result: (ExitReason, RegSize, Gas, Registers, RamMemory, HostCallContext)
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
                        let page = post_ram.pages[page_number as usize].as_ref().unwrap();
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


mod tests {
    use super::*;
    use crate::types::StandardProgram;
    use crate::utils::codec::generic::decode;
    use crate::pvm::isa::skip;
    use crate::pvm::isa::one_reg_one_ext_imm::load_imm_64;

    #[test]
    fn test_init_program() {
        let program_blob_str = "0x09626f6f747374726170000000000000020000200032040000050283ae7900300194012f027802280d00000028ae00000028ab029511e07b10187b15107b1608648664783309043307000001ac967c9566fc5106769587047d7833050159083a8489ff003305025329c0002d3305035329e000253305045329f0001d3305055329f800153305065329fc000d8898fe009a850801ac564564587b175010029c026478e45607c95707d88709e48707c98707887720d479098217c8750594983307000001da95072805330801821018821510821608951120320000951150ff7b10a8007b15a0007b169800330908ac98e7003309fcaa97e5015107e101958af8957508510a457d583306015908408489ff003306025329c0002d3306035329e000253306045329f0001d3306055329f800153306065329fc000d8898fe009a860801ae6a092890003306017b166457646864a6501004e501821a51077be4a607c9a70753176072c85a089576a09587607b1751064c7d783305015908378489ff003305025329c0002d3305035329e000253305045329f0001d3305055329f800153305065329fc000d8898fe009a850801ac562a016458501006810128073305330701e45608c95808e47808c97808330921ae981d33083307000001018210a8008215a000821698009511b00032008219c89505c857077c792051090933083307286e958adf957521510a547d573306015907378477ff003306025327c0002d3306035327e000253306045327f0001d3306055327f800153306065327fc000d8877fe009a7608017b1aac6a920064576468501008e6006478821a28073306330801c86507e46a09c96909e6890801c878088088fc330964330a640a0964757b1708481114951714330804951908330a040a0395171833098000330850100a4a330820a107330964951a1864570a0b81180833070000023b080000029889183b090300029889103b090200029888083b080100023308202806ff0000003307000001330832008d7a84aa07c8a70b510a0e647c0178c895cc01acbcfbc9a903843cf8c8cb0a580c1d8482ff0014090101010101010101ca920c017bbc95bb08acabfb843907520905280ec8a9090178a895aa01ac9afb320051089b0064797c77510791005127ff0090006c7a570a09330a330828735527c0000d330a01330b80284a5527e0000e330a02330b40ff283c5527f0000e330a03330b20ff282e5527f8000e330a04330b10ff28205527fc000e330a05330b08ff2812887afe00330b04ff93ab02ff85aa0701ae8a2b3308c8b70764ab01c8b90c7ccc97880895bbffd4c808520bf28aa903cf9707c88707320032000000002124492a21494a22212121212132154a9224a5909a248d88482422494924242424244426ad0a258924a524121212121222a3504d92a43022a292a44a52120909090909914585aa26c924a924494a1421a984909090903c54495a92241140962465495111942a24854421514814124544a6342549923a";
        
        use crate::utils::codec::{Decode, BytesReader};
        use crate::utils::codec::generic::decode_unsigned;
        use crate::pvm::hostcall::program_init::init_std_program;

        let program_blob = hex::decode(&program_blob_str[2..]).unwrap();
        
        struct AccountCode {
            metadata: Vec<u8>,
            code: Vec<u8>,
        }

        let mut reader = BytesReader::new(&program_blob);
        let metadata_len = decode_unsigned(&mut reader).unwrap();
        let metadata = reader.read_bytes(metadata_len as usize).unwrap();
        let code = reader.read_bytes(reader.data.len() - metadata_len as usize - 1).unwrap();

        let std_program = init_std_program(&code, &[]);

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