use std::fs::OpenOptions;
use std::io::Read;
use std::path::Path;
use error_stack::{Report, ResultExt};
use http_msgsign_draft::sign::SignerKey;
use rsa::pkcs1::DecodeRsaPrivateKey;
use rsa::pkcs1v15::SigningKey;
use rsa::signature::{SignatureEncoding, Signer};
use crate::error::KeyLoadError;

#[derive(Debug)]
pub struct RsaSignerKey {
    url: String,
    key: SigningKey<sha2::Sha256>
}

impl RsaSignerKey {
    pub fn load(
        hostname: String, 
        owner_id: String, 
        path: impl AsRef<Path>
    ) -> Result<RsaSignerKey, Report<KeyLoadError>> {
        let mut load = OpenOptions::new()
            .read(true)
            .open(path)
            .change_context_lazy(|| KeyLoadError::Io)?;
        
        let mut buf = String::new();
        load.read_to_string(&mut buf)
            .change_context_lazy(|| KeyLoadError::Io)?;
        
        let key = SigningKey::from_pkcs1_pem(&buf)
            .change_context_lazy(|| KeyLoadError::IncorrectKey)
            .attach("key format only supports the PKCS#1v1.5 format, which is common on ActivityPub.")?;
        
        Ok(Self { 
            url: format!("https://{}/{}#main-key", hostname, owner_id),
            key 
        })
    }
}

impl SignerKey for RsaSignerKey {
    fn id(&self) -> String {
        self.url.clone()
    }
    
    fn algorithm(&self) -> String {
        "rsa-sha256".to_string()
    }
    
    fn sign(&self, target: &[u8]) -> Vec<u8> {
        self.key
            .sign(target)
            .to_vec()
    }
}
