use serde::{Deserialize, Serialize};

use notes_proto::notes::v1::{
    CreateNoteRequest, CreateNoteResponse, ListNotesResponse, Note,
};

// Wire DTOs. proto types are the schema contract; DTOs are the JSON wire
// format. Field names use snake_case to match notes.proto and the Swift
// client's CodingKeys. One-to-one conversions keep the boundary explicit.

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NoteDto {
    pub id: String,
    pub body: String,
    pub created_at_unix: i64,
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
            created_at_unix: n.created_at_unix,
        }
    }
}

impl From<NoteDto> for Note {
    fn from(d: NoteDto) -> Self {
        Self {
            id: d.id,
            body: d.body,
            created_at_unix: d.created_at_unix,
        }
    }
}

impl From<Vec<Note>> for ListNotesResponseDto {
    fn from(notes: Vec<Note>) -> Self {
        Self {
            notes: notes.into_iter().map(NoteDto::from).collect(),
        }
    }
}

impl From<ListNotesResponse> for ListNotesResponseDto {
    fn from(resp: ListNotesResponse) -> Self {
        Self::from(resp.notes)
    }
}

impl From<CreateNoteRequestDto> for CreateNoteRequest {
    fn from(d: CreateNoteRequestDto) -> Self {
        Self { body: d.body }
    }
}

impl From<Note> for CreateNoteResponseDto {
    fn from(n: Note) -> Self {
        Self {
            note: NoteDto::from(n),
        }
    }
}

impl From<CreateNoteResponse> for CreateNoteResponseDto {
    fn from(resp: CreateNoteResponse) -> Self {
        Self {
            note: resp.note.map(NoteDto::from).unwrap_or(NoteDto {
                id: String::new(),
                body: String::new(),
                created_at_unix: 0,
            }),
        }
    }
}
