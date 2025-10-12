pub mod client;
pub mod server;
pub mod net_utils;

use std::error::Error;

use crate::client::run_client;
use crate::server::run_server;

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

#[tokio::main]
async fn main() -> Result<()> {

    run_client().await?;
    //run_server().await?;
    
    Ok(())
}
