# Test Plan

## Unit Tests

### Rust Backend
- `NotesStore::insert()` — appends to map
- `NotesStore::list()` — returns all notes
- Model validation — body length, timestamp fields
- Location: `services/api/src/tests/` or inline via `#[cfg(test)]`

### iOS (Phase 8 — done)
- `NotesViewModelTests` — 6 cases: load from idle, error no-cache, error keeps-cache, create success, create error, empty-body guard
- `APIClientTests` + `APIErrorUserMessageTests` — URL composition, list decode, create decode, server error envelope, unknown 500, all `userMessage` cases
- `FakeNotesAPI` / `FailingNotesAPI` / `SwitchingNotesAPI` for ViewModel tests; `StubURLProtocol` for `APIClient` tests
- Location: `apps/ios/Tests/`
- Run: `bazel test //apps/ios:NotesTests`

## Integration Tests

### Backend
Single integration test hitting the live router:
- POST `/notes` → captures ID, body
- GET `/notes` → returns list with posted note
- Location: `services/api/tests/integration_test.rs`

### iOS
Manual E2E round-trip via simulator + live backend (see §E2E below). No separate integration test suite; `StubURLProtocol` covers the network boundary.

## E2E (Manual)

1. Start backend: `bazel run //services/api`
2. Open iOS simulator, run app
3. Create note in app
4. Verify note appears in list (local backend round-trip)

## Load Smoke Test

`oha` benchmark (Phase 6):
```bash
# Backend must be running: bazel run //services/api:notes_api
bash tools/bench/bench.sh
# Measures p50/p99 latency under sustained load (1000 GET, 500 POST)
```

Requires `oha`: `brew install oha`. Measures end-to-end HTTP latency including tower middleware (tracing, timeouts, concurrency limits).

## Failure Modes

- Backend unreachable → iOS shows `.error` inline Retry row (no cache) or `lastLoadError` alert (stale cache kept)
- Empty list → iOS shows "No notes yet" placeholder
- Invalid body → backend 400; iOS client guards trim+empty before send

## TODO

- [x] Flesh out unit tests per component (Phase 5–7)
- [x] Write integration test (Phase 5)
- [x] Add load test script (Phase 6)
- [x] XCTest harness for iOS (Phase 8)
