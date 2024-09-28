use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

extern crate vinwolf;

use vinwolf::refine::RefineContext;

use vinwolf::work::package::WorkItem;
use vinwolf::work::package::WorkPackage;
use vinwolf::work::package::WorkResult;
use vinwolf::work::package::WorkReport;
use vinwolf::extrinsic::TicketEnvelope;
use vinwolf::extrinsic::DisputesExtrinsic;


#[derive(Deserialize, Debug, PartialEq)]
struct Testcase {
    /*name: String,
    #[serde(rename = "initial-regs")]
    initial_regs: [u32; 13],
    #[serde(rename = "initial-pc")]
    initial_pc: u32,
    #[serde(rename = "initial-page-map")]
    initial_page_map: Vec<PageMap>,
    #[serde(rename = "initial-memory")]
    initial_memory: Vec<PageMap>,
    #[serde(rename = "initial-gas")]
    initial_gas: i64,
    program: Vec<u8>,
    #[serde(rename = "expected-status")]
    expected_status: String,
    #[serde(rename = "expected-regs")]
    expected_regs: [u32; 13],
    #[serde(rename = "expected-pc")]
    expected_pc: u32,
    #[serde(rename = "expected-memory")]
    expected_memory: Vec<PageMap>,
    #[serde(rename = "expected-gas")]
    expected_gas: i64,*/
}

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

    #[test]
    fn run_refine_context_test() {
        let test = read_codec_test("data/codec/data/refine_context.bin");
        let refine_decoded: RefineContext = RefineContext::decode(&test);
        let refine_result = refine_decoded.encode();
        assert_eq!(test, refine_result);
    }

    #[test]
    fn run_work_item_test() {
        let test = read_codec_test("data/codec/data/work_item.bin");
        let work_item_decoded = WorkItem::decode(&test);
        let work_item_result = work_item_decoded.encode();
        assert_eq!(test, work_item_result);
    }

    #[test]
    fn run_work_package_test() {
        let test = read_codec_test("data/codec/data/work_package.bin");
        let work_pkg_decoded = WorkPackage::decode(&test);
        let work_pkg_result = work_pkg_decoded.encode();
        assert_eq!(test, work_pkg_result);
    }

    #[test]
    fn run_work_result_0() {
        let test = read_codec_test("data/codec/data/work_result_0.bin");
        let work_result_decoded = WorkResult::decode(&test);
        let work_result = work_result_decoded.encode();
        assert_eq!(test, work_result);
    }

    #[test]
    fn run_work_result_1() {
        let test = read_codec_test("data/codec/data/work_result_1.bin");
        let work_result_decoded = WorkResult::decode(&test);
        let work_result = work_result_decoded.encode();
        assert_eq!(test, work_result);
    }

    #[test]
    fn run_work_report() {
        let test = read_codec_test("data/codec/data/work_report.bin");
        let work_report_decoded = WorkReport::decode(&test);
        let res = work_report_decoded.encode();
        assert_eq!(test, res);
    }

    #[test]
    fn run_tickets_extrinsic() {
        let test = read_codec_test("data/codec/data/tickets_extrinsic.bin");
        let ticket_decoded = TicketEnvelope::decode(&test);
        let res = TicketEnvelope::encode(&ticket_decoded);
        assert_eq!(test, res);
    }

    #[test]
    fn run_disputes_extrinsic() {
        let test = read_codec_test("data/codec/data/disputes_extrinsic.bin");
        let disputes_decoded = DisputesExtrinsic::decode(&test);
        let res = DisputesExtrinsic::encode(&disputes_decoded);
        assert_eq!(test, res);
    }

}
