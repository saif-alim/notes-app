# Test Plan

## Unit Tests

### Rust Backend
No separate unit-test target — the `NotesStore` trait, `InMemoryNotesStore`, `ApiError` → `IntoResponse`, and DTO conversions are all exercised end-to-end through the integration suite below (handlers run their real store + error paths in-process via `tower::ServiceExt::oneshot`). Keeps a single test target + covers every code path reachable from HTTP. Revisit when persistence swaps in and a store impl grows logic not reachable via routes.

### iOS (Phase 8 — done)
Three test files + one support file under `apps/ios/Tests/` — 21 XCTest cases total:

- `NotesViewModelTests.swift` — `NotesViewModelTests` class, 8 cases: load from idle, error no-cache, error keeps-cache, create success, create error, empty-body guard, load non-APIError wraps into `.transport`, create non-APIError wraps into `.transport`.
- `NotesListViewTests.swift` — `NotesListViewTests` class, 7 cases: view transitions loaded / loaded-empty / error, draft preserved on create failure, stale-while-revalidate keeps cache on refresh failure, Add button empty-body guard, Add button calls API on non-empty draft.
- `APIClientTests.swift` — two `XCTestCase` classes:
  - `APIClientTests` — 6 cases: URL resolution with leading slash, list decode, create decode, server error envelope, unknown 500, decoding error on malformed JSON.
  - `APIErrorUserMessageTests` — 1 case covering every `APIError.userMessage` branch.
- `TestDoubles.swift` (support, not a test case) — `FakeNotesAPI`, `FailingNotesAPI`, `SwitchingNotesAPI` for ViewModel tests; `StubURLProtocol` for `APIClient` tests.

Run: `bazel test //apps/ios:NotesTests`. Target carries `tags = ["manual"]` so `bazel test //...` skips it — invoke explicitly. `.bazelrc` pins `--ios_simulator_device="iPhone 16 Pro" --ios_simulator_version=18.4` so the default xctestrunner device doesn't miss on current SDKs.

## Integration Tests

### Backend
25 integration tests hitting the live router in-process. Uses `tower::ServiceExt::oneshot` to drive `create_router(store)` without binding a socket — full suite runs in ~0.3s.

Coverage:
- **Happy path** — POST → GET round-trip, `created_at_ms` ascending order, body trim canonicalization.
- **Validation** — empty body → 400, whitespace-only → 400, missing `body` field → 400, null `body` → 400, non-string `body` → 400.
- **Content-Type / parsing** — wrong `Content-Type` → 415, missing Content-Type, malformed JSON → 400.
- **Unicode** — emoji, RTL, CJK round-trip intact.
- **Limits** — oversized body (>64KB) → 413.
- **Routing** — unknown route → 404, wrong method → 405.
- **Concurrency** — concurrent writes across tasks preserve all entries (no lost writes under `parking_lot::RwLock`).

Location: `services/api/tests/notes_integration.rs`. Run: `bazel test //services/api:integration_test`.

### iOS
Manual E2E round-trip via simulator + live backend (see §E2E below). No separate integration test suite; `StubURLProtocol` covers the network boundary.

## E2E (Manual)

1. Start backend: `bazel run //services/api:notes_api`
2. Launch app: `./tools/run-ios-sim.sh` (builds via Bazel, installs + launches on booted simulator)
3. Type body → tap **Add** → note appears in list with relative timestamp
4. Kill + relaunch app → list re-fetches from backend (confirms data is server-side, not cached locally)
5. Kill backend mid-refresh → pull-to-refresh keeps stale `.loaded` list + surfaces error alert (stale-while-revalidate)

## Load Smoke Test

`oha` benchmark (Phase 6):
```bash
# Backend must be running: bazel run //services/api:notes_api
bash tools/bench/bench.sh
# Measures p50/p99 latency under sustained load (1000 GET, 500 POST)
```

Requires `oha`: `brew install oha`. Measures end-to-end HTTP latency including tower middleware (tracing, timeouts, concurrency limits).

Expected baseline (local, M4 MacBook Pro, in-memory store): ~0.2ms p50 GET, ~0.3ms p50 POST. Middleware overhead unmeasurable in the noise. Treat as a regression signal — deviations >10× warrant investigation.

## Failure Modes

- Backend unreachable → iOS shows `.error` inline Retry row (no cache) or `lastLoadError` alert (stale cache kept)
- Empty list → iOS shows "No notes yet" placeholder
- Invalid body → backend 400; iOS client guards trim+empty before send

