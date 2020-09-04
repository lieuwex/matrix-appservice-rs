use std::collections::BTreeMap;

use ruma::identifiers::UserId;
use ruma_client::{Error, HttpsClient};

/// A builder for a request to the Matrix homeserver.
#[derive(Debug, Clone)]
pub struct RequestBuilder<'a, R>
where
    R: ruma::api::OutgoingRequest + std::fmt::Debug,
{
    client: &'a HttpsClient,
    request: R,

    params: BTreeMap<String, String>,
}

impl<'a, R> RequestBuilder<'a, R>
where
    R: ruma::api::OutgoingRequest + std::fmt::Debug,
{
    /// Create a new `RequestBuilder`, with the given `HttpsClient` and the given `request`.
    pub fn new(client: &'a HttpsClient, request: R) -> Self {
        Self {
            client,
            request,

            params: BTreeMap::new(),
        }
    }

    /// Set the `user_id` url parameter, returning the current builder to allow method chaining.
    pub fn user_id(&mut self, user_id: &UserId) -> &mut Self {
        self.params
            .insert(String::from("user_id"), user_id.to_string());
        self
    }

    /// Set the `ts` url parameter, returning the current builder to allow method chaining.
    pub fn timestamp(&mut self, timestamp: i64) -> &mut Self {
        self.params
            .insert(String::from("ts"), timestamp.to_string());
        self
    }

    /// Submit the request, waiting on the response.
    /// This will consume the current builder.
    pub async fn request(self) -> Result<R::IncomingResponse, Error<R::EndpointError>> {
        self.client
            .request_with_url_params(self.request, Some(self.params))
            .await
    }
}
