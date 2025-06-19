use std::sync::{Arc, RwLock};
use tracing::debug;
use crate::repo::db::{InMemoryDatabase, KVDatabase};

/// Application state that holds all the app dependency singletons.
#[derive(Clone)]
pub struct ApplicationState {
    // Note: Locking database client with `Arc<Mutex>` or `Arc<RwLock>` is not ideal for throughput,
    //   it means we can only issue one operation at a time.
    //   In practice, make sure that the database client is either one of the followings:
    //   - Bitwise copyable, i.e. it only clones pointers to the connection pool.
    //   - Allows you to get a pointer to the shared underlying resource with e.g. `get_ref()` or `get_mut()`.
    // Library documentation typically states this clearly.
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