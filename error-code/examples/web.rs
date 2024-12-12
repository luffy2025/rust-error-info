use anyhow::Result;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use error_code::ToErrorInfo;
use std::backtrace::Backtrace;
use thiserror::Error;
use tokio::net::TcpListener;
use tracing::{info, warn};

#[allow(unused)]
#[derive(Debug, Error, ToErrorInfo)]
#[error_info(app_type = "http::StatusCode", prefix = "0A")]
enum AppError {
    #[error("Invalid param: {0}")]
    #[error_info(code = "IP", app_code = "400")]
    InvalidParam(String),

    #[error("Item {0} Not found")]
    #[error_info(code = "NF", app_code = "404")]
    NotFound(String),

    #[error("Internal server error: {0}")]
    #[error_info(
        code = "SE",
        app_code = "500",
        client_msg = "we had a server problem, please try again later"
    )]
    ServerError(String),

    #[error("Unknown error")]
    #[error_info(code = "UK", app_code = "500")]
    Unknown,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let info = self.to_error_info();
        let status = info.app_code;

        if status.is_server_error() {
            warn!("{:?}", info);
        } else {
            info!("{:?}", info);
        }

        Response::builder()
            .status(status)
            .body(info.to_string().into())
            .unwrap()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let app = Router::new().route("/", get(index_handler));

    let addr = "0.0.0.0:8080";
    info!("Listening on {}", addr);

    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

async fn index_handler() -> Result<&'static str, AppError> {
    let b = Backtrace::force_capture();
    Err(AppError::ServerError(format!("\n{}", b)))
}
