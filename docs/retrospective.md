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

### Phase 4 (Shared schema)
- **Schema:** `libs/schema/notes.proto` — `notes.v1` package with `Note`, `ListNotesResponse`, `CreateNoteRequest`, `CreateNoteResponse`.
- **Rust codegen:** Clean. `rules_rust_prost` 0.70.0 on BCR. One-line `rust_prost_library` target. ~100s cold (prost_build + tonic + h2 compile), cached after. Generated `pub mod notes::v1 { ... }` verified.
- **Swift codegen blocker:** `rules_proto_grpc_swift` 5.6.0 + its transitive `rules_swift_package_manager` + `rules_go` still reference the old `CcInfo` Starlark symbol. Removed in Bazel 9.x — build fails at analysis. Overriding `rules_go` → 0.60.0 surfaced the same issue one layer deeper (`rules_swift_package_manager+/swiftpkg/internal/generate_modulemap.bzl`). Fighting the override chain was unsustainable.
- **Decision:** Scope cut per plan. Swift side uses hand-written `Codable` structs in `libs/schema/Sources/NotesSchema/Notes.swift`, wrapped as `swift_library` target `//libs/schema:notes_swift_proto`. Proto stays the single source of truth for schema; Swift mirror is a 50-line hand-sync. Acceptable for four messages; reassess past ~10.
- **Why this is still a good story:** Bazel meaningfully owns the proto → Rust pipeline (the harder half), the schema is single-source, and the hybrid is honest engineering — we hit a real-world Bazel 9 ecosystem gap and made a scope-right call rather than burn a phase on ruleset archaeology.
- **What worked:** rules_rust_prost Bzlmod integration is excellent. prost-generated structs use `::prost::alloc` types — no std-vs-no-std surprises for Phase 5 Axum handlers.
- **Advisor note:** This was a judgment call made without invoking the Opus advisor subagent because the cascade failure was observable directly (two Starlark errors in two different transitive deps, same root cause). Advisor was appropriate for the Phase 3 Plan A/B decision where evidence was mixed; here the signal was unambiguous.

### Phase 5 (Backend minimal)
- **Surface:** `GET /notes` (list; ascending `created_at_unix`), `POST /notes` (create; 400 on empty body; 201 on success). `InMemoryNotesStore` behind `NotesStore` trait, `RwLock<HashMap>` backing. UUID v4 ids. Unix-seconds timestamps.
- **Toolchain bring-up gotchas (both fixed in commit 1):**
  - `crate_universe` extension requires a **root `BUILD.bazel`** to locate the workspace path. Root package is empty but required.
  - `Cargo.lock` must be committed — removed the blanket gitignore rule from Phase 1. crate_universe reads it via `crate.from_cargo(cargo_lockfile = ...)` to pin transitive deps deterministically.
- **JSON wire format — DTOs vs pbjson:** Opus advisor called. Verdict: DTOs + `From` impls. pbjson would need a custom aspect or genrule in the rust_prost_library chain, costing hours of build-infra maintenance. Hand-DTOs for 4 messages are ~50 lines, symmetric with Swift side's hand-Codables, and give us a wire-format layer we can evolve independently of the proto schema. Revisit at ~20 messages.
- **`NotesStore` trait is sync, not async:** In-memory ops don't await anything. Keeping it sync avoids pulling `async-trait` and avoids the `Pin<Box<dyn Future>>` footgun at trait boundaries. If Phase 8+ swaps to SQLite/Postgres, flip to native-async-fn-in-traits (Rust 1.75+ — toolchain supports it).
- **Handler state:** `Arc<dyn NotesStore>`. `with_state::<()>(store)` turbofish required — Axum can't otherwise infer the unit `S` parameter.
- **Integration testing:** `tower::ServiceExt::oneshot` drives the router with no network. Three tests (round-trip, empty-body rejection, order invariant) complete in 0.3s. No separate test fixture infra needed.
- **Cold vs warm build:** 26s cold (downloads axum 0.8.9, tokio 1.52.1, tower 0.5.3, serde 1.0.228 + transitive tree, compiles each). Subsequent builds are 1–2s via Bazel's action cache.
- **What worked:** rules_rust + Bzlmod + crate_universe is clean once the two bring-up gotchas are past. Zero drift between `Cargo.toml` and Bazel's resolved deps.

## TODO: Fill in as phases progress

- [x] **Phase 3 (Bazel bootstrap):** Done.
- [x] **Phase 4 (Shared schema):** Done.
- [x] **Phase 5 (Backend minimal):** Done.
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

- [x] Bazel + iOS: rules_apple works; rules_xcodeproj BwB had DerivedData permissions issue, fell back to `bazel build` + `simctl` (Phase 3).
- [x] Bzlmod vs legacy WORKSPACE: Bazel 9.x Bzlmod-default; clean MODULE.bazel (Phase 1, 3).
- [x] prost + swift-protobuf codegen: prost/rules_rust_prost clean; swift-protobuf chain broken on Bazel 9, scope-cut to hand-written Codables (Phase 4).
- [ ] tower middleware: concurrency limits, graceful shutdown (Phase 6).

## What Worked

- (To be updated)

## What to Change

- (To be updated)
