use std::sync::Arc;

use axum::{routing::get, Router};

pub mod dto;
pub mod error;
pub mod routes;
pub mod store;

pub use error::ApiError;
pub use store::{InMemoryNotesStore, NotesStore};

use routes::{create_note, list_notes, AppState};

pub fn create_router(store: Arc<dyn NotesStore>) -> Router {
    Router::new()
        .route("/notes", get(list_notes).post(create_note))
        .with_state::<()>(store as AppState)
}
