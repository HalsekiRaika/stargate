use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicKey {
    id: String,
    owner: String,
    public_key_pem: String,
}

impl PublicKey {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn owner(&self) -> &str {
        &self.owner
    }

    pub fn public_key_pem(&self) -> &str {
        &self.public_key_pem
    }
}


#[cfg(test)]
mod test {
    use serde::{Deserialize, Serialize};
    use crate::entities::links::types::PublicKey;
    
    #[test]
    fn test_deserialize() {
        // language=JSON
        let json = r#"
{
  "publicKey": {
    "id": "key-123",
    "owner": "user-456",
    "publicKeyPem": ""
  }
}
        "#;
        
        #[derive(Deserialize, Serialize)]
        struct Flatten(PublicKey);
        let key: Flatten = serde_json::from_str(json).unwrap();
        let key = key.0;
        assert_eq!(key.id(), "key-123");
    }
}