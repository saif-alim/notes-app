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

## Tradeoffs

| Aspect | Choice | Why | Alternative |
|--------|--------|-----|-------------|
| Storage | In-memory RwLock | Zero setup, pluggable trait | SQLite (Phase 4) |
| API | REST JSON | Simple, tools-friendly | gRPC (later phase) |
| Build | Bazel (Bzlmod) | Unified mono-repo, rules_proto | xcodebuild + cargo separate |
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
