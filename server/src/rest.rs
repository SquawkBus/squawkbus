use std::future::ready;
use std::io;
use std::{net::SocketAddr, sync::Arc};

use axum::{Router, routing::get};
use axum_server::tls_rustls::RustlsConfig;

use metrics_exporter_prometheus::PrometheusHandle;

use tokio_rustls::rustls;

pub async fn start_rest_server(
    addr: SocketAddr,
    rustls_config: Option<Arc<rustls::ServerConfig>>,
    prometheus_handle: PrometheusHandle,
) -> io::Result<()> {
    // initialize tracing
    // tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/metrics", get(move || ready(prometheus_handle.render())));

    match rustls_config {
        Some(config) => {
            axum_server::bind_rustls(addr, RustlsConfig::from_config(config))
                .serve(app.into_make_service())
                .await
        }
        None => axum_server::bind(addr).serve(app.into_make_service()).await,
    }
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}

async fn health_check() -> &'static str {
    "OK"
}
