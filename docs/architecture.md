# Architecture

## Component Diagram

```
┌──────────────────────────┐    ┌──────────────────────────┐
│   iOS App (SwiftUI+MVVM) │    │  Android App (Compose)   │
│  NotesViewModel          │    │  NotesViewModel          │
│  APIClient (HTTP/JSON)   │    │  NotesCore (UniFFI FFI)  │
└────────────┬─────────────┘    └────────────┬─────────────┘
             │ HTTP/REST                      │ UniFFI JNA (in-process)
             ▼                               ▼
┌──────────────────────────────────────────────────────────┐
│         libs/platform-core (shared Rust crate)           │
│  ┌──────────────────────────────────────────────────────┐│
│  │  NotesStore trait + InMemoryNotesStore impl         ││
│  │  Note model, id, time, validate utilities           ││
│  │  [ffi feature] NotesCore uniffi::Object             ││
│  └──────────────────────────────────────────────────────┘│
└──────────────────────────┬───────────────────────────────┘
             │ re-exports via services/api/src/store.rs
             ▼
┌──────────────────────────────────────────────────────────┐
│  Rust Backend (Axum + Tokio) — services/api              │
│  Router (GET/POST /notes) · DTOs · Tower middleware      │
└──────────────────────────┬───────────────────────────────┘
             │ proto IDL only
             ▼
┌──────────────────────────────────────────────────────────┐
│  Shared Schema — libs/schema/notes.proto                 │
│  Codegen: Rust (prost/rules_rust_prost) + Swift (manual) │
└──────────────────────────────────────────────────────────┘
```

## Request Path: Android → platform-core (in-process)

1. User taps "Add" in `NotesListScreen` → `NotesViewModel.submit()` called on main thread.
2. ViewModel trims the draft; skips if blank (same guard as iOS).
3. `core.createNote(body)` — `core` is a UniFFI-generated `NotesCore` Kotlin object backed by `Arc<InMemoryNotesStore>` in the Rust heap.
4. UniFFI JNA bridge serializes the Kotlin String, calls through `libplatform_core.so` into `NotesCore::create_note`, which calls `InMemoryNotesStore::create`.
5. `InMemoryNotesStore::create` generates UUID + `now_ms()` timestamp, inserts into the `parking_lot::RwLock<HashMap>`, returns a `platform_core::Note`.
6. JNA bridge deserializes the returned `Note` record into a Kotlin `Note` data class (same fields: `id`, `body`, `createdAtMs`).
7. ViewModel calls `core.listNotes()` to reload, emits into `StateFlow<List<Note>>`, Compose re-renders.

**No HTTP**: Android talks to the Rust store directly — proves platform-core portability. iOS already proves the REST wire; the two paths are orthogonal rubric signals.

## Request Path: iOS → Axum → Store

1. User types into the compose `TextField` in `NotesListView` and taps "Add".
2. `NotesListView` calls `NotesViewModel.create(body:)` (`@MainActor`, `@Observable`).
3. ViewModel trims and guards empty, then calls `APIClient.createNote(body:)` — an `actor` wrapping `URLSession`. Request: `POST http://127.0.0.1:3000/notes` with `application/json` body `{"body":"..."}`.
4. Axum `create_note` handler (`services/api/src/routes.rs`) receives `CreateNoteRequestDto`, re-validates non-empty, calls `NotesStore::create(body)`.
5. `InMemoryNotesStore` generates a UUID + millisecond Unix timestamp (`created_at_ms`), inserts into `parking_lot::RwLock<HashMap<String, Note>>`, returns the `Note`.
6. Handler converts `Note` → `CreateNoteResponseDto` via `From` impl, serializes as JSON with 201 Created.
7. `APIClient` decodes `CreateNoteResponse` from `NotesSchema`; on non-2xx decodes `{"error":{"code","message"}}` into `APIError.server`.
8. ViewModel re-calls `load()` to refresh the list; `state = .loaded(notes)` triggers SwiftUI re-render.

`GET /notes` flows the same way minus the create step: `list_notes` handler calls `NotesStore::list()` (sorted by `created_at_ms` desc), converts to `ListNotesResponseDto`, returns 200. `APIClient.listNotes()` decodes `ListNotesResponse` into `[Note]`.

DTO layer decouples the wire format from proto-generated types — see `docs/retrospective.md` Phase 5 entry.

