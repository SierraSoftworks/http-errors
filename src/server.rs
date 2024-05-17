use std::convert::Infallible;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::rt::{Read, Write};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioTimer;
use log::{debug, error};
use std::net::SocketAddr;

use crate::assets::{self, get_static};
use crate::template;

pub async fn handle_connection<I>(io: I, addr: SocketAddr)
where
    I: Read + Write + Unpin,
{
    debug!("Handling new incoming connection from {addr}");
    let conn = http1::Builder::new()
        .keep_alive(true)
        .timer(TokioTimer::new())
        .serve_connection(io, service_fn(handle));

    if let Err(err) = conn.await {
        error!("Error serving connection from {addr}: {err:?}");
    }
}

async fn handle<B: hyper::body::Body>(req: Request<B>) -> Result<Response<Full<Bytes>>, Infallible> {
    match req.uri().path() {
        "/401" => Ok(build_response(&TemplateVars {
            code: StatusCode::UNAUTHORIZED,
            title: "Unauthorized",
            message: "The request requires user authentication.",
        })),
        "/403" => Ok(build_response(&TemplateVars {
            code: StatusCode::FORBIDDEN,
            title: "Forbidden",
            message: "The request was denied due to insufficient permissions.",
        })),
        "/404" => Ok(build_response(&TemplateVars {
            code: StatusCode::NOT_FOUND,
            title: "Not Found",
            message: "The requested service could not be found.",
        })),
        "/405" => Ok(build_response(&TemplateVars {
            code: StatusCode::METHOD_NOT_ALLOWED,
            title: "Method Not Allowed",
            message: "The requested service does not support the specified HTTP method.",
        })),
        "/409" => Ok(build_response(&TemplateVars {
            code: StatusCode::CONFLICT,
            title: "Conflict",
            message: "The request could not be completed due to a conflict with the current state of the resource.",
        })),
        "/500" => Ok(build_response(&TemplateVars {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            title: "Internal Server Error",
            message: "The server encountered an unexpected condition that prevented it from fulfilling the request.",
        })),
        "/501" => Ok(build_response(&TemplateVars {
            code: StatusCode::NOT_IMPLEMENTED,
            title: "Not Implemented",
            message: "The requested service is not available at this endpoint.",
        })),
        "/502" => Ok(build_response(&TemplateVars {
            code: StatusCode::BAD_GATEWAY,
            title: "Bad Gateway",
            message: "The server received an invalid response from an upstream server.",
        })),
        "/503" => Ok(build_response(&TemplateVars {
            code: StatusCode::SERVICE_UNAVAILABLE,
            title: "Service Unavailable",
            message: "The server is currently unable to handle the request due to a temporary overloading or maintenance of the server.",
        })),
        "/504" => Ok(build_response(&TemplateVars {
            code: StatusCode::GATEWAY_TIMEOUT,
            title: "Gateway Timeout",
            message: "The server did not receive a timely response from an upstream server.",
        })),
        "/505" => Ok(build_response(&TemplateVars {
            code: StatusCode::HTTP_VERSION_NOT_SUPPORTED,
            title: "HTTP Version Not Supported",
            message: "The server does not support the HTTP protocol version used in the request.",
        })),
        _ => Ok(build_response(&TemplateVars {
            code: StatusCode::NOT_IMPLEMENTED,
            title: "Not Implemented",
            message: "The requested service is not available at this endpoint.",
        })),
    }
}

struct TemplateVars {
    code: StatusCode,
    title: &'static str,
    message: &'static str,
}

fn build_response(vars: &TemplateVars) -> Response<Full<Bytes>> {
    let styles = assets::get_static("stylesheet.css").unwrap_or_default();

    match get_static("template.html.tpl") {
        Some(template) => {
            let mut vars_map = std::collections::HashMap::new();
            vars_map.insert("code", format!("{}", vars.code.as_u16()));
            vars_map.insert("title", vars.title.into());
            vars_map.insert("message", vars.message.into());
            vars_map.insert("stylesheet", styles);

            let body = template::template_replace(&template, vars_map);

            Response::builder()
                .status(vars.code)
                .header("Content-Type", "text/html")
                .body(Full::new(Bytes::from(body)))
                .unwrap()
        }
        None => Response::builder()
            .status(vars.code)
            .header("Content-Type", "text/plain")
            .body(Full::new(Bytes::from(format!(
                "{} {}\n{}",
                vars.code, vars.title, vars.message
            ))))
            .unwrap(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_response() {
        let vars = TemplateVars {
            code: StatusCode::NOT_FOUND,
            title: "Not Found",
            message: "The requested service could not be found.",
        };

        let response = build_response(&vars);
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(response.headers().get("Content-Type").unwrap(), "text/html");
    }

    #[tokio::test]
    async fn test_handle() {
        let cases = &[
            StatusCode::UNAUTHORIZED,
            StatusCode::FORBIDDEN,
            StatusCode::NOT_FOUND,
            StatusCode::METHOD_NOT_ALLOWED,
            StatusCode::CONFLICT,
            StatusCode::INTERNAL_SERVER_ERROR,
            StatusCode::NOT_IMPLEMENTED,
            StatusCode::BAD_GATEWAY,
            StatusCode::SERVICE_UNAVAILABLE,
            StatusCode::GATEWAY_TIMEOUT,
            StatusCode::HTTP_VERSION_NOT_SUPPORTED,
        ];

        for code in cases {
            let req = Request::builder().uri(format!("/{}", code.as_u16())).body(Full::new(Bytes::new())).unwrap();
            let response = handle(req).await.unwrap();
            assert_eq!(response.status(), *code);
            assert_eq!(response.headers().get("Content-Type").unwrap(), "text/html");
        }
    }
}
