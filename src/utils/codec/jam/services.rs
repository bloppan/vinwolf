use crate::jam_types::{ ServiceId, Gas, OpaqueHash, ServiceInfo, ServiceItem, Services };
use crate::utils::codec::{Encode, EncodeLen, EncodeSize, Decode, DecodeLen, BytesReader, ReadError};

impl Encode for ServiceInfo {

    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::with_capacity(std::mem::size_of::<Self>());
        
        self.code_hash.encode_to(&mut blob);
        self.balance.encode_size(8).encode_to(&mut blob);
        self.acc_min_gas.encode_size(8).encode_to(&mut blob);
        self.xfer_min_gas.encode_size(8).encode_to(&mut blob);
        self.bytes.encode_size(8).encode_to(&mut blob);
        self.items.encode_size(4).encode_to(&mut blob);
        
        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for ServiceInfo {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(ServiceInfo {
            code_hash: OpaqueHash::decode(blob)?,
            balance: u64::decode(blob)?,
            acc_min_gas: Gas::decode(blob)?,
            xfer_min_gas: Gas::decode(blob)?,
            bytes: u64::decode(blob)?,
            items: u32::decode(blob)?,
        })
    }
}

impl Encode for ServiceItem {

    fn encode(&self) -> Vec<u8> {
        
        let mut blob = Vec::with_capacity(std::mem::size_of::<Self>());

        self.id.encode_to(&mut blob);
        self.info.encode_to(&mut blob);
        
        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for ServiceItem {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(ServiceItem{
            id: ServiceId::decode(blob)?,
            info: ServiceInfo::decode(blob)?,
        })
    }
}

impl Encode for Services {

    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::with_capacity(std::mem::size_of::<Self>() * self.0.len());
        self.0.encode_len().encode_to(&mut blob);

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for Services {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(Services{0: Vec::<ServiceItem>::decode_len(blob)?})
    }
}