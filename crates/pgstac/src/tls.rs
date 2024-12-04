use rustls::{
    client::danger::{ServerCertVerified, ServerCertVerifier},
    crypto::{verify_tls12_signature, verify_tls13_signature, CryptoProvider},
    pki_types::{CertificateDer, ServerName, UnixTime},
    ClientConfig, Error, RootCertStore,
};
use std::sync::Arc;
use tokio_postgres_rustls::MakeRustlsConnect;
use webpki_roots::TLS_SERVER_ROOTS;

/// Make an unverified tls.
///
/// # Examples
///
/// ```
/// #[cfg(feature = "tls")]
/// {
/// let tls = pgstac::make_unverified_tls();
/// }
/// ```
pub fn make_unverified_tls() -> MakeRustlsConnect {
    let mut root_cert_store = RootCertStore::empty();
    root_cert_store.extend(TLS_SERVER_ROOTS.iter().cloned());

    let verifier = DummyTlsVerifier::default();
    let config = ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(verifier))
        .with_no_client_auth();
    MakeRustlsConnect::new(config)
}

// A TLS verifier copied from the `sqlx` library
// [here](https://github.com/launchbadge/sqlx/blob/a892ebc6e283f443145f92bbc7fce4ae44547331/sqlx-core/src/net/tls/tls_rustls.rs#L208)
// This verifier _does not_ verify certificates, but instead decrypts TLS
// connections using default cipher codes
#[derive(Debug)]
struct DummyTlsVerifier {
    provider: Arc<CryptoProvider>,
}

impl ServerCertVerifier for DummyTlsVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, Error> {
        verify_tls12_signature(
            message,
            cert,
            dss,
            &self.provider.signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, Error> {
        verify_tls13_signature(
            message,
            cert,
            dss,
            &self.provider.signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        self.provider
            .signature_verification_algorithms
            .supported_schemes()
    }
}

impl Default for DummyTlsVerifier {
    fn default() -> Self {
        Self {
            provider: Arc::new(rustls::crypto::ring::default_provider()),
        }
    }
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn connect() {
        let tls = super::make_unverified_tls();
        let (_, _) = tokio_postgres::connect("host=/var/run/postgresql", tls)
            .await
            .unwrap();
    }
}
