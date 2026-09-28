use block::extrinsic;
use jam_types::*;
use tools::log;

pub fn process(
    services: &mut ServiceAccounts, 
    post_tau: &TimeSlot, 
    preimages_extrinsic: &[Preimage]
) -> Result<OutputPreimages, ImportError> {

    log::debug!("Process the preimages extrinsic");

    if preimages_extrinsic.len() == 0 {
        log::debug!("No preimages to process");
        return Ok(OutputPreimages::Ok());
    }
    
    extrinsic::preimages::process(preimages_extrinsic, services, post_tau)?;

    log::debug!("Preimages extrinsic processed successfully");
    Ok(OutputPreimages::Ok())
}


pub fn validate_preimages(
    services: &ServiceAccounts, 
    preimages_extrinsic: &[Preimage]
) -> Result<(), ImportError> {

    if preimages_extrinsic.len() == 0 {
        return Ok(());
    }
    extrinsic::preimages::validate(preimages_extrinsic, services)
}

pub fn integrate_preimages(
    services: &mut ServiceAccounts, 
    post_tau: &TimeSlot, 
    preimages_extrinsic: &[Preimage]
) {

    if preimages_extrinsic.len() == 0 {
        return;
    }
    let pairs = preimages_extrinsic.iter().map(|preimage| (preimage.requester, preimage.blob.clone())).collect::<Vec<_>>();
    extrinsic::preimages::integrate(services, &pairs, post_tau);
    log::debug!("Preimages extrinsic integrated");
}
