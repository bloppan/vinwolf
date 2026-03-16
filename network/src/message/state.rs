use crate::jamnp_types::*;
use crate::message::{NetworkMessage, STATE_REQUEST};
use codec::{BytesReader, Decode, Encode};
use jam_types::{*};
use quinn::Connection;
use tools::{hex, log};

pub struct StateRequestInfo {
    pub header_hash: OpaqueHash,
    pub keyvals: Vec<KeyValue>,
    pub max_size: u32,
}

pub async fn request(header_hash: OpaqueHash, connection: Connection) -> Result<GlobalState, NetworkError> {

    let (mut send_stream, mut recv_stream) = connection.open_bi().await?;

    let key_start = [0u8; 31];
    let key_end = [0xFFu8; 31];
    let max_size = u32::MAX;
    
    let payload = [header_hash.encode(), key_start.encode(), key_end.encode(), max_size.encode()].concat();

    NetworkMessage::send(STATE_REQUEST, payload, &mut send_stream).await?;
    log::debug!("State request sent to address: {:?} for header: {}", connection.remote_address(), hex::encode(&header_hash));
    
    let _boundary_nodes = NetworkMessage::recv(&mut recv_stream).await?;
    let state_keyvals_blob = NetworkMessage::recv(&mut recv_stream).await?;
    
    drop(recv_stream);

    let mut reader = BytesReader::new(&state_keyvals_blob);
    let mut keyvals: Vec<KeyValue> = vec![];
    let mut c: usize = 0;

    while reader.get_position() < state_keyvals_blob.len() {

        c = c.saturating_add(1);

        let keyval = KeyValue::decode(&mut reader).map_err(|e| {
            log::error!("Failed to decode keyvalue {c}: {:?}", e);
            NetworkError::Decode(e)
        })?;

        keyvals.push(keyval);
    }

    let mut global_state = GlobalState::default();
    
    misc::parse_state_keyvals(&keyvals, &mut global_state)?;
    
    // Initialize the verifiers 
    safrole::verifier::init_all(&global_state);
    
    log::debug!("Total keyvalues decoded: {c}");

    Ok(global_state)
}