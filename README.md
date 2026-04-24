# notes-app

A minimal two-endpoint notes application demonstrating a full-stack iOS + Android + Rust + Bazel architecture with a shared platform-core crate.

## What is this?

Notes app: SwiftUI iOS client + Jetpack Compose Android client + Rust/Axum backend + shared `.proto` schema + `libs/platform-core` shared Rust crate, built with Bazel (Bzlmod). Optimizes for clear separation of concerns and pluggable persistence. iOS talks to Axum over HTTP; Android calls platform-core in-process via UniFFI — two different portability stories.

## Quickstart

```bash
# Build everything
bazel build //...

# Run backend server (starts on :3000)
bazel run //services/api:notes_api

# Run iOS app in simulator
./tools/run-ios-sim.sh
```

## Configuration

- `NOTES_API_BASE_URL` — override backend URL in the iOS app. Defaults to `http://127.0.0.1:3000`.
  ```bash
  NOTES_API_BASE_URL=http://192.168.1.42:3000 ./tools/run-ios-sim.sh
  ```

## Repo Map

- `apps/ios/` — SwiftUI app (MVVM, HTTP to Axum)
- `apps/android/` — Jetpack Compose app (UniFFI in-process via platform-core)
- `services/api/` — Rust/Axum backend
- `libs/platform-core/` — shared Rust crate (store, model, validators; UniFFI FFI surface)
- `libs/schema/` — .proto shared schema (IDL)
- `tools/` — scripts, benchmarks, uniffi-bindgen
- `docs/` — architecture, test plan, retrospective

## How to add

- **New endpoint:** see `docs/recipes/add-endpoint.md`
- **New iOS screen:** see `docs/recipes/add-ios-screen.md`
- **New proto message:** see `libs/schema/CLAUDE.md` (Phase 4)

## Android Quickstart

**Prerequisites:** `cargo install cargo-ndk`, Android NDK installed (`ANDROID_NDK_HOME` set), Gradle wrapper.

```bash
# 1. Cross-compile the Rust shared library
./tools/build-android.sh

# 2. Build and install on emulator / device
cd apps/android
./gradlew installDebug
```

Kotlin bindings (`uniffi/platform_core/platform_core.kt`) are committed — no re-generation needed unless the FFI surface changes (see `apps/android/CLAUDE.md`).

## Test Commands

```bash
bazel test //...                              # all Bazel tests
bazel test //libs/platform-core:platform_core_test  # platform-core unit tests
bazel test //services/api:integration_test    # 24 backend integration tests
bazel test //apps/ios:NotesTests              # iOS XCTest suite

# Rust platform-core with FFI feature
cargo test -p platform_core --features ffi

# Load smoke test (backend must be running; requires `brew install oha`)
bash tools/bench/bench.sh
```

## Known Limitations

- In-memory storage only (pluggable to SQLite/Postgres via `NotesStore` trait)
- iOS app is simulator-only (device provisioning out of scope)
- iOS dev loop uses `tools/run-ios-sim.sh` (Bazel build + simctl); Xcode indexing via `bazel run //apps/ios:NotesXcodeProj`
- Android requires NDK + `cargo-ndk` for the `.so` (not pre-installed; `jniLibs/` is gitignored)
- Plaintext HTTP to localhost (iOS/ATS exception active); no TLS, no auth

## Documentation

- [Architecture](docs/architecture.md)
- [Test Plan](docs/test-plan.md)
- [Retrospective](docs/retrospective.md)
- [Root navigation](CLAUDE.md)