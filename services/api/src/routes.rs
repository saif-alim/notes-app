use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};

use crate::dto::{
    CreateNoteRequestDto, CreateNoteResponseDto, ListNotesResponseDto, NoteDto,
};
use crate::error::ApiError;
use crate::store::NotesStore;

pub type AppState = Arc<dyn NotesStore>;

pub async fn list_notes(
    State(store): State<AppState>,
) -> Result<Json<ListNotesResponseDto>, ApiError> {
    let notes = store.list();
    let dtos: Vec<NoteDto> = notes.into_iter().map(NoteDto::from).collect();
    Ok(Json(ListNotesResponseDto { notes: dtos }))
}

pub async fn create_note(
    State(store): State<AppState>,
    Json(req): Json<CreateNoteRequestDto>,
) -> Result<(StatusCode, Json<CreateNoteResponseDto>), ApiError> {
    if req.body.trim().is_empty() {
        return Err(ApiError::ValidationError("body must not be empty"));
    }
    let note = store.create(req.body);
    Ok((
        StatusCode::CREATED,
        Json(CreateNoteResponseDto::from(note)),
    ))
}
