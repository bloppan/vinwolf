pub mod client;
pub mod dev_accounts;
pub mod server;
pub mod net_utils;
pub mod jamnp_codec;
pub mod jamnp_types;

use std::error::Error;

use crate::client::run_client;
use crate::server::run_server;

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

#[tokio::main]
async fn main() -> Result<()> {
    
    utils::log::Builder::from_env(utils::log::Env::default().default_filter_or("debug"))
        .with_dotenv(true)
        .init();

    
    //let client_handler = tokio::spawn(run_client());
    let server_handler = tokio::spawn(run_server());

    //client_handler.await;
    server_handler.await;

    Ok(())
}
