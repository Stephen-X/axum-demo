use std::sync::Arc;
use crate::api::handler::get_api_routes;
use crate::configuration::Settings;
use crate::dependency::ApplicationState;
use axum::extract::State;
use axum::routing::get;
use axum::Router;

/// Extension trait for adding routes to the server router.
pub trait ApplicationRoute {
    /// Adds application-specific routes to the server router.
    /// # Arguments
    /// * `config`: The global settings.
    fn add_routes(self, config: Arc<Settings>) -> Self;
}

impl ApplicationRoute for Router<ApplicationState> {
    fn add_routes(self, _config: Arc<Settings>) -> Self {
        self.route("/", get(|_: State<ApplicationState>| async { "Root dir" }))
            .nest("/api", get_api_routes())
    }
}
