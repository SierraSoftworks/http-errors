use std::collections::HashMap;
use std::convert::Infallible;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::service::Service;
use hyper::{Request, Response, StatusCode};
use std::future::{ready, Ready};

use crate::errors::{ErrorType, ERRORS};

#[derive(Clone)]
pub struct ErrorService {
    default: Response<Full<Bytes>>,
    cache: HashMap<String, Response<Full<Bytes>>>,
}

impl ErrorService {
    pub fn new(default_status: StatusCode) -> Self {
        let mut cache = HashMap::new();

        let mut default = ErrorType::new(
            StatusCode::NOT_IMPLEMENTED,
            "Not Implemented",
            "The requested service is not available at this endpoint.",
        )
        .into();

        for error in ERRORS.iter() {
            let response: Response<Full<Bytes>> = error.clone().into();

            if error.code() == default_status {
                default = response.clone();
            }

            cache.insert(error.path(), response);
        }

        Self { cache, default }
    }
}

impl<B> Service<Request<B>> for ErrorService {
    type Response = Response<Full<Bytes>>;
    type Error = Infallible;
    type Future = Ready<Result<Response<Full<Bytes>>, Infallible>>;

    fn call(&self, req: Request<B>) -> Self::Future {
        match req.uri().path() {
            "/.well-known/health" => {
                return ready(Ok(Response::builder()
                    .status(StatusCode::OK)
                    .body(Full::new(Bytes::new()))
                    .unwrap()))
            }
            "/favicon.ico" => {
                return ready(Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Full::new(Bytes::new()))
                    .unwrap()))
            }
            _ => {}
        }

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
    async fn test_handle_health_check() {
        let service = ErrorService::new(StatusCode::NOT_FOUND);
        let req = Request::builder()
            .uri("/.well-known/health")
            .body(Full::new(Bytes::new()))
            .unwrap();
        let response = service.call(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_handle_favicon() {
        let service = ErrorService::new(StatusCode::NOT_FOUND);
        let req = Request::builder()
            .uri("/favicon.ico")
            .body(Full::new(Bytes::new()))
            .unwrap();
        let response = service.call(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_handle_default_error() {
        let service = ErrorService::new(StatusCode::NOT_FOUND);
        let req = Request::builder()
            .uri("/unknown")
            .body(Full::new(Bytes::new()))
            .unwrap();
        let response = service.call(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_handle_known_errors() {
        let service = ErrorService::new(StatusCode::NOT_FOUND);

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
