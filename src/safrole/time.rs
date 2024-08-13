
// The length of an epoch in timeslots
/*
const E: u32 = 600; 
/**
    @tau:   Slot index
    @epoch: Epoch
    @m:     Slot phase index     
*/
struct Timekeeping {
    pub tau: u32, 
    pub epoch: u32,
    pub m: u32,
}

impl Timekeeping {

    pub fn get_time_info(tau: u32) -> Timekeeping {
        Timekeeping {
            tau,
            epoch: tau / E,
            m: tau % E,
        }
    }
}
*/