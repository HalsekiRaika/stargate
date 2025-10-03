use error_stack::{Report, ResultExt};
use http_msgsign_draft::errors::VerificationError;
use http_msgsign_draft::sign::VerifierKey;
use rsa::pkcs1::DecodeRsaPublicKey;
use rsa::pkcs1v15::{Signature, VerifyingKey};
use rsa::RsaPublicKey;
use rsa::signature::Verifier;
use crate::error::KeyLoadError;

#[derive(Debug, Clone)]
pub struct RsaVerifierKey {
    url: String,
    key: VerifyingKey<sha2::Sha256>
}

impl RsaVerifierKey {
    pub fn load(url: String, key: impl AsRef<str>) -> Result<RsaVerifierKey, Report<KeyLoadError>> {
        let key = RsaPublicKey::from_pkcs1_pem(key.as_ref())
            .change_context_lazy(|| KeyLoadError::IncorrectKey)
            .attach("key format only supports the PKCS#1v1.5 format, which is common on ActivityPub.")?;
        
        Ok(Self {
            url,
            key: VerifyingKey::new(key),
        })
    }
}

impl VerifierKey for RsaVerifierKey {
    fn id(&self) -> String {
        self.url.clone()
    }
    
    fn algorithm(&self) -> String {
        "rsa-sha256".to_string()
    }
    
    fn verify(&self, target: &[u8], sig: &[u8]) -> Result<(), VerificationError> {
        let sig = Signature::try_from(sig)
            .map_err(|e| VerificationError::Crypto(Box::new(e)))?;
        self.key.verify(target, &sig)
            .map_err(|e| VerificationError::Crypto(Box::new(e)))?;
        Ok(())
    }
}
