use crate::NetResult;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use rustls::{
    client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier},
    pki_types::{CertificateDer, PrivatePkcs8KeyDer, ServerName, UnixTime},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{fs, io::Write, net::SocketAddr, path::Path, sync::Arc, time::Duration};
use subtle::ConstantTimeEq;

const ALPN: &[u8] = b"midnight-survival/1";
const SERVER_NAME: &str = "midnight-survival";

/// Deliberately not Debug: the invitation contains the access secret.
#[derive(Clone)]
pub struct Invitation {
    pub fingerprint: [u8; 32],
    pub token: [u8; 32],
}

impl Invitation {
    pub fn parse(code: &str) -> NetResult<Self> {
        let code = code.trim();
        if code.len() > 128 {
            return Err("Некоректний код доступу".into());
        }
        let bytes = URL_SAFE_NO_PAD
            .decode(
                code.strip_prefix("MS1-")
                    .ok_or("Код має починатися з MS1-")?,
            )
            .map_err(|_| "Некоректний код доступу")?;
        if bytes.len() != 64 {
            return Err("Некоректний код доступу".into());
        }
        Ok(Self {
            fingerprint: bytes[..32].try_into().unwrap(),
            token: bytes[32..].try_into().unwrap(),
        })
    }

    pub fn code(&self) -> String {
        let bytes: Vec<_> = self
            .fingerprint
            .iter()
            .chain(&self.token)
            .copied()
            .collect();
        format!("MS1-{}", URL_SAFE_NO_PAD.encode(bytes))
    }

    pub fn authenticates(&self, token: &[u8; 32]) -> bool {
        bool::from(self.token.ct_eq(token))
    }
}

#[derive(Serialize, Deserialize)]
pub struct ServerIdentity {
    certificate: Vec<u8>,
    private_key: Vec<u8>,
    token: [u8; 32],
}

impl ServerIdentity {
    pub fn generate() -> NetResult<Self> {
        let cert = rcgen::generate_simple_self_signed(vec![SERVER_NAME.into()])
            .map_err(|e| e.to_string())?;
        let mut token = [0; 32];
        getrandom::fill(&mut token).map_err(|e| e.to_string())?;
        Ok(Self {
            certificate: cert.cert.der().to_vec(),
            private_key: cert.key_pair.serialize_der(),
            token,
        })
    }

    pub fn load_or_create(directory: &Path) -> NetResult<Self> {
        let path = directory.join("identity.json");
        if path.exists() {
            return serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?)
                .map_err(|e| format!("Cannot read server identity: {e}"));
        }
        fs::create_dir_all(directory).map_err(|e| e.to_string())?;
        let identity = Self::generate()?;
        let temporary = directory.join("identity.json.tmp");
        let mut options = fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options
            .open(&temporary)
            .map_err(|e| format!("Cannot create server identity: {e}"))?;
        file.write_all(&serde_json::to_vec(&identity).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        file.sync_all().map_err(|e| e.to_string())?;
        fs::rename(temporary, path).map_err(|e| e.to_string())?;
        Ok(identity)
    }

    pub fn invitation(&self) -> Invitation {
        Invitation {
            fingerprint: Sha256::digest(&self.certificate).into(),
            token: self.token,
        }
    }

    pub fn endpoint(&self, address: SocketAddr) -> NetResult<quinn::Endpoint> {
        let provider = Arc::new(rustls::crypto::ring::default_provider());
        let mut crypto = rustls::ServerConfig::builder_with_provider(provider)
            .with_protocol_versions(&[&rustls::version::TLS13])
            .map_err(|e| e.to_string())?
            .with_no_client_auth()
            .with_single_cert(
                vec![CertificateDer::from(self.certificate.clone())],
                PrivatePkcs8KeyDer::from(self.private_key.clone()).into(),
            )
            .map_err(|e| e.to_string())?;
        crypto.alpn_protocols = vec![ALPN.to_vec()];
        let mut config = quinn::ServerConfig::with_crypto(Arc::new(
            quinn::crypto::rustls::QuicServerConfig::try_from(crypto).map_err(|e| e.to_string())?,
        ));
        config.transport_config(transport());
        quinn::Endpoint::server(config, address).map_err(|e| e.to_string())
    }
}

