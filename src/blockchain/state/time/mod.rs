use once_cell::sync::Lazy;
use std::sync::Mutex;

use crate::types::TimeSlot;

// eta0
static CURRENT_SLOT: Lazy<Mutex<TimeSlot>> = Lazy::new(|| {
    Mutex::new(TimeSlot::default())
});

pub fn set_current_slot(slot: &TimeSlot) {
    let mut current_slot = CURRENT_SLOT.lock().unwrap();
    *current_slot = *slot;
}

pub fn get_current_slot() -> TimeSlot {
    let current_slot = CURRENT_SLOT.lock().unwrap();
    current_slot.clone()
}