use std::convert::Infallible;
use std::future::Future;
use std::net::ToSocketAddrs;

use ruma::events::AnyEvent;
use ruma::Raw;

use bytes::buf::ext::BufExt;

use hyper::service::{make_service_fn, service_fn};
use hyper::{body::aggregate, Body, Request, Response, Server};
use hyper::{header, StatusCode};

use serde_json::value::to_raw_value;

/// Listen on `addrs` for incoming events, and use the given `handler` to handle those events.
pub async fn serve<S, F, R>(addrs: S, handler: F) -> Result<(), hyper::Error>
where
    S: ToSocketAddrs,
    F: Fn(String, Vec<Raw<AnyEvent>>) -> R + Sync + Send + Clone + 'static,
    R: Future<Output = Result<String, Infallible>> + Send,
{
    let service = make_service_fn(move |_| {
        let handler = handler.clone();
        async {
            let f = service_fn(move |req: Request<Body>| {
                let handler = handler.clone();
                async move {
                    let (parts, body) = req.into_parts();

                    // skip "/transactions/"
                    let txn_id = parts.uri.path()[14..].to_string();

                    let body = aggregate(body).await.unwrap();
                    let json: serde_json::Value = serde_json::from_reader(body.reader()).unwrap();

                    let events: Vec<Raw<AnyEvent>> = json
                        .as_object()
                        .unwrap()
                        .get("events")
                        .unwrap()
                        .as_array()
                        .unwrap()
                        .into_iter()
                        .map(|e| {
                            let raw_value = to_raw_value(e).unwrap();
                            Raw::from_json(raw_value)
                        })
                        .collect();

                    match handler(txn_id, events).await {
                        Err(_) => {} // TODO
                        Ok(_) => {}
                    }

                    let response = Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, "application/json")
                        .body(Body::from("{}"))
                        .unwrap();
                    Ok::<_, Infallible>(response)
                }
            });

            Ok::<_, Infallible>(f)
        }
    });

    let addr = addrs.to_socket_addrs().unwrap().next().unwrap();
    let server = Server::bind(&addr).serve(service);

    server.await
}
