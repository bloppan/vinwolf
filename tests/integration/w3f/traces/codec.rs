
use frame_support::traits::dynamic_params::Key;
use vinwolf::utils::codec::{BytesReader, Decode, DecodeLen, EncodeLen, Encode, ReadError};
use vinwolf::types::{Block, RawState, KeyValue};


#[derive(Debug, Clone, PartialEq)]
pub struct TestCase {

    pub pre_state: RawState,
    pub block: Block,
    pub post_state: RawState,
}

