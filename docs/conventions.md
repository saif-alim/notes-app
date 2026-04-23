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

- Unit tests in same file as source (Rust: `#[cfg(test)]`, Swift: dedicated Tests/)
- Integration tests in `tests/` or `Tests/` at package root
- Test naming: `test_<function>_<scenario>` (e.g., `test_insert_note_creates_id`)
- Fixtures and mocks in `tests/common/` or `Tests/Helpers/`

## Documentation

- Code comments: "WHY" only (why = non-obvious constraint, invariant, workaround)
- Don't comment "what" — well-named identifiers do that
- Block comments for algorithmic notes; inline comments rare
- README + docs/ are first-class artifacts, updated in same commit as code changes

## TODO

- [ ] Flesh out as conventions are established per phase
- [ ] Add Bazel naming conventions (targets, packages)
- [ ] Add proto import/dependency conventions
