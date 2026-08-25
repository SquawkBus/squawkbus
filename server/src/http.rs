use std::future::ready;
use std::io;
use std::{net::SocketAddr, sync::Arc};

use axum::http::StatusCode;
use axum::{Router, routing::get};
use axum_server::tls_rustls::RustlsConfig;

use log::info;
use metrics_exporter_prometheus::PrometheusHandle;

use tokio_rustls::rustls;

pub async fn start_http_server(
    addr: SocketAddr,
    rustls_config: Option<Arc<rustls::ServerConfig>>,
    prometheus_handle: PrometheusHandle,
) -> io::Result<()> {
    info!("Serving http on address {addr}");

    let app = Router::new()
        .route("/health/readiness", get(readiness_check))
        .route("/health/liveness", get(liveness_check))
        .route("/health/startup", get(startup_check))
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

async fn readiness_check() -> (StatusCode, &'static str) {
    (StatusCode::OK, "OK")
}

async fn liveness_check() -> (StatusCode, &'static str) {
    (StatusCode::OK, "OK")
}

async fn startup_check() -> (StatusCode, &'static str) {
    (StatusCode::OK, "OK")
}
