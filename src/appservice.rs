use ruma::api::exports::http::Uri;
use ruma::identifiers::ServerName;

use serde::{Deserialize, Serialize};

pub use ruma::api::appservice::{Namespace, Namespaces, Registration, RegistrationInit};

#[cfg(feature = "rand")]
pub fn new_registration_rand(
    id: String,
    namespaces: Namespaces,
    sender_localpart: String,
    url: Uri,
    rate_limited: bool,
) -> Registration {
    use crate::util::random_alphanumeric;
    Registration::from(RegistrationInit {
        id,
        as_token: random_alphanumeric(64),
        hs_token: random_alphanumeric(64),
        namespaces,
        url: url.to_string(),
        sender_localpart,
        rate_limited: Some(rate_limited),
        protocols: None,
    })
}

/// A struct containing information required by an application service.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApplicationService {
    server_name: Box<ServerName>,
    server_url: String,
}

impl ApplicationService {
    /// Create a new ApplicationService struct with the given information.
    pub fn new(server_name: Box<ServerName>, server_url: Uri) -> Self {
        Self {
            server_name,
            server_url: server_url.to_string(),
        }
    }

    /// Get a reference to the server name in this ApplicationService instance.
    pub fn server_name(&self) -> &ServerName {
        self.server_name.as_ref()
    }
    /// Get a reference to the server url in this ApplicationService instance.
    pub fn server_url(&self) -> Uri {
        self.server_url.parse().unwrap()
    }
}
