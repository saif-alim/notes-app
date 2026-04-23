# Test Plan

## Unit Tests

### Rust Backend
- `NotesStore::insert()` — appends to map
- `NotesStore::list()` — returns all notes
- Model validation — body length, timestamp fields
- Location: `services/api/src/tests/` or inline via `#[cfg(test)]`

### iOS
- NotesViewModel — observe state updates on successful POST
- APIClient — mock network responses
- Location: `apps/ios/Tests/` (XCTest)

## Integration Tests

### Backend
Single integration test hitting the live router:
- POST `/notes` → captures ID, body
- GET `/notes` → returns list with posted note
- Location: `services/api/tests/integration_test.rs`

### iOS
One round-trip XCTest:
- Mock APIClient or point to local backend on :3000
- Create note, fetch list, verify in UI
- Location: `apps/ios/Tests/integration/`

## E2E (Manual)

1. Start backend: `bazel run //services/api`
2. Open iOS simulator, run app
3. Create note in app
4. Verify note appears in list (local backend round-trip)

## Load Smoke Test

`oha` benchmark (Phase 6):
```bash
# Installed in Phase 6 tooling
bash tools/bench/load.sh
# Measures p50/p99 latency under sustained load
```

See `tools/bench/` for script.

## Failure Modes

- Backend unreachable → iOS shows error state (Phase 8)
- Empty list → iOS shows empty state (Phase 8)
- Invalid body → backend rejects, iOS surface error (Phase 6)

## TODO

- [ ] Flesh out unit tests per component (Phase 5–7)
- [ ] Write integration test (Phase 5)
- [ ] Add load test script (Phase 6)
- [ ] XCTest harness for iOS (Phase 7)
