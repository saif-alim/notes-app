# notes-app — Navigation

**One-page map. See per-area CLAUDE.mds for local details.**

## What is this?

Notes app: SwiftUI iOS client + Jetpack Compose Android client + Rust/Axum backend + shared .proto schema + shared `libs/platform-core` Rust crate, built with Bazel (Bzlmod).

**Guiding principle:** Foundation > features. Clean vertical slice beats half-baked breadth.

## Where is X?

| Component | Path | Comes up in |
|-----------|------|------------|
| iOS app | `apps/ios/` | Phase 7+ |
| Android app | `apps/android/` | Phase 11 |
| Shared Rust core | `libs/platform-core/` | Phase 11 |
| Rust backend | `services/api/` | Phase 5+ |
| Proto schema | `libs/schema/` | Phase 4 |
| Build config | `MODULE.bazel`, `.bazelrc` | Phase 1+ |
| Docs | `docs/` | Phases 1–12 |
| Agent swarm | `.claude/agents/` | Phase 2+ |
| Benchmarks | `tools/bench/` | Phase 6+ |
| UniFFI bindgen | `tools/uniffi-bindgen/` | Phase 11 |

## How to run

```bash
bazel build //...                    # build all Bazel targets
bazel run //services/api:notes_api   # start backend on :3000
bazel test //...                     # run all tests

# iOS: ./tools/run-ios-sim.sh        (Bazel build + simctl)
# Android: see apps/android/CLAUDE.md (requires NDK + cargo-ndk)
```

## How to add

| Task | See |
|------|-----|
| New backend endpoint | `docs/recipes/add-endpoint.md` |
| New iOS screen | `docs/recipes/add-ios-screen.md` |
| New proto message | `libs/schema/CLAUDE.md` |
| New platform-core function (FFI) | `libs/platform-core/CLAUDE.md` |

## Per-area navigation

- `services/api/CLAUDE.md` — routes, handlers, NotesStore, test commands
- `apps/ios/CLAUDE.md` — views, viewmodels, networking, file layout
- `apps/android/CLAUDE.md` — Compose UI, UniFFI, NDK build, Gradle
- `libs/schema/CLAUDE.md` — how to add a message, codegen wiring
- `libs/platform-core/CLAUDE.md` — shared core, FFI surface, Bazel/cargo split

## Docs at a glance

| Doc | Purpose |
|-----|---------|
| `README.md` | What this is, quickstart, repo map, tests, limitations |
| `docs/architecture.md` | Component diagram, request path, tradeoffs, perf story |
| `docs/test-plan.md` | Unit / integration / E2E / load test strategy |
| `docs/retrospective.md` | Living log of decisions, what worked, what to change |
| `docs/conventions.md` | Naming, error handling, commit style, testing patterns |
| `docs/recipes/` | Playbooks for common tasks |

## Docs freshness rule

**After any non-trivial code change**, before marking done, check if `README.md`, `docs/architecture.md`, `docs/test-plan.md`, `docs/retrospective.md`, and the relevant area `CLAUDE.md` still describe reality. Update in the same commit if stale. If genuinely unaffected, note so in the commit body.

(Enforced at commit time via stop hook in Phase 2.)

## Build phases

| Phase | Goal |
|-------|------|
| 1 | Scaffold: repo skeleton, Bazel stubs, doc outlines ✓ |
| 2 | Agent tooling: MCP setup, context-mode, hooks ✓ |
| 3 | Bazel bootstrap: rules_rust/apple/xcodeproj, iOS path decision ✓ |
| 4 | Shared schema: `.proto` codegen for Rust + Swift ✓ |
| 5 | Backend minimal: Axum + NotesStore + 2 endpoints ✓ |
| 6 | Backend hardening: tower middleware, tracing, bench ✓ |
| 7 | iOS minimal: SwiftUI, views, viewmodels, round-trip ✓ |
| 8 | iOS polish: error/loading states, cache, XCTest ✓ |
| 9 | Docs pass: README verify, architecture finalize, test-plan complete ✓ |
| 10 | Retro finalize: coherence pass on living doc ✓ |
| 11 | Bonus: platform-core + Android (UniFFI in-process) ← current |
| 12 | Final verify: fresh-clone smoke test, git history review |

Full details: [PLAN.md](PLAN.md).

## Key files

- `PLAN.md` — full engineering plan (decisions, scope, phases, agents)
- `.bazelrc` — common Bazel flags
- `MODULE.bazel` — Bazel dependencies (Bzlmod)
- `Cargo.toml` — workspace root (services/api + libs/platform-core)

## Known limitations

- No persistence — in-memory `NotesStore` only (trait designed for SQLite/Postgres swap)
- Simulator only (iOS) / emulator only (Android) — no device provisioning
- Plaintext HTTP to localhost; ATS exception active, no TLS, no auth
- Android requires NDK + `cargo-ndk` to build the `.so`; `jniLibs/` is gitignored

## Questions?

- How do I add a route? → `docs/recipes/add-endpoint.md` or `services/api/CLAUDE.md`
- How do I add an iOS screen? → `docs/recipes/add-ios-screen.md` or `apps/ios/CLAUDE.md`
- How do I add an Android screen? → `apps/android/CLAUDE.md`
- How do I extend the FFI surface? → `libs/platform-core/CLAUDE.md`
- What's the architecture? → `docs/architecture.md`
- How do I run tests? → `README.md` or phase-specific CLAUDE.md
