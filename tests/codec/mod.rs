use std::cmp::min;

extern crate vinwolf;

use crate::{read_test_file};

use vinwolf::codec::{Encode, Decode, BytesReader, ReadError};
use vinwolf::codec::refine_context::RefineContext;
use vinwolf::codec::work_item::WorkItem;
use vinwolf::codec::work_package::WorkPackage;
use vinwolf::codec::work_result::WorkResult;
use vinwolf::codec::work_report::WorkReport;
use vinwolf::codec::tickets_extrinsic::TicketsExtrinsic;
use vinwolf::codec::disputes_extrinsic::{DisputesExtrinsic, DisputesRecords, DisputesState, OutputDisputes};
use vinwolf::codec::preimages_extrinsic::PreimagesExtrinsic;
use vinwolf::codec::assurances_extrinsic::AssurancesExtrinsic;
use vinwolf::codec::guarantees_extrinsic::GuaranteesExtrinsic;
use vinwolf::codec::header::Header;
use vinwolf::codec::block::Block;
use vinwolf::codec::safrole::{Input as InputSafrole, SafroleState, Output as OutputSafrole};
use vinwolf::codec::history::{Input as InputHistory, State as StateHistory};

fn find_first_difference(data1: &[u8], data2: &[u8], _part: &str) -> Option<usize> {
    data1.iter()
        .zip(data2.iter())
        .position(|(byte1, byte2)| byte1 != byte2)
        .map(|pos| {
            println!("First 32 bytes expected:  {:0X?}", &data1[pos..min(data1.len(), pos + 32)]);
            println!("First 32 bytes of result: {:0X?}", &data2[pos..min(data2.len(), pos + 32)]);
            pos
        })
}

pub enum TestBody {
    RefineContext,
    WorkItem,
    WorkPackage,
    WorkResult,
    WorkReport,
    TicketsExtrinsic,
    DisputesExtrinsic,
    PreimagesExtrinsic,
    AssurancesExtrinsic,
    GuaranteesExtrinsic,
    Header,
    Block,
    InputHistory,
    StateHistory,
    InputSafrole,
    SafroleState,
    OutputSafrole,
    DisputesRecords,
    DisputesState,
    OutputDisputes,
}

struct TestContext<'a, 'b> {
    reader: &'a mut BytesReader<'b>,
    blob: &'b [u8],
    global_position: usize,
}
use hex::encode;
impl<'a, 'b> TestContext<'a, 'b> {
    fn process_test_part<T: Encode + Decode + std::fmt::Debug>(
        &mut self,
        part_name: &str,
        decode_fn: fn(&mut BytesReader) -> Result<T, ReadError>,
        encode_fn: fn(&T) -> Vec<u8>,
    ) -> Result<(), ReadError> {
        let part = decode_fn(self.reader)?;
        let encoded_part = encode_fn(&part);
        let end_position = self.reader.get_position();

        if let Some(diff_pos) = find_first_difference(
            &self.blob[self.global_position..end_position],
            &encoded_part,
            part_name,
        ) {
            println!("\nDifference found in '{}' at byte position {}\n\n", part_name, self.global_position + diff_pos);
            println!("Result decoded: \n\n {:0x?}", part);
            println!("\n\nResult encoded: \n\n {:0x?}\n\n", encoded_part);
        }

        assert_eq!(
            &self.blob[self.global_position..end_position],
            &encoded_part
        );
        
        if end_position > self.blob.len() {
            println!("{}: Out of test bounds | end part position = {}", part_name, end_position);
        } 

        self.global_position = end_position;

        Ok(())
    }
}

