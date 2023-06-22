use {
    crate::handlers::ResponseError,
    axum::response::{IntoResponse, Response},
    hyper::StatusCode,
    tracing::error,
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Envy(#[from] envy::Error),

    #[error(transparent)]
    Trace(#[from] opentelemetry::trace::TraceError),

    #[error(transparent)]
    Metrics(#[from] opentelemetry::metrics::MetricsError),

    #[error(transparent)]
    Prometheus(#[from] prometheus_core::Error),

    #[error(transparent)]
    Database(#[from] sqlx::Error),

    #[error("database migration failed: {0}")]
    DatabaseMigration(#[from] sqlx::migrate::MigrateError),

    #[error(transparent)]
    Store(#[from] crate::stores::StoreError),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        error!("responding with error ({:?})", self);
        match self {
            Error::Database(e) => crate::handlers::Response::new_failure(StatusCode::INTERNAL_SERVER_ERROR, vec![
                ResponseError {
                    name: "sqlx".to_string(),
                    message: e.to_string(),
                }
            ], vec![]),
            e => crate::handlers::Response::new_failure(StatusCode::INTERNAL_SERVER_ERROR, vec![
                ResponseError {
                    name: "unknown_error".to_string(),
                    message: "This error should not have occurred. Please file an issue at: https://github.com/walletconnect/rust-http-starter".to_string(),
                },
                ResponseError {
                    name: "dbg".to_string(),
                    message: format!("{e:?}"),
                }
            ], vec![])
        }.into_response()
    }
}
