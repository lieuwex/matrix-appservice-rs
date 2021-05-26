use std::convert::Infallible;
use std::future::Future;
use std::net::ToSocketAddrs;

use hyper::body::to_bytes;
use ruma::api::appservice::event::push_events;
use ruma::api::IncomingRequest;
use ruma::events::AnyRoomEvent;
use ruma::serde::Raw;

use hyper::server::Server;
use hyper::service::{make_service_fn, service_fn};
use hyper::{header, StatusCode};
use hyper::{Body, Request, Response};

/// Listen on `addrs` for incoming events, and use the given `handler` to handle those events.
pub async fn serve<S, F, R>(addrs: S, handler: F) -> Result<(), hyper::Error>
where
    S: ToSocketAddrs,
    F: Fn(String, Vec<Raw<AnyRoomEvent>>) -> R + Sync + Send + Clone + 'static,
    R: Future<Output = Result<String, Infallible>> + Send,
{
    let service = make_service_fn(move |_| {
        let handler = handler.clone();
        async {
            let f = service_fn(move |req: Request<Body>| {
                let handler = handler.clone();
                async move {
                    let (parts, body) = req.into_parts();
                    let body = to_bytes(body).await.unwrap();
                    let req: Request<&[u8]> = Request::from_parts(parts, &body);
                    let req = push_events::v1::IncomingRequest::try_from_http_request(req).unwrap();

                    match handler(req.txn_id, req.events).await {
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
