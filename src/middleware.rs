use crate::configuration::{Environment, Settings};
use crate::dependency::ApplicationState;
use axum::body::Body;
use axum::error_handling::HandleErrorLayer;
use axum::http::{Request, StatusCode};
use axum::response::IntoResponse;
use axum::Router;
use std::borrow::Cow;
use std::sync::Arc;
use std::time::Duration;
use tower::{BoxError, ServiceBuilder};
use tower_http::trace::{DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tower_http::LatencyUnit;
use tracing::{Level, Span};
use uuid::Uuid;

/// Extension trait for adding middleware to the Axum router.
pub trait Middleware {
    /// Adds global middleware to the Axum router.
    fn add_middleware(self, config: Arc<Settings>) -> Self;
}

impl Middleware for Router<ApplicationState> {
    fn add_middleware(self, config: Arc<Settings>) -> Self {
        self.layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handle_tower_error))
                .load_shed()
                .concurrency_limit(config.application.max_concurrent_requests)
                .timeout(Duration::from_secs(config.application.request_timeout_s))
                // TODO: How do I add a trace layer for non-HTTP logs?
                // tower-http middleware for logging
                // Ref: https://docs.rs/tower-http/latest/tower_http/trace/index.html
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(move |request: &Request<Body>| build_trace_span(request, config.clone()))
                        .on_request(DefaultOnRequest::new().level(Level::INFO))
                        .on_response(
                            DefaultOnResponse::new()
                                .level(Level::INFO)
                                .latency_unit(LatencyUnit::Micros),
                        )
                        .on_failure(
                            DefaultOnFailure::new()
                                .level(Level::ERROR)
                                .latency_unit(LatencyUnit::Micros),
                        ),
                ),
        )
    }
}

fn build_trace_span(request: &Request<Body>, config: Arc<Settings>) -> Span {
    // Extract the trace ID from the request headers, or generate a new one.
    let trace_id = request
        .headers()
        .get("X-Trace-ID")
        .and_then(|value| match value.to_str().ok() {
            Some(val) => Some(val.to_string()),
            _ => None,
        })
        .unwrap_or(Uuid::new_v4().to_string());

    // Note: Doc for the `%` and `?` sigils: https://docs.rs/tracing/latest/tracing/#recording-fields
    if config.environment == Environment::Local.as_str() {
        tracing::span!(
            Level::TRACE,
            "request",
            trace_id = %trace_id,
            method = %request.method(),
            uri = %request.uri(),
            version = ?request.version(),
            headers = ?request.headers()
        )
    } else {
        tracing::span!(
            Level::INFO,
            "request",
            trace_id = %trace_id,
            method = %request.method(),
            uri = %request.uri(),
            version = ?request.version(),
            headers = ?request.headers()
        )
    }
}

/// Error code mapping for tower middlewares.
// Ref: https://docs.rs/axum/latest/axum/error_handling/index.html
async fn handle_tower_error(error: BoxError) -> impl IntoResponse {
    if error.is::<tower::timeout::error::Elapsed>() {
        return (StatusCode::REQUEST_TIMEOUT, Cow::from("Request timed out."));
    }

    if error.is::<tower::load_shed::error::Overloaded>() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Cow::from("Service is overloaded, try again later."),
        );
    }

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Cow::from("Internal server error."),
    )
}
