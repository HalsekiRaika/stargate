use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Read;
use std::path::Path;

use error_stack::{Report, ResultExt};
use serde::{Deserialize, Serialize};

use crate::error::ConfigError;

pub fn init_or_load(path: impl AsRef<Path>) -> Result<Config, Report<ConfigError>> {
    let path = path.as_ref();
    let mut load = OpenOptions::new()
        .read(true)
        .open(path)
        .change_context_lazy(|| ConfigError::Io)?;
    
    let mut buf = Vec::new();
    load.read_to_end(&mut buf)
        .change_context_lazy(|| ConfigError::Io)?;
    
    let val: Config = toml::from_slice(&buf)
        .change_context_lazy(|| ConfigError::InvalidFormat)?;
    
    Ok(val)
}


#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(test, derive(Eq, PartialEq))]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub server: ServerConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(test, derive(Eq, PartialEq))]
#[serde(rename_all = "kebab-case")]
pub struct ServerConfig {
    pub bind_address: String,
    pub bind_port: Option<u16>,
    pub host_name: String,
    pub host_key: String,
    pub overrides: HashMap<String, Overrides>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(test, derive(Eq, PartialEq))]
#[serde(rename_all = "kebab-case")]
pub struct Overrides {
    pub certificate: Option<String>,
    pub authority: Option<String>
}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn config_load() {
        let loaded_config = init_or_load("../config.toml").unwrap();
        let template_config = Config {
            server: ServerConfig {
                bind_address: "0.0.0.0".to_string(),
                bind_port: Some(55555),
                host_name: "stargate.localhost".to_string(),
                host_key: "../private.key".to_string(),
                overrides: vec![
                    ("misskey.localhost".to_string(), Overrides { 
                        certificate: None,
                        authority: Some("misskey.localhost:4430".to_string()) 
                    }),
                    ("mastodon.localhost".to_string(), Overrides { 
                        certificate: None,
                        authority: Some ("mastodon.localhost:4431".to_string()) 
                    }),
                ].into_iter().collect(),
            },
        };
        assert_eq!(loaded_config, template_config)
    }
}
