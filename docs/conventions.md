# Conventions

## Naming

### Rust
- Modules: `snake_case` (e.g., `notes_store`, `api_client`)
- Functions: `snake_case` (e.g., `insert_note`, `get_notes`)
- Constants: `SCREAMING_SNAKE_CASE` (e.g., `MAX_NOTE_LENGTH`)
- Types: `PascalCase` (e.g., `NotesStore`, `Note`)

### Swift
- Classes/Structs: `PascalCase` (e.g., `NotesViewModel`, `APIClient`)
- Properties/Methods: `camelCase` (e.g., `notesList`, `fetchNotes()`)
- Constants: `camelCase` (e.g., `defaultTimeout`)

### Proto
- Messages: `PascalCase` (e.g., `Note`, `NotesList`)
- Fields: `snake_case` (e.g., `note_id`, `created_at`)

## Error Handling

### Rust Backend
- Return `Result<T, ApiError>` from handlers
- ApiError variants: `ValidationError`, `NotFound`, `InternalError`
- Log errors with `tracing::error!()` at error boundaries
- User-facing responses: JSON error objects with code and message

### iOS
- Async/await with do-try-catch or Result handling
- User-facing errors: show alert with localized message
- Silence network logs in release builds

## Commit Style

**Imperative present tense:**
```
add GET /notes endpoint
fix NotesStore concurrent access race
refactor APIClient for retry logic
```

**Not:** "Added", "Fixed", "Refactored", "WIP", "TODO"

**Format:**
```
<action> <thing> [reason or context]

[optional body if "why" is non-obvious]
```

Example with body:
```
add tower concurrency middleware

Prevents thundering herd on backend. Set to 100 concurrent requests
per connection (tuned in Phase 6 bench).
```

## Testing

- Rust backend: no `#[cfg(test)]` unit modules — all `src/` logic is reachable through HTTP, so one integration suite in `services/api/tests/notes_integration.rs` drives the real router via `tower::ServiceExt::oneshot`. Add an integration case per handler branch. Revisit if a store impl grows logic unreachable from routes.
- Rust library (`libs/platform-core`): `#[cfg(test)]` module inline with source — no HTTP surface to exercise.
- Swift iOS: three XCTest files under `apps/ios/Tests/` — ViewModel state transitions, View driver tests (state → rendered state), APIClient decode / error-envelope / URL assembly. `TestDoubles.swift` holds fakes + `StubURLProtocol`; not a test case.
- Test naming: `test_<function>_<scenario>` in Rust (e.g., `post_oversized_body_returns_413`); `test_<scenario>_<expectedOutcome>` in Swift (e.g., `test_load_fromIdle_transitionsLoadingThenLoaded`).
- `ios_unit_test` targets carry `tags = ["manual"]` to stay out of `bazel test //...` — invoke explicitly via `bazel test //apps/ios:NotesTests`. Simulator device pinned in `.bazelrc`.

## Bazel

- Target naming: `<dir>_<kind>` (e.g., `notes_rust_proto`, `notes_swift_proto`, `platform_core_test`, `integration_test`). `NotesApp` / `NotesTests` / `NotesLib` follow Apple rules conventions.
- `tags = ["manual"]` on iOS application + test targets — keeps simulator-dependent targets out of `//...` so CI without a simulator configured still works.
- `.bazelignore` lists `.claude/worktrees` (reviewer-swarm sandboxes) + `target` (Cargo scratch). Without it, Bazel traverses stale BUILD files inside worktrees and fails on `all_crate_deps`.
- Bzlmod only (`MODULE.bazel`). No legacy `WORKSPACE`.
- `rust_prost_library` for proto codegen; hand-written Codables on the Swift side (see `libs/schema/CLAUDE.md` and retro §Phase 4 for why the Swift proto chain is scope-cut on Bazel 9).

## Documentation

- Code comments: "WHY" only (why = non-obvious constraint, invariant, workaround)
- Don't comment "what" — well-named identifiers do that
- Block comments for algorithmic notes; inline comments rare
- README + docs/ are first-class artifacts, updated in same commit as code changes
