
use crate::types::{Account, DeferredTransfer, ExitReason, Gas, Balance, HostCallFn, RamMemory, Registers, ServiceId, TimeSlot, ServiceAccounts};
use crate::blockchain::state::services::decode_preimage;
use crate::pvm::hostcall::{hostcall_argument, HostCallContext};
use crate::utils::codec::Encode;

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

pub fn on_transfer_dispatcher(n: HostCallFn, gas: Gas, reg: Registers, ram: RamMemory, ctx: HostCallContext) 

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


