use std::collections::HashMap;
use std::convert::Infallible;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::rt::{Read, Write};
use hyper::server::conn::http1;
use hyper::service::Service;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioTimer;
use log::{debug, error};
use std::future::{ready, Ready};
use std::net::SocketAddr;

use crate::errors::{ErrorType, ERRORS};

pub async fn handle_connection<I>(io: I, addr: SocketAddr)
where
    I: Read + Write + Unpin,
{
    debug!("Handling new incoming connection from {addr}");
    let conn = http1::Builder::new()
        .keep_alive(true)
        .timer(TokioTimer::new())
        .serve_connection(io, ErrorService::new());

    if let Err(err) = conn.await {
        error!("Error serving connection from {addr}: {err:?}");
    }
}

struct ErrorService {
    default: Response<Full<Bytes>>,
    cache: HashMap<String, Response<Full<Bytes>>>,
}

impl ErrorService {
    fn new() -> Self {
        let mut cache = HashMap::new();

        for error in ERRORS.iter() {
            let response: Response<Full<Bytes>> = error.clone().into();
            cache.insert(error.path(), response);
        }

        let default = ErrorType::new(
            StatusCode::NOT_IMPLEMENTED,
            "Not Implemented",
            "The requested service is not available at this endpoint.",
        )
        .into();

        Self { cache, default }
    }
}

impl<B> Service<Request<B>> for ErrorService {
    type Response = Response<Full<Bytes>>;
    type Error = Infallible;
    type Future = Ready<Result<Response<Full<Bytes>>, Infallible>>;

    fn call(&self, req: Request<B>) -> Self::Future {
        if let Some(response) = self.cache.get(req.uri().path()) {
            return ready(Ok(response.clone()));
        } else if let Some(response) = self.cache.get("/504") {
            return ready(Ok(response.clone()));
        } else {
            return ready(Ok(self.default.clone()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_handle() {
        let service = ErrorService::new();

        for error in ERRORS.iter() {
            let req = Request::builder()
                .uri(error.path())
                .body(Full::new(Bytes::new()))
                .unwrap();
            let response = service.call(req).await.unwrap();
            assert_eq!(response.status(), error.code());
            assert_eq!(response.headers().get("Content-Type").unwrap(), "text/html");
        }
    }
}
