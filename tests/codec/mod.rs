use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

extern crate vinwolf;

use vinwolf::refine::RefineContext;
use vinwolf::refine::{encode_refine_ctx, decode_refine_ctx};
use vinwolf::work::package::{encode_work_item, decode_work_item, encode_work_pkg, decode_work_pkg};

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
        let mut file = File::open(&path).expect("Failed to open JSON file");
        let mut content = Vec::new();
        let success = file.read_to_end(&mut content);
        return content;
    }

    #[test]
    fn run_refine_context_test() {
        let refine_test = read_codec_test("data/codec/data/refine_context.bin");
        let refine_decoded = decode_refine_ctx(&refine_test);
        let refine_result = encode_refine_ctx(&refine_decoded);
        assert_eq!(refine_test, refine_result);
    }

    #[test]
    fn run_work_item_test() {
        let work_item_test = read_codec_test("data/codec/data/work_item.bin");
        let work_item_decoded = decode_work_item(&work_item_test);
        let work_item_result = encode_work_item(&work_item_decoded);
        assert_eq!(work_item_test, work_item_result);
    }

    #[test]
    fn run_work_package_test() {
        let work_pkg_test = read_codec_test("data/codec/data/work_package.bin");
        let work_pkg_decoded = decode_work_pkg(&work_pkg_test);
        let work_pkg_result = encode_work_pkg(&work_pkg_decoded);
        assert_eq!(work_pkg_test, work_pkg_result);
    }

}
