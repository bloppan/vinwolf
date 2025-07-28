#![allow(dead_code)]
#![allow(unused_variables)]

use std::path::PathBuf;

use vinwolf::node::fuzz::*;
use vinwolf::node::utils::*;

use dotenv::dotenv;

fn print_help() {    
    println!("vinwolf target");
    println!();
    //println!("\x1b[1m\x1b[4mUsage:\x1b[0m\x1b[1m vinwolf\x1b[0m [OPTIONS] <command>");
    println!("\x1b[1mUsage example:\x1b[0m\n");
    //println!("\x1b[4mCommands\x1b[0m");
    println!("vinwolf --fuzz\t\t\t The default path is /tmp/jam_conformance.sock");
    println!("vinwolf --fuzz other_path.sock");
    println!();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let args = std::env::args().collect::<Vec<_>>();

    if args.len() == 1 {
        print_help();
        return Ok(());
    }

    dotenv().ok();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    //env_logger::init();

    match args[1].as_ref() { 
        /*"--prueba" => {
            let mut array = [1, 2, 3, 4, 5, 6, 7];
            println!("array: {:02x?}", array[0..4].to_vec());
        },*/
        "--help" | "-h" => {
            print_help();
            return Ok(())
        },
        "--version" | "-v" => {
            println!("vinwolf GP version: 0.6.6");
            return Ok(())
        },
        /*"--target" => {
            let mut path: PathBuf = PathBuf::from("/tmp/jam_conformance.sock");

            if args.len() > 2 {
                let args: Vec<String> = std::env::args().collect();
                path = PathBuf::from(&args[2]);
            }

            let socket_path = path.to_str().unwrap();

            connect_to_unix_socket(socket_path).await?;
        }
        "--dir_test" => {
            let files = match read_filenames_in_dir(&args[2]) {
                Ok(files) => files,
                Err(_) => return Ok(())
            };

            for file in files.iter() {
                let _ = import_block(file);
            }
        },
        "--file_test" => {
            let file_path = std::path::Path::new(&args[2]);
            let _ = import_block(&file_path);
        },*/
        "--fuzz" => {

            let mut path: PathBuf = PathBuf::from("/tmp/jam_conformance.sock");

            if args.len() > 2 {
                let args: Vec<String> = std::env::args().collect();
                path = PathBuf::from(&args[2]);
            }

            let _ = std::fs::remove_file(path.clone());
            
            let socket_path = path.to_str().unwrap();
            run_unix_server(socket_path).await?;
        }
        _ => {
            println!("Error: Unknown argument '{}'", args[1]);
            print_help();
        },
    };

    return Ok(())
    
}

