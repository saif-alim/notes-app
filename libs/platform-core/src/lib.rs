// uniffi scaffolding only when building the FFI cdylib (cargo --features ffi).
// The Bazel rust_library target omits this feature, keeping the sandbox
// free of uniffi's proc-macro Cargo.toml file I/O.
#[cfg(feature = "ffi")]
uniffi::setup_scaffolding!();

#[cfg(feature = "ffi")]
pub mod ffi;
pub mod id;
pub mod model;
pub mod store;
pub mod time;
pub mod validate;

#[cfg(feature = "ffi")]
pub use ffi::NotesCore;
pub use model::Note;
pub use store::{InMemoryNotesStore, NotesStore};
pub use validate::{trim_note_body, ValidationError};
