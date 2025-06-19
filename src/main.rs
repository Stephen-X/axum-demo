use std::sync::Arc;
use axum::Router;
use axum_demo::configuration::{get_configuration, Environment, Settings};
use axum_demo::dependency::ApplicationState;
use axum_demo::middleware::Middleware;
use axum_demo::route::ApplicationRoute;
use tokio::net::TcpListener;
use tracing::{debug, Level};
use tracing_subscriber::fmt;

// Axum reference code: https://github.com/tokio-rs/axum/tree/main/examples
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Arc::new(get_configuration().expect("Failed to read configuration."));
    init_tracing(config.clone());

    // Using the State extractor: https://docs.rs/axum/latest/axum/#using-the-state-extractor
    let global_state = ApplicationState::default();
    let address = format!("{}:{}", config.application.host, config.application.port);

    // Build application with routes
    let router = Router::new()
        .add_middleware(config.clone())
        .add_routes(config.clone())
        // Ref: https://docs.rs/axum/latest/axum/struct.Router.html#returning-routers-with-states-from-functions
        .with_state(global_state);

    // Run server
    let listener = TcpListener::bind(address).await?;
    debug!("Listening on {}...", listener.local_addr()?);
    axum::serve(listener, router).await?;
    Ok(())
}

/// Initializes the tracing subscriber for logging.
fn init_tracing(config: Arc<Settings>) {
    if config.environment == Environment::Local.as_str() {
        let format = fmt::format()
            .with_level(true)
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .compact();

        tracing_subscriber::fmt()
            .event_format(format)
            .with_max_level(Level::TRACE)
            .init()
    } else {
        let format = fmt::format()
            .with_level(true)
            .with_target(true)
            .compact();

        tracing_subscriber::fmt()
            .event_format(format)
            .with_max_level(Level::INFO)
            .init()
    }
}
