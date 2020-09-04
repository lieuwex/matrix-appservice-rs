use ruma_identifiers::ServerName;

use url::Url;

use serde::{Deserialize, Serialize};

use crate::util::random_alphanumeric;

/// A namespace defined by an application service.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Namespace {
    pub exclusive: bool,
    pub regex: String, // REVIEW
}

/// Namespaces defined by an application service.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Namespaces {
    pub users: Vec<Namespace>,
    pub aliases: Vec<Namespace>,
}

/// Information required in the registration yaml file that a homeserver needs.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Registration {
    pub id: String,
    pub as_token: String,
    pub hs_token: String,
    pub namespaces: Namespaces,
    pub url: String, // REVIEW
    pub sender_localpart: String,
    pub rate_limited: bool,
}

impl Registration {
    /// Create a new Registration with the given options and random tokens.
    pub fn new(
        id: String,
        namespaces: Namespaces,
        sender_localpart: String,
        url: Url,
        rate_limited: bool,
    ) -> Self {
        Registration {
            id,
            as_token: random_alphanumeric(64),
            hs_token: random_alphanumeric(64),
            namespaces,
            url: url.to_string(),
            sender_localpart,
            rate_limited,
        }
    }

    /// Create a registration from the given yaml value.
    pub fn from_yaml(value: serde_yaml::Value) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_value(value)
    }
}

/// A struct containing information required by an application service.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApplicationService {
    server_name: Box<ServerName>,
    server_url: String,
}

impl ApplicationService {
    /// Create a new ApplicationService struct with the given information.
    pub fn new(server_name: Box<ServerName>, server_url: Url) -> Self {
        Self {
            server_name,
            server_url: server_url.to_string(),
        }
    }

    /// Create a new ApplicationService struct from the given yaml value.
    pub fn from_yaml(value: serde_yaml::Value) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_value(value)
    }

    /// Get a reference to the server name in this ApplicationService instance.
    pub fn server_name(&self) -> &ServerName {
        self.server_name.as_ref()
    }
    /// Get a reference to the server url in this ApplicationService instance.
    pub fn server_url(&self) -> Url {
        Url::parse(&self.server_url).unwrap()
    }
}
