use crate::types::{
    ServiceId, Gas, OpaqueHash, ServiceInfo, ServiceItem, Services
};
use crate::utils::codec::{Encode, EncodeSize, Decode, BytesReader, ReadError};
use crate::utils::codec::generic::{encode_unsigned, decode_unsigned};

impl Encode for ServiceInfo {

    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::with_capacity(std::mem::size_of::<Self>());
        
        self.code_hash.encode_to(&mut blob);
        self.balance.encode_size(8).encode_to(&mut blob);
        self.min_item_gas.encode_size(8).encode_to(&mut blob);
        self.min_memo_gas.encode_size(8).encode_to(&mut blob);
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
            min_item_gas: Gas::decode(blob)?,
            min_memo_gas: Gas::decode(blob)?,
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

        encode_unsigned(self.0.len()).encode_to(&mut blob);

        for item in &self.0 {
            item.encode_to(&mut blob);
        }

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for Services {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        let len = decode_unsigned(blob)?;
        let mut services = Vec::with_capacity(std::mem::size_of::<Self>() * len);

        for _ in 0..len {
            services.push(ServiceItem::decode(blob)?);
        }

        Ok(Services{0: services})
    }
}