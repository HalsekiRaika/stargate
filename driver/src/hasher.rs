use http_msgsign_draft::digest::{ContentHasher, DigestHash};

pub struct Sha256Hasher;

impl ContentHasher for Sha256Hasher {
    const DIGEST_ALG: &'static str = "SHA-256";
    
    fn hash(content: &[u8]) -> DigestHash {
        use sha2::Digest;
        let mut hasher = <sha2::Sha256 as Digest>::new();
        hasher.update(content);
        DigestHash::new(hasher.finalize().to_vec())
    }
}
