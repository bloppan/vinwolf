use constants::node::VALIDATORS_COUNT;
use jam_types::{ValidatorIndex, Ed25519Public};
use quinn::{ClientConfig, ServerConfig, TransportConfig};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName, UnixTime, PrivatePkcs8KeyDer};
use rustls::crypto::{verify_tls12_signature, verify_tls13_signature};
use rustls::client::danger::{ServerCertVerifier, ServerCertVerified, HandshakeSignatureValid};
use rustls::server::danger::{ClientCertVerifier, ClientCertVerified};
use rustls::{SignatureScheme, DistinguishedName};
use rustls::crypto::ring::default_provider;
use rustls::crypto::CryptoProvider;
use std::sync::Arc;
use std::error::Error;

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

pub fn am_i_the_preferred_initiator(my_key: &Ed25519Public, peer_key: &Ed25519Public) -> bool {
    let cond = ((my_key[31] > 127) ^ (peer_key[31] > 127)) ^ (my_key < peer_key);
    cond
}

/// Grid width W = floor(sqrt(V)) as defined in JAMNP-S.
pub const fn grid_width() -> usize {
    // Integer sqrt: largest w such that w*w <= V
    let mut w = 1;
    while (w + 1) * (w + 1) <= VALIDATORS_COUNT {
        w += 1;
    }
    w
}

/// Two validators in the same epoch are grid neighbours if they share
/// the same row (index / W) or the same column (index % W).
pub fn is_grid_neighbour(my_index: ValidatorIndex, other_index: ValidatorIndex) -> bool {
    if my_index == other_index {
        return false;
    }
    let w = grid_width();
    let same_row = (my_index as usize / w) == (other_index as usize / w);
    let same_col = (my_index as usize % w) == (other_index as usize % w);
    same_row || same_col
}

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

pub fn parse_pem_certs(pem_data: &[u8]) -> Result<Vec<CertificateDer<'static>>> {
    
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

pub fn parse_pem_private_key(pem_data: &[u8]) -> Result<PrivateKeyDer<'static>> {

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

const ALPN_PROTOCOL: &[u8] = "jamnp-s/0/2bf11dc5".as_bytes();

pub fn load_credentials(validator_index: ValidatorIndex) -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>), > { 

    let cert_pem = std::fs::read(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("src/certs/node{}/cert.pem", validator_index)))?;
    let key_pem = std::fs::read(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("src/certs/node{}/key.pem", validator_index)))?;

    let certs: Vec<CertificateDer> = parse_pem_certs(&cert_pem).expect("Error parsing certificate");
    if certs.is_empty() {
        return Err("No valid certificates found in cert.pem".into());
    }

    let key_der = parse_pem_private_key(&key_pem)?;

    return Ok((certs, key_der));
}

pub fn load_server_config(certs: Vec<CertificateDer<'static>>, key_der: PrivateKeyDer<'static>) -> Result<ServerConfig, > {

    let mut server_crypto = rustls::ServerConfig::builder()
        .with_client_cert_verifier(SkipClientVerification::new())
        .with_single_cert(certs, key_der)?;

    server_crypto.alpn_protocols = vec![ALPN_PROTOCOL.to_vec()];

    let mut server_config = ServerConfig::with_crypto(Arc::new(
        quinn::crypto::rustls::QuicServerConfig::try_from(server_crypto)?
    ));
    let mut transport_config = TransportConfig::default();
    transport_config.max_concurrent_bidi_streams(100u32.into());
    server_config.transport = Arc::new(transport_config);

    return Ok(server_config);
}

pub fn load_client_config(certs: Vec<CertificateDer<'static>>, key_der: PrivateKeyDer<'static>) -> Result<ClientConfig, > {

    let mut client_crypto = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(SkipServerVerification::new())
        .with_client_auth_cert(certs, key_der)?;

    client_crypto.alpn_protocols = vec![ALPN_PROTOCOL.to_vec()];

    let mut client_config = ClientConfig::new(Arc::new(
        quinn::crypto::rustls::QuicClientConfig::try_from(client_crypto)?,
    ));
    let mut transport_config = TransportConfig::default();
    transport_config.max_concurrent_bidi_streams(100u32.into());
    client_config.transport_config(Arc::new(transport_config));

    return Ok(client_config);
}

#[derive(Debug)]
pub struct SkipClientVerification(Arc<CryptoProvider>);

impl SkipClientVerification {
    pub fn new() -> Arc<Self> {
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