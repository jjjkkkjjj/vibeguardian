pub mod handler;

use anyhow::Result;
use tokio::sync::oneshot;

use crate::config::project::ProxyRoute;

/// Start the axum proxy server in a background task.
/// Returns a sender that, when dropped or fired, shuts the server down.
pub async fn start(
    port: u16,
    routes: Vec<ProxyRoute>,
    store: serde_json::Value,
) -> Result<oneshot::Sender<()>> {
    let (tx, rx) = oneshot::channel::<()>();
    let app = handler::build_router(routes, store)?;
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("[Vibeguard] Proxy started at http://localhost:{}", port);
    tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async { rx.await.ok(); })
            .await
            .ok();
    });
    Ok(tx)
}
