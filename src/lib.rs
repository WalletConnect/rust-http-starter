pub mod config;
pub mod error;
mod handlers;
pub mod log;
mod metrics;
mod state;
mod stores;

use {
    crate::{config::Configuration, state::AppState},
    axum::{routing::get, Router},
    opentelemetry::{
        sdk::Resource,
        // util::tokio_interval_stream,
        KeyValue,
    },
    sqlx::{
        postgres::{PgConnectOptions, PgPoolOptions},
        ConnectOptions,
    },
    std::{net::SocketAddr, str::FromStr, sync::Arc, time::Duration},
    tokio::{select, sync::broadcast},
    tower::ServiceBuilder,
    tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
    tracing::info,
    tracing::log::LevelFilter,
    tracing::Level,
};

build_info::build_info!(fn build_info);

pub type Result<T> = std::result::Result<T, error::Error>;

pub async fn bootstap(mut shutdown: broadcast::Receiver<()>, config: Configuration) -> Result<()> {
    let pg_options = PgConnectOptions::from_str(&config.database_url)?
        .log_statements(LevelFilter::Debug)
        .log_slow_statements(LevelFilter::Info, Duration::from_millis(250))
        .clone();

    let store = PgPoolOptions::new()
        .max_connections(5)
        .connect_with(pg_options)
        .await?;

    sqlx::migrate!("./migrations").run(&store).await?;

    let mut state = AppState::new(config.clone(), Arc::new(store.clone()))?;

    if state.config.telemetry_prometheus_port.is_some() {
        state.set_metrics(metrics::Metrics::new(Resource::new(vec![
            KeyValue::new("service_name", "rust-http-starter"),
            KeyValue::new(
                "service_version",
                state.build_info.crate_info.version.clone().to_string(),
            ),
        ]))?);
    }

    let port = state.config.port;

    let state_arc = Arc::new(state);

    let global_middleware = ServiceBuilder::new().layer(
        TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::new().include_headers(true))
            .on_request(DefaultOnRequest::new().level(Level::INFO))
            .on_response(
                DefaultOnResponse::new()
                    .level(Level::INFO)
                    .include_headers(true),
            ),
    );

    let app = Router::new()
        .route("/health", get(handlers::health::handler))
        .layer(global_middleware)
        .with_state(state_arc.clone());

    let private_app = Router::new()
        .route("/metrics", get(handlers::metrics::handler))
        .with_state(state_arc);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let private_addr = SocketAddr::from((
        [0, 0, 0, 0],
        config.telemetry_prometheus_port.unwrap_or(3001),
    ));

    select! {
        _ = axum::Server::bind(&addr).serve(app.into_make_service()) => info!("Server terminating"),
        _ = axum::Server::bind(&private_addr).serve(private_app.into_make_service()) => info!("Internal Server terminating"),
        _ = shutdown.recv() => info!("Shutdown signal received, killing servers"),
    }

    Ok(())
}
