pub mod block;
pub mod ticket;
pub mod state;

use crate::jamnp_types::*;
use codec::{BytesReader, Decode, Encode};
use codec::generic_codec::decode_from_bytes;
use quinn::{SendStream, RecvStream, Connection};
use std::collections::{BTreeMap, HashMap};
use std::sync::{LazyLock, Mutex, OnceLock};
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::sync::{mpsc, oneshot};
use tools::{hex, log};

pub const BLOCK_ANNOUNCEMENT: StreamKind = 0;
pub const BLOCK_REQUEST: StreamKind = 128;
pub const STATE_REQUEST: StreamKind = 129;
pub const TICKET_GENERATION: StreamKind = 131;
pub const TICKET_PROXY: StreamKind = 132;

pub struct ConnectionInfo {
    pub connection: Connection,
    pub send_stream: SendStream,
    pub recv_stream: RecvStream,
    pub kind: u8,
}

pub struct NetworkMessage {
    pub msg: Vec<u8>
}

impl NetworkMessage {
    
    pub fn new(kind: u8, payload: Vec<u8>) -> Vec<u8> {
        let len = payload.len() as u32;
        let len_bytes = len.to_le_bytes();
        let mut msg = Vec::with_capacity(5 + payload.len());
        msg.push(kind);
        msg.extend_from_slice(&len_bytes);
        msg.extend(payload);
        msg
    }

    pub async fn recv(recv_stream: &mut RecvStream) -> Result<Vec<u8>, NetworkError> {
        let mut len_msg = [0u8; 4];
        recv_stream.read_exact(&mut len_msg).await?;
        let len_msg = u32::from_le_bytes(len_msg) as usize;
        let mut buffer = vec![0u8; len_msg];
        recv_stream.read_exact(&mut buffer).await?;
        Ok(buffer)
    }

    pub async fn send(msg_kind: u8, payload: Vec<u8>, send_stream: &mut SendStream) -> Result<(), NetworkError> {
        let message = NetworkMessage::new(msg_kind, payload);
        send_stream.write_all(&message).await?;
        send_stream.finish()?;
        Ok(())
    }

    pub async fn send_up(msg_kind: u8, payload: Vec<u8>, send_stream: &mut SendStream) -> Result<(), NetworkError> {
        let message = NetworkMessage::new(msg_kind, payload);
        send_stream.write_all(&message).await?;
        Ok(())
    }

    // Sends a length-prefixed message without kind byte, then finishes the stream.
    // Use for CE stream responses where the kind byte was already sent at stream open.
    pub async fn reply(payload: Vec<u8>, send_stream: &mut SendStream) -> Result<(), NetworkError> {
        let len_bytes = (payload.len() as u32).to_le_bytes();
        send_stream.write_all(&len_bytes).await?;
        send_stream.write_all(&payload).await?;
        send_stream.finish()?;
        Ok(())
    }
} 

