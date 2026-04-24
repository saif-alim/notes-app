# services/api — Axum Backend

Single crate. Axum 0.8 + tokio. In-memory store behind a trait. REST JSON transport; proto is IDL, DTOs are wire format. Tower middleware: tracing, concurrency limits, timeouts.

## Layout

```
services/api/
├── BUILD.bazel                    # rust_library + rust_binary + rust_test
├── Cargo.toml                     # direct deps (committed; read by crate_universe)
├── Cargo.lock                     # pinned by cargo; committed
├── src/
│   ├── lib.rs                     # create_router(store) — composable entrypoint
│   ├── main.rs                    # tokio runtime + bind :3000
│   ├── routes.rs                  # handler fns (list_notes, create_note)
│   ├── store.rs                   # NotesStore trait + InMemoryNotesStore
│   └── dto.rs                     # serde wire types + From impls ↔ prost types
├── tests/
│   └── notes_integration.rs       # oneshot against create_router
└── CLAUDE.md
```

## Routes

| Method | Path | Body | Response | Status |
|--------|------|------|----------|--------|
| `GET` | `/notes` | — | `{"notes":[...]}` | 200 |
| `POST` | `/notes` | `{"body":"..."}` | `{"note":{"id":..,"body":..,"created_at_ms":..}}` | 201 on success, 400 if body empty |

Note ordering on list: ascending by `created_at_ms`. Empty store returns `{"notes":[]}`. Errors return a JSON envelope: `{"error":{"code":"VALIDATION_ERROR","message":"..."}}`. Body text is trimmed before storage; `"  hello  "` canonicalizes to `"hello"`.

## How to add a route

1. Add a handler function in `src/routes.rs`:
   ```rust
   pub async fn update_note(
       State(store): State<AppState>,
       Path(id): Path<String>,
       Json(req): Json<UpdateNoteRequestDto>,
   ) -> Result<Json<NoteDto>, StatusCode> { /* ... */ }
   ```
2. Register it in `src/lib.rs::create_router`:
   ```rust
   .route("/notes/:id", put(routes::update_note))
   ```
3. If the request/response shape is new, add DTOs in `src/dto.rs` with `#[derive(Serialize, Deserialize)]` and snake_case field names (matches `notes.proto` and the Swift side).
4. If the schema is net-new, edit `libs/schema/notes.proto` first, then the DTO mirrors it — see `libs/schema/CLAUDE.md`.
5. Add an integration test case in `tests/notes_integration.rs` using `tower::ServiceExt::oneshot`.
6. `bazel test //services/api:integration_test` before committing.

## Store

`NotesStore` is a sync trait (not `async_trait`). `InMemoryNotesStore` uses `parking_lot::RwLock<HashMap<String, Note>>` — no lock poisoning, no `.expect` on the lock path. Swap for a persistent impl by implementing the trait on a new struct and changing the `Arc::new(...)` line in `main.rs`. No handler changes needed.

IDs: UUIDv4. Timestamps: `i64` milliseconds since Unix epoch via `SystemTime`.

## DTOs vs proto types

Handlers consume/emit serde DTOs from `dto.rs`. Proto types live at `//libs/schema:notes_rust_proto` (crate name `notes_proto`, module `notes::v1`). Only `From<Note>` conversions exist — the opposite-direction impls were deleted in Phase 5.5 after review flagged them as unused dead code with a silent-sentinel failure mode. DTO layer provides the serde surface prost doesn't emit by default; if schema and wire format ever diverge, the DTO is the place to encode the difference. See `docs/retrospective.md` Phase 5 entry for the DTOs-vs-pbjson call.

## Errors

`src/error.rs` defines `ApiError`. Handlers return `Result<T, ApiError>`; `ApiError` implements `IntoResponse` and emits the envelope:
```json
{"error": {"code": "VALIDATION_ERROR", "message": "body must not be empty"}}
```
Single code path for errors across endpoints. Extend by adding a variant + match arm in `error.rs`.

## Middleware (Phase 6+)

Tower middleware stack in `src/lib.rs::create_router` (built via `ServiceBuilder`). Constants: `MAX_CONCURRENT=100`, `REQUEST_TIMEOUT_SECS=5`, `MAX_BODY_BYTES=65536`.

| Layer | Position | What it does |
|-------|----------|--------------|
| `TraceLayer` | outermost | emits `tower_http` spans; captures full latency + status |
| `RequestBodyLimitLayer` | 2nd | rejects request bodies >64KB with 413; no handler work done |
| `HandleErrorLayer` | 3rd | catches `tower::BoxError` from ConcurrencyLimitLayer → 503 |
| `ConcurrencyLimitLayer` | 4th | rejects when >100 requests in-flight |
| `TimeoutLayer` | innermost | 5s wall-clock timeout per request; returns 408 if exceeded |

Control log level via `RUST_LOG` env var:
```bash
RUST_LOG=notes_api=debug,tower_http=debug bazel run //services/api:notes_api
```
Default: `notes_api=debug,tower_http=debug`. Set to `error` or `info` to reduce verbosity.

Benchmark with `oha`:
```bash
brew install oha
bazel run //services/api:notes_api &  # start server in background
./tools/bench/bench.sh                 # run load test (1000 GET, 500 POST)
```

## How to run

```bash
# Run the server
bazel run //services/api:notes_api
# Binds 127.0.0.1:3000 by default; override with BIND_ADDR=0.0.0.0:8080 etc.

# Hit it
curl -s localhost:3000/notes
curl -X POST localhost:3000/notes -H 'content-type: application/json' -d '{"body":"first"}'

# Tests
bazel test //services/api:integration_test
```

## Gotchas

- `create_router` requires an `Arc<dyn NotesStore>`; the `with_state::<()>(...)` turbofish is needed because Axum can't otherwise infer the unit `S` type parameter of the returned `Router`.
- Cold build pulls ~100 crates (tokio, axum, serde, tower tree). 30s–2min depending on CPU. Cached builds are sub-second.
- `Cargo.lock` is committed — `crate_universe` needs it pinned. Do not add the repo-level `Cargo.lock` gitignore rule back.
