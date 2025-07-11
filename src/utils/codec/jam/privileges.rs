use std::collections::HashMap;

use crate::jam_types::{Privileges, ServiceId};
use crate::utils::codec::{Encode, Decode, BytesReader, ReadError};

impl Encode for Privileges {
    fn encode(&self) -> Vec<u8> {
        
        let mut blob = Vec::new();

        self.bless.encode_to(&mut blob);
        self.assign.encode_to(&mut blob);
        self.designate.encode_to(&mut blob);
        self.always_acc.encode_to(&mut blob);

        return blob;
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for Privileges {
    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(Privileges {
            bless: ServiceId::decode(blob)?,
            assign: ServiceId::decode(blob)?,
            designate: ServiceId::decode(blob)?,
            always_acc: HashMap::decode(blob)?,
        })
    }
}