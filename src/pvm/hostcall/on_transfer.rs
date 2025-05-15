use {once_cell::sync::Lazy, std::sync::Mutex};

use crate::types::{Account, DeferredTransfer, ExitReason, Gas, Balance, HostCallFn, RamMemory, Registers, ServiceId, TimeSlot, ServiceAccounts};
use crate::constants::WHAT;
use crate::blockchain::state::services::decode_preimage;
use crate::pvm;
use super::general_fn::{write, info, read, lookup};
use crate::pvm::hostcall::{hostcall_argument, HostCallContext};
use crate::utils::codec::Encode;

static SERVICE_ACCOUNTS: Lazy<Mutex<ServiceAccounts>> = Lazy::new(|| {
    Mutex::new(ServiceAccounts::default())
});

static ACCOUNT: Lazy<Mutex<Account>> = Lazy::new(|| {
    Mutex::new(Account::default())
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
fn set_account(account: Account) {
    *ACCOUNT.lock().unwrap() = account;
}
fn get_account() -> &'static Mutex<Account> {
    &ACCOUNT
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
        set_service_accounts(service_accounts.clone());
        set_account(s_account.clone());
        set_service(service_id.clone());
        println!("\nexec hostcall_argument\n");
        let (gas_used, 
             _result, 
             ctx) = hostcall_argument(&preimage_data.code, 10, gas, &arg, on_transfer_dispatcher, HostCallContext::OnTransfer(s_account.clone()));
    
        //println!("ctx storage: {:x?}", ctx)
        let HostCallContext::OnTransfer(modified_account) = ctx else {
            unreachable!("Invalid context");
        };

        return (modified_account, gas_used);
    }
    println!("No preimage code found");
    return (s_account, 0);
}

pub fn on_transfer_dispatcher(n: HostCallFn, mut gas: Gas, mut reg: Registers, ram: RamMemory, ctx: HostCallContext) 

-> (ExitReason, Gas, Registers, RamMemory, HostCallContext) 
{
    println!("ON TRANSFER DISPATCHER!!! ");
    let service_accounts = get_service_accounts().lock().unwrap().clone();
    let service_id = get_service().lock().unwrap().clone();

    match n {
        HostCallFn::Lookup => {
            println!("Lookup");
            let HostCallContext::OnTransfer(account) = ctx else {
                unreachable!("On transfer dispatcher: Invalid context");
            };
            let (exit_reason, gas, reg, ram, modified_account) = lookup(gas, reg, ram, account, service_id, service_accounts);
            return (exit_reason, gas, reg, ram, HostCallContext::OnTransfer(modified_account));
        }
        HostCallFn::Read => {
            println!("Read");
            let HostCallContext::OnTransfer(account) = ctx else {
                unreachable!("On transfer dispatcher: Invalid context");
            };
            let (exit_reason, gas, reg, ram, modified_account) = read(gas, reg, ram, account, service_id, service_accounts);
            return (exit_reason, gas, reg, ram, HostCallContext::OnTransfer(modified_account));
        }
        HostCallFn::Write => {
            println!("Write");
            let HostCallContext::OnTransfer(account) = ctx else {
                unreachable!("On transfer dispatcher: Invalid context");
            };
            let (exit_reason, gas, reg, ram, modified_account) = write(gas, reg, ram, account, service_id);
            return (exit_reason, gas, reg, ram, HostCallContext::OnTransfer(modified_account));
        }
        HostCallFn::Gas => {
            println!("Gas");
            let (exit_reason, gas, reg, ram, ctx) = pvm::hostcall::general_fn::gas(gas, reg, ram, ctx);
            let HostCallContext::OnTransfer(account) = ctx else {
                unreachable!("On transfer dispatcher: Invalid context");
            };
            return (exit_reason, gas, reg, ram, HostCallContext::OnTransfer(account));
        }
        HostCallFn::Info => {
            println!("Info");
            let (exit_reason, gas, reg, ram, modified_account) = info(gas, reg, ram, service_id, service_accounts);
            return (exit_reason, gas, reg, ram, HostCallContext::OnTransfer(modified_account));
        }
        HostCallFn::Log => {
            println!("Log");
            let HostCallContext::OnTransfer(account) = ctx else {
                unreachable!("On transfer dispatcher: Invalid context");
            };
            return (ExitReason::Continue, gas, reg, ram, HostCallContext::OnTransfer(account));
        }
        _ => {
            println!("Unknown on transfer hostcall function: {:?}", n);
            gas -= 10;
            reg[7] = WHAT;
            let HostCallContext::OnTransfer(account) = ctx else {
                unreachable!("On transfer dispatcher: Invalid context");
            };
            return (ExitReason::Continue, gas, reg, ram, HostCallContext::OnTransfer(account));
        }
    }
}


