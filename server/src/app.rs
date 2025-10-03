use std::ops::Deref;
use std::sync::Arc;
use error_stack::Report;
use driver::config::Config;
use crate::error::UnrecoverableError;

pub async fn init(config: Config) -> Result<AppModule, Report<UnrecoverableError>> {
    Ok(AppModule(
        Arc::new(Handler {
            
        })
    ))
}

#[derive(Debug)]
pub struct AppModule(Arc<Handler>);

impl Clone for AppModule {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl Deref for AppModule {
    type Target = Handler;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct Handler {
    
}