use std::convert::Infallible;
use std::convert::TryInto;
use std::future::Future;
use std::net::ToSocketAddrs;

use ruma::api::appservice::event::push_events;
use ruma::events::AnyEvent;
use ruma::Raw;

use bytes::Buf;

use hyper::service::{make_service_fn, service_fn};
use hyper::{body::aggregate, Body, Request, Response, Server};
use hyper::{header, StatusCode};

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

                    let json: push_events::v1::IncomingRequest = {
                        let body = aggregate(body).await.unwrap();
                        let body = body.bytes().to_vec();
                        let new_req = Request::builder().method("POST").body(body).unwrap();
                        new_req.try_into().unwrap()
                    };

                    match handler(txn_id, json.events).await {
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
