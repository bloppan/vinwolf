use vinwolf::types::{TimeSlot, ValidatorsData, HeaderHash, AvailabilityAssignments, AssurancesExtrinsic};
use vinwolf::utils::codec::{BytesReader, Decode, Encode, ReadError};

#[derive(Debug, Clone, PartialEq)]
pub struct InputPreimages {
    pub assurances: AssurancesExtrinsic,
    pub slot: TimeSlot,
    pub parent: HeaderHash
}
/*
State ::= SEQUENCE {
    -- [..] Accounts.
    accounts SEQUENCE OF AccountsMapEntry,
}

Input ::= SEQUENCE {
    -- [E_P] Preimages extrinsic.
    preimages PreimagesExtrinsic,

    -- [H_t] Block's timeslot.
    slot TimeSlot
}

-- State transition function execution error.
-- Error codes **are not specified** in the the Graypaper.
-- Feel free to ignore the actual value.
ErrorCode ::= ENUMERATED {
    preimage-unneeded (0)
}

Output ::= CHOICE {
    ok  NULL,
    err ErrorCode
}

TestCase ::= SEQUENCE {
    input        Input,
    pre-state    State,
    output       Output,
    post-state   State
}*/