use serde::{Deserialize, Serialize};

use platform_core::Note;

// Wire DTOs. platform_core::Note is the domain model; DTOs are the JSON wire
// format. Field names use snake_case to match notes.proto and the Swift
// client's CodingKeys. Only the `From<Note> for NoteDto` direction exists —
// the opposite-direction impls were deleted in Phase 5.5 after review flagged
// them as dead code with a silent-sentinel failure mode.

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NoteDto {
    pub id: String,
    pub body: String,
    pub created_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ListNotesResponseDto {
    pub notes: Vec<NoteDto>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateNoteRequestDto {
    pub body: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateNoteResponseDto {
    pub note: NoteDto,
}

impl From<Note> for NoteDto {
    fn from(n: Note) -> Self {
        Self {
            id: n.id,
            body: n.body,
            created_at_ms: n.created_at_ms,
        }
    }
}

impl From<Note> for CreateNoteResponseDto {
    fn from(n: Note) -> Self {
        Self {
            note: NoteDto::from(n),
        }
    }
}
