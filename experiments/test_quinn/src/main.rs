use quinn::{ClientConfig, Endpoint, TransportConfig};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName, UnixTime};
use rustls::crypto::{verify_tls12_signature, verify_tls13_signature};
use rustls::client::danger::{ServerCertVerifier, ServerCertVerified, HandshakeSignatureValid};
use rustls::{Error as RustlsError, SignatureScheme};
use rustls::crypto::ring::default_provider;
use rustls::crypto::CryptoProvider;
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::sync::Arc;
use anyhow::{Result, bail};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug)]
struct SkipServerVerification(Arc<CryptoProvider>);

impl SkipServerVerification {
    fn new() -> Arc<Self> {
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
    ) -> Result<ServerCertVerified, RustlsError> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, RustlsError> {
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
    ) -> Result<HandshakeSignatureValid, RustlsError> {
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

#[tokio::main]
async fn main() -> Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|e| anyhow::anyhow!("Failed to install ring provider: "))?;

    let node_alt_name = "elfaiiixcuzmzroa34lajwp52cdsucikaxdviaoeuvnygdi3imtba";
    let node_addr = SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 40005);

    let genesis_hash = "2bf11dc5";
    let alpn_protocol = format!("jamnp-s/0/{}", genesis_hash).into_bytes();

    let mut cert_file = std::io::Cursor::new(std::fs::read("cert.pem")?);
    let mut key_file = std::io::Cursor::new(std::fs::read("key.pem")?);

    let certs: Vec<CertificateDer> = certs(&mut cert_file)
        .map(|result| result.map(CertificateDer::from))
        .collect::<Result<Vec<_>, _>>()?;
    if certs.is_empty() {
        bail!("No valid certificates found in cert.pem");
    }

    let mut keys = pkcs8_private_keys(&mut key_file);
    let key = PrivateKeyDer::from(
        keys.next()
            .ok_or_else(|| anyhow::anyhow!("No valid private keys found in key.pem"))??,
    );

    let mut client_crypto = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(SkipServerVerification::new())
        .with_client_auth_cert(certs, key)?;

    client_crypto.alpn_protocols = vec![alpn_protocol];

    let mut client_config = ClientConfig::new(Arc::new(
        quinn::crypto::rustls::QuicClientConfig::try_from(client_crypto)?,
    ));
    let mut transport_config = TransportConfig::default();
    transport_config.max_concurrent_bidi_streams(100u32.into());
    client_config.transport_config(Arc::new(transport_config));

    let bind_addr = SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), 0);
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

    // Envía handshake de respuesta (para testing, copia el recibido; ajusta con tu data real)
    let handshake = vec![15, 140, 101, 194, 104, 174, 233, 240, 82, 49, 141, 19, 229, 55, 117, 252, 165, 108, 150, 250, 80, 25, 40, 178, 168, 52, 196, 232, 108, 37, 140, 85, 138, 102, 59, 0, 1, 15, 140, 101, 194, 104, 174, 233, 240, 82, 49, 141, 19, 229, 55, 117, 252, 165, 108, 150, 250, 80, 25, 40, 178, 168, 52, 196, 232, 108, 37, 140, 85, 138, 102, 59, 0];
    let len_bytes = (handshake.len() as u32).to_le_bytes();
    send_stream.write_all(&len_bytes).await?;
    send_stream.write_all(&handshake).await?;
    println!("Sent handshake response");

    // NO CIERRES send_stream; mantén abierto para persistencia

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
                        // Opcional: Responde con anuncio si es necesario (e.g., si recibes nuevo bloque)
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



use base32::{Alphabet, encode};

const PUBKEY: [u8; 32] = [
    0x3b, 0x6a, 0x27, 0xbc, 0xce, 0xb6, 0xa4, 0x2d,
    0x62, 0xa3, 0xa8, 0xd0, 0x2a, 0x6f, 0x0d, 0x73,
    0x65, 0x32, 0x15, 0x77, 0x1d, 0xe2, 0x43, 0xa6,
    0x3a, 0xc0, 0x48, 0xa1, 0x8b, 0x59, 0xda, 0x29,
];

//const PUBKEY: [u8; 32] = [57u8, 100, 133, 71, 244, 149, 234, 172, 144, 159, 120, 64, 51, 14, 115, 205, 248, 55, 248, 219, 138, 79, 88, 55, 40, 116, 44, 149, 138, 218, 200, 43];


/*const PUBKEY: [u8; 32] = [
    0xf8, 0xfd, 0x75, 0xc5, 0xd9, 0x52, 0xbf, 0x61,
    0x37, 0x7d, 0xa7, 0x4b, 0x8a, 0x03, 0x59, 0x51,
    0x89, 0x9c, 0x75, 0xe4, 0xc1, 0x52, 0xb9, 0x3f,
    0x34, 0x28, 0xec, 0x1b, 0x17, 0xa6, 0x48, 0x08,
];*/

fn dns_alt_name_from_pubkey(pk: &[u8; 32]) -> String {
    let b32 = encode(Alphabet::Rfc4648 { padding: false }, pk).to_lowercase();
    format!("e{b32}")
}

#[test]
fn alternative_name_test() {
    let result = dns_alt_name_from_pubkey(&PUBKEY);
    println!("result ascii: {}", result);
    println!("result hex: {}", hex::encode(result));
}