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

### Phase 2 (Agent tooling)
- **Discovery:** codebase-memory-mcp + caveman already installed user-globally; cbm-code-discovery-gate + cbm-session-reminder hooks pre-wired. Only rtk + context-mode were net-new.
- **Decision:** Commit `.claude/agents/`, `.claude/commands/`, `.claude/hooks/`, `.claude/settings.json` — keep only `settings.local.json` and `worktrees/` local. Reviewer gets same toolchain on clone.
- **Decision:** Doc-freshness hook non-blocking (exit 0 always). Hard fail too noisy given many edits legitimately don't need doc updates.
- **Install friction:** rtk via brew one-shot. context-mode via `/plugin marketplace add mksglu/context-mode` + `/plugin install` + restart. Both smooth.
- **Verified:** rtk 0.37.2, 61.3% token savings on 12 commands. codebase-memory-mcp indexed scaffold (45 nodes, 44 edges, `fast` mode — baseline for Phase 3+ growth).
- **What worked:** Splitting the plan into user-driven vs Claude-driven steps — installs that need interactive CLI don't block file moves / hook wiring.
- **What to watch:** `.claude/worktrees/` gitignore rule prevents accidental commit of `/worktree` sandboxes. Re-check in Phase 3 when worktrees used for qa-engineer.

### Phase 3 (Bazel bootstrap)
- **Decision:** Plan A (rules_xcodeproj 4.0.1) confirmed — `bazel build //apps/ios:NotesXcodeProj` passed, project generated successfully.
- **Blocker:** rules_xcodeproj BwB (Build with Bazel) mode creates `bazel-out/` paths inside Xcode's DerivedData with 0555 (read-only) permissions by design. Xcode cannot write Info.plist there. Full BwB fix requires replacing XCBBuildService (non-trivial, disproportionate to scope). `Package.swift` `.executableTarget` also attempted and failed — SPM doesn't produce a valid iOS app bundle (nil bundle ID at runtime).
- **Decision (workaround):** `bazel build //apps/ios:NotesApp` produces a signed `.ipa`; `tools/run-ios-sim.sh` extracts and installs on simulator via `simctl`. Unblocks progress without adding scope. **Future work:** wire a hand-maintained `Notes.xcodeproj` (committed, not Bazel-generated) pointing at `Sources/` for Xcode IDE loop — this is the proper long-term solution for Phase 7+ development ergonomics.
- **What worked:** Bazelisk + Bazel 9.1.0 installed cleanly. BCR module resolution resolved version bumps transparently (apple_support 1.22.1 → 2.2.0, etc.). Opus advisor correctly predicted Plan A viability + 3hr time-box; resolved in ~15min.
- **Tradeoff recorded:** `docs/architecture.md` — iOS build path section.

## TODO: Fill in as phases progress

- [ ] **Phase 3 (Bazel bootstrap):** ✅ Done — see above.
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
