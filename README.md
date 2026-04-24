# notes-app

A minimal two-endpoint notes application demonstrating a full-stack iOS + Rust + Bazel architecture.

## What is this?

Notes app: SwiftUI iOS client + Rust/Axum backend + shared .proto schema, built with Bazel (Bzlmod). Optimizes for clear separation of concerns and pluggable persistence.

## Quickstart

```bash
# Build everything
bazel build //...

# Terminal 1 — start backend (keep running on :3000)
bazel run //services/api:notes_api

# Terminal 2 — build + install + launch iOS app on simulator
# (backend must be reachable on :3000 or the app shows "can't reach server")
./tools/run-ios-sim.sh
```

## Configuration

- `NOTES_API_BASE_URL` — override backend URL in the iOS app. Defaults to `http://127.0.0.1:3000`.
  ```bash
  NOTES_API_BASE_URL=http://192.168.1.42:3000 ./tools/run-ios-sim.sh
  ```

## Repo Map

- `apps/ios/` — SwiftUI app (MVVM)
- `services/api/` — Rust/Axum backend
- `libs/schema/` — .proto shared schema
- `tools/` — scripts, benchmarks
- `docs/` — architecture, test plan, retrospective

## How to add

- **New endpoint:** see `docs/recipes/add-endpoint.md`
- **New iOS screen:** see `docs/recipes/add-ios-screen.md`
- **New proto message:** see `libs/schema/CLAUDE.md` (Phase 4)

## Test Commands

```bash
bazel test //...                     # all tests
bazel test //services/api/...        # backend unit + integration tests
bazel test //apps/ios/...            # iOS tests (all)
bazel test //apps/ios:NotesTests     # iOS XCTest suite (explicit label)

# Load smoke test (backend must be running; requires `brew install oha`)
bash tools/bench/bench.sh
```

## Known Limitations

- In-memory storage only (Phase 4+: pluggable to SQLite/Postgres)
- iOS app is simulator-only (device testing out of scope)
- iOS dev loop uses `tools/run-ios-sim.sh` (Bazel build + simctl); Xcode indexing via `bazel run //apps/ios:NotesXcodeProj`

## Documentation

- [Architecture](docs/architecture.md)
- [Test Plan](docs/test-plan.md)
- [Retrospective](docs/retrospective.md)
- [Root navigation](CLAUDE.md)