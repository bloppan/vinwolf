use std::cmp::min;

extern crate vinwolf;

use crate::integration::read_test_file;

use vinwolf::types::{
    RefineContext, WorkItem, WorkPackage, WorkResult, TicketsExtrinsic, DisputesExtrinsic, PreimagesExtrinsic, AssurancesExtrinsic, 
    GuaranteesExtrinsic, Header, Block, BlockHistory, WorkReport};
use vinwolf::utils::codec::{Encode, Decode, BytesReader, ReadError};

use vinwolf::utils::codec::work_report::{InputWorkReport, WorkReportState, OutputWorkReport};

use vinwolf::blockchain::block::extrinsic::assurances::OutputAssurances;
use vinwolf::blockchain::block::extrinsic::disputes::{DisputesState, OutputDisputes};
use vinwolf::blockchain::state::safrole::codec::{Input as InputSafrole, SafroleState, Output as OutputSafrole};
use vinwolf::blockchain::state::recent_history::codec::Input as InputHistory;
use crate::integration::assurances::schema::{InputAssurances, StateAssurances};
use crate::integration::authorization::schema::{InputAuthorizations, StateAuthorizations};
use crate::integration::statistics::schema::{InputStatistics, StateStatistics};

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
    BlockHistory,
    InputSafrole,
    SafroleState,
    OutputSafrole,
    DisputesState,
    OutputDisputes,
    InputWorkReport,
    WorkReportState,
    OutputWorkReport,
    InputAssurances,
    StateAssurances,
    OutputAssurances,
    InputAuthorizations,
    StateAuthorizations,
    InputStatistics,
    StateStatistics,
}

