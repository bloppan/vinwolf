use std::cmp::min;

extern crate vinwolf;

use crate::integration::w3f::read_test_file;

use vinwolf::types::{
    RefineContext, WorkItem, WorkPackage, WorkResult, TicketsExtrinsic, DisputesExtrinsic, PreimagesExtrinsic, AssurancesExtrinsic, 
    GuaranteesExtrinsic, Header, Block, BlockHistory, WorkReport, OutputAssurances, OutputSafrole, OutputPreimages, OutputAccumulation,
    AuthPools, AuthQueues, Safrole, DisputesRecords, EntropyPool, ValidatorsData, AvailabilityAssignments, TimeSlot, Privileges, Statistics,
    AccumulatedHistory, ReadyQueue, StateKey, KeyValue, StorageKey, RawState, StateRoot
};
use vinwolf::utils::codec::{Encode, Decode, BytesReader, ReadError};

use crate::integration::w3f::safrole::codec::{InputSafrole, SafroleState};
use crate::integration::w3f::disputes::codec::{DisputesState, OutputDisputes};
use crate::integration::w3f::assurances::codec::{InputAssurances, StateAssurances};
use crate::integration::w3f::authorization::codec::{InputAuthorizations, StateAuthorizations};
use crate::integration::w3f::history::codec::InputHistory;
use crate::integration::w3f::statistics::codec::{InputStatistics, StateStatistics};
use crate::integration::w3f::reports::codec::{InputWorkReport, WorkReportState, OutputWorkReport};
use crate::integration::w3f::preimages::codec::{InputPreimages, PreimagesState};
use crate::integration::w3f::accumulate::codec::{InputAccumulate, StateAccumulate};
use crate::integration::testnet::codec::{GlobalStateTest, ServiceAccounts};

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

