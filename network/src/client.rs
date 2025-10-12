use quinn::{ClientConfig, Endpoint, TransportConfig};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName, UnixTime, PrivatePkcs8KeyDer};
use rustls::crypto::{verify_tls12_signature, verify_tls13_signature};
use rustls::client::danger::{ServerCertVerifier, ServerCertVerified, HandshakeSignatureValid};
use rustls::{Error as RustlsError, SignatureScheme};
use rustls::crypto::ring::default_provider;
use rustls::crypto::CryptoProvider;
use std::io::{Cursor, Read};
use std::net::{IpAddr, Ipv6Addr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::error::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

use crate::net_utils::{parse_pem_private_key, parse_pem_certs};


pub async fn run_client() -> Result<()> {

    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|e| format!("Failed to install ring provider: {:?}", e))?;

    let node_alt_name = "elfaiiixcuzmzroa34lajwp52cdsucikaxdviaoeuvnygdi3imtba";
    let node_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 40005);

    let genesis_hash = "2bf11dc5";
    let alpn_protocol = format!("jamnp-s/0/{}", genesis_hash).into_bytes();

    
    let cert_pem = std::fs::read("/home/bernar/workspace/vinwolf/network/src/node0/cert.pem")?;
    let key_pem = std::fs::read("/home/bernar/workspace/vinwolf/network/src/node0/key.pem")?;

    let certs: Vec<CertificateDer> = parse_pem_certs(&cert_pem)?;
    if certs.is_empty() {
        return Err("No valid certificates found in cert.pem".into());
    }

    let key_der = parse_pem_private_key(&key_pem)?;

    let mut client_crypto = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(SkipServerVerification::new())
        .with_client_auth_cert(certs, key_der)?;

    client_crypto.alpn_protocols = vec![alpn_protocol];

    let mut client_config = ClientConfig::new(Arc::new(
        quinn::crypto::rustls::QuicClientConfig::try_from(client_crypto)?,
    ));
    let mut transport_config = TransportConfig::default();
    transport_config.max_concurrent_bidi_streams(100u32.into());
    client_config.transport_config(Arc::new(transport_config));

    let bind_addr = SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), 40000);
    let mut endpoint = Endpoint::client(bind_addr)?;
    endpoint.set_default_client_config(client_config);

    println!("Connecting to {} at {}", node_alt_name, node_addr);

    let connection = endpoint
        .connect(node_addr, node_alt_name)?
        .await?;

    println!("Connected to {}", node_alt_name);

    let (mut send_stream, mut recv_stream) = connection.open_bi().await?;
    send_stream.write_all(&[0]).await?;
    println!("Sent stream kind 0");

    let handshake = vec![15, 140, 101, 194, 104, 174, 233, 240, 82, 49, 141, 19, 229, 55, 117, 252, 165, 108, 150, 250, 80, 25, 40, 178, 168, 52, 196, 232, 108, 37, 140, 85, 138, 102, 59, 0, 1, 15, 140, 101, 194, 104, 174, 233, 240, 82, 49, 141, 19, 229, 55, 117, 252, 165, 108, 150, 250, 80, 25, 40, 178, 168, 52, 196, 232, 108, 37, 140, 85, 138, 102, 59, 0];
    let len_bytes = (handshake.len() as u32).to_le_bytes();
    send_stream.write_all(&len_bytes).await?;
    send_stream.write_all(&handshake).await?;
    println!("Sent handshake response");

    loop {
        let mut len_buf = [0u8; 4];
        match recv_stream.read_exact(&mut len_buf).await {
            Ok(()) => {
                let len = u32::from_le_bytes(len_buf) as usize;
                if len > 1024 * 1024 {
                    println!("Received unreasonably large message length: {}", len);
                    break;
                }
                let mut buffer = vec![0u8; len];
                match recv_stream.read_exact(&mut buffer).await {
                    Ok(()) => {
                        println!("Received message ({} bytes): {:?}", len, buffer);
                    }
                    Err(e) => {
                        println!("Error reading message content: {}", e);
                        break;
                    }
                }
            }
            Err(e) => {
                println!("Error reading message length: {}", e);
                break;
            }
        }
    }

    connection.closed().await;
    println!("Connection closed");

    Ok(())
}

#[derive(Debug)]
pub struct SkipServerVerification(Arc<CryptoProvider>);

impl SkipServerVerification {
    pub fn new() -> Arc<Self> {
        Arc::new(Self(Arc::new(default_provider())))
    }
}

impl ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp: &[u8],
        _now: UnixTime,
    ) -> std::result::Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
        verify_tls12_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
        verify_tls13_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.0.signature_verification_algorithms.supported_schemes()
    }
}