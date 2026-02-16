/*
    We presume a pre-existing consensus over time specifically for block production and import. While this was not an 
    assumption of Polkadot, pragmatic and resilient solutions exist including the ntp protocol and network.
    We utilize this assumption in only one way: we require that blocks be considered temporarily invalid if their timeslot 
    is in the future. This is specified in detail in the Safrole module.

    Formally, we define the time in terms of seconds passed since the beginning of the Jam Common Era, 12:00 UTC on
    January 1, 2025. Midday utc is selected to ensure that all major timezones are on the same date at any exact 24-hour 
    multiple from the beginning of the common era. 

    Unlike the YP Ethereum with its proof-of-work consensus system, Jam defines a proof-ofauthority consensus mechanism, 
    with the authorized validators presumed to be identified by a set of public keys and decided by a staking mechanism 
    residing within some system hosted by Jam. The staking system is out of scope for the present work; instead there is 
    an api which may be utilized to update these keys, and we presume that whatever logic is needed for the staking system 
    will be introduced and utilize this api as needed.

    The Safrole mechanism subdivides time following genesis into fixed length epochs with each epoch divided into E = 600 
    timeslots each of uniform length P = 6 seconds, given an epoch period of E ⋅ P = 3600 seconds or one hour. This six-second 
    slot period represents the minimum time between Jam blocks, and through Safrole we aim to strictly minimize forks arising 
    both due to contention within a slot (where two valid blocks may be produced within the same six-second period) and due to 
    contention over multiple slots (where two valid blocks are produced in different time slots but with the same parent).

    Formally when identifying a timeslot index, we use a natural less than 2^32 (in compute parlance, a 32-bit unsigned integer) 
    indicating the number of six-second timeslots from the Jam Common Era. 

    This implies that the lifespan of the proposed protocol takes us to mid-August of the year 2840, which with the current 
    course that humanity is on should be ample.
*/

use constants::node::{JAM_COMMON_ERA, SLOT_PERIOD};
use jam_types::*;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tools::log;

pub fn current_slot() -> Result<TimeSlot, ImportError> {

    let now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| {
            log::error!("Time error: {:?}", e);
            ImportError::TimeError(TimeErrorCode::TimeWentBackwards)
        })?
        .as_secs();

    Ok(((now_secs.saturating_sub(JAM_COMMON_ERA)) as TimeSlot / SLOT_PERIOD) as TimeSlot)
}

pub async fn wait_until_next_slot(current_slot: TimeSlot) -> Result<(), ImportError> {

    let era_millis = JAM_COMMON_ERA * 1000;
    let slot_millis = (SLOT_PERIOD as u64) * 1000;

    // Exact timestamp when the next slot starts
    let next_slot_start = era_millis + (current_slot as u64 + 1) * slot_millis;

    let now_millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| {
            log::error!("Time error: {:?}", e);
            ImportError::TimeError(TimeErrorCode::TimeWentBackwards)
        })?
        .as_millis() as u64;

    let millis_to_wait = next_slot_start.saturating_sub(now_millis);
    
    Ok(tokio::time::sleep(Duration::from_millis(millis_to_wait)).await)
}
