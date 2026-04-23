# notes-app

A minimal two-endpoint notes application demonstrating a full-stack iOS + Rust + Bazel architecture.

## What is this?

Notes app: SwiftUI iOS client + Rust/Axum backend + shared .proto schema, built with Bazel (Bzlmod). Optimizes for clear separation of concerns and pluggable persistence.

## Quickstart

```bash
# Build everything
bazel build //...

# Run backend server (starts on :3000)
bazel run //services/api

# Run iOS app in simulator
# Open apps/ios/Package.swift in Xcode, select simulator, run
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
bazel test //...               # all tests
bazel test //services/api/...  # backend tests only
bazel test //apps/ios/...      # iOS tests only
```

## Known Limitations

- In-memory storage only (Phase 4+: pluggable to SQLite/Postgres)
- iOS app is simulator-only in Phase 1–2 (device testing in Phase 3+)
- Bazel-iOS integration may fall back to xcodebuild wrapper if rules_xcodeproj resists (see Phase 3 decision)

## Documentation

- [Architecture](docs/architecture.md)
- [Test Plan](docs/test-plan.md)
- [Retrospective](docs/retrospective.md)
- [Root navigation](CLAUDE.md)