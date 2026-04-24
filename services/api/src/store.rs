// Domain trait and in-memory impl live in platform-core so they can be shared
// with the Android FFI layer. Re-export here so existing callers are unchanged.
pub use platform_core::{InMemoryNotesStore, NotesStore};
