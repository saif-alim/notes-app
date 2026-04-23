# Add a New Endpoint

**Scope:** Add a new REST endpoint to the Axum backend.

## Steps

1. **Define the proto message** (if needed)
   - Edit `libs/schema/notes.proto`
   - Run codegen (Phase 4+)

2. **Add handler in `services/api/src/routes/`**
   ```rust
   pub async fn get_note_by_id(
       Path(id): Path<String>,
       State(store): State<Arc<NotesStore>>,
   ) -> Json<Note> {
       // ...
   }
   ```

3. **Wire into router in `services/api/src/main.rs`**
   ```rust
   .route("/notes/:id", get(routes::get_note_by_id))
   ```

4. **Write integration test in `services/api/tests/integration_test.rs`**
   - Test happy path and error cases

5. **Update `docs/architecture.md`** if the request path changes

See `services/api/CLAUDE.md` (Phase 5+) for more detailed patterns.
