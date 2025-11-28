use crate::{dev_accounts, net_utils};
use crate::message;
use jam_types::{*};
use quinn::{Connection, Endpoint};
use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use utils::{hex, log};

pub async fn run_server(validator_index: ValidatorIndex, endpoint: Endpoint) -> std::result::Result<(), Box<dyn Error + Send + Sync>> {

    /*rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|e| format!("Failed to install ring provider: {:?}", e))?;*/
    
    /*let port = 40000 + validator_index;
    let bind_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
    let server_config = net_utils::load_server_config(validator_index)?;
    
    let endpoint = match quinn::Endpoint::server(server_config, bind_addr) {
        Ok(endpoint) => endpoint,
        Err(e) => {
            log::error!("Failed to create the endpoint: {:?}", e);
            return Err(Box::new(e));
        }
    };*/

    

    while let Some(conn) = endpoint.accept().await {
        log::debug!("Incoming connection attempt from {}", conn.remote_address());
        tokio::spawn(async move {
            match conn.await {
                Ok(connection) => {
                    let id_account = connection.remote_address().port().saturating_sub(40000);
                    let dev_accounts = dev_accounts::parse_dev_accounts();
                    log::debug!("New connection established from {} bandersnatch public: {}", connection.remote_address(), hex::encode(&dev_accounts[id_account as usize].bandersnatch_public));
                    dev_accounts::add_dev_account(dev_accounts[id_account as usize].bandersnatch_public, connection.clone());
                    handle_connection(connection).await;
                }
                Err(e) => {
                    log::error!("Connection error: {}", e);
                }
            }
        });
    }

    endpoint.wait_idle().await;

    Ok(())
}


async fn handle_connection(connection: Connection) {

    log::debug!("New connection established from {}", connection.remote_address());
    let conn_clone = connection.clone();
    // Wait for a new stream
    while let Ok((send_stream, mut recv_stream)) = connection.accept_bi().await {
        let conn_clone = conn_clone.clone(); // Clone for each spawn
        tokio::spawn(async move {
            let mut stream_kind_buf = [0u8; 1];
            if recv_stream.read_exact(&mut stream_kind_buf).await.is_ok() {
                log::debug!("Received stream kind {:?}", stream_kind_buf);
                let conn_info = message::ConnectionInfo {
                    connection: conn_clone,
                    send_stream,
                    recv_stream,
                    kind: stream_kind_buf[0]
                };
                message::handle_stream(conn_info).await;
            }
        });
    }
}




