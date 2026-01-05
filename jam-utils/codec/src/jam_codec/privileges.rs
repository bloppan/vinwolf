use constants::node::CORES_COUNT;
use crate::{generic_codec::{decode_unsigned, encode_unsigned}, BytesReader, Decode, Encode, ReadError};
use jam_types::{Privileges, ServiceId, Gas};
use std::collections::HashMap;

impl Encode for Privileges {

    fn encode(&self) -> Vec<u8> {
        
        let mut blob = Vec::new();

        self.manager.encode_to(&mut blob);
        self.assigners.encode_to(&mut blob);
        self.delegator.encode_to(&mut blob);
        self.registrar.encode_to(&mut blob);

        encode_unsigned(self.always_acc.len()).encode_to(&mut blob);

        for (key, value) in self.always_acc.iter() {
            encode_unsigned(*key as usize).encode_to(&mut blob);
            encode_unsigned(*value as usize).encode_to(&mut blob);
        }

        return blob;
    }
    
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for Privileges {
    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(Privileges {
            manager: ServiceId::decode(blob)?,
            assigners: Box::<[ServiceId; CORES_COUNT]>::decode(blob)?,
            delegator: ServiceId::decode(blob)?,
            registrar: ServiceId::decode(blob)?,
            always_acc: {
                let len = decode_unsigned(blob)?;
                let mut always_acc_map: HashMap<ServiceId, Gas> = HashMap::new();
                for _ in 0..len {
                    let service = decode_unsigned(blob)? as ServiceId;
                    let gas = decode_unsigned(blob)? as Gas;
                    always_acc_map.insert(service, gas);
                }
                always_acc_map
            }
        })
    }
}