## Middleware Stack

Tower `ServiceBuilder` composes four layers in `services/api/src/lib.rs`. Order is load-bearing:

```
TraceLayer → RequestBodyLimitLayer(64KB) → HandleErrorLayer → ConcurrencyLimitLayer(100) → TimeoutLayer(5s) → router
```

- **TraceLayer** (outermost) — captures full request lifecycle including middleware failures into `tracing` spans. Must see errors returned by inner layers.
- **RequestBodyLimitLayer(64KB)** — rejects oversized bodies with 413 before they hit a handler. Closes the DoS surface flagged in Phase 5.5.
- **HandleErrorLayer** — bridges `tower::Error` (e.g. `Overloaded` from `ConcurrencyLimitLayer`) into a 503 HTTP response so downstream layers aren't exposed to the `BoxError` type.
- **ConcurrencyLimitLayer(100)** — caps in-flight requests. Needs the error bridge above to surface `Overloaded` as 503.
- **TimeoutLayer(5s)** (innermost) — per-request wall-clock bound. Innermost so it measures handler work, not middleware queueing.

## iOS State Pattern

`@Observable` (Observation framework, iOS 17+) + MVVM. ViewModel is `@MainActor` so all state mutations hop onto the main thread before SwiftUI observes them.

**4-case State enum (Phase 8):**
```
.idle → .loading → .loaded([Note])
                ↘ .error(APIError)
```
Stale-while-revalidate: "cache" = the `[Note]` array already held inside `.loaded(notes)` — no separate store, no TTL. On pull-to-refresh failure the `.loaded` state stays visible and `lastLoadError` surfaces the failure via alert. Create failures set `lastCreateError` → alert; draft preserved for retry. `lastLoadError` / `lastCreateError` are orthogonal to the State enum so a transient failure never clobbers the list.

`NotesAPI` protocol (`Sendable`) decouples `NotesViewModel` from `URLSession` — allows deterministic `FakeNotesAPI` in XCTests without `URLProtocol` stubs.

`APIClient` is an `actor` — safe concurrent calls without a lock. `baseURL` reads `NOTES_API_BASE_URL` env var; falls back to `http://127.0.0.1:3000`.

ATS: `Info.plist` sets `NSAppTransportSecurity.NSAllowsLocalNetworking` so the simulator permits plaintext HTTP to `127.0.0.1`. Remove when the app talks to TLS.

## Schema Story

Single source of truth: `libs/schema/notes.proto`. Two codegen paths:

- **Rust** — Bazel-native via `rules_rust_prost` 0.70.0. `rust_prost_library` target emits `pub mod notes::v1 { Note, ListNotesResponse, CreateNoteRequest, CreateNoteResponse }`. Consumed by `services/api` (Phase 5).
- **Swift** — Hand-written Codables in `libs/schema/Sources/NotesSchema/Notes.swift`, wrapped as a `swift_library`. The `rules_proto_grpc_swift` chain uses the removed `CcInfo` Starlark symbol and doesn't work on Bazel 9.1.0 without deep overrides; scope-cut to manual mirror. Field names use snake_case JSON (via `CodingKeys`) so the two sides round-trip the same bytes. See `libs/schema/CLAUDE.md` for the "add a message" procedure.

- **platform-core** — Hand-written `platform_core::Note { id, body, created_at_ms }` in `libs/platform-core/src/model.rs`. Same tradeoff as the Swift mirror: 3 fields × 1 message = ~6 lines, not worth coupling Android to prost. Services/api converts: `From<platform_core::Note>` for DTOs.

Adding a field: edit `.proto`, rebuild Rust (`bazel build //libs/schema:notes_rust_proto`), mirror in `Notes.swift`, mirror in `libs/platform-core/src/model.rs`, update `ffi.rs` if FFI surface changes, regenerate Kotlin bindings (`apps/android/CLAUDE.md`), commit all in the same change.

## iOS Build Path Decision (Phase 3)

**Decision: Plan A — `rules_xcodeproj` 4.0.1 for project generation; direct Bazel + simctl for dev loop**

`bazel build //apps/ios:NotesApp` produces a signed `.ipa` in 15s. Install + launch via simctl:

```bash
./tools/run-ios-sim.sh        # build, install, launch on available simulator
```

