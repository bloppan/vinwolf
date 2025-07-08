use {once_cell::sync::Lazy, std::sync::Mutex};

use crate::types::{Account, DeferredTransfer, ExitReason, Gas, Balance, HostCallFn, RamMemory, Registers, ServiceId, TimeSlot, ServiceAccounts};
use crate::constants::{WHAT, MAX_SERVICE_CODE_SIZE};
use crate::blockchain::state::services::{decode_preimage, parse_preimage};
use crate::pvm;
use crate::utils::codec::generic::encode_unsigned;
use super::general_fn::{write, info, read, lookup};
use crate::pvm::hostcall::{hostcall_argument, HostCallContext};
use crate::utils::codec::{Encode, EncodeLen};

static SERVICE_ACCOUNTS: Lazy<Mutex<ServiceAccounts>> = Lazy::new(|| {
    Mutex::new(ServiceAccounts::default())
});

static SERVICE_ID: Lazy<Mutex<ServiceId>> = Lazy::new(|| {
    Mutex::new(ServiceId::default())
});

fn set_service_accounts(service_accounts: ServiceAccounts) {
    *SERVICE_ACCOUNTS.lock().unwrap() = service_accounts;
}
fn get_service_accounts() -> &'static Mutex<ServiceAccounts> {
    &SERVICE_ACCOUNTS
}
fn set_service(id: ServiceId) {
    *SERVICE_ID.lock().unwrap() = id;
}
fn get_service() -> &'static Mutex<ServiceId> {
    &SERVICE_ID
}

pub fn invoke_on_transfer(
    service_accounts: &ServiceAccounts, 
    slot: &TimeSlot, 
    service_id: &ServiceId, 
    transfers: Vec<DeferredTransfer>) 
-> (Account, Gas) {
    
    //println!("Invoke on transfer");
    //println!("Service ID: {:?}", service_id);
    let mut s_account = service_accounts.get(service_id).unwrap().clone();
    
    if transfers.is_empty() {
        //println!("No transfers");
        return (s_account, 0);
    }
    //println!("Hay transfers!");
    s_account.balance += transfers.iter().map(|transfer| transfer.amount).sum::<Balance>();

    if let Some(preimage_blob) = s_account.preimages.get(&s_account.code_hash) {

        let preimage = match decode_preimage(&preimage_blob) {
            Ok(preimage_data) => { preimage_data },
            Err(_) => { return (s_account, 0); },
        };

        if preimage.code.len() > MAX_SERVICE_CODE_SIZE {
            return (s_account, 0);
        }

        let gas = transfers.iter().map(|transfer| transfer.gas_limit).sum::<Gas>();
        let arg = [encode_unsigned(*slot as usize), encode_unsigned(*service_id as usize), encode_unsigned(transfers.len())].concat();
        set_service_accounts(service_accounts.clone());
        set_service(service_id.clone());
        let (gas_used, 
             _result, 
             ctx) = hostcall_argument(&preimage.code, 10, gas, &arg, dispatch_xfer, HostCallContext::OnTransfer(s_account.clone()));
    
        //println!("ctx storage: {:x?}", ctx)
        let HostCallContext::OnTransfer(modified_account) = ctx else {
            unreachable!("Invalid context");
        };

        return (modified_account, gas_used);
    }

    return (s_account, 0);
}

fn dispatch_xfer(n: HostCallFn, mut gas: Gas, mut reg: Registers, ram: RamMemory, ctx: HostCallContext) 

-> (ExitReason, Gas, Registers, RamMemory, HostCallContext) 
{
    //println!("ON TRANSFER DISPATCHER!!! ");
    let service_accounts = get_service_accounts().lock().unwrap().clone();
    let service_id = get_service().lock().unwrap().clone();

    match n {
        HostCallFn::Lookup => {
            //println!("Lookup");
            let account = ctx.to_xfer_ctx();
            let (exit_reason, gas, reg, ram, modified_account) = lookup(gas, reg, ram, account, service_id, service_accounts);
            return (exit_reason, gas, reg, ram, HostCallContext::OnTransfer(modified_account));
        }
        HostCallFn::Read => {
            //println!("Read");
            let account = ctx.to_xfer_ctx();
            let (exit_reason, gas, reg, ram, modified_account) = read(gas, reg, ram, account, service_id, service_accounts);
            return (exit_reason, gas, reg, ram, HostCallContext::OnTransfer(modified_account));
        }
        HostCallFn::Write => {
            //println!("Write");
            let account = ctx.to_xfer_ctx();
            let (exit_reason, gas, reg, ram, modified_account) = write(gas, reg, ram, account, service_id);
            return (exit_reason, gas, reg, ram, HostCallContext::OnTransfer(modified_account));
        }
        HostCallFn::Gas => {
            //println!("Gas");
            let (exit_reason, gas, reg, ram, ctx) = pvm::hostcall::general_fn::gas(gas, reg, ram, ctx);
            let account = ctx.to_xfer_ctx();
            return (exit_reason, gas, reg, ram, HostCallContext::OnTransfer(account));
        }
        HostCallFn::Info => {
            //println!("Info");
            let (exit_reason, gas, reg, ram, modified_account) = info(gas, reg, ram, service_id, service_accounts);
            return (exit_reason, gas, reg, ram, HostCallContext::OnTransfer(modified_account));
        }
        HostCallFn::Log => {
            //println!("Log");
            let account = ctx.to_xfer_ctx();
            return (ExitReason::Continue, gas, reg, ram, HostCallContext::OnTransfer(account));
        }
        _ => {
            println!("Unknown on transfer hostcall function: {:?}", n);
            gas -= 10;
            reg[7] = WHAT;
            let account = ctx.to_xfer_ctx();
            return (ExitReason::Continue, gas, reg, ram, HostCallContext::OnTransfer(account));
        }
    }
}


