use crate::{Encode, EncodeSize, Decode, BytesReader, ReadError};
use crate::generic_codec::{encode_unsigned, decode_unsigned};
use jam_types::{TimeSlot, WorkReport, ValidatorIndex, Ed25519Signature, ReportGuarantee, ValidatorSignature};

