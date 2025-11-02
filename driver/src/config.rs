use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Read;
use std::net::{IpAddr, SocketAddr};
use std::path::Path;
use std::str::FromStr;

use error_stack::{Report, ResultExt};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::ConfigError;

pub fn init_or_load(path: impl AsRef<Path>) -> Result<Config, Report<ConfigError>> {
    let path = path.as_ref();
    let mut load = OpenOptions::new()
        .read(true)
        .open(path)
        .change_context_lazy(|| ConfigError::Io)
        .attach_with(|| format!("Check that {path:?}/config.toml is valid."))?;
    
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
    pub keypair: KeypairConfig,
    
    pub overrides: HashMap<String, Overrides>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(test, derive(Eq, PartialEq))]
#[serde(rename_all = "kebab-case")]
pub struct KeypairConfig {
    pub private: String,
    pub public: String,
}

#[derive(Debug, Clone)]
pub enum ResolveAddr {
    Socket(SocketAddr),
    Ip(IpAddr),
}

impl ResolveAddr {
    pub fn to_socket_addr(&self, default_port: u16) -> SocketAddr {
        match self {
            ResolveAddr::Socket(addr) => *addr,
            ResolveAddr::Ip(ip) => SocketAddr::new(*ip, default_port),
        }
    }
    
    pub fn socket_addr(&self) -> Option<SocketAddr> {
        match self {
            ResolveAddr::Socket(addr) => Some(*addr),
            ResolveAddr::Ip(_) => None,
        }
    }
    
    pub fn ip_addr(&self) -> IpAddr {
        match self {
            ResolveAddr::Socket(addr) => addr.ip(),
            ResolveAddr::Ip(ip) => *ip,
        }
    }
}

impl FromStr for ResolveAddr {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(socket_addr) = s.parse::<SocketAddr>() {
            Ok(ResolveAddr::Socket(socket_addr))
        } else if let Ok(ip_addr) = s.parse::<IpAddr>() {
            Ok(ResolveAddr::Ip(ip_addr))
        } else {
            Err(format!("Invalid address format: {}", s))
        }
    }
}

impl Serialize for ResolveAddr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ResolveAddr::Socket(addr) => serializer.serialize_str(&addr.to_string()),
            ResolveAddr::Ip(ip) => serializer.serialize_str(&ip.to_string()),
        }
    }
}

impl<'de> Deserialize<'de> for ResolveAddr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        ResolveAddr::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
impl PartialEq for ResolveAddr {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ResolveAddr::Socket(a), ResolveAddr::Socket(b)) => a == b,
            (ResolveAddr::Ip(a), ResolveAddr::Ip(b)) => a == b,
            _ => false,
        }
    }
}

#[cfg(test)]
impl Eq for ResolveAddr {}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(test, derive(Eq, PartialEq))]
#[serde(rename_all = "kebab-case")]
pub struct Overrides {
    pub certificate: Option<String>,
    pub resolve: Option<ResolveAddr>,
}

#[cfg(test)]
mod test {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};
    
    #[test]
    fn config_load() {
        let loaded_config = init_or_load("../config.toml").unwrap();
        let template_config = Config {
            server: ServerConfig {
                bind_address: "0.0.0.0".to_string(),
                bind_port: Some(12864),
                host_name: "shuttlepub.localhost".to_string(),
                keypair: KeypairConfig {
                    private: "./.keys/private.pem".to_string(),
                    public: "./.keys/public.pem".to_string(),
                },
                overrides: vec![
                    ("misskey.localhost".to_string(), Overrides { 
                        certificate: Some("./.certs/misskey.crt".to_string()),
                        resolve: "127.0.0.1:4430".parse().ok(),
                    }),
                    ("mastodon.localhost".to_string(), Overrides { 
                        certificate: None,
                        resolve: None,
                    }),
                ].into_iter().collect(),
            },
        };
        assert_eq!(loaded_config, template_config)
    }
    
    #[test]
    fn parse_resolve_addr() {
        // Test SocketAddr parsing
        let socket_v4: ResolveAddr = "127.0.0.1:8080".parse().unwrap();
        assert!(matches!(socket_v4, ResolveAddr::Socket(_)));
        assert_eq!(socket_v4.to_socket_addr(9090), "127.0.0.1:8080".parse().unwrap());
        
        let socket_v6: ResolveAddr = "[::1]:8080".parse().unwrap();
        assert!(matches!(socket_v6, ResolveAddr::Socket(_)));
        
        // Test IpAddr parsing
        let ip_v4: ResolveAddr = "192.168.1.1".parse().unwrap();
        assert!(matches!(ip_v4, ResolveAddr::Ip(_)));
        assert_eq!(ip_v4.to_socket_addr(443), SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 443));
        
        let ip_v6: ResolveAddr = "::1".parse().unwrap();
        assert!(matches!(ip_v6, ResolveAddr::Ip(_)));
        assert_eq!(ip_v6.ip_addr(), IpAddr::V6(Ipv6Addr::LOCALHOST));
        
        // Test invalid format
        let invalid = "invalid".parse::<ResolveAddr>();
        assert!(invalid.is_err());
    }
}
