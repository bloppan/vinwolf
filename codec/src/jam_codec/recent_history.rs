use std::collections::VecDeque;

use jam_types::{BlockHistory, BlockInfo, Hash, Mmr, MmrPeak, ReportedWorkPackages};
use crate::{Encode, EncodeLen, Decode, DecodeLen, BytesReader, ReadError};
use crate::generic_codec::{encode_unsigned, decode_unsigned};

impl Encode for BlockInfo {

    fn encode(&self) -> Vec<u8> {

        let mut block_info = Vec::with_capacity(std::mem::size_of::<Self>());

        self.header_hash.encode_to(&mut block_info);
        self.mmr.peaks.encode_len().encode_to(&mut block_info);
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
            header_hash: Hash::decode(block_info)?,
            mmr: Mmr {peaks: Vec::<MmrPeak>::decode_len(block_info)?},
            state_root: Hash::decode(block_info)?,
            reported_wp: ReportedWorkPackages::decode(block_info)?,
        })
    }
}

impl Encode for BlockHistory {

    fn encode(&self) -> Vec<u8> {

        let len = self.blocks.len();
        let mut state = Vec::with_capacity(std::mem::size_of::<Self>() * len);
        encode_unsigned(len).encode_to(&mut state);

        for item in &self.blocks {
            item.encode_to(&mut state);
        }

        return state;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for BlockHistory {

    fn decode(state: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(BlockHistory {
            blocks: {
                let len = decode_unsigned(state)?;
                let mut blocks_vec = VecDeque::with_capacity(len);
                for _ in 0..len {
                    blocks_vec.push_back(BlockInfo::decode(state)?);
                }
                blocks_vec
            }
        })
    }
}
