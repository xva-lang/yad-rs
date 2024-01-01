use axum_server::tls_rustls::RustlsConfig;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct SSLConfig {
    pub certificate: String,
    pub private_key: String,
}

impl SSLConfig {
    pub(crate) async fn to_pem_file(&self) -> std::io::Result<RustlsConfig> {
        RustlsConfig::from_pem_file(self.certificate.as_str(), self.private_key.as_str()).await
    }
}
