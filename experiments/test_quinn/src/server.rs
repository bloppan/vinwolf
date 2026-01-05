use quinn::{Endpoint, TransportConfig, ServerConfig};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName, UnixTime, PrivatePkcs8KeyDer};
use rustls::crypto::{verify_tls12_signature, verify_tls13_signature};
use rustls::client::danger::{ServerCertVerifier, ServerCertVerified, HandshakeSignatureValid};
use rustls::server::danger::{ClientCertVerifier, ClientCertVerified};
use rustls::{SignatureScheme, DistinguishedName};
use rustls::crypto::ring::default_provider;
use rustls::crypto::CryptoProvider;
use std::io::Read;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::error::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

fn base64_decode(input: &str) -> Result<Vec<u8>> {
    let table = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = Vec::new();
    let mut buffer = 0u32;
    let mut bits = 0u32;
    let mut padding = 0u8;

    for &b in input.as_bytes() {
        if b == b'=' {
            padding += 1;
            continue;
        }
        if let Some(idx) = table.iter().position(|&c| c == b) {
            buffer = (buffer << 6) | idx as u32;
            bits += 6;
            if bits >= 8 {
                bits -= 8;
                output.push(((buffer >> bits) & 0xff) as u8);
            }
        } else {
            return Err("Invalid base64 character".into());
        }
    }

    if padding > 2 {
        return Err("Invalid padding".into());
    }

    Ok(output)
}

fn parse_pem_certs(pem_data: &[u8]) -> Result<Vec<CertificateDer<'static>>> {
    let pem_str = std::str::from_utf8(pem_data)?;
    let lines = pem_str.lines().map(|l| l.trim()).collect::<Vec<_>>();
    let mut certs = Vec::new();
    let mut collecting = false;
    let mut b64_lines = Vec::new();

    for line in lines {
        if line == "-----BEGIN CERTIFICATE-----" {
            collecting = true;
            b64_lines.clear();
        } else if line == "-----END CERTIFICATE-----" {
            if collecting {
                let b64 = b64_lines.join("");
                let der = base64_decode(&b64)?;
                certs.push(CertificateDer::from(der));
                collecting = false;
            }
        } else if collecting && !line.is_empty() {
            b64_lines.push(line);
        }
    }

    Ok(certs)
}

fn parse_pem_private_key(pem_data: &[u8]) -> Result<PrivateKeyDer<'static>> {
    let pem_str = std::str::from_utf8(pem_data)?;
    let lines = pem_str.lines().map(|l| l.trim()).collect::<Vec<_>>();
    let mut collecting = false;
    let mut b64_lines = Vec::new();

    for line in lines {
        if line == "-----BEGIN PRIVATE KEY-----" {
            collecting = true;
            b64_lines.clear();
        } else if line == "-----END PRIVATE KEY-----" {
            if collecting {
                let b64 = b64_lines.join("");
                let der = base64_decode(&b64)?;
                let pkcs8 = PrivatePkcs8KeyDer::from(der);
                return Ok(PrivateKeyDer::Pkcs8(pkcs8));
            }
        } else if collecting && !line.is_empty() {
            b64_lines.push(line);
        }
    }

    Err("No private key found".into())
}

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

#[derive(Debug)]
struct SkipClientVerification(Arc<CryptoProvider>);

impl SkipClientVerification {
    fn new() -> Arc<Self> {
        Arc::new(Self(Arc::new(default_provider())))
    }
}

impl ClientCertVerifier for SkipClientVerification {
    fn root_hint_subjects(&self) -> &[DistinguishedName] {
        &[]
    }

    fn verify_client_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _now: UnixTime,
    ) -> std::result::Result<ClientCertVerified, rustls::Error> {
        Ok(ClientCertVerified::assertion())
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

pub async fn run_server() -> Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|e| format!("Failed to install ring provider: {:?}", e))?;

    let bind_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 40000);

    let genesis_hash = "2bf11dc5";
    let alpn_protocol = format!("jamnp-s/0/{}", genesis_hash).into_bytes();

    let cert_pem = std::fs::read("node0/cert.pem")?;
    let key_pem = std::fs::read("node0/key.pem")?;

    let certs: Vec<CertificateDer> = parse_pem_certs(&cert_pem)?;
    if certs.is_empty() {
        return Err("No valid certificates found in cert.pem".into());
    }

    let key_der = parse_pem_private_key(&key_pem)?;

    let mut server_crypto = rustls::ServerConfig::builder()
        .with_client_cert_verifier(SkipClientVerification::new())
        .with_single_cert(certs, key_der)?;

    server_crypto.alpn_protocols = vec![alpn_protocol];

    let mut server_config = ServerConfig::with_crypto(Arc::new(
        quinn::crypto::rustls::QuicServerConfig::try_from(server_crypto)?
    ));
    let mut transport_config = TransportConfig::default();
    transport_config.max_concurrent_bidi_streams(100u32.into());
    server_config.transport = Arc::new(transport_config);

    let endpoint = Endpoint::server(server_config, bind_addr)?;

    println!("Listening on {}", bind_addr);

    while let Some(conn) = endpoint.accept().await {
        println!("Incoming connection attempt from {}", conn.remote_address());
        tokio::spawn(async move {
            match conn.await {
                Ok(connection) => {
                    println!("New connection established from {}", connection.remote_address());

                    while let Ok((mut send_stream, mut recv_stream)) = connection.accept_bi().await {
                        tokio::spawn(async move {
                            let mut kind_buf = [0u8; 1];
                            if recv_stream.read_exact(&mut kind_buf).await.is_ok() {
                                println!("Received stream kind {:?}", kind_buf);
                                let handshake = vec![15, 140, 101, 194, 104, 174, 233, 240, 82, 49, 141, 19, 229, 55, 117, 252, 165, 108, 150, 250, 80, 25, 40, 178, 168, 52, 196, 232, 108, 37, 140, 85, 138, 102, 59, 0, 1, 15, 140, 101, 194, 104, 174, 233, 240, 82, 49, 141, 19, 229, 55, 117, 252, 165, 108, 150, 250, 80, 25, 40, 178, 168, 52, 196, 232, 108, 37, 140, 85, 138, 102, 59, 0];
                                let len_bytes = (handshake.len() as u32).to_le_bytes();
                                send_stream.write_all(&len_bytes).await.ok();
                                send_stream.write_all(&handshake).await.ok();
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
                            }
                        });
                    }
                }
                Err(e) => {
                    println!("Connection error: {}", e);
                }
            }
        });
    }

    endpoint.wait_idle().await;

    Ok(())
}