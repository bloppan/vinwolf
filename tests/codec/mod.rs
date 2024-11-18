use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

extern crate vinwolf;

use vinwolf::codec::{Encode, Decode, BytesReader};
use vinwolf::codec::refine_context::RefineContext;
use vinwolf::codec::work_item::WorkItem;
use vinwolf::codec::work_package::WorkPackage;
use vinwolf::codec::work_result::WorkResult;
use vinwolf::codec::work_report::WorkReport;
use vinwolf::codec::tickets_extrinsic::TicketsExtrinsic;
use vinwolf::codec::disputes_extrinsic::DisputesExtrinsic;
use vinwolf::codec::preimages_extrinsic::PreimagesExtrinsic;
use vinwolf::codec::assurances_extrinsic::AssurancesExtrinsic;
use vinwolf::codec::guarantees_extrinsic::GuaranteesExtrinsic;
use vinwolf::codec::header::Header;
use vinwolf::codec::block::Block;

pub fn find_first_difference(data1: &[u8], data2: &[u8], _part: &str) -> Option<usize> {
    data1.iter()
        .zip(data2.iter())
        .position(|(byte1, byte2)| byte1 != byte2)
        .map(|pos| {
            println!("First 32 bytes of data1: {:0X?}", &data1[pos..pos + 64.min(data1.len())]);
            println!("First 32 bytes of data2: {:0X?}", &data2[pos..pos + 64.min(data2.len())]);
            pos
        })
}



fn read_codec_test(filename: &str) -> Vec<u8> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(filename);
    let mut file = File::open(&path).expect("Failed to open file");
    let mut content = Vec::new();
    let _success = file.read_to_end(&mut content);
    return content;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_refine_context_test() {
        let test = read_codec_test("data/codec/data/refine_context.bin");
        let mut refine_test = BytesReader::new(&test);
        let refine_decoded: RefineContext = RefineContext::decode(&mut refine_test).expect("Error decoding RefineContext");
        let res = refine_decoded.encode();
        assert_eq!(test, res);
    }

    #[test]
    fn run_work_item_test() {
        let test = read_codec_test("data/codec/data/work_item.bin");
        let mut item_test = BytesReader::new(&test);
        let work_item_decoded: WorkItem = WorkItem::decode(&mut item_test).expect("Error decoding WorkItem");
        let res = work_item_decoded.encode();
        assert_eq!(test, res);
    }

    #[test]
    fn run_work_package_test() {
        let test = read_codec_test("data/codec/data/work_package.bin");
        let mut work_pkg_test = BytesReader::new(&test);
        let work_pkg_decoded = WorkPackage::decode(&mut work_pkg_test).expect("Error decoding WorkPackage");
        let res = work_pkg_decoded.encode();
        assert_eq!(test, res);
    }

    #[test]
    fn run_work_result_0() {
        let test = read_codec_test("data/codec/data/work_result_0.bin");
        let mut work_result_test = BytesReader::new(&test);
        let work_result_decoded = WorkResult::decode(&mut work_result_test).expect("Error decoding WorkResult 0");
        let res = work_result_decoded.encode();
        assert_eq!(test, res);
    }

    #[test]
    fn run_work_result_1() {
        let test = read_codec_test("data/codec/data/work_result_1.bin");
        let mut work_result_test = BytesReader::new(&test);
        let work_result_decoded = WorkResult::decode(&mut work_result_test).expect("Error decoding WorkResult 1");
        let res = work_result_decoded.encode();
        assert_eq!(test, res);
    }

    #[test]
    fn run_work_report() {
        let test = read_codec_test("data/codec/data/work_report.bin");
        let mut work_report_test = BytesReader::new(&test);
        let work_report_decoded = WorkReport::decode(&mut work_report_test).expect("Error decoding WorkReport");
        let res = work_report_decoded.encode();
        /*println!("work_report decoded: {:0X?}", work_report_decoded);

        if let Some(diff_pos) = find_first_difference(&test, &res, "WorkReport") {
            panic!("Difference found at byte position {}", diff_pos);
        }*/
        assert_eq!(test, res);
    }

    #[test]
    fn run_tickets_extrinsic() {
        let test = read_codec_test("data/codec/data/tickets_extrinsic.bin");
        let mut tickets_extrinsic_test = BytesReader::new(&test);
        let ticket_decoded = TicketsExtrinsic::decode(&mut tickets_extrinsic_test).expect("Error decoding TicketEnvelope");
        let res = TicketsExtrinsic::encode(&ticket_decoded);
        assert_eq!(test, res);
    }

    #[test]
    fn run_disputes_extrinsic() {
        let test = read_codec_test("data/codec/data/disputes_extrinsic.bin");
        let mut disputes_extrinsic_test = BytesReader::new(&test);
        let disputes_decoded = DisputesExtrinsic::decode(&mut disputes_extrinsic_test).expect("Error decoding DisputesExtrinsic");
        let res = DisputesExtrinsic::encode(&disputes_decoded);
        assert_eq!(test, res);
    }

    #[test]
    fn run_preimages_extrinsic() {
        let test = read_codec_test("data/codec/data/preimages_extrinsic.bin");
        let mut preimages_extrinsic_test = BytesReader::new(&test);
        let preimages_decoded = PreimagesExtrinsic::decode(&mut preimages_extrinsic_test).expect("Error decoding PreimagesExtrinsic");
        let res = PreimagesExtrinsic::encode(&preimages_decoded);
        assert_eq!(test, res);
    }

    #[test]
    fn run_assurances_extrinsic() {
        let test = read_codec_test("data/codec/data/assurances_extrinsic.bin");
        let mut assurances_extrinsic_test = BytesReader::new(&test);
        let assurances_decoded = AssurancesExtrinsic::decode(&mut assurances_extrinsic_test).expect("Error decoding AssurancesExtrinsic");
        let res = AssurancesExtrinsic::encode(&assurances_decoded);
        assert_eq!(test, res);
    }

    #[test]
    fn run_guarantees_extrinsic() {
        let test = read_codec_test("data/codec/data/guarantees_extrinsic.bin");
        let mut guarantees_extrinsic_test = BytesReader::new(&test);
        let guarantees_decoded = GuaranteesExtrinsic::decode(&mut guarantees_extrinsic_test).expect("Error decoding GuaranteesExtrinsic");
        let res = GuaranteesExtrinsic::encode(&guarantees_decoded);
        assert_eq!(test, res);
    }

    #[test]
    fn run_header_0() {
        let test = read_codec_test("data/codec/data/header_0.bin");
        let mut header_test = BytesReader::new(&test);
        let header_decoded = Header::decode(&mut header_test).expect("Error decoding Header");
        let res = Header::encode(&header_decoded);
        assert_eq!(test, res);
    }

    #[test]
    fn run_header_1() {
        let test = read_codec_test("data/codec/data/header_1.bin");
        let mut header_test = BytesReader::new(&test);
        let header_decoded = Header::decode(&mut header_test).expect("Error decoding Header");
        let res = Header::encode(&header_decoded);
        assert_eq!(test, res);
    }

    #[test]
    fn run_block() {
        let test = read_codec_test("data/codec/data/block.bin");
        let mut block_test = BytesReader::new(&test);
        let block_decoded = Block::decode(&mut block_test).expect("Error decoding Block");
        let res = Block::encode(&block_decoded);
        assert_eq!(test, res);
    }
}
