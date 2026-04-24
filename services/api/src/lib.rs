use std::sync::Arc;
use std::time::Duration;

use axum::{routing::get, Router};
use tower::ServiceBuilder;
use tower::limit::ConcurrencyLimitLayer;
use tower_http::{timeout::TimeoutLayer, trace::TraceLayer};

pub mod dto;
pub mod error;
pub mod routes;
pub mod store;

pub use error::ApiError;
pub use store::{InMemoryNotesStore, NotesStore};

use routes::{create_note, list_notes, AppState};

const MAX_CONCURRENT: usize = 100;
const REQUEST_TIMEOUT_SECS: u64 = 5;

pub fn create_router(store: Arc<dyn NotesStore>) -> Router {
    Router::new()
        .route("/notes", get(list_notes).post(create_note))
        .with_state::<()>(store as AppState)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(ConcurrencyLimitLayer::new(MAX_CONCURRENT))
                .layer(TimeoutLayer::new(Duration::from_secs(REQUEST_TIMEOUT_SECS)))
        )
}
