use error_stack::{Report, ResultExt};
use http_msgsign_draft::errors::VerificationError;
use http_msgsign_draft::sign::VerifierKey;
use rsa::pkcs8::DecodePublicKey;
use rsa::pkcs1v15::{Signature, VerifyingKey};
use rsa::signature::{Verifier};
use rsa::RsaPublicKey;
use rsa::sha2::Sha256;
use crate::config::Config;
use crate::error::KeyLoadError;

#[derive(Debug, Clone)]
pub struct RsaVerifierKey {
    url: String,
    raw: String,
    key: VerifyingKey<Sha256>
}

impl RsaVerifierKey {
    pub fn new(url: String, key: impl AsRef<str>) -> Result<RsaVerifierKey, Report<KeyLoadError>> {
        let key = key.as_ref();
        let pubkey = RsaPublicKey::from_public_key_pem(key)
            .change_context_lazy(|| KeyLoadError::IncorrectKey)
            .attach("key format only supports the PKCS#1v1.5 format, which is common on ActivityPub.")?;
        
        Ok(Self {
            url,
            raw: key.to_string(),
            key: VerifyingKey::new(pubkey),
        })
    }
    
    // noinspection DuplicatedCode
    pub fn read_local_file(config: Config) -> Result<RsaVerifierKey, Report<KeyLoadError>> {
        let host = config.server.host_name;
        let path = config.server.keypair.public;
        let key = std::fs::read_to_string(&path)
            .change_context_lazy(|| KeyLoadError::Io)
            .attach(format!("failed to read public key from {}", path))?;
        Self::new(host, key)
    }
    
    pub fn as_pem(&self) -> &str {
        &self.raw
    }
}

impl VerifierKey for RsaVerifierKey {
    fn id(&self) -> String {
        self.url.clone()
    }
    
    fn algorithm(&self) -> String {
        "rsa-sha256".to_string()
    }
    
    #[tracing::instrument(skip_all, name = "rsa-rs")]
    fn verify(&self, target: &[u8], sig: &[u8]) -> Result<(), VerificationError> {
        let sig = match Signature::try_from(sig) {
            Ok(sig) => sig,
            Err(e) => {
                tracing::error!("signature format error: {}", e);
                return Err(VerificationError::Crypto(Box::new(e)));
            },
        };
        if let Err(e) = self.key.verify(target, &sig) {
            tracing::error!("actual signature: {:x?}", sig);
            tracing::error!("signature verification failed: {:?}", e);
            return Err(VerificationError::Crypto(Box::new(e)));
        }
        Ok(())
    }
}
