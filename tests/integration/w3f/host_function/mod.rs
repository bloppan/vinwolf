use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

extern crate vinwolf;


use vinwolf::constants::{NUM_REG, PAGE_SIZE};
use vinwolf::pvm::invoke_pvm;
use vinwolf::types::{Context, ExitReason, MemoryChunk, PageMap, PageFlags, RamAddress, Gas, Page, PageTable};



#[cfg(test)]
mod tests {

    use super::*;
    fn run_hostcall_test(filename: &str) {


    }

}