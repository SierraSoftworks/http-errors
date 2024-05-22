use std::net::SocketAddr;

use hyper::{server::conn::http1, StatusCode};
use hyper_util::rt::{TokioIo, TokioTimer};
use log::{debug, error, info};
use tokio::net::TcpListener;

use crate::server::ErrorService;

mod assets;
mod errors;
mod server;
mod template;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    pretty_env_logger::init();

    let default_status_code: StatusCode = std::env::var("DEFAULT_STATUS_CODE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(StatusCode::NOT_IMPLEMENTED);

    let service = ErrorService::new(default_status_code);

    let port = std::env::var("PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3000);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let listener = TcpListener::bind(addr).await?;
    info!("Listening on http://{addr}");

    loop {
        let (stream, addr) = listener.accept().await?;

        let io = TokioIo::new(stream);
        let service = service.clone();

        tokio::task::spawn(async move {
            //server::handle_connection(default_status_code, io, addr).await;
            debug!("Handling new incoming connection from {addr}");
            let conn = http1::Builder::new()
                .keep_alive(true)
                .timer(TokioTimer::new())
                .serve_connection(io, service);

            if let Err(err) = conn.await {
                error!("Error serving connection from {addr}: {err:?}");
            }
        });
    }
}
