#[cfg(test)]
mod tests {
    
    use std::collections::HashSet;
    use std::path::Path;
    use utils::log;
    use vinwolf_target::{process_all_bins, process_all_dirs, process_trace};
    
    const REPORTS_FUZZER_DIR: &str = "/home/bernar/workspace/jam-conformance/fuzz-reports/0.7.0/traces";

    #[test]
    fn run_reports_fuzzer_tests() {

        log::Builder::from_env(log::Env::default().default_filter_or("debug"))
        .with_dotenv(true)
        .init();

        let dir_base = Path::new(REPORTS_FUZZER_DIR);
        let skip: HashSet<String> = ["RETIRED"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        
        let all_dirs = process_all_dirs(dir_base, &skip).unwrap();
        
        for dir in all_dirs.iter() {
            log::info!("Test {:?} processed successfully", dir);
        }
    }
    // 1758637363
    
    const FUZZ_REPORT: &str = "/home/bernar/workspace/jam-conformance/fuzz-reports/0.7.0/traces/_new/1758621173";
    //const FUZZ_REPORT: &str = "/home/bernar/workspace/vinwolf/tests/javajam-trace/stf/state_transitions/";

    #[test]
    fn run_single_fuzz_report() {

        log::Builder::from_env(log::Env::default().default_filter_or("debug"))
        .with_dotenv(true)
        .init();

        let dir_base = Path::new(FUZZ_REPORT);
        let _ = process_all_bins(dir_base);
    }   

    const TRACES_DIR: &str = "/home/bernar/workspace/jam-conformance/fuzz-reports/0.7.0/traces/_new/";

    #[test]
    fn run_all_traces_tests() {

        log::Builder::from_env(log::Env::default().default_filter_or("debug"))
        .with_dotenv(true)
        .init();
        
        let dir_base = Path::new(TRACES_DIR);
        let skip: HashSet<String> = ["1758708840"]
            .iter()
            .map(|s| s.to_string())
            .collect();

        let all_dirs = process_all_dirs(dir_base, &skip).unwrap();
        for dir in all_dirs.iter() {
            log::info!("Test {:?} processed successfully", dir);
        }
    }   

    #[test]
    fn run_preimages_light_traces_tests() {
        log::Builder::from_env(log::Env::default().default_filter_or("debug"))
        .with_dotenv(true)
        .init();
    
        let dir_base = Path::new(TRACES_DIR).join("preimages_light");
        run_traces(&dir_base);
    }

    #[test]
    fn run_preimages_traces_tests() {
        log::Builder::from_env(log::Env::default().default_filter_or("debug"))
        .with_dotenv(true)
        .init();

        let dir_base = Path::new(TRACES_DIR).join("preimages");
        run_traces(&dir_base);
    }

    #[test]
    fn run_storage_light_traces_tests() {
        log::Builder::from_env(log::Env::default().default_filter_or("debug"))
        .with_dotenv(true)
        .init();

        let dir_base = Path::new(TRACES_DIR).join("storage_light");
        run_traces(&dir_base);
    }

    #[test]
    fn run_storage_traces_tests() {
        log::Builder::from_env(log::Env::default().default_filter_or("debug"))
        .with_dotenv(true)
        .init();

        let dir_base = Path::new(TRACES_DIR).join("storage");
        run_traces(&dir_base);
    }

    #[test]
    fn run_safrole_traces_tests() {

        let dir_base = Path::new(TRACES_DIR).join("safrole");
        run_traces(&dir_base);
    }


    #[test]
    fn run_fallback_traces_tests() {
        log::Builder::from_env(log::Env::default().default_filter_or("debug"))
        .with_dotenv(true)
        .init();

        let dir_base = Path::new(TRACES_DIR).join("fallback");
        run_traces(&dir_base);
    }

    #[test]
    fn run_single_trace() {

        log::Builder::from_env(log::Env::default().default_filter_or("debug"))
        .with_dotenv(true)
        .init();

        let start = std::time::Instant::now();
        process_trace(Path::new("/home/bernar/workspace/vinwolf/tests/jamtestvectors/traces/safrole/00000089.bin"));
        let duration = start.elapsed();
        log::info!("* TOTAL time taken: {:?}", duration);
    }

    fn run_traces(path: &Path) {
        /*log::Builder::from_env(log::Env::default().default_filter_or("debug"))
        .with_dotenv(true)
        .init();*/

        let start = std::time::Instant::now();
        let _ = process_all_bins(path);
        let end = start.elapsed();
        println!("All tests processed in: {:?}", end);
    }
}