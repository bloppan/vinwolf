#[cfg(test)]
mod tests {
    
    use std::collections::HashSet;
    use std::path::Path;
    use dotenv::dotenv;
    use vinwolf_target::{process_all_bins, process_all_dirs, process_trace};
    
    const REPORTS_FUZZER_DIR: &str = "/home/bernar/workspace/jam-stuff/fuzz-reports/0.6.7/traces";

    #[test]
    fn run_reports_fuzzer_tests() {

        use dotenv::dotenv;
        dotenv().ok();
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).init();

        let dir_base = Path::new(REPORTS_FUZZER_DIR);
        let skip: HashSet<String> = ["1754982087", "RETIRED"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        
        let all_dirs = process_all_dirs(dir_base, &skip).unwrap();
        
        for dir in all_dirs.iter() {
            log::info!("Test {:?} processed successfully", dir);
        }
    }

    const FUZZ_REPORT: &str = "/home/bernar/workspace/jam-stuff/fuzz-reports/0.6.7/traces/TESTING/1755796851";

    #[test]
    fn run_single_fuzz_report() {

        use dotenv::dotenv;
        dotenv().ok();
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).init();

        let dir_base = Path::new(FUZZ_REPORT);
        let _ = process_all_bins(dir_base);
    }   

    const TRACES_DIR: &str = "/home/bernar/workspace/vinwolf/tests/jamtestvectors/traces";

    #[test]
    fn run_all_traces_tests() {

        dotenv().ok();
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).init();
        
        let dir_base = Path::new(TRACES_DIR);
        let skip: HashSet<String> = [""]
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

        let dir_base = Path::new(TRACES_DIR).join("preimages_light");
        run_traces(&dir_base);
    }

    #[test]
    fn run_preimages_traces_tests() {

        let dir_base = Path::new(TRACES_DIR).join("preimages");
        run_traces(&dir_base);
    }

    #[test]
    fn run_storage_light_traces_tests() {

        let dir_base = Path::new(TRACES_DIR).join("storage_light");
        run_traces(&dir_base);
    }

    #[test]
    fn run_storage_traces_tests() {

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

        let dir_base = Path::new(TRACES_DIR).join("fallback");
        run_traces(&dir_base);
    }

    #[test]
    fn run_single_trace() {
        
        dotenv().ok();
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).init();
        let start = std::time::Instant::now();
        process_trace(Path::new("/home/bernar/workspace/vinwolf/tests/jamtestvectors/traces/safrole/00000089.bin"));
        let duration = start.elapsed();
        log::info!("* TOTAL time taken: {:?}", duration);
    }

    fn run_traces(path: &Path) {

        dotenv().ok();
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).init();
        let _ = process_all_bins(path);
    }
}