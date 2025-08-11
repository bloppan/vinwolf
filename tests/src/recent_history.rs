#[cfg(test)]
mod tests {

    //use crate::TestBody;
    use codec::{Decode, BytesReader};
    use jam_types::{RecentBlocks, ReportedWorkPackages};
    use crate::test_types::InputHistory;

    fn run_test(filename: &str) {
        
        let test_content = utils::common::read_bin_file(std::path::Path::new(&format!("jamtestvectors/history/data/{}", filename))).unwrap();

        /*let test_body: Vec<TestBody> = vec![TestBody::InputHistory, TestBody::RecentBlocks, TestBody::RecentBlocks];

        if encode_decode_test(&test_content, &test_body).is_err() {
            panic!("Error encoding/decoding test file: {}", filename);
        }*/
        
        let mut reader = BytesReader::new(&test_content);
        let input = InputHistory::decode(&mut reader).expect("Error decoding InputHistory");
        let expected_pre_state = RecentBlocks::decode(&mut reader).expect("Error decoding pre RecentBlocks");
        let expected_post_state = RecentBlocks::decode(&mut reader).expect("Error decoding post RecentBlocks");
        
        let mut reported_work_packages = ReportedWorkPackages::default();
        for wp in &input.work_packages {
            reported_work_packages.push((wp.hash, wp.exports_root));
        }

        state_handler::recent_history::set(expected_pre_state.clone());

        let mut recent_history_state = state_handler::recent_history::get();
        assert_eq!(expected_pre_state, recent_history_state);

        recent_history::process(&mut recent_history_state,
                            &input.header_hash, 
                            &input.parent_state_root, 
                            &reported_work_packages);

        recent_history::finalize(&mut recent_history_state,
                                &input.header_hash, 
                                &input.accumulate_root, 
                                &reported_work_packages);

        assert_eq!(expected_post_state, recent_history_state);
    }

    #[test]
    fn run_recent_history_tests() {
        
        dotenv::dotenv().ok();
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
        log::info!("Recent history tests");
        
        let test_files = vec![
            // Empty history queue.
            "progress_blocks_history-1.bin",
            // Not empty nor full history queue.
            "progress_blocks_history-2.bin",
            // Fill the history queue.
            "progress_blocks_history-3.bin",
            // Shift the history queue.
            "progress_blocks_history-4.bin",
        ];
        for file in test_files {
            log::info!("Running test: {}", file);
            run_test(file);
        }
    }
}   