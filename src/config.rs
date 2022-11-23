use serde::{Deserialize, Serialize};

/// The configuration for the server.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ServerConfig {
    /// The port to listen on.
    pub http_listen_port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            http_listen_port: 9498,
        }
    }
}

/// Represents the configuration for the application.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    /// The configuration for the server.
    pub server: ServerConfig,

    /// A list of Mastodon instances to monitor.
    pub instance_info: Vec<String>,

    /// A list of Mastodon accounts to monitor.
    ///
    /// The first value is the name of the instance, the second is the account's id.
    pub accounts: Vec<(String, String)>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            instance_info: vec!["mas.to".to_string(), "mastodon.social".to_string()],
            accounts: Vec::new(),
        }
    }
}