#[allow(dead_code)]
#[allow(non_snake_case)]
#[derive(Debug)]
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
    InputPreimages,
    PreimagesState,
    OutputPreimages,
    InputAccumulate,
    StateAccumulate,
    OutputAccumulation,
    GlobalStateTest,
    AuthQueues,
    AuthPools,
    Safrole,
    DisputesRecords,
    EntropyPool,
    ValidatorsData,
    AvailabilityAssignments,
    TimeSlot,
    Privileges,
    Statistics,
    AccumulatedHistory,
    ReadyQueue,
    ServiceAccounts,
    KeyValue,
    RawState,
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

        /*println!("part_name: {}", part_name);
        println!("decoded: {:0x?}", part);*/
        
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
                context.process_test_part("Safrole", SafroleState::decode, SafroleState::encode)?;
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
            TestBody::InputPreimages => {
                context.process_test_part("InputPreimages", InputPreimages::decode, InputPreimages::encode)?;
            }
            TestBody::PreimagesState => {
                context.process_test_part("PreimagesState", PreimagesState::decode, PreimagesState::encode)?;
            }
            TestBody::OutputPreimages => {
                context.process_test_part("OutputPreimages", OutputPreimages::decode, OutputPreimages::encode)?;
            }
            TestBody::InputAccumulate => {
                context.process_test_part("InputAccumulate", InputAccumulate::decode, InputAccumulate::encode)?;
            }
            TestBody::StateAccumulate => {
                context.process_test_part("StateAccumulate", StateAccumulate::decode, StateAccumulate::encode)?;
            }
            TestBody::OutputAccumulation => {
                context.process_test_part("OutputAccumulation", OutputAccumulation::decode, OutputAccumulation::encode)?;
            }
            TestBody::GlobalStateTest => {
                context.process_test_part("GlobalStateTest", GlobalStateTest::decode, GlobalStateTest::encode)?;
            }
            TestBody::AuthQueues => {
                context.process_test_part("AuthQueues", AuthQueues::decode, AuthQueues::encode)?;
            }
            TestBody::AuthPools => {
                context.process_test_part("AuthPools", AuthPools::decode, AuthPools::encode)?;
            }
            TestBody::Safrole => {
                context.process_test_part("Safrole", Safrole::decode, Safrole::encode)?;
            }
            TestBody::DisputesRecords => {
                context.process_test_part("DisputesRecords", DisputesRecords::decode, DisputesRecords::encode)?;
            }
            TestBody::EntropyPool => {
                context.process_test_part("EntropyPool", EntropyPool::decode, EntropyPool::encode)?;
            }
            TestBody::ValidatorsData => {
                context.process_test_part("ValidatorsData", ValidatorsData::decode, ValidatorsData::encode)?;
            }
            TestBody::AvailabilityAssignments => {
                context.process_test_part("AvailabilityAssignments", AvailabilityAssignments::decode, AvailabilityAssignments::encode)?;
            }
            TestBody::TimeSlot => {
                context.process_test_part("TimeSlot", TimeSlot::decode, TimeSlot::encode)?;
            }
            TestBody::Privileges => {
                context.process_test_part("Privileges", Privileges::decode, Privileges::encode)?;
            }
            TestBody::Statistics => {
                context.process_test_part("Statistics", Statistics::decode, Statistics::encode)?;
            }
            TestBody::AccumulatedHistory => {
                context.process_test_part("AccumulatedHistory", AccumulatedHistory::decode, AccumulatedHistory::encode)?;
            }
            TestBody::ReadyQueue => {
                context.process_test_part("ReadyQueue", ReadyQueue::decode, ReadyQueue::encode)?;
            }
            TestBody::ServiceAccounts => {
                context.process_test_part("ServiceAccounts", ServiceAccounts::decode, ServiceAccounts::encode)?;
            }
            TestBody::KeyValue => {
                context.process_test_part("KeyValue", KeyValue::decode, KeyValue::encode)?;
            }
            TestBody::RawState => {
                context.process_test_part("RawState", RawState::decode, RawState::encode)?;
            }
        }
    }

    /*println!("blob.len() = {}", blob.len());
    println!("context.global_position = {}", context.global_position);*/
    
    if context.global_position != blob.len() {
        println!("Codec test was not readed properly! Readed {} bytes. The test file has {} bytes", context.global_position, blob.len());
        assert_eq!(blob.len(), context.global_position);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    const TEST_DIR: &str = "tests/test_vectors/w3f/jamtestvectors/codec/data";

    #[test]
    fn run_refine_context_test() {
        let test_content = read_test_file(&format!("{}/refine_context.bin", TEST_DIR));
        let test_body: Vec<TestBody> = vec![TestBody::RefineContext];
        let result = encode_decode_test(&test_content, &test_body);
        assert_eq!(result.is_ok(), true);
    }

    #[test]
    fn run_work_item_test() {
        let test_content = read_test_file(&format!("{}/work_item.bin", TEST_DIR));
        let test_body: Vec<TestBody> = vec![TestBody::WorkItem];
        let result = encode_decode_test(&test_content, &test_body);
        assert_eq!(result.is_ok(), true);
    }
    
    #[test]
    fn run_work_package_test() {
        let test_content = read_test_file(&format!("{}/work_package.bin", TEST_DIR));
        let test_body: Vec<TestBody> = vec![TestBody::WorkPackage];
        let result = encode_decode_test(&test_content, &test_body);
        assert_eq!(result.is_ok(), true);
    }
    
    #[test]
    fn run_work_result_0() {
        let test_content = read_test_file(&format!("{}/work_result_0.bin", TEST_DIR));
        let test_body: Vec<TestBody> = vec![TestBody::WorkResult];
        let result = encode_decode_test(&test_content, &test_body);
        assert_eq!(result.is_ok(), true);
    }
    
    #[test]
    fn run_work_result_1() {
        let test_content = read_test_file(&format!("{}/work_result_1.bin", TEST_DIR));
        let test_body: Vec<TestBody> = vec![TestBody::WorkResult];
        let result = encode_decode_test(&test_content, &test_body);
        assert_eq!(result.is_ok(), true);
    }
    
    #[test]
    fn run_work_report() {
        let test_content = read_test_file(&format!("{}/work_report.bin", TEST_DIR));
        let test_body: Vec<TestBody> = vec![TestBody::WorkReport];
        let result = encode_decode_test(&test_content, &test_body);
        assert_eq!(result.is_ok(), true);
    }
    
    #[test]
    fn run_tickets_extrinsic() {
        let test_content = read_test_file(&format!("{}/tickets_extrinsic.bin", TEST_DIR));
        let test_body: Vec<TestBody> = vec![TestBody::TicketsExtrinsic];
        let result = encode_decode_test(&test_content, &test_body);
        assert_eq!(result.is_ok(), true);
    }
    
   #[test]
    fn run_disputes_extrinsic() {
        let test_content = read_test_file(&format!("{}/disputes_extrinsic.bin", TEST_DIR));
        let test_body: Vec<TestBody> = vec![TestBody::DisputesExtrinsic];
        let result = encode_decode_test(&test_content, &test_body);
        assert_eq!(result.is_ok(), true);
    }
    
    #[test]
    fn run_preimages_extrinsic() {
        let test_content = read_test_file(&format!("{}/preimages_extrinsic.bin", TEST_DIR));
        let test_body: Vec<TestBody> = vec![TestBody::PreimagesExtrinsic];
        let result = encode_decode_test(&test_content, &test_body);
        assert_eq!(result.is_ok(), true);
    }
    
    #[test]
    fn run_assurances_extrinsic() {
        let test_content = read_test_file(&format!("{}/assurances_extrinsic.bin", TEST_DIR));
        let test_body: Vec<TestBody> = vec![TestBody::AssurancesExtrinsic];
        let result = encode_decode_test(&test_content, &test_body);
        assert_eq!(result.is_ok(), true);
    }
    
    #[test]
    fn run_guarantees_extrinsic() {
        let test_content = read_test_file(&format!("{}/guarantees_extrinsic.bin", TEST_DIR));
        let test_body: Vec<TestBody> = vec![TestBody::GuaranteesExtrinsic];
        let result = encode_decode_test(&test_content, &test_body);
        assert_eq!(result.is_ok(), true);
    }
    
    #[test]
    fn run_header_0() {
        let test_content = read_test_file(&format!("{}/header_0.bin", TEST_DIR));
        let test_body: Vec<TestBody> = vec![TestBody::Header];
        let result = encode_decode_test(&test_content, &test_body);
        assert_eq!(result.is_ok(), true);
    }
    
    #[test]
    fn run_header_1() {
        let test_content = read_test_file(&format!("{}/header_1.bin", TEST_DIR));
        let test_body: Vec<TestBody> = vec![TestBody::Header];
        let result = encode_decode_test(&test_content, &test_body);
        assert_eq!(result.is_ok(), true);
    }
    
    #[test]
    fn run_block() {
        let test_content = read_test_file(&format!("{}/block.bin", TEST_DIR));
        let test_body: Vec<TestBody> = vec![TestBody::Block];
        let result = encode_decode_test(&test_content, &test_body);
        assert_eq!(result.is_ok(), true);
    }
}
