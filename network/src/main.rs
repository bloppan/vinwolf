pub mod client;
pub mod dev_accounts;
pub mod message;
pub mod net_utils;
pub mod jamnp_codec;
pub mod jamnp_types;
pub mod server;

use std::error::Error;

use network::node_config;

use crate::client::run_client;
use crate::server::run_server;

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

fn print_help() {    
    println!("vinwolf network");
    println!();
    println!("\x1b[1mUsage example:\x1b[0m\n");
    println!("cargo run --dev-validator N");
    println!();
}

#[tokio::main]
async fn main() -> Result<()> {
    
    utils::log::Builder::from_env(utils::log::Env::default().default_filter_or("debug"))
        .with_dotenv(true)
        .init();

    let args = std::env::args().collect::<Vec<_>>();
    let mut validator_index = 0;

    match args[1].as_ref() { 
        "--dev-validator" => {
            validator_index = args[2].parse().expect("Error parsing --dev-validator index");
            println!("Validator index: {validator_index}");
        },
        _ => {
            println!("Error: Unknown argument '{}'", args[1]);
            print_help();
        },
    };

    node_config::set_account_id(validator_index);

    //let client_handler = tokio::spawn(run_client());
    let server_handler = tokio::spawn(run_server(validator_index));

    //client_handler.await;
    server_handler.await;

    Ok(())
}
