use std::sync::Arc;

use crate::model::Note;
use crate::store::{InMemoryNotesStore, NotesStore};

/// UniFFI opaque object. Android Kotlin gets a `NotesCore` class; constructor
/// produces an Arc<Self> as required by uniffi::Object.
///
/// The in-process store is intentional: Android uses platform-core directly
/// rather than talking to the Axum backend, proving the shared-core story.
#[derive(uniffi::Object)]
pub struct NotesCore {
    store: InMemoryNotesStore,
}

#[uniffi::export]
impl NotesCore {
    #[uniffi::constructor]
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            store: InMemoryNotesStore::new(),
        })
    }

    /// Returns the newly created note. Panics if body is empty or whitespace-only.
    /// Android ViewModel validates before calling.
    pub fn create_note(&self, body: String) -> Note {
        assert!(!body.trim().is_empty(), "body must not be empty");
        self.store.create(body)
    }

    pub fn list_notes(&self) -> Vec<Note> {
        self.store.list()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ffi_create_and_list() {
        let core = NotesCore::new();
        let note = core.create_note("hello".to_string());
        assert_eq!(note.body, "hello");
        assert!(!note.id.is_empty());
        assert!(note.created_at_ms > 0);

        let notes = core.list_notes();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].body, "hello");
    }

    #[test]
    fn ffi_multiple_notes_ordered_by_timestamp() {
        let core = NotesCore::new();
        core.create_note("first".to_string());
        core.create_note("second".to_string());
        let notes = core.list_notes();
        assert_eq!(notes.len(), 2);
        assert!(notes[0].created_at_ms <= notes[1].created_at_ms);
    }
}
