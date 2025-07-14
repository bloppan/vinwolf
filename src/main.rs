#![allow(dead_code)]
#![allow(unused_variables)]

use std::path::PathBuf;

use vinwolf::node::fuzz::*;
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
use log::{info, warn, debug, trace, error};

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

    // Generar algunos mensajes de log
    /*debug!("Este es un mensaje de bbbb");
    info!("Este es un mensaje de info");
    warn!("Este es un mensaje de advertencia");
    trace!("este es un mensaje de trace");
    error!("este es un mensaje de error");*/

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
        /*"--dir_test" => {
            let files = match read_files_in_directory(&args[2]) {
                Ok(files) => files,
                Err(_) => return Ok(())
            };

            for file in files.iter() {
                let _ = import_state_block(file);
            }
        },
        "--file_test" => {
            let file_path = std::path::Path::new(&args[2]);
            let _ = import_state_block(&file_path);
        },*/
        "--fuzz" => {

            let mut path: PathBuf = PathBuf::from("/tmp/jam_conformance.sock");

            if args.len() > 2 {
                let args: Vec<String> = std::env::args().collect();
                path = PathBuf::from(&args[2]);
            }

            if let Err(e) = std::fs::remove_file(path.clone()) {
                if e.kind() != std::io::ErrorKind::NotFound {
                // Si el error es diferente de "No encontrado", lo reportamos
                }
            }
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

