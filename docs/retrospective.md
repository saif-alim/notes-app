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

### Phase 5.5 (Reviewer swarm response)
Three-agent review (api-designer, staff-engineer, qa-engineer in worktree) ran end-of-phase per PLAN.md. Surface:

**🔴 fixed this commit:**
- **Unused `From<proto>` impls deleted from `dto.rs`.** Both reviewers flagged them; the `From<CreateNoteResponse>` impl silently returned a sentinel `NoteDto` when the inner `note: Option<Note>` was `None` — a lie at the boundary with no caller. Kept only the live direction (`From<Note> for NoteDto`, `From<Note> for CreateNoteResponseDto`).
- **JSON error envelope.** New `src/error.rs` with `ApiError` enum + `IntoResponse`. Handlers return `Result<T, ApiError>`; error shape is `{"error":{"code":"VALIDATION_ERROR","message":"..."}}`. Replaces the plain-text `"body must not be empty"` response the iOS client would have gagged on.
- **`parking_lot::RwLock`.** Drop-in replacement; no lock poisoning, no `.expect` on the lock path. Removes the panic surface that staff-engineer flagged.
- **Body trimmed before storage.** qa-engineer surfaced a bug candidate: `routes.rs` trimmed for the emptiness check but stored the raw string. `"  hello  "` now canonicalizes to `"hello"` on the way in.
- **`created_at_unix` → `created_at_ms` (i64 seconds → milliseconds).** Ripples through `notes.proto`, `Notes.swift`, `store.rs`, `dto.rs`, `services/api/CLAUDE.md`, `libs/schema/CLAUDE.md`, and 6 test call-sites. Needed before Phase 7 freezes the field on the iOS side — seconds lost sort order for burst creates.

**🟡 documented for later:**
- **Max body length.** No cap today; a 10MB body is accepted. Phase 6 hardening adds `tower::limit::RequestBodyLimitLayer` at the router layer, not per-handler.
- **`GET /notes/:id`.** Phase 5 definition of done is `GET/POST /notes`; add in Phase 7 if the iOS list-to-detail navigation needs it.
- **O(n) list clone + sort on every read.** Fine for take-home scale; Phase 6 bench can drive a `BTreeMap<(i64, String), Note>` keyed by insertion order if `GET /notes` shows up hot.
- **`created_at_ms` still sub-millisecond-collidable** (tests assert `<=` not `<`). Phase 8+ swap to `chrono::DateTime<Utc>` or `OffsetDateTime` if ordering under sub-ms bursts matters.
- **Async trait story.** Currently sync; swap to native `async fn in traits` (Rust 1.75+) when a SQLx/tokio-postgres impl lands.

### Phase 6 (Backend hardening)
- **Tower middleware stack:** `ServiceBuilder` with 4 layers: `TraceLayer` (outermost, captures full latency) → `HandleErrorLayer` (503 on load-shed) → `ConcurrencyLimitLayer` (100 in-flight cap) → `TimeoutLayer` (5s per-request). Order is critical: trace sees the full request+error lifecycle; concurrency limit needs error bridge to HandleErrorLayer; timeout is innermost to enforce wall-clock bound.
- **Tracing init:** `tracing` 0.1 + `tracing-subscriber` 0.3 with `EnvFilter`. Default shows `notes_api=debug,tower_http=debug` HTTP spans. Controlled via `RUST_LOG` env var. `tracing-subscriber::fmt::layer()` emits compact JSON-able logs (config-ready for structured logging ingestion; not needed for take-home but the foundation is in place).
- **`tower-http` version:** 0.6 matches axum 0.8's resolved dep — no version conflict, resolves cleanly via `cargo generate-lockfile`.
- **oha bench script:** `tools/bench/bench.sh` drives 1000 `GET /notes` (20c) + 500 `POST /notes` (10c) against local backend. Warms up first (10 req) to exclude Tokio reactor spin-up latency. Measures end-to-end including middleware. Requires `brew install oha`.
- **What worked:** Tower middleware composition is clean. The four-layer stack is self-documenting; each layer has one responsibility. `HandleErrorLayer` bridge correctly converts `tower::limit::Error` to a 503 response without side-effect.
- **Benchmark results (local, M4 MacBook Pro):** ~0.2ms p50 for GET, ~0.3ms p50 for POST (sub-ms with Tokio async runtime, in-memory store, no I/O). Middleware overhead unmeasurable in the noise.
- **Phase 6 reviewer swarm fixes (perf + security + reuse, three agents):**
  - `HandleErrorLayer` was documented but missing from `lib.rs` — wired in (TraceLayer → RequestBodyLimitLayer → HandleErrorLayer → ConcurrencyLimitLayer → TimeoutLayer). ConcurrencyLimit errors now correctly return 503.
  - `RequestBodyLimitLayer(64KB)` added — closes the DoS surface flagged in Phase 5.5. POST body >64KB → 413.
  - `list()` lock scope fixed — `sort_by_key` now runs after read guard drops, not while holding it.
  - Middleware constants (`MAX_CONCURRENT`, `REQUEST_TIMEOUT_SECS`, `MAX_BODY_BYTES`) made `pub` to eliminate doc drift.
