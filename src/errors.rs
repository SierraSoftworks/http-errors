use http_body_util::Full;
use hyper::{body::Bytes, Response, StatusCode};

use crate::{
    assets::{self, get_static},
    template,
};

lazy_static::lazy_static! {
    pub static ref ERRORS: Vec<ErrorType> = vec![
        ErrorType {
            code: StatusCode::BAD_REQUEST,
            title: "Bad Request",
            message: "The request could not be completed due to invalid input. Check the request parameters and try again.",
        },
        ErrorType {
            code: StatusCode::UNAUTHORIZED,
            title: "Unauthorized",
            message: "You have not provided a valid Authorization header with your request. Check your access token and try again.",
        },
        ErrorType {
            code: StatusCode::FORBIDDEN,
            title: "Forbidden",
            message: "You do not have permission to access the requested service. Check your account permissions and try again.",
        },
        ErrorType {
            code: StatusCode::NOT_FOUND,
            title: "Not Found",
            message: "The requested service could not be found. Check the URL and try again.",
        },
        ErrorType {
            code: StatusCode::METHOD_NOT_ALLOWED,
            title: "Method Not Allowed",
            message: "The requested service does not support the provided HTTP method. Check the documentation and try again.",
        },
        ErrorType {
            code: StatusCode::CONFLICT,
            title: "Conflict",
            message: "The request could not be completed due to a conflict with the current state of the resource. Check the resource state and try again.",
        },
        ErrorType {
            code: StatusCode::TOO_MANY_REQUESTS,
            title: "Too Many Requests",
            message: "You have exceeded the rate limit for this service. Check the rate limit and try again later.",
        },

        ErrorType {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            title: "Internal Server Error",
            message: "An unexpected error occurred while processing your request. Please try again later.",
        },
        ErrorType {
            code: StatusCode::NOT_IMPLEMENTED,
            title: "Not Implemented",
            message: "The requested service is not implemented. Check the documentation and try again later.",
        },
        ErrorType {
            code: StatusCode::BAD_GATEWAY,
            title: "Bad Gateway",
            message: "The server received an invalid response from an upstream server. Please try again later.",
        },
        ErrorType {
            code: StatusCode::SERVICE_UNAVAILABLE,
            title: "Service Unavailable",
            message: "The requested service is temporarily unavailable. Please try again later.",
        },
        ErrorType {
            code: StatusCode::GATEWAY_TIMEOUT,
            title: "Gateway Timeout",
            message: "The server did not receive a timely response from an upstream server. Please try again later.",
        },
        ErrorType {
            code: StatusCode::HTTP_VERSION_NOT_SUPPORTED,
            title: "HTTP Version Not Supported",
            message: "The server does not support the HTTP protocol version used in the request. Please try again with a different version.",
        },
    ];
}

#[derive(Clone)]
pub struct ErrorType {
    code: StatusCode,
    title: &'static str,
    message: &'static str,
}

impl ErrorType {
    pub fn new(code: StatusCode, title: &'static str, message: &'static str) -> Self {
        Self {
            code,
            title,
            message,
        }
    }

    pub fn code(&self) -> StatusCode {
        self.code
    }

    pub fn path(&self) -> String {
        format!("/.well-known/http-{}", self.code.as_u16())
    }
}

impl Into<Response<Full<hyper::body::Bytes>>> for ErrorType {
    fn into(self) -> Response<Full<Bytes>> {
        let styles = assets::get_static("stylesheet.css").unwrap_or_default();

        match get_static("template.html.tpl") {
            Some(template) => {
                let mut vars_map = std::collections::HashMap::new();
                vars_map.insert("code", format!("{}", self.code.as_u16()));
                vars_map.insert("title", self.title.into());
                vars_map.insert("message", self.message.into());
                vars_map.insert("stylesheet", styles);

                let body = template::template_replace(&template, vars_map);

                Response::builder()
                    .status(self.code)
                    .header("Content-Type", "text/html")
                    .body(Full::new(Bytes::from(body)))
                    .unwrap()
            }
            None => Response::builder()
                .status(self.code)
                .header("Content-Type", "text/plain")
                .body(Full::new(Bytes::from(format!(
                    "{} {}\n{}",
                    self.code, self.title, self.message
                ))))
                .unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_response() {
        let vars = ErrorType {
            code: StatusCode::NOT_FOUND,
            title: "Not Found",
            message: "The requested service could not be found.",
        };

        let response: Response<Full<Bytes>> = vars.into();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(response.headers().get("Content-Type").unwrap(), "text/html");
    }
}