pub fn encode_decode_test(blob: &[u8], test_body: &Vec<TestBody>) -> Result<(), ReadError> {
    let mut test_reader = BytesReader::new(blob);
    let mut context = TestContext {
        reader: &mut test_reader,
        blob,
        global_position: 0,
    };

    for part in test_body {
        match part {
            TestBody::RefineContext => {
                context.process_test_part("RefineContext", RefineContext::decode, RefineContext::encode)?;
            }
            TestBody::WorkItem => {
                context.process_test_part("WorkItem", WorkItem::decode, WorkItem::encode)?;
            }
            TestBody::WorkPackage => {
                context.process_test_part("WorkPackage", WorkPackage::decode, WorkPackage::encode)?;
            }
            TestBody::WorkResult => {
                context.process_test_part("WorkResult", WorkResult::decode, WorkResult::encode)?;
            }
            TestBody::WorkReport => {
                context.process_test_part("WorkReport", WorkReport::decode, WorkReport::encode)?;
            }
            TestBody::TicketsExtrinsic => {
                context.process_test_part("TicketsExtrinsic", TicketsExtrinsic::decode, TicketsExtrinsic::encode)?;
            }
            TestBody::DisputesExtrinsic => {
                context.process_test_part("DisputesExtrinsic", DisputesExtrinsic::decode, DisputesExtrinsic::encode)?;
            }
            TestBody::PreimagesExtrinsic => {
                context.process_test_part("PreimagesExtrinsic", PreimagesExtrinsic::decode, PreimagesExtrinsic::encode)?;
            }
            TestBody::AssurancesExtrinsic => {
                context.process_test_part("AssurancesExtrinsic", AssurancesExtrinsic::decode, AssurancesExtrinsic::encode)?;
            }
            TestBody::GuaranteesExtrinsic => {
                context.process_test_part("GuaranteesExtrinsic", GuaranteesExtrinsic::decode, GuaranteesExtrinsic::encode)?;
            }
            TestBody::Header => {
                context.process_test_part("Header", Header::decode, Header::encode)?;
            }
            TestBody::Block => {
                context.process_test_part("Block", Block::decode, Block::encode)?;
            }
            TestBody::InputHistory => {
                context.process_test_part("InputHistory", InputHistory::decode, InputHistory::encode)?;
            }
            TestBody::StateHistory => {
                context.process_test_part("StateHistory", StateHistory::decode, StateHistory::encode)?;
            }
            TestBody::InputSafrole => {
                context.process_test_part("InputSafrole", InputSafrole::decode, InputSafrole::encode)?;
            }
            TestBody::SafroleState => {
                context.process_test_part("SafroleState", SafroleState::decode, SafroleState::encode)?;
            }
            TestBody::OutputSafrole => {
                context.process_test_part("OutputSafrole", OutputSafrole::decode, OutputSafrole::encode)?;
            }
            TestBody::DisputesRecords => {
                context.process_test_part("DisputesRecords", DisputesRecords::decode, DisputesRecords::encode)?;
            }
            TestBody::DisputesState => {
                context.process_test_part("DisputesState", DisputesState::decode, DisputesState::encode)?;
            }
            TestBody::OutputDisputes => {
                context.process_test_part("OutputDisputes", OutputDisputes::decode, OutputDisputes::encode)?;
            }
        }
    }

    if context.global_position != blob.len() {
        println!("Codec test was not readed properly! Readed {} bytes. The test file has {} bytes", context.global_position, blob.len());
        assert_eq!(context.global_position, blob.len());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_refine_context_test() {
        let test_content = read_test_file("data/codec/data/refine_context.bin");
        let test_body: Vec<TestBody> = vec![TestBody::RefineContext];
        let _ = encode_decode_test(&test_content, &test_body);
    }

    #[test]
    fn run_work_item_test() {
        let test_content = read_test_file("data/codec/data/work_item.bin");
        let test_body: Vec<TestBody> = vec![TestBody::WorkItem];
        let _ = encode_decode_test(&test_content, &test_body);
    }
    
    #[test]
    fn run_work_package_test() {
        let test_content = read_test_file("data/codec/data/work_package.bin");
        let test_body: Vec<TestBody> = vec![TestBody::WorkPackage];
        let _ = encode_decode_test(&test_content, &test_body);
    }
    
    #[test]
    fn run_work_result_0() {
        let test_content = read_test_file("data/codec/data/work_result_0.bin");
        let test_body: Vec<TestBody> = vec![TestBody::WorkResult];
        let _ = encode_decode_test(&test_content, &test_body);
    }
    
    #[test]
    fn run_work_result_1() {
        let test_content = read_test_file("data/codec/data/work_result_1.bin");
        let test_body: Vec<TestBody> = vec![TestBody::WorkResult];
        let _ = encode_decode_test(&test_content, &test_body);
    }
    
    #[test]
    fn run_work_report() {
        let test_content = read_test_file("data/codec/data/work_report.bin");
        let test_body: Vec<TestBody> = vec![TestBody::WorkReport];
        let _ = encode_decode_test(&test_content, &test_body);
    }
    
    #[test]
    fn run_tickets_extrinsic() {
        let test_content = read_test_file("data/codec/data/tickets_extrinsic.bin");
        let test_body: Vec<TestBody> = vec![TestBody::TicketsExtrinsic];
        let _ = encode_decode_test(&test_content, &test_body);
    }
    
    #[test]
    fn run_disputes_extrinsic() {
        let test_content = read_test_file("data/codec/data/disputes_extrinsic.bin");
        let test_body: Vec<TestBody> = vec![TestBody::DisputesExtrinsic];
        let _ = encode_decode_test(&test_content, &test_body);
    }
    
    #[test]
    fn run_preimages_extrinsic() {
        let test_content = read_test_file("data/codec/data/preimages_extrinsic.bin");
        let test_body: Vec<TestBody> = vec![TestBody::PreimagesExtrinsic];
        let _ = encode_decode_test(&test_content, &test_body);
    }
    
    #[test]
    fn run_assurances_extrinsic() {
        let test_content = read_test_file("data/codec/data/assurances_extrinsic.bin");
        let test_body: Vec<TestBody> = vec![TestBody::AssurancesExtrinsic];
        let _ = encode_decode_test(&test_content, &test_body);
    }
    
    #[test]
    fn run_guarantees_extrinsic() {
        let test_content = read_test_file("data/codec/data/guarantees_extrinsic.bin");
        let test_body: Vec<TestBody> = vec![TestBody::GuaranteesExtrinsic];
        let _ = encode_decode_test(&test_content, &test_body);
    }
    
    #[test]
    fn run_header_0() {
        let test_content = read_test_file("data/codec/data/header_0.bin");
        let test_body: Vec<TestBody> = vec![TestBody::Header];
        let _ = encode_decode_test(&test_content, &test_body);
    }
    
    #[test]
    fn run_header_1() {
        let test_content = read_test_file("data/codec/data/header_1.bin");
        let test_body: Vec<TestBody> = vec![TestBody::Header];
        let _ = encode_decode_test(&test_content, &test_body);
    }
    
    #[test]
    fn run_block() {
        let test_content = read_test_file("data/codec/data/block.bin");
        let test_body: Vec<TestBody> = vec![TestBody::Block];
        let _ = encode_decode_test(&test_content, &test_body);
    }
}
