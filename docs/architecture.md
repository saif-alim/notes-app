# Architecture

## Component Diagram

```
┌─────────────────────────────────────┐
│      iOS App (SwiftUI + MVVM)       │
│  ┌─────────────────────────────────┐│
│  │  NotesView (list + detail)     ││
│  │  NotesViewModel (state mgmt)   ││
│  │  APIClient (HTTP calls)        ││
│  └─────────────────────────────────┘│
└─────────────┬───────────────────────┘
              │
              │ HTTP (REST)
              ▼
┌─────────────────────────────────────┐
│  Rust Backend (Axum + Tokio)        │
│  ┌─────────────────────────────────┐│
│  │  Router (GET/POST /notes)      ││
│  │  NotesStore (trait + impl)     ││
│  │  Tracing + Tower middleware    ││
│  └─────────────────────────────────┘│
└─────────────┬───────────────────────┘
              │
              ▼
┌─────────────────────────────────────┐
│  Shared Schema (.proto)             │
│  ├─ Note message                    │
│  └─ Codegen: Rust (prost) + Swift   │
└─────────────────────────────────────┘
```

## Request Path: iOS → Axum → Store

1. User taps "Add note"
2. SwiftUI posts `POST /notes { body: "..." }` via APIClient
3. Axum handler receives, validates, calls `NotesStore::insert()`
4. In-memory `RwLock<HashMap>` appends note
5. Response serialized as JSON → iOS
6. NotesViewModel updates, SwiftUI re-renders

## Schema Story

Single source of truth: `libs/schema/notes.proto`. Codegen via Bazel:
- **Rust**: `prost-build` generates Rust structs
- **iOS**: `swift-protobuf` generates Swift Codables

Adding a field: edit `.proto`, regenerate, update handlers/views.

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

## Performance Notes

**Latency story:**
- Tokio async runtime: non-blocking I/O
- tower concurrency limits + timeouts (Phase 6)
- serde zero-copy JSON
- HTTP/2 keep-alive via hyper
- Structured logging with tracing

**Benchmarks:** see `tools/bench/` and `docs/test-plan.md` (Phase 6+).

## TODO

- [ ] Component diagram visual (ASCII OK for now)
- [ ] Request flow diagram (mermaid or ASCII)
- [ ] Filled in at end of Phase 5 (backend) and Phase 7 (iOS)
