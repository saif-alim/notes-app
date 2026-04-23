# Retrospective — Living Document

**Updated continuously from Phase 1 onward.** Reflects decisions made, tradeoffs, what worked, what didn't, unfamiliar territory tackled.

## Decisions Made (from PLAN.md)

| Decision | Choice | Rationale | Status |
|----------|--------|-----------|--------|
| Product | Notes app | 2 endpoints, trivial schema/UI; caching & optimistic updates as crit talking points | ✓ Locked |
| iOS framework | SwiftUI + MVVM | Fastest, least boilerplate for single screen | ✓ Locked |
| Backend framework | Axum + Tokio | Best ergonomics + ecosystem; non-blocking I/O | ✓ Locked |
| Storage | In-memory `RwLock<HashMap>` behind `NotesStore` trait | Zero setup; allows pluggable swap | ✓ Locked |
| Schema | `.proto` → `prost` (Rust) + `swift-protobuf` (iOS) | Single source of truth; machine-readable | ✓ Locked |
| Bazel-iOS | Smoke-test `rules_xcodeproj`; Plan B: `sh_binary` wrapping `xcodebuild` | "Meaningfully involved" > "exclusively" | ✓ Phase 3 |
| Build tooling | Bazel (Bzlmod) | Unified mono-repo for Rust + Swift + schema codegen | ✓ Locked |

## Key Insights

### Phase 1 (Scaffold)
- **Discovery:** Repo was empty except PLAN.md and agent files.
- **Decision:** Fix agent count typo in PLAN.md (line 208: unleash goes to commands/, not agents/) before Phase 2.
- **Blocker:** None.
- **What worked:** Bazel MODULE.bazel + .bazelrc stubs minimal and clear.

## TODO: Fill in as phases progress

- [ ] **Phase 2 (Agent tooling):** Reflect on MCP install friction, context-mode efficiency.
- [ ] **Phase 3 (Bazel bootstrap):** rules_xcodeproj success/fallback decision.
- [ ] **Phase 5 (Backend minimal):** Axum ergonomics, Tokio learning curve, NotesStore trait design.
- [ ] **Phase 6 (Backend hardening):** tower middleware tuning, tracing setup, bench insights.
- [ ] **Phase 7 (iOS minimal):** SwiftUI state management, APIClient pattern, codegen integration.
- [ ] **Phase 8 (iOS polish):** Cache design, error state UX.
- [ ] **Phase 9 (Docs pass):** README clarity, architecture diagram completeness, test-plan accuracy.
- [ ] **Phase 12 (Final):** What would an interviewer probe first? Unfamiliar territory conquered?

## Alternatives Considered

- **Persistence:** SQLite/Postgres deferred to Phase 4+; in-memory proves trait abstraction works.
- **API style:** REST chosen for simplicity; gRPC deferred.
- **Bazel fallback:** xcodebuild wrapper (Plan B) ready if rules_xcodeproj stalls Phase 3.
- **iOS framework:** UIKit discarded; SwiftUI faster for this scope.

## Unfamiliar Territory

- [ ] Bazel + iOS: rules_apple/rules_xcodeproj learning curve (Phase 3).
- [ ] Bzlmod vs legacy WORKSPACE: Bazel 7.x modern idiom (Phase 1).
- [ ] prost + swift-protobuf codegen: first time wiring cross-language schema (Phase 4).
- [ ] tower middleware: concurrency limits, graceful shutdown (Phase 6).

## What Worked

- (To be updated)

## What to Change

- (To be updated)
