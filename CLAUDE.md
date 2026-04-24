# notes-app — Navigation

**One-page map. See per-area CLAUDE.mds for local details.**

## What is this?

Notes app: SwiftUI iOS client + Rust/Axum backend + shared .proto schema, built with Bazel (Bzlmod).

**Guiding principle:** Foundation > features. Clean vertical slice beats half-baked breadth.

## Where is X?

| Component | Path | Comes up in |
|-----------|------|------------|
| iOS app | `apps/ios/` | Phase 7+ |
| Rust backend | `services/api/` | Phase 5+ |
| Proto schema | `libs/schema/` | Phase 4 |
| Platform core (shared Rust) | `libs/platform-core/` | Phase 11 |
| Build config | `MODULE.bazel`, `.bazelrc`, `.bazelignore` | Phase 1+ |
| Docs | `docs/` | Phases 1–12 |
| Agent swarm | `agents/` → `.claude/agents/` (Phase 2) | Phase 2+ |
| Benchmarks | `tools/bench/` | Phase 6+ |

## How to run

```bash
bazel build //...               # build all
bazel run //services/api:notes_api   # start backend on :3000 (keep running)
bazel test //...                # Rust tests (platform-core + integration)
bazel test //apps/ios:NotesTests     # iOS XCTest suite (manual tag — explicit invoke)

# iOS: ./tools/run-ios-sim.sh   (builds via Bazel, installs on simulator)
# Backend must already be running on :3000 — the script does not start it.
```

## How to add

| Task | See |
|------|-----|
| New backend endpoint | `docs/recipes/add-endpoint.md` |
| New iOS screen | `docs/recipes/add-ios-screen.md` |
| New proto message | `libs/schema/CLAUDE.md` (Phase 4) |

## Per-area navigation

- `services/api/CLAUDE.md` — routes, handlers, NotesStore, test commands
- `apps/ios/CLAUDE.md` — views, viewmodels, networking, file layout
- `libs/schema/CLAUDE.md` — how to add a message, codegen wiring

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
| 11 | **Bonus:** `libs/platform-core` + UniFFI ✓ on main; Android Gradle scaffold on `phase-11` (NDK install gap) |
| 12 | Final verify: fresh-clone smoke test, git history review ✓ |

Full details: [PLAN.md](PLAN.md).

## Key files

- `PLAN.md` — full engineering plan (decisions, scope, phases, agents)
- `.bazelrc` — common Bazel flags
- `MODULE.bazel` — Bazel dependencies (Bzlmod)

## Known limitations

- No persistence — in-memory `NotesStore` only (trait designed for SQLite/Postgres swap)
- Simulator only — no device provisioning
- Plaintext HTTP to localhost; ATS exception active, no TLS
- Single user, no auth

## Questions?

- How do I add a route? → `docs/recipes/add-endpoint.md` or `services/api/CLAUDE.md`
- How do I add a screen? → `docs/recipes/add-ios-screen.md` or `apps/ios/CLAUDE.md`
- What's the architecture? → `docs/architecture.md`
- How do I run tests? → `README.md` or phase-specific CLAUDE.md
