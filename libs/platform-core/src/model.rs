// uniffi::Record derive is only active when the `ffi` feature is enabled.
// Without it, Note is a plain Rust struct — safe to compile in Bazel without
// uniffi proc-macro touching Cargo.toml.
#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    pub id: String,
    pub body: String,
    pub created_at_ms: i64,
}
