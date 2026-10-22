mod db;
mod domain;
mod handlers;

use axum::{
    routing::{get, post},
    Router,
};
use axum_prometheus::PrometheusMetricLayer;
use std::net::SocketAddr;
use std::time::Duration;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "api_rust=debug,tower_http=debug,axum=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@postgres.storage.svc.cluster.local:5432/ledger_db".to_string()
    });

    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT must be a valid number");

    info!("Starting Ledger API (Rust/Axum)...");
    let pool = db::create_pool(&database_url).await?;

    // `pair()` registers a global metrics recorder and returns the middleware
    // that observes every request plus the handle used to render the snapshot.
    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();

    let app = Router::new()
        .route("/health", get(handlers::health))
        .route("/metrics", get(|| async move { metric_handle.render() }))
        .route("/accounts", post(handlers::create_account).get(handlers::list_accounts))
        .route("/transactions", post(handlers::create_transaction))
        .route("/accounts/:id/balance", get(handlers::get_account_balance))
        .layer(TraceLayer::new_for_http())
        // Outermost layer: measures the whole request, database round-trip
        // included.
        .layer(prometheus_layer)
        .with_state(pool);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Ledger API listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Server gracefully stopped.");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => info!("Received Ctrl+C, initiating graceful shutdown..."),
        _ = terminate => info!("Received SIGTERM, initiating graceful shutdown..."),
    }

    // Allow in-flight requests 5 seconds to complete
    tokio::time::sleep(Duration::from_secs(5)).await;
}