pub fn transport() -> Arc<quinn::TransportConfig> {
    let mut config = quinn::TransportConfig::default();
    config
        .max_concurrent_bidi_streams(1u8.into())
        .max_concurrent_uni_streams(0u8.into())
        .max_idle_timeout(Some(
            Duration::from_secs(10)
                .try_into()
                .expect("constant timeout"),
        ))
        .keep_alive_interval(Some(Duration::from_secs(1)))
        .stream_receive_window((64u32 * 1024).into())
        .receive_window((128u32 * 1024).into())
        .send_window(128 * 1024)
        .datagram_receive_buffer_size(Some(64 * 1024))
        .datagram_send_buffer_size(16 * 1024);
    Arc::new(config)
}

pub fn client_endpoint(bind: SocketAddr, invitation: &Invitation) -> NetResult<quinn::Endpoint> {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let mut crypto = rustls::ClientConfig::builder_with_provider(provider.clone())
        .with_protocol_versions(&[&rustls::version::TLS13])
        .map_err(|e| e.to_string())?
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(PinnedCertificate {
            fingerprint: invitation.fingerprint,
            provider,
        }))
        .with_no_client_auth();
    crypto.alpn_protocols = vec![ALPN.to_vec()];
    let mut config = quinn::ClientConfig::new(Arc::new(
        quinn::crypto::rustls::QuicClientConfig::try_from(crypto).map_err(|e| e.to_string())?,
    ));
    config.transport_config(transport());
    let mut endpoint = quinn::Endpoint::client(bind).map_err(|e| e.to_string())?;
    endpoint.set_default_client_config(config);
    Ok(endpoint)
}

pub fn connect(endpoint: &quinn::Endpoint, address: SocketAddr) -> NetResult<quinn::Connecting> {
    endpoint
        .connect(address, SERVER_NAME)
        .map_err(|e| e.to_string())
}

/// The out-of-band invitation pins the exact certificate. Rustls still verifies
/// the TLS handshake signature and possession of the corresponding private key.
#[derive(Debug)]
struct PinnedCertificate {
    fingerprint: [u8; 32],
    provider: Arc<rustls::crypto::CryptoProvider>,
}

impl ServerCertVerifier for PinnedCertificate {
    fn verify_server_cert(
        &self,
        certificate: &CertificateDer<'_>,
        _: &[CertificateDer<'_>],
        _: &ServerName<'_>,
        _: &[u8],
        _: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        let actual: [u8; 32] = Sha256::digest(certificate.as_ref()).into();
        if actual != self.fingerprint {
            return Err(rustls::Error::General(
                "Сертифікат сервера не відповідає коду запрошення".into(),
            ));
        }
        Ok(ServerCertVerified::assertion())
    }
    fn verify_tls12_signature(
        &self,
        message: &[u8],
        certificate: &CertificateDer<'_>,
        signature: &rustls::DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls12_signature(
            message,
            certificate,
            signature,
            &self.provider.signature_verification_algorithms,
        )
    }
    fn verify_tls13_signature(
        &self,
        message: &[u8],
        certificate: &CertificateDer<'_>,
        signature: &rustls::DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls13_signature(
            message,
            certificate,
            signature,
            &self.provider.signature_verification_algorithms,
        )
    }
    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        self.provider
            .signature_verification_algorithms
            .supported_schemes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invitation_roundtrip_and_wrong_token() {
        let identity = ServerIdentity::generate().unwrap();
        let invite = identity.invitation();
        let parsed = Invitation::parse(&invite.code()).unwrap();
        assert_eq!(parsed.fingerprint, invite.fingerprint);
        assert!(parsed.authenticates(&invite.token));
        assert!(!parsed.authenticates(&[0; 32]));
        assert!(Invitation::parse("1234").is_err());
    }
}
