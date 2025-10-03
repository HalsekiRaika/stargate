use std::sync::Arc;
use error_stack::{Report, ResultExt};
use redb::{Database, TableDefinition};
use crate::error::SetupError;


pub trait ActorPublicKeyCache: 'static + Sync + Send {

}


const ACTOR_PUBLIC_KEY_CACHE_TABLE: TableDefinition<&str, Vec<u8>> = TableDefinition::new("actor_public_key_cache");

#[derive(Debug, Clone)]
pub struct ActorPublicKeyCacheClient {
    client: Arc<Database>
}

impl ActorPublicKeyCacheClient {
    pub fn setup() -> Result<Self, Report<SetupError>> {
        let temp = tempfile::NamedTempFile::new()
            .change_context_lazy(|| SetupError)
            .attach("cannot create temporary file.")?;
        
        let db = Database::create(temp)
            .change_context_lazy(|| SetupError)?;
        
        Ok(Self {
            client: Arc::new(db)
        })
    }
}