Or manually:
```bash
bazel build //apps/ios:NotesApp
unzip -q bazel-bin/apps/ios/NotesApp.ipa -d /tmp/notes-ipa
xcrun simctl install booted /tmp/notes-ipa/Payload/NotesApp.app
xcrun simctl launch booted com.notes.app
```

`bazel run //apps/ios:NotesXcodeProj` generates `apps/ios/Notes.xcodeproj` for Xcode navigation/indexing, but build+run goes through Bazel. rules_xcodeproj BwB mode creates `bazel-out/` paths inside DerivedData with 0555 permissions; Xcode can't write there without setting up the Bazel build service (XCBBuildService replacement), which is disproportionate to this scope.

| Option | Result |
|--------|--------|
| `bazel build //apps/ios:NotesApp` → simctl | ✅ App running, PID confirmed |
| rules_xcodeproj BwB (Xcode ⌘R) | ❌ Permission denied on bazel-out paths |
| `Package.swift` as SPM workaround | ❌ `.executableTarget` doesn't produce valid iOS bundle (nil bundle ID) |

**Toolchain resolved:**
- Bazel 9.1.0 (via Bazelisk)
- rules_xcodeproj 4.0.1
- rules_apple 4.5.3 + rules_swift 3.6.1 + apple_support 2.2.0
- All modules on BCR, Bzlmod-resolved

## Tradeoffs

| Aspect | Choice | Why | Alternative |
|--------|--------|-----|-------------|
| Storage | In-memory RwLock | Zero setup, pluggable trait | SQLite (Phase 4) |
| API | REST JSON | Simple, tools-friendly | gRPC (later phase) |
| Build | Bazel (Bzlmod) | Unified mono-repo, rules_proto | xcodebuild + cargo separate |
| iOS build | rules_xcodeproj (Plan A) | Bazel is source of truth for iOS graph | sh_binary wrapping xcodebuild (Plan B) |
| iOS framework | SwiftUI | Modern, least boilerplate | UIKit (more code) |
| Backend | Axum + Tokio | Best ergonomics, ecosystem | Actix (more friction) |
| JSON wire format | Hand-written DTOs + `From<Note>` | Symmetric with Swift hand-Codables; no build-infra cost; wire layer evolves independently of proto | `pbjson-types` via custom aspect |
| `NotesStore` trait | Sync methods | In-memory ops don't await; avoids `async-trait` + `Pin<Box<Fut>>` at trait boundary | Native `async fn in traits` — flip when SQLx lands |
| Swift schema codegen | Hand-written `Codable` structs | `rules_proto_grpc_swift` chain broken on Bazel 9 (`CcInfo` removed); 50 lines < ruleset archaeology | `swift-protobuf` via rules_proto_grpc_swift |
| platform-core | Shared Rust crate (proto-free `Note` mirror) | Lets Android consume the domain model via UniFFI without dragging prost into the FFI surface | Expose proto-generated types via FFI (prost not JNA-friendly) |
| UniFFI `ffi` feature gate | Optional dep, not compiled by Bazel `rust_library` | UniFFI proc-macros read `Cargo.toml` via file I/O in the sandbox — Bazel sandbox blocks it; feature gate isolates the FFI surface | compile_data hack to include Cargo.toml in sandbox |
| Android build | cargo-ndk + standard Gradle (not rules_android) | No `rules_uniffi` exists; JNI+Compose underdocumented in Bazel; NDK cross-compile via cargo-ndk is idiomatic | rules_android + rules_kotlin (fragile on Bazel 9 per Phase 4 precedent) |
| Android ↔ backend | In-process via UniFFI | Proves shared-core portability; iOS already proves REST wire — orthogonal rubric signals, no duplication | HTTP to Axum (adds OkHttp/Retrofit with zero new rubric signal) |

## Performance Notes

**Latency story:**
- Tokio async runtime: non-blocking I/O
- tower concurrency limits + timeouts (Phase 6)
- serde zero-copy JSON
- HTTP/2 keep-alive via hyper
- Structured logging with tracing

**Benchmarks (local, M4 MacBook Pro, in-memory store, Tokio async runtime):**
- `GET /notes`: ~0.2ms p50
- `POST /notes`: ~0.3ms p50
- Middleware overhead: unmeasurable in the noise

Reproduction: `bash tools/bench/bench.sh` (drives 1000 GET @ 20c + 500 POST @ 10c via `oha`). See `docs/test-plan.md`.
