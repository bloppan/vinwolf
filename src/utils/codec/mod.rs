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

#[derive(Debug, PartialEq)]
pub enum ReadError {
    NotEnoughData,
    InvalidData,
    ConversionError,
}

pub struct BytesReader<'a> {
    pub position: usize,
    pub data: &'a [u8],
}

impl<'a> BytesReader<'a> {

    pub fn new(data: &'a [u8]) -> Self {
        BytesReader { data, position: 0 }
    }

    pub fn read_bytes(&mut self, length: usize) -> Result<&[u8], ReadError> {

        if self.position + length > self.data.len() {
            return Err(ReadError::NotEnoughData);
        }

        let bytes = &self.data[self.position..self.position + length];
        self.position += length;

        Ok(bytes)
    }

    pub fn read_byte(&mut self) -> Result<u8, ReadError> {

        if self.position + 1 > self.data.len() {
            return Err(ReadError::NotEnoughData);
        }

        let byte = self.data[self.position] as u8;
        self.position += 1;
        
        Ok(byte)
    }

    pub fn get_position(&self) -> usize {
        self.position
    }
}