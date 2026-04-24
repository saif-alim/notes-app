# libs/platform-core — Shared Rust Core

Proto-free crate containing domain logic shared between the Axum backend
and the Android FFI layer.

## What's in here

| Module | Contents |
|--------|----------|
| `model` | `Note { id, body, created_at_ms }` — domain struct |
| `id` | `new_note_id()` — UUID v4 string |
| `time` | `now_ms()` — `SystemTime` → i64 milliseconds |
| `validate` | `trim_note_body(&str) -> Result<String, ValidationError>` |
| `store` | `NotesStore` trait + `InMemoryNotesStore` (parking_lot::RwLock) |
| `ffi` | `NotesCore` uniffi::Object — Android FFI surface (feature-gated) |

## Build

```bash
# Standard (used by Bazel rust_library and services/api dep):
cargo check -p platform_core

# With FFI surface (for Android cdylib):
cargo build -p platform_core --features ffi --release
# → target/release/libplatform_core.dylib (macOS) / .so (Linux)

# Tests (all):
cargo test -p platform_core           # no-ffi path (matches Bazel)
cargo test -p platform_core --features ffi  # validates FFI facade
bazel test //libs/platform-core:platform_core_test
```

## Bazel target

`//libs/platform-core:platform_core` — `rust_library` (rlib).
Does NOT enable the `ffi` feature. Used by `//services/api:notes_api_lib`.
The `rust_test` target (`platform_core_test`) exercises `id`, `time`,
`validate`, `store` modules.

## Why proto-free?

`services/api` uses `notes_proto::notes::v1::Note` (prost-generated) as the
IDL type. platform-core deliberately does NOT depend on the proto crate:
- keeps the Android `.so` small (no prost in FFI surface)
- lets Android consume `platform_core::Note` directly (plain Rust struct)
- mirrors the Swift hand-Codable pattern from Phase 4 (same tradeoff,
  same decision: 4 messages × 3 fields = ~20 lines, not worth build-infra tax)

`services/api/src/dto.rs` does `From<platform_core::Note>` for its JSON DTOs.

## UniFFI surface (`ffi` feature)

`NotesCore` is a `uniffi::Object` that wraps `InMemoryNotesStore`. Kotlin gets:

```kotlin
val core = NotesCore()
val note = core.createNote("hello")  // panics on empty — ViewModel validates first
val notes = core.listNotes()         // Vec<Note> ascending by created_at_ms
core.destroy()                        // call in ViewModel.onCleared()
```

Generated Kotlin bindings are committed in
`apps/android/app/src/main/java/uniffi/platform_core/platform_core.kt`.

## How to add a function to the FFI surface

1. Add the function to `src/ffi.rs` with `#[uniffi::export]` on the impl.
2. Add a test in `ffi.rs`.
3. If a new return type is needed, add `#[derive(uniffi::Record)]` (inside `#[cfg(feature = "ffi")]`).
4. Rebuild dylib: `cargo build -p platform_core --features ffi --release`
5. Regenerate Kotlin bindings: `cargo run --manifest-path tools/uniffi-bindgen/Cargo.toml -- generate --language kotlin --library target/release/libplatform_core.dylib --out-dir apps/android/app/src/main/java`
6. Commit updated `platform_core.kt`.
7. Rebuild Android .so: `./tools/build-android.sh`

## Feature flag rationale

`uniffi::setup_scaffolding!()` and `#[derive(uniffi::Record)]` macros read
`Cargo.toml` at compile time via file I/O — which fails in Bazel's hermetic
sandbox (Cargo.toml is not in the sandbox inputs). Feature-gating avoids
this: Bazel compiles without `ffi`; the Android cargo build enables it.

This is the same class of Bazel-ecosystem friction as Phase 4's
`rules_proto_grpc_swift` → CcInfo breakage.
