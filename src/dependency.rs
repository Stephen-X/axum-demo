use std::sync::{Arc, RwLock};
use tracing::debug;
use crate::repo::db::{InMemoryDatabase, KVDatabase};

/// Application state that holds all the app dependency singletons.
#[derive(Clone)]
pub struct ApplicationState {
    pub db: Arc<RwLock<dyn KVDatabase<String, String>>>,
}

impl Default for ApplicationState {
    fn default() -> Self {
        debug!("Creating new AppState...");
        Self {
            db: Arc::new(RwLock::new(InMemoryDatabase::new())),
        }
    }
}

impl ApplicationState {
    pub fn build() -> Arc<Self> {
        Arc::new(Self::default())
    }
}