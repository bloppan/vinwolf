use std::collections::VecDeque;

use jam_types::{ServiceId, BlockInfo, Mmr, MmrPeak, OpaqueHash, RecentAccOutputs, RecentBlocks, ReportedWorkPackages};
use crate::{Encode, EncodeSize, Decode, DecodeLen, BytesReader, ReadError};
use crate::generic_codec::{encode_unsigned, decode_unsigned};

impl Encode for BlockInfo {

    fn encode(&self) -> Vec<u8> {

        let mut block_info = Vec::with_capacity(std::mem::size_of::<Self>());

        self.header_hash.encode_to(&mut block_info);
        self.beefy_root.encode_to(&mut block_info);
        self.state_root.encode_to(&mut block_info);
        self.reported_wp.encode_to(&mut block_info);

        return block_info;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for BlockInfo {

    fn decode(block_info: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(BlockInfo {
            header_hash: OpaqueHash::decode(block_info)?,
            beefy_root: OpaqueHash::decode(block_info)?,
            state_root: OpaqueHash::decode(block_info)?,
            reported_wp: ReportedWorkPackages::decode(block_info)?,
        })
    }
}

impl Encode for RecentBlocks {

    fn encode(&self) -> Vec<u8> {

        let len = self.history.len();
        let mut state = Vec::with_capacity(std::mem::size_of::<Self>() * len);
        encode_unsigned(len).encode_to(&mut state);

        for item in &self.history {
            item.encode_to(&mut state);
        }

        self.mmr.peaks.encode_to(&mut state);

        return state;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for RecentBlocks {

    fn decode(state: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(RecentBlocks {
            history: {
                let len = decode_unsigned(state)?;
                let mut blocks_vec = VecDeque::with_capacity(len);
                for _ in 0..len {
                    blocks_vec.push_back(BlockInfo::decode(state)?);
                }
                blocks_vec
            },
            mmr: Mmr {peaks: Vec::<MmrPeak>::decode_len(state)?},
        })
    }
}

impl Encode for RecentAccOutputs {

    fn encode(&self) -> Vec<u8> {
        
        let mut blob = Vec::new();

        encode_unsigned(self.pairs.len()).encode_to(&mut blob);

        for output in &self.pairs {
            output.0.encode_size(4).encode_to(&mut blob);
            output.1.encode_to(&mut blob);
        }

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for RecentAccOutputs {

    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        
        Ok(RecentAccOutputs {
            pairs: {
                let len = decode_unsigned(reader)?;
                let mut recent_acc_outputs = RecentAccOutputs::default();
                for _ in 0..len {
                    let service = ServiceId::decode(reader)?;
                    let hash = OpaqueHash::decode(reader)?;
                    recent_acc_outputs.pairs.push((service, hash));
                }
                recent_acc_outputs.pairs
            }
        })
    }
}