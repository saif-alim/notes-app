use std::sync::Arc;
use std::time::Duration;

use axum::{http::StatusCode, routing::get, Router};
use tower::ServiceBuilder;
use tower::limit::ConcurrencyLimitLayer;
use tower::util::HandleErrorLayer;
use tower_http::{limit::RequestBodyLimitLayer, timeout::TimeoutLayer, trace::TraceLayer};

pub mod dto;
pub mod error;
pub mod routes;
pub mod store;

pub use error::ApiError;
pub use store::{InMemoryNotesStore, NotesStore};

use routes::{create_note, list_notes, AppState};

pub const MAX_CONCURRENT: usize = 100;
pub const REQUEST_TIMEOUT_SECS: u64 = 5;
pub const MAX_BODY_BYTES: usize = 64 * 1024;

pub fn create_router(store: Arc<dyn NotesStore>) -> Router {
    Router::new()
        .route("/notes", get(list_notes).post(create_note))
        .with_state::<()>(store as AppState)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(RequestBodyLimitLayer::new(MAX_BODY_BYTES))
                .layer(HandleErrorLayer::new(|_: tower::BoxError| async {
                    StatusCode::SERVICE_UNAVAILABLE
                }))
                .layer(ConcurrencyLimitLayer::new(MAX_CONCURRENT))
                .layer(TimeoutLayer::new(Duration::from_secs(REQUEST_TIMEOUT_SECS)))
        )
}
