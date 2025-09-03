use {once_cell::sync::Lazy, std::sync::Mutex};

use jam_types::{Account, DeferredTransfer, Gas, Balance, ServiceId, TimeSlot, ServiceAccounts, StateKeyType};
use crate::pvm_types::{ExitReason, HostCallFn, RamMemory, Registers};
use constants::pvm::WHAT;
use constants::node::MAX_SERVICE_CODE_SIZE;
use utils::common::decode_preimage;
use utils::serialization::{StateKeyTrait, construct_preimage_key};
use codec::generic_codec::encode_unsigned;
use super::general_fn::{write, info, read, lookup};
use crate::hostcall::{hostcall_argument, HostCallContext};

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
    
    log::debug!("Invoke on transfer for service {:?}", *service_id);
    let mut s_account = service_accounts.get(service_id).unwrap().clone();
    
    if transfers.is_empty() {
        log::debug!("No transfers");
        return (s_account, 0);
    }
    
    s_account.balance += transfers.iter().map(|transfer| transfer.amount).sum::<Balance>();
    let preimage_key = StateKeyType::Account(*service_id, construct_preimage_key(&s_account.code_hash)).construct();

    if let Some(preimage_blob) = s_account.storage.get(&preimage_key) {

        let preimage = match decode_preimage(&preimage_blob) {
            Ok(preimage_data) => { preimage_data },
            Err(_) => { 
                log::error!("Failed to decode preimage");
                return (s_account, 0); 
            },
        };

        if preimage.code.len() > MAX_SERVICE_CODE_SIZE {
            log::error!("The preimage code len is greater than the max service code size allowed");
            return (s_account, 0);
        }

        let gas = transfers.iter().map(|transfer| transfer.gas_limit).sum::<Gas>();
        let arg = [encode_unsigned(*slot as usize), encode_unsigned(*service_id as usize), encode_unsigned(transfers.len())].concat();
        set_service_accounts(service_accounts.clone());
        set_service(service_id.clone());
        let mut ctx = HostCallContext::OnTransfer(Some(s_account.clone()));

        let (gas_used, 
             _result) = hostcall_argument(&preimage.code, 10, gas, &arg, dispatch_xfer, &mut ctx);
    
        let HostCallContext::OnTransfer(modified_account) = ctx else {
            unreachable!("Invalid context");
        };

        log::debug!("Exit on transfer invokation");
        return (modified_account.unwrap(), gas_used);
    }

    log::error!("Preimage key {} code hash {} not found", hex::encode(&preimage_key), hex::encode(s_account.code_hash));
    return (s_account, 0);
}

fn dispatch_xfer(n: HostCallFn, gas: &mut Gas, reg: &mut Registers, ram: &mut RamMemory, ctx: &mut HostCallContext) -> ExitReason {

    let service_accounts = get_service_accounts().lock().unwrap().clone();
    let service_id = get_service().lock().unwrap().clone();
    log::debug!("Dispatch on transfer hostcall {:?} for service {:?}", n, service_id);
    let account = ctx.to_xfer_ctx();
    
    match n {
        HostCallFn::Lookup => lookup(gas, reg, ram, account, service_id, &service_accounts),
        HostCallFn::Read => read(gas, reg, ram, account, service_id, &service_accounts),
        HostCallFn::Write => write(gas, reg, ram, account, &service_id),
        HostCallFn::Gas => crate::hostcall::general_fn::gas(gas, reg),
        HostCallFn::Info => info(gas, reg, ram, service_id, &service_accounts, account),
        HostCallFn::Log => crate::hostcall::general_fn::log(&reg, &ram, &service_id),
        _ => {
            log::error!("Unknown on transfer hostcall function: {:?}", n);
            *gas -= 10;
            reg[7] = WHAT;
            return ExitReason::Continue;
        }
    }
}


