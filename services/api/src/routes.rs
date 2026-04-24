use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use crate::dto::{
    CreateNoteRequestDto, CreateNoteResponseDto, ListNotesResponseDto, NoteDto,
};
use crate::store::NotesStore;

pub type AppState = Arc<dyn NotesStore>;

pub async fn list_notes(State(store): State<AppState>) -> Json<ListNotesResponseDto> {
    let notes = store.list();
    let dtos: Vec<NoteDto> = notes.into_iter().map(NoteDto::from).collect();
    Json(ListNotesResponseDto { notes: dtos })
}

pub async fn create_note(
    State(store): State<AppState>,
    Json(req): Json<CreateNoteRequestDto>,
) -> Response {
    if req.body.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, "body must not be empty").into_response();
    }
    let note = store.create(req.body);
    (
        StatusCode::CREATED,
        Json(CreateNoteResponseDto::from(note)),
    )
        .into_response()
}
