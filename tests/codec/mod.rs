use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

extern crate vinwolf;

use vinwolf::codec::BytesReader;

use vinwolf::refine::RefineContext;

use vinwolf::work::package::WorkItem;
use vinwolf::work::package::WorkPackage;
use vinwolf::work::package::WorkResult;
use vinwolf::work::package::WorkReport;

use vinwolf::extrinsic::TicketEnvelope;
use vinwolf::extrinsic::DisputesExtrinsic;
use vinwolf::extrinsic::PreimagesExtrinsic;
use vinwolf::extrinsic::AssurancesExtrinsic;
use vinwolf::extrinsic::GuaranteesExtrinsic;

use vinwolf::header::Header;
use vinwolf::block::Block;

#[cfg(test)]
mod tests {
    use super::*;
    
    fn read_codec_test(filename: &str) -> Vec<u8> {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push(filename);
        let mut file = File::open(&path).expect("Failed to open file");
        let mut content = Vec::new();
        let _success = file.read_to_end(&mut content);
        return content;
    }

    fn run_refine_context_test() {
        let test = read_codec_test("data/codec/data/refine_context.bin");
        let mut refine_test = BytesReader::new(&test);
        let refine_decoded: RefineContext = RefineContext::decode(&mut refine_test).expect("Error decoding RefineContext");
        let res = refine_decoded.encode().expect("Error encoding RefineContext");
        assert_eq!(test, res);
    }

    #[test]
    fn run_work_item_test() {
        let test = read_codec_test("data/codec/data/work_item.bin");
        let mut item_test = BytesReader::new(&test);
        let work_item_decoded: WorkItem = WorkItem::decode(&mut item_test).expect("Error decoding WorkItem");
        let res = work_item_decoded.encode().expect("Error decoding WorkItem");
        assert_eq!(test, res);
    }

    #[test]
    fn run_work_package_test() {
        let test = read_codec_test("data/codec/data/work_package.bin");
        let mut work_pkg_test = BytesReader::new(&test);
        let work_pkg_decoded = WorkPackage::decode(&mut work_pkg_test).expect("Error decoding WorkPackage");
        let res = work_pkg_decoded.encode().expect("Error encode WorkPackage");
        assert_eq!(test, res);
    }

    #[test]
    fn run_work_result_0() {
        let test = read_codec_test("data/codec/data/work_result_0.bin");
        let mut work_result_test = BytesReader::new(&test);
        let work_result_decoded = WorkResult::decode(&mut work_result_test).expect("Error decoding WorkResult 0");
        let res = work_result_decoded.encode().expect("Error encoding WorkResult 0");
        assert_eq!(test, res);
    }

    #[test]
    fn run_work_result_1() {
        let test = read_codec_test("data/codec/data/work_result_1.bin");
        let mut work_result_test = BytesReader::new(&test);
        let work_result_decoded = WorkResult::decode(&mut work_result_test).expect("Error decoding WorkResult 1");
        let res = work_result_decoded.encode().expect("Error encoding WorkResult 1");
        assert_eq!(test, res);
    }

    #[test]
    fn run_work_report() {
        let test = read_codec_test("data/codec/data/work_report.bin");
        let mut work_report_test = BytesReader::new(&test);
        let work_report_decoded = WorkReport::decode(&mut work_report_test).expect("Error decoding WorkReport");
        let res = work_report_decoded.encode().expect("Error encoding WorkReport");
        assert_eq!(test, res);
    }

    #[test]
    fn run_tickets_extrinsic() {
        let test = read_codec_test("data/codec/data/tickets_extrinsic.bin");
        let mut tickets_extrinsic_test = BytesReader::new(&test);
        let ticket_decoded = TicketEnvelope::decode(&mut tickets_extrinsic_test).expect("Error decoding TicketEnvelope");
        let res = TicketEnvelope::encode(&ticket_decoded.as_slice()).expect("Error encoding TicketEnvelope");
        assert_eq!(test, res);
    }

    #[test]
    fn run_disputes_extrinsic() {
        let test = read_codec_test("data/codec/data/disputes_extrinsic.bin");
        let mut disputes_extrinsic_test = BytesReader::new(&test);
        let disputes_decoded = DisputesExtrinsic::decode(&mut disputes_extrinsic_test).expect("Error decoding DisputesExtrinsic");
        let res = DisputesExtrinsic::encode(&disputes_decoded).expect("Error encoding DisputesExtrinsic");
        assert_eq!(test, res);
    }

    #[test]
    fn run_preimages_extrinsic() {
        let test = read_codec_test("data/codec/data/preimages_extrinsic.bin");
        let mut preimages_extrinsic_test = BytesReader::new(&test);
        let preimages_decoded = PreimagesExtrinsic::decode(&mut preimages_extrinsic_test).expect("Error decoding PreimagesExtrinsic");
        let res = PreimagesExtrinsic::encode(&preimages_decoded).expect("Error encoding PreimagesExtrinsic");
        assert_eq!(test, res);
    }

    #[test]
    fn run_assurances_extrinsic() {
        let test = read_codec_test("data/codec/data/assurances_extrinsic.bin");
        let mut assurances_extrinsic_test = BytesReader::new(&test);
        let assurances_decoded = AssurancesExtrinsic::decode(&mut assurances_extrinsic_test).expect("Error decoding AssurancesExtrinsic");
        let res = AssurancesExtrinsic::encode(&assurances_decoded).expect("Error encoding AssurancesExtrinsic");
        assert_eq!(test, res);
    }

    #[test]
    fn run_guarantees_extrinsic() {
        let test = read_codec_test("data/codec/data/guarantees_extrinsic.bin");
        let mut guarantees_extrinsic_test = BytesReader::new(&test);
        let guarantees_decoded = GuaranteesExtrinsic::decode(&mut guarantees_extrinsic_test).expect("Error decoding GuaranteesExtrinsic");
        let res = GuaranteesExtrinsic::encode(&guarantees_decoded).expect("Error encoding GuaranteesExtrinsic");
        assert_eq!(test, res);
    }

    #[test]
    fn run_header_0() {
        let test = read_codec_test("data/codec/data/header_0.bin");
        let mut header_test = BytesReader::new(&test);
        let header_decoded = Header::decode(&mut header_test).expect("Error decoding Header");
        let res = Header::encode(&header_decoded).expect("Error encoding Header");
        assert_eq!(test, res);
    }

    #[test]
    fn run_header_1() {
        let test = read_codec_test("data/codec/data/header_1.bin");
        let mut header_test = BytesReader::new(&test);
        let header_decoded = Header::decode(&mut header_test).expect("Error decoding Header");
        let res = Header::encode(&header_decoded).expect("Error encoding Header");
        assert_eq!(test, res);
    }

    #[test]
    fn run_block() {
        let test = read_codec_test("data/codec/data/block.bin");
        let mut block_test = BytesReader::new(&test);
        let block_decoded = Block::decode(&mut block_test).expect("Error decoding Header");
        let res = Block::encode(&block_decoded).expect("Error encoding Header");
        assert_eq!(test, res);
    }
}
