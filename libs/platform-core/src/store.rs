use std::collections::HashMap;

use parking_lot::RwLock;

use crate::model::Note;
use crate::{id, time};

pub trait NotesStore: Send + Sync + 'static {
    fn list(&self) -> Vec<Note>;
    fn create(&self, body: String) -> Note;
}

pub struct InMemoryNotesStore {
    notes: RwLock<HashMap<String, Note>>,
}

impl InMemoryNotesStore {
    pub fn new() -> Self {
        Self {
            notes: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryNotesStore {
    fn default() -> Self {
        Self::new()
    }
}

impl NotesStore for InMemoryNotesStore {
    fn list(&self) -> Vec<Note> {
        let mut out: Vec<Note> = self.notes.read().values().cloned().collect();
        // guard dropped; sort outside lock so writers don't block during O(N log N)
        out.sort_by_key(|n| n.created_at_ms);
        out
    }

    fn create(&self, body: String) -> Note {
        // Caller has already validated the body (trim + non-empty).
        // We trim again here so the canonical stored form never has surrounding whitespace.
        let note = Note {
            id: id::new_note_id(),
            body: body.trim().to_string(),
            created_at_ms: time::now_ms(),
        };
        let mut guard = self.notes.write();
        guard.insert(note.id.clone(), note.clone());
        note
    }
}
