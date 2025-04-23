use std::collections::{HashMap, HashSet};

use sp_core::blake2_256;
use sp_core::ecdsa::PUBLIC_KEY_SERIALIZED_SIZE;

use crate::blockchain::state::entropy::get_recent_entropy;
use crate::blockchain::state::{get_entropy, get_time};
use crate::types::{
    Account, AccumulationContext, AccumulationOperand, AccumulationPartialState, DeferredTransfer, ExitReason, Gas, Hash, Balance,
    HostCallFn, OpaqueHash, RamAddress, RamMemory, RegSize, Registers, ServiceId, TimeSlot, WorkExecResult, ServiceAccounts
};
use crate::constants::{CASH, CORE, FULL, HUH, LOW, NONE, OK, OOB, PAGE_SIZE, VALIDATORS_COUNT, WHAT, WHO, TRANSFER_MEMO_SIZE};
use crate::utils::codec::{Encode, DecodeSize, BytesReader};
use crate::pvm::hostcall::{hostcall_argument, is_readable, HostCallContext};
use crate::pvm::hostcall::general_functions::info;
use crate::blockchain::state::services::{decode_preimage, historical_preimage_lookup};
use crate::utils::common;


pub fn invoke_on_transfer(
    service_accounts: &ServiceAccounts, 
    slot: &TimeSlot, 
    service_id: &ServiceId, 
    transfers: Vec<DeferredTransfer>) 
-> (Account, Gas) {
    
    println!("Invoke on transfer");
    println!("Service ID: {:?}", service_id);
    let mut s_account = service_accounts.service_accounts.get(service_id).unwrap().clone();
    
    if transfers.is_empty() {
        println!("No transfers");
        return (s_account, 0);
    }

    s_account.balance += transfers.iter().map(|transfer| transfer.amount).sum::<Balance>();

    if let Some(preimage_code) = s_account.preimages.get(&s_account.code_hash) {

        let preimage_data = decode_preimage(&preimage_code).unwrap(); // TODO handle error

        let gas = transfers.iter().map(|transfer| transfer.gas_limit).sum::<Gas>();
        let arg = [slot.encode(), service_id.encode(), transfers.encode()].concat();

        let (gas_used, 
             _result, 
             ctx) = hostcall_argument(&preimage_data.code, 10, gas, &arg, on_transfer_dispatcher, HostCallContext::OnTransfer(s_account.clone()));
    
        let HostCallContext::OnTransfer(modified_account) = ctx else {
            unreachable!("Invalid context");
        };

        return (modified_account, gas_used);
    }
    println!("No preimage code found");
    return (s_account, 0);
}

pub fn on_transfer_dispatcher(n: HostCallFn, mut gas: Gas, mut reg: Registers, mut ram: RamMemory, mut ctx: HostCallContext) 

-> (ExitReason, Gas, Registers, RamMemory, HostCallContext) 
{

    match n {
        HostCallFn::Lookup => {
            println!("Lookup");
        }
        HostCallFn::Read => {
            println!("Read");
        }
        HostCallFn::Write => {
            println!("Write");
        }
        HostCallFn::Gas => {
            println!("Gas");
        }
        HostCallFn::Info => {
            println!("Info");
        }
        _ => {
            println!("Unknown on transfer hostcall function");
        }
    }

    return (ExitReason::Continue, gas, reg, ram, ctx);
}


