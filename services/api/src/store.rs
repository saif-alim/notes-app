use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

use notes_proto::notes::v1::Note;
use uuid::Uuid;

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
        let guard = self.notes.read().expect("RwLock poisoned");
        let mut out: Vec<Note> = guard.values().cloned().collect();
        out.sort_by_key(|n| n.created_at_unix);
        out
    }

    fn create(&self, body: String) -> Note {
        let note = Note {
            id: Uuid::new_v4().to_string(),
            body,
            created_at_unix: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock before epoch")
                .as_secs() as i64,
        };
        let mut guard = self.notes.write().expect("RwLock poisoned");
        guard.insert(note.id.clone(), note.clone());
        note
    }
}
