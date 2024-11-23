use crate::types::{Hash};
use crate::codec::{Encode, Decode, BytesReader, ReadError};
use crate::codec::{encode_unsigned, decode_unsigned};

#[derive(Clone)]
pub enum MmrPeak {
    None,
    Some(Hash),
}

impl Encode for MmrPeak {

    fn encode(&self) -> Vec<u8> {

        let mut mmrpeak_blob = Vec::new();

        match self {
            MmrPeak::Some(hash) => {
                mmrpeak_blob.push(1);
                hash.encode_to(&mut mmrpeak_blob);
            }
            MmrPeak::None => {
                mmrpeak_blob.push(0);
            }
        }

        return mmrpeak_blob;
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for MmrPeak {

    fn decode(mmrpeak_blob: &mut BytesReader) -> Result<Self, ReadError> {

        let option = mmrpeak_blob.read_byte()?;

        match option {
            0 => Ok(MmrPeak::None),
            1 => {
                let hash = Hash::decode(mmrpeak_blob)?; 
                Ok(MmrPeak::Some(hash))
            }
            _ => Err(ReadError::InvalidData), 
        }
    }
}

#[derive(Clone)]
pub struct Mmr {
    pub peaks: Vec<MmrPeak>,
}

impl Encode for Mmr {

    fn encode(&self) -> Vec<u8> {

        let len = self.peaks.len();
        let mut mmr_blob = Vec::with_capacity(std::mem::size_of::<MmrPeak>() * len);
        encode_unsigned(len).encode_to(&mut mmr_blob);

        for peak in &self.peaks {
            peak.encode_to(&mut mmr_blob);
        }

        return mmr_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for Mmr {

    fn decode(mmr_blob: &mut BytesReader) -> Result<Self, ReadError> {

        let len = decode_unsigned(mmr_blob)?;
        let mut peaks = Mmr { peaks: Vec::with_capacity(len) };

        for _ in 0..len {
            peaks
                .peaks
                .push(MmrPeak::decode(mmr_blob)?);
        }

        Ok(peaks)
    }
}

#[derive(Clone)]
struct ReportedWorkPackage {
    hash: Hash,
    exports_root: Hash,
}

impl Encode for ReportedWorkPackage {

    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::with_capacity(std::mem::size_of::<Self>());
        
        self.hash.encode_to(&mut blob);
        self.exports_root.encode_to(&mut blob);
        
        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for ReportedWorkPackage {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(ReportedWorkPackage{
            hash: Hash::decode(blob)?,
            exports_root: Hash::decode(blob)?,
        })
    }
}

#[derive(Clone)]
pub struct ReportedWorkPackages {
    pub reported_work_packages: Vec<ReportedWorkPackage>,
}

impl Encode for ReportedWorkPackages {

    fn encode(&self) -> Vec<u8> {

        let len = self.reported_work_packages.len();
        let mut reported_work_packages = Vec::with_capacity(std::mem::size_of::<ReportedWorkPackage>() * len);
        encode_unsigned(len).encode_to(&mut reported_work_packages); 
        
        for item in &self.reported_work_packages {
            item.encode_to(&mut reported_work_packages);
        }

        return reported_work_packages;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for ReportedWorkPackages {
    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        let len = decode_unsigned(blob)?;
        let mut reported_work_packages = Vec::with_capacity(len);

        for _ in 0..len {
            reported_work_packages.push(ReportedWorkPackage::decode(blob)?);
        }

        Ok(ReportedWorkPackages {
            reported_work_packages,
        })
    }
}

#[derive(Clone)]
pub struct BlockInfo {
    header_hash: Hash,
    pub mmr: Mmr,
    state_root: Hash,
    reported: ReportedWorkPackages,
}

impl Encode for BlockInfo {

    fn encode(&self) -> Vec<u8> {

        let mut block_info = Vec::with_capacity(std::mem::size_of::<Self>());

        self.header_hash.encode_to(&mut block_info);
        self.mmr.encode_to(&mut block_info);
        self.state_root.encode_to(&mut block_info);
        self.reported.encode_to(&mut block_info);

        return block_info;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for BlockInfo {

    fn decode(block_info: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(BlockInfo {
            header_hash: Hash::decode(block_info)?,
            mmr: Mmr::decode(block_info)?,
            state_root: Hash::decode(block_info)?,
            reported: ReportedWorkPackages::decode(block_info)?,
        })
    }
}

#[derive(Clone)]
pub struct State {
    pub beta: Vec<BlockInfo>,
}

impl Encode for State {

    fn encode(&self) -> Vec<u8> {

        let len = self.beta.len();
        let mut state = Vec::with_capacity(std::mem::size_of::<Self>() * len);
        encode_unsigned(len).encode_to(&mut state);

        for item in &self.beta {
            item.encode_to(&mut state);
        }

        return state;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for State {

    fn decode(state: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(State {
            beta: {
                let len = decode_unsigned(state)?;
                let mut beta_vec = Vec::with_capacity(std::mem::size_of::<Self>() * len);
                for _ in 0..len {
                    beta_vec.push(BlockInfo::decode(state)?);
                }
                beta_vec
            }
        })
    }
}

pub struct Input {
    pub header_hash: Hash,
    pub parent_state_root: Hash,
    pub accumulate_root: Hash,
    pub work_packages: ReportedWorkPackages,
}

impl Encode for Input {

    fn encode(&self) -> Vec<u8> {

        let mut input_blob = Vec::with_capacity(std::mem::size_of::<Self>());

        self.header_hash.encode_to(&mut input_blob);
        self.parent_state_root.encode_to(&mut input_blob);
        self.accumulate_root.encode_to(&mut input_blob);
        self.work_packages.encode_to(&mut input_blob);

        return input_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for Input {

    fn decode(input_blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(Input {
            header_hash: Hash::decode(input_blob)?,
            parent_state_root: Hash::decode(input_blob)?,
            accumulate_root: Hash::decode(input_blob)?,
            work_packages: ReportedWorkPackages::decode(input_blob)?,
        })
    }
}