struct TestContext<'a, 'b> {
    reader: &'a mut BytesReader<'b>,
    blob: &'b [u8],
    global_position: usize,
}

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

        /*println!("---------- {}: {:0x?}", part_name, part);
        use std::io::{stdin, stdout, Write};
        print!("Presiona Enter para continuar...");
        stdout().flush().unwrap();
        let mut _input = String::new();
        stdin().read_line(&mut _input).unwrap();*/

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
            TestBody::BlockHistory => {
                context.process_test_part("BlockHistory", BlockHistory::decode, BlockHistory::encode)?;
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
            TestBody::DisputesState => {
                context.process_test_part("DisputesState", DisputesState::decode, DisputesState::encode)?;
            }
            TestBody::OutputDisputes => {
                context.process_test_part("OutputDisputes", OutputDisputes::decode, OutputDisputes::encode)?;
            }
            TestBody::InputWorkReport => {
                context.process_test_part("InputWorkReport", InputWorkReport::decode, InputWorkReport::encode)?;
            }
            TestBody::WorkReportState => {
                context.process_test_part("WorkReportState", WorkReportState::decode, WorkReportState::encode)?;
            }
            TestBody::OutputWorkReport => {
                context.process_test_part("OutputWorkReport", OutputWorkReport::decode, OutputWorkReport::encode)?;
            }
            TestBody::InputAssurances => {
                context.process_test_part("InputAssurances", InputAssurances::decode, InputAssurances::encode)?;
            }
            TestBody::StateAssurances => {
                context.process_test_part("StateAssurances", StateAssurances::decode, StateAssurances::encode)?;
            }
            TestBody::OutputAssurances => {
                context.process_test_part("OutputAssurances", OutputAssurances::decode, OutputAssurances::encode)?;
            }
            TestBody::InputAuthorizations => {
                context.process_test_part("InputAuthorizations", InputAuthorizations::decode, InputAuthorizations::encode)?;
            }
            TestBody::StateAuthorizations => {
                context.process_test_part("StateAuthorizations", StateAuthorizations::decode, StateAuthorizations::encode)?;
            }
            TestBody::InputStatistics => {
                context.process_test_part("InputStatistics", InputStatistics::decode, InputStatistics::encode)?;
            }
            TestBody::StateStatistics => {
                context.process_test_part("StateStatistics", StateStatistics::decode, StateStatistics::encode)?;
            }
        }
    }

    if context.global_position != blob.len() {
        println!("Codec test was not readed properly! Readed {} bytes. The test file has {} bytes", context.global_position, blob.len());
        assert_eq!(blob.len(), context.global_position);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_refine_context_test() {
        let test_content = read_test_file("tests/jamtestvectors/codec/data/refine_context.bin");
        let test_body: Vec<TestBody> = vec![TestBody::RefineContext];
        let _ = encode_decode_test(&test_content, &test_body);
    }

    #[test]
    fn run_work_item_test() {
        let test_content = read_test_file("tests/jamtestvectors/codec/data/work_item.bin");
        let test_body: Vec<TestBody> = vec![TestBody::WorkItem];
        let _ = encode_decode_test(&test_content, &test_body);
    }
    
    #[test]
    fn run_work_package_test() {
        let test_content = read_test_file("tests/jamtestvectors/codec/data/work_package.bin");
        let test_body: Vec<TestBody> = vec![TestBody::WorkPackage];
        let _ = encode_decode_test(&test_content, &test_body);
    }
    
    #[test]
    fn run_work_result_0() {
        let test_content = read_test_file("tests/jamtestvectors/codec/data/work_result_0.bin");
        let test_body: Vec<TestBody> = vec![TestBody::WorkResult];
        let _ = encode_decode_test(&test_content, &test_body);
    }
    
    #[test]
    fn run_work_result_1() {
        let test_content = read_test_file("tests/jamtestvectors/codec/data/work_result_1.bin");
        let test_body: Vec<TestBody> = vec![TestBody::WorkResult];
        let _ = encode_decode_test(&test_content, &test_body);
    }
    
    #[test]
    fn run_work_report() {
        let test_content = read_test_file("tests/jamtestvectors/codec/data/work_report.bin");
        let test_body: Vec<TestBody> = vec![TestBody::WorkReport];
        let _ = encode_decode_test(&test_content, &test_body);
    }
    
    #[test]
    fn run_tickets_extrinsic() {
        let test_content = read_test_file("tests/jamtestvectors/codec/data/tickets_extrinsic.bin");
        let test_body: Vec<TestBody> = vec![TestBody::TicketsExtrinsic];
        let _ = encode_decode_test(&test_content, &test_body);
    }
    
    #[test]
    fn run_disputes_extrinsic() {
        let test_content = read_test_file("tests/jamtestvectors/codec/data/disputes_extrinsic.bin");
        let test_body: Vec<TestBody> = vec![TestBody::DisputesExtrinsic];
        let _ = encode_decode_test(&test_content, &test_body);
    }
    
    #[test]
    fn run_preimages_extrinsic() {
        let test_content = read_test_file("tests/jamtestvectors/codec/data/preimages_extrinsic.bin");
        let test_body: Vec<TestBody> = vec![TestBody::PreimagesExtrinsic];
        let _ = encode_decode_test(&test_content, &test_body);
    }
    
    #[test]
    fn run_assurances_extrinsic() {
        let test_content = read_test_file("tests/jamtestvectors/codec/data/assurances_extrinsic.bin");
        let test_body: Vec<TestBody> = vec![TestBody::AssurancesExtrinsic];
        let _ = encode_decode_test(&test_content, &test_body);
    }
    
    #[test]
    fn run_guarantees_extrinsic() {
        let test_content = read_test_file("tests/jamtestvectors/codec/data/guarantees_extrinsic.bin");
        let test_body: Vec<TestBody> = vec![TestBody::GuaranteesExtrinsic];
        let _ = encode_decode_test(&test_content, &test_body);
    }
    
    #[test]
    fn run_header_0() {
        let test_content = read_test_file("tests/jamtestvectors/codec/data/header_0.bin");
        let test_body: Vec<TestBody> = vec![TestBody::Header];
        let _ = encode_decode_test(&test_content, &test_body);
    }
    
    #[test]
    fn run_header_1() {
        let test_content = read_test_file("tests/jamtestvectors/codec/data/header_1.bin");
        let test_body: Vec<TestBody> = vec![TestBody::Header];
        let _ = encode_decode_test(&test_content, &test_body);
    }
    
    #[test]
    fn run_block() {
        let test_content = read_test_file("tests/jamtestvectors/codec/data/block.bin");
        let test_body: Vec<TestBody> = vec![TestBody::Block];
        let _ = encode_decode_test(&test_content, &test_body);
    }
}
