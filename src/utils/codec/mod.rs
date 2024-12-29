pub mod generic;
pub mod jam;

pub trait Encode {
    fn encode(&self) -> Vec<u8>;
    fn encode_to(&self, writer: &mut Vec<u8>);
}

pub trait EncodeSize {
    fn encode_size(&self, l: usize) -> Vec<u8>;
}

pub trait EncodeLen {
    fn encode_len(&self) -> Vec<u8>;
}

pub trait Decode: Sized {
    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError>;
}

pub trait DecodeLen: Sized {
    fn decode_len(reader: &mut BytesReader) -> Result<Self, ReadError>;
}

pub struct BytesReader<'a> {
    position: usize,
    data: &'a [u8],
}

#[derive(Debug, PartialEq)]
pub enum ReadError {
    NotEnoughData,
    InvalidData,
    ConversionError,
}