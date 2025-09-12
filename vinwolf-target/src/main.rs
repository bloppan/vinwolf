#![allow(dead_code)]
#![allow(unused_variables)]
// Vamos Marcos!
use std::path::PathBuf;
use std::collections::HashSet;
use utils::log;

mod fuzz;
use fuzz::*;
use fuzz::VINWOLF_INFO;
use constants::BUILD_PROFILE;

fn print_help() {    
    println!("vinwolf-target mode {}", BUILD_PROFILE);
    println!();
    //println!("\x1b[1m\x1b[4mUsage:\x1b[0m\x1b[1m vinwolf\x1b[0m [OPTIONS] <command>");
    println!("\x1b[1mUsage example:\x1b[0m\n");
    //println!("\x1b[4mCommands\x1b[0m");
    println!("vinwolf --fuzz\t\t\t The default path is /tmp/jam_conformance.sock");
    println!("vinwolf --fuzz other_path.sock");
    println!();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let vinwolf_info = &*VINWOLF_INFO;

    let args = std::env::args().collect::<Vec<_>>();

    if args.len() == 1 {
        print_help();
        return Ok(());
    }

    /*log::Builder::from_env(log::Env::default().default_filter_or("debug"))
        .with_dotenv(true)
        .init();*/

    /*dotenv().ok();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();*/
    //env_logger::init();


    /*dotenv().ok();
    env_logger::Builder::new()
    .filter_level(log::LevelFilter::Debug)
    .init();*/

    match args[1].as_ref() { 
        "--help" | "-h" => {
            print_help();
            return Ok(())
        },
        "--version" | "-v" => {
            println!("{:?} target-version: {}.{}.{} GP-version: {}.{}.{} {} mode",
            String::from_utf8(vinwolf_info.name.clone()).unwrap(),
            vinwolf_info.app_version.major, 
                vinwolf_info.app_version.minor, 
                vinwolf_info.app_version.patch,
                vinwolf_info.jam_version.major, 
                vinwolf_info.jam_version.minor, 
                vinwolf_info.jam_version.patch,
                BUILD_PROFILE,
        );
            return Ok(())
        },
        "--target" => {
            let mut path: PathBuf = PathBuf::from("/tmp/jam_conformance.sock");
            
            if args.len() > 2 {
                let args: Vec<String> = std::env::args().collect();
                path = PathBuf::from(&args[2]);
            }
            
            let socket_path = path.to_str().unwrap();

            let result = run_fuzzer(socket_path);

            println!("End target: {:?}", result);
        }
        "--fuzz" => {

            let mut path: PathBuf = PathBuf::from("/tmp/jam_conformance.sock");

            if args.len() > 2 {
                let args: Vec<String> = std::env::args().collect();
                path = PathBuf::from(&args[2]);
            }

            let _ = std::fs::remove_file(path.clone());
            
            let socket_path = path.to_str().unwrap();
            let _ = run_unix_server(socket_path);
        },
        "--process-dirs" => {

            if args.len() < 3 {
                println!("Bad arguments");
                return Ok(());
            }

            let mut skip_dirs: HashSet<String> = HashSet::new();

            if args.len() > 3 {
                for i in 3..args.len() {
                    skip_dirs.insert(args[i].clone());
                }
            }
            println!("Start to process all dirs");
            let start = std::time::Instant::now();
            let _ = vinwolf_target::process_all_dirs(&PathBuf::from(&args[2]), &skip_dirs);
            let end = start.elapsed();
            println!("All tests processed in {:?}", end);
        },
        "--process-traces" => {

            if args.len() != 3 {
                println!("Bad arguments");
                return Ok(());
            }
            let start = std::time::Instant::now();
            let _ = vinwolf_target::process_all_bins(&PathBuf::from(&args[2]));
            let end = start.elapsed();
            println!("All tests processed in {:?}", end);
        },
        "--process-trace" => {

            if args.len() != 3 {
                println!("Bad arguments");
                return Ok(());
            }
            let start = std::time::Instant::now();
            vinwolf_target::process_trace(&PathBuf::from(&args[2]));
            let end = start.elapsed();
            println!("All tests processed in {:?}", end);
        }
        "--speed-test" => {

            if args.len() != 3 {
                println!("Bad arguments");
                return Ok(());
            }
            let start = std::time::Instant::now();
            for _ in 0..50 {
                let _ = vinwolf_target::process_all_bins(&PathBuf::from(&args[2]));
            }
            let end = start.elapsed();
            println!("All tests processed in {:?}", end);
        },
        _ => {
            println!("Error: Unknown argument '{}'", args[1]);
            print_help();
        },
    };

    return Ok(())
    
}

