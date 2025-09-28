use jam_types::{ Gas, OpaqueHash, ServiceId, ServiceInfo, ServiceItem, Services, TimeSlot };
use crate::{Encode, EncodeLen, EncodeSize, Decode, DecodeLen, DecodeSize, BytesReader, ReadError};

impl Encode for ServiceInfo {

    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::with_capacity(std::mem::size_of::<Self>());
        
        self.code_hash.encode_to(&mut blob);
        self.balance.encode_size(8).encode_to(&mut blob);
        self.acc_min_gas.encode_size(8).encode_to(&mut blob);
        self.xfer_min_gas.encode_size(8).encode_to(&mut blob);
        self.octets.encode_size(8).encode_to(&mut blob);
        self.gratis_storage_offset.encode_size(8).encode_to(&mut blob);
        self.items.encode_size(4).encode_to(&mut blob);
        self.created_at.encode_size(4).encode_to(&mut blob);
        self.last_acc.encode_size(4).encode_to(&mut blob);
        self.parent_service.encode_size(4).encode_to(&mut blob);
        
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
            acc_min_gas: Gas::decode_size(blob, 8)? as Gas,
            xfer_min_gas: Gas::decode_size(blob, 8)? as Gas,
            octets: u64::decode(blob)?,
            gratis_storage_offset: u64::decode(blob)?,
            items: u32::decode(blob)?,
            created_at: TimeSlot::decode(blob)?,
            last_acc: TimeSlot::decode(blob)?,
            parent_service: ServiceId::decode(blob)?,
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