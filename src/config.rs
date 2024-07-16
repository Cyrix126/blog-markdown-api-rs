use std::{
    net::{Ipv4Addr, SocketAddr},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};
#[cfg(feature = "update_cache")]
use url::Url;

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    #[cfg(feature = "update_cache")]
    pub cache_uri: Url,
    #[cfg(feature = "update_cache")]
    pub cache_password: Option<PathBuf>,
    #[cfg(feature = "update_cache")]
    pub hostname: String,
    pub listen: SocketAddr,
    pub assets_path: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            #[cfg(feature = "update_cache")]
            cache_uri: Url::parse("http://127.0.0.1:8001").unwrap(),
            #[cfg(feature = "update_cache")]
            cache_password: None,
            #[cfg(feature = "update_cache")]
            hostname: "blog.example.net".to_string(),
            listen: SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8500),
            assets_path: PathBuf::from("/media/ssd/www/blog-markdown-api/assets"),
        }
    }
}