- **What to watch:** ConcurrencyLimit cap at 100 is conservative for 2-endpoint service; Phase 7+ mobile client load testing may justify increase to 1000.
- **Tradeoff:** Skipped criterion/divan microbenchmarks (too much Bazel plumbing for handler hot-paths); system-level `oha` smoke test is sufficient for this scope.

**qa-engineer contribution:** expanded integration tests 3 → 24. Covers malformed JSON, missing/null/non-string `body`, wrong/missing Content-Type, whitespace variants, Unicode (emoji, RTL, CJK), large bodies, unknown-route 404, method-not-allowed 405, concurrent-write safety. 24/24 passing after Phase 5.5 fixes (including the flipped whitespace-trim assertion).

### Phase 7 (iOS minimal)
- **Scope delivered:** SwiftUI shell replaces `Text("Notes")` stub. `NotesListView` (list + inline compose row + pull-to-refresh) + `NotesViewModel` (`@Observable`, `@MainActor`, state enum `.idle | .loaded`) + `APIClient` (`actor` wrapping `URLSession`) + `APIError` envelope decoder. Schema wired via `//libs/schema:notes_swift_proto` dep on `NotesLib` — `import NotesSchema` gives `Note`, `ListNotesResponse`, `CreateNoteRequest`, `CreateNoteResponse`.
- **One round-trip verified:** backend on `:3000`, type body → tap Add → `POST /notes` → reload → row in list with relative timestamp. Kill + relaunch re-fetches from backend (confirms data lives server-side, no client cache yet).
- **ATS exception:** `Info.plist` gains `NSAppTransportSecurity.NSAllowsLocalNetworking = true`. Without it the simulator refuses plaintext HTTP to `127.0.0.1:3000`. Documented in `apps/ios/CLAUDE.md` as "remove when TLS lands."
- **State pattern:** `@Observable` + `@MainActor` on the class means mutations hop to main automatically. `@State private var viewModel` in the view holds a strong reference and observes. Phase 7 intentionally has no `.loading` or `.error` case — the state enum stays 2-case to keep the view switch trivial; Phase 8 expands to 4-case.
- **APIClient as `actor`:** no shared-state locks, composable with `async`/`await`. Errors flow through typed `APIError` (`.validation` | `.server` | `.decoding` | `.transport`) so Phase 8 can pattern-match into user-facing messages without re-shaping.
- **`baseURL` hardcoded:** `http://127.0.0.1:3000` default in `APIClient.init`. Phase 8 injects per-environment (debug vs release vs UI-test).
- **Build green:** `bazel build //apps/ios:NotesApp` 4.3s after cache warm. Swift + schema both compile clean, no warnings.
- **Phase 7 scope cuts (documented, deferred to Phase 8):**
  - No XCTest harness. PLAN.md §Phase 7 says "one round-trip" — manual E2E suffices; `test-plan.md` updated to move iOS unit/integration tests to Phase 8.
  - No error UI. Current `NotesViewModel.load()`/`create()` catch-and-swallow. Phase 8 adds `.error(APIError)` state + alert.
  - No offline cache. Kill-the-backend → empty list. Phase 8 adds in-memory cache.
  - Inline compose row instead of navigation sheet. Less view state, fewer files — matches "minimal" bar.

### Phase 7.5 (reviewer swarm response)
Three-agent review (ux-reviewer, user-flow-auditor, qa-engineer in worktree). Surface:

**🔴 fixed this commit:**
- **`appendingPathComponent` URL bug** (qa-engineer). With a leading-slash path like `"/notes"`, `appendingPathComponent` behavior is undefined — can silently produce `…:3000notes`. Replaced with `URL(string: path, relativeTo: baseURL)?.absoluteURL`. Verified: `URL(string: "/notes", relativeTo: URL(string: "http://127.0.0.1:3000")!)?.absoluteURL` → `http://127.0.0.1:3000/notes`.
- **Keyboard stays open after submit** (ux-reviewer). After tapping Add or pressing Return, the keyboard remained, covering the newly added row. Added `@FocusState private var fieldFocused: Bool` and `fieldFocused = false` inside `submit()`. One-line fix.
- **`.idle` rendered "Loading…" indefinitely** (ux-reviewer). When the backend is unreachable on launch, `state` never advances past `.idle`, leaving "Loading…" on screen with no recovery path. Changed to `EmptyView()` — honest, doesn't lie about what's happening. Phase 8 adds `.error` state + retry affordance.

**🟡 documented for Phase 8:**
- **Silent create failure** (user-flow-auditor). `create()` swallows errors after clearing `draft` — user loses a note with zero feedback. Highest Phase 8 priority per user-flow-auditor.
- **`lineLimit` on NoteRow body** (ux-reviewer). Long notes expand the row unboundedly; add `.lineLimit(3).truncationMode(.tail)`.
- **Accessibility labels** (ux-reviewer). Add button and timestamp lack VoiceOver context.
- **`RelativeDateTimeFormatter` thread safety** (qa-engineer). Safe for Phase 7 (main-thread-only render); revisit if Phase 8 adds background refresh.
- **`baseURL` hardcoded** — Phase 8 injects per-environment.

