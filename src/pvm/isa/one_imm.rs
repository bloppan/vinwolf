/*
    Instructions with Arguments of One Immediate.
*/

use std::cmp::{min, max};
use crate::pvm;
use crate::types::{Context, ExitReason, Program, RegSigned, RegSize};
use crate::pvm::isa::{skip, extend_sign, signed};
use crate::utils::codec::{BytesReader};
use crate::utils::codec::generic::{decode_integer};