use std::net::SocketAddr;

use hyper_util::rt::TokioIo;
use log::info;
use tokio::net::TcpListener;

mod assets;
mod errors;
mod server;
mod template;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    pretty_env_logger::init();

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
        tokio::task::spawn(async move {
            server::handle_connection(io, addr).await;
        });
    }
}
