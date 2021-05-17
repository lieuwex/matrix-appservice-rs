use std::collections::HashMap;

use ruma::identifiers::UserId;
use ruma_client::{Client, HttpClient, ResponseResult};

use hyper::Uri;

/// A builder for a request to the Matrix homeserver.
#[derive(Debug, Clone)]
pub struct RequestBuilder<'a, C, R>
where
    C: HttpClient,
    R: ruma::api::OutgoingRequest,
{
    client: &'a Client<C>,
    request: R,

    params: HashMap<String, String>,
}

impl<'a, C, R> RequestBuilder<'a, C, R>
where
    C: HttpClient,
    R: ruma::api::OutgoingRequest,
{
    /// Create a new `RequestBuilder`, with the given `Client` and the given `request`.
    pub fn new(client: &'a Client<C>, request: R) -> Self {
        Self {
            client,
            request,

            params: HashMap::new(),
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
    pub async fn request(self) -> ResponseResult<C, R> {
        let mut new_params = String::new();
        for (i, s) in self
            .params
            .into_iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .enumerate()
        {
            if i > 0 {
                new_params.push('&');
            }
            new_params.push_str(&s);
        }

        self.client
            .send_customized_request(self.request, |req| {
                let uri = req.uri_mut();
                let new_path_and_query = match uri.query() {
                    Some(params) => format!("{}?{}&{}", uri.path(), params, new_params),
                    None => format!("{}?{}", uri.path(), new_params),
                };

                let mut parts = uri.clone().into_parts();
                parts.path_and_query = Some(new_path_and_query.parse()?);
                *uri = Uri::from_parts(parts)?;

                Ok(())
            })
            .await
    }
}