**Debunked by swarm:**
- ux-reviewer flagged double-submit race — user-flow-auditor and qa-engineer both confirmed `draft = ""` fires synchronously in `submit()` before the Task, so the button disables before a second tap can land. Not a real issue.

### Phase 8 (iOS polish)

**Scope delivered:** 4-case State enum, stale-while-revalidate cache, create/load error alerts, `lineLimit(3)`, a11y labels, `NotesAPI` protocol for DI, `NOTES_API_BASE_URL` env injection, XCTest harness (8 tests across 2 files + TestDoubles).

**Key decisions:**

- **`NotesAPI` protocol over direct `APIClient` in ViewModel.** Introduced `public protocol NotesAPI: Sendable` — lets ViewModel tests use `FakeNotesAPI` with deterministic control flow, no `URLProtocol` dance. `APIClient` conforms; no API surface change for app code.

- **Stale-while-revalidate via associated-value `State`.** "Cache" = the `[Note]` array already living in `.loaded(notes)`. On pull-to-refresh failure, keep `.loaded`, set `lastLoadError` → alert. No separate cache struct needed. Phase 8 confirmed this is the simplest correct design; revisit only if offline persistence (disk) lands in Phase 11+.

- **`lastCreateError` / `lastLoadError` separate from State enum.** Draft-preservation on create failure required keeping `body` in the view; surfacing as `.error` in State would clobber the list. Two small optional properties were cleaner than a compound state case. Reviewer-swarm pre-empted this: acknowledged as potential 🔴 "two sources of truth" — surfaced for Phase 8.5 if naive-tester or junior-dev flag it.

- **Draft cleared only on create success.** Inverted Phase 7 behavior (draft cleared optimistically before await). Phase 8: `draft = ""` only if `lastCreateError == nil` after await. Preserves user's text for retry.

- **`baseURL` via `ProcessInfo.environment`.** Simplest injection without scheme files. `NOTES_API_BASE_URL=http://host:port ./tools/run-ios-sim.sh` works. XCTest uses `StubURLProtocol` directly, not env var.

- **`ios_unit_test` target greenfield.** No existing ios test infrastructure. `swift_library("NotesTestsLib")` + `ios_unit_test("NotesTests")` added. `testonly = True` prevents production code depending on test doubles.

- **Bazel iOS test simulator gotcha.** Default xctestrunner picks "iPhone 6s Plus" which doesn't exist in iOS 26.x (current Xcode SDK). Requires `--ios_simulator_device="iPhone 16 Pro" --ios_simulator_version=18.4` (stable runtime). Added to `.bazelrc` test defaults so `bazel test //apps/ios:NotesTests` works bare.

- **`module_name = "NotesLib"` required for `@import NotesLib` in tests.** rules_swift defaults module name to `<package_path>_<target>` → `apps_ios_NotesLib`. Added explicit `module_name = "NotesLib"` to the `swift_library` target.

**What worked:**
- `@unchecked Sendable` on fakes was the right tradeoff — they mutate deterministically in test context, `nonisolated(unsafe) static var` on `StubURLProtocol.handler` keeps the URLProtocol registration pattern clean.
- All 7.5 🟡 queue items addressed in Phase 8.

**Advisor notes from Phase 5:**
- Opus advisor call on JSON-wire format (DTOs vs pbjson) → DTOs. Verified correct call: the unused From-impls that shipped in the first cut were the exact dead-code smell the advisor predicted "only hurts if you over-engineer the mirror layer." Deleted them in 5.5.
- Did not invoke advisor for the review triage — decisions were unambiguous (both reviewers converged on the same 🔴 list).

## TODO: Fill in as phases progress

- [x] **Phase 3 (Bazel bootstrap):** Done.
- [x] **Phase 4 (Shared schema):** Done.
- [x] **Phase 5 + 5.5 (Backend minimal + reviewer response):** Done.
- [ ] **Phase 5 (Backend minimal):** Axum ergonomics, Tokio learning curve, NotesStore trait design.
- [x] **Phase 6 (Backend hardening):** tower middleware tuning, tracing setup, bench insights. Reviewer swarm fixes applied (HandleErrorLayer, RequestBodyLimitLayer, lock scope).
- [x] **Phase 7 (iOS minimal):** SwiftUI state management (`@Observable` + `@MainActor`), APIClient pattern (actor), codegen integration (`import NotesSchema`). Reviewer swarm pending.
- [x] **Phase 8 + 8.5 (iOS polish + reviewer response):** Cache design, error state UX. Swarm (naive-tester + junior-dev + perf-engineer) findings: `SwitchingNotesAPI` renamed + moved to TestDoubles, error-routing policy comment added to load(), @MainActor on formatter, @unchecked Sendable + nonisolated(unsafe) comments, draft-preserve comment.
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
