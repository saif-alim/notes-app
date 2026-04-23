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

**Decision: Plan A — `rules_xcodeproj` 4.0.1**

Smoke test passed: `bazel build //apps/ios:NotesXcodeProj` succeeded on first valid attempt with Bazel 9.1.0 + Bzlmod. rules_xcodeproj generates the Xcode project from the Bazel graph; Xcode is a viewer, not the source of truth.

| Option | Result |
|--------|--------|
| Plan A (`rules_xcodeproj` 4.0.1) | ✅ Passed — `NotesXcodeProj-runner.sh` emitted, 11 actions, 3s |
| Plan B (`sh_binary` wrapping `xcodebuild`) | Not needed |

**Rationale:** Single-screen SwiftUI app is well within rules_xcodeproj scope. Known issues (Xcode version sync, scheme generation) have documented workarounds. Plan B remains available if Phase 7 reveals blockers — keeping MODULE.bazel deps, just swapping the target rule (~1hr cost).

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
