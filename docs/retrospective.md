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

### Phase 9 (Docs pass)

- **Audit method:** three Explore agents in parallel (one per doc — README, architecture, test-plan) against current code state. Findings triaged into punch list, then edits applied from retrospective as single source of truth.
- **README.md:** added `NOTES_API_BASE_URL` Configuration section (Phase 8 injection point was undocumented), promoted `bazel test //apps/ios:NotesTests` from glob to explicit label, added `bash tools/bench/bench.sh` under Test Commands. Quickstart labels already matched `BUILD.bazel` — no breakage.
- **docs/architecture.md:** deleted stale "component diagram TODO" checklist (diagram exists since Phase 7). Added **Middleware Stack** subsection with the 4-layer order (`TraceLayer → RequestBodyLimitLayer(64KB) → HandleErrorLayer → ConcurrencyLimitLayer(100) → TimeoutLayer(5s)`) + why-each-sits-where. Inlined bench numbers from Phase 6 (`~0.2ms p50 GET, ~0.3ms p50 POST on M4`) instead of just linking `tools/bench/`. Extended Tradeoffs table with three rows the retro had but architecture.md didn't: DTOs vs pbjson, sync vs `async_trait` `NotesStore`, hand-Codables vs `rules_proto_grpc_swift`. Clarified Phase 8 cache = `[Note]` inside `.loaded(notes)` (no TTL, no separate struct); `lastLoadError`/`lastCreateError` orthogonal to State so failures never clobber the list.
- **docs/test-plan.md:** fixed wrong path (`integration_test.rs` → `notes_integration.rs`). Reframed Rust unit-test section — no `#[cfg(test)]` modules exist in `src/`; store/error/DTO logic is exercised end-to-end through the 24 integration tests via `tower::ServiceExt::oneshot`. Expanded Backend integration coverage list (round-trip, validation, Content-Type/parsing, Unicode, 413/404/405, concurrent writes). Inlined bench baseline with a "deviations >10× warrant investigation" rule-of-thumb. Clarified iOS test layout: two `.swift` files + `TestDoubles.swift` support, three `XCTestCase` classes. Dropped stale TODO block (all items were already checked).
- **CLAUDE.md (root):** replaced "No loading/error states yet (Phase 8)" + "No caching (Phase 8)" with the honest current limits (no persistence, no TLS, no auth, simulator-only). Checked off phases 1–8 in build-phases table; marked Phase 9 as current.
- **What worked:** punch-list-from-swarm cadence. Three Explore agents ran in parallel in ~30s and surfaced every stale line + path. Pure doc edits, no code risk.
- **Scope boundary respected:** did not touch retro's `## What Worked` / `## What to Change` finalize sections — those are Phase 10 per PLAN.md:49.
- **Deferred:** Phase 10 is the coherence pass on this retrospective itself (merge duplicates, collapse reviewer-swarm sub-entries, write final narrative). Phase 11 (bonus) and Phase 12 (fresh-clone verify + `/unleash`) remain.

### Phase 11 (Bonus: platform-core + Android + UniFFI)

**Scope delivered:**
- `libs/platform-core` — proto-free Rust crate. Extracted `NotesStore` trait + `InMemoryNotesStore`, `Note` model, `id`/`time`/`validate` helpers from `services/api/src/store.rs`. `services/api` re-exports from platform-core; 24 integration tests pass unmodified.
- Cargo workspace root added (`Cargo.toml`, `Cargo.lock`) — `MODULE.bazel` now resolves `crate_universe` from workspace-level manifest, picking up both `services/api` and `libs/platform-core`.
- UniFFI `ffi` feature: `NotesCore` (`uniffi::Object`) wrapping `InMemoryNotesStore`; `Note` gets `uniffi::Record`. 2 host unit tests (create-and-list, ordering). Feature-gated so Bazel `rust_library` compiles without uniffi.
- Kotlin bindings generated via `tools/uniffi-bindgen/` (standalone crate, not in workspace). Committed to `apps/android/app/src/main/java/uniffi/platform_core/platform_core.kt`.
- Android Gradle project scaffold: `settings.gradle.kts`, `build.gradle.kts`, `libs.versions.toml`, `AndroidManifest.xml`, `MainActivity.kt`, `NotesViewModel.kt` (wraps `NotesCore`), `NotesListScreen.kt` (Jetpack Compose list + inline compose row).
- `tools/build-android.sh` — cargo-ndk cross-compile script (arm64-v8a + x86_64). Requires NDK + cargo-ndk.
- `apps/android/CLAUDE.md` — file layout, run command, FFI update recipe.

**Key decisions:**

- **proto-free platform-core.** `platform_core::Note` mirrors `notes::v1::Note` (same 3 fields). Not coupled to prost — keeps Android `.so` small, avoids JNA-over-prost friction. Same tradeoff as Swift hand-Codables (Phase 4). `services/api/src/dto.rs` swaps `From<notes_proto::Note>` to `From<platform_core::Note>`.

- **UniFFI `ffi` feature gate.** `uniffi::setup_scaffolding!()` and `#[derive(uniffi::Record)]` proc-macros read `Cargo.toml` via file I/O at compile time — Bazel's hermetic sandbox blocks this. Feature-gating isolates the FFI surface from the Bazel build; the Android `.so` is built with `cargo build --features ffi`. Same class of Bazel-ecosystem friction as Phase 4's `rules_proto_grpc_swift` → `CcInfo` breakage.

- **`uniffi_testing` isolation.** The `cli` feature of `uniffi` pulls in `uniffi_testing`, which uses `env!("CARGO")` — also fails in Bazel. Bindgen binary moved to `tools/uniffi-bindgen/` (own `[workspace]`, not in main workspace) so it never pollutes the `crate_universe` dep graph.

- **cargo-ndk over Bazel-native Android.** No `rules_uniffi` exists; `rules_android` + JNI + Compose is underdocumented in Bazel; NDK cross-compile via `cargo-ndk` is the industry pattern (used by Firefox application-services). Gradle owns the Android packaging; Bazel owns the Rust. Mirrors Phase 3 (`bazel build` + simctl) and Phase 4 (hand-Codables) scope-cut precedent.

- **Android in-process (no HTTP to Axum).** Android calls `InMemoryNotesStore` via UniFFI directly. iOS already proves the REST wire. Two orthogonal portability stories, zero duplication of network stack setup.

- **NDK not pre-installed.** Scope-cut per plan: `tools/build-android.sh` is the deliverable. Android project scaffolding + committed Kotlin bindings show the architecture. Instrumented test deferred to machine-with-NDK.

**What worked:**
- `cargo test -p platform_core --features ffi` — both FFI tests pass on the host (macOS dylib). 
- Feature gate is clean: Bazel `rust_library` builds without uniffi dep; cargo with `--features ffi` activates the whole FFI surface. No wrapper crate, no proc-macro workarounds.
- `tools/uniffi-bindgen/` standalone workspace: `cargo run --manifest-path tools/uniffi-bindgen/Cargo.toml -- generate` works without poisoning the main workspace.
- Platform-core extraction was zero-behavior-change: 24 backend integration tests pass with no modifications.

## TODO: Fill in as phases progress

- [x] **Phase 3 (Bazel bootstrap):** Done.
- [x] **Phase 4 (Shared schema):** Done.
- [x] **Phase 5 + 5.5 (Backend minimal + reviewer response):** Done.
- [x] **Phase 6 (Backend hardening):** tower middleware tuning, tracing setup, bench insights. Reviewer swarm fixes applied (HandleErrorLayer, RequestBodyLimitLayer, lock scope).
- [x] **Phase 7 (iOS minimal):** SwiftUI state management (`@Observable` + `@MainActor`), APIClient pattern (actor), codegen integration (`import NotesSchema`). Reviewer swarm pending.
- [x] **Phase 8 + 8.5 (iOS polish + reviewer response):** Cache design, error state UX. Swarm (naive-tester + junior-dev + perf-engineer) findings: `SwitchingNotesAPI` renamed + moved to TestDoubles, error-routing policy comment added to load(), @MainActor on formatter, @unchecked Sendable + nonisolated(unsafe) comments, draft-preserve comment.
- [x] **Phase 9 (Docs pass):** README adds Configuration (`NOTES_API_BASE_URL`), explicit `NotesTests` label, bench command. architecture.md adds Middleware Stack section (4-layer order + rationale), inline bench numbers (~0.2ms p50 GET, ~0.3ms p50 POST), 3 tradeoff rows (DTOs vs pbjson, sync trait, Swift hand-Codables), cache-strategy clarification; TODO block removed. test-plan.md corrects integration path (`notes_integration.rs`), reframes Rust unit coverage as integration-only (24 cases), documents `tower::ServiceExt::oneshot` pattern, inlines bench baseline, clarifies iOS test file layout, drops stale TODO. Root `CLAUDE.md` replaces Phase-8 stale "no cache/no error states" with honest limits (no persistence, no TLS, no auth) + checks off phases 1–8.
- [x] **Phase 11 (Bonus):** platform-core extraction, UniFFI feature gate, Android scaffold, cargo-ndk build script, Kotlin bindings committed. NDK absent → instrumented test deferred.
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
- [x] tower middleware: concurrency limits, graceful shutdown (Phase 6). Built 4-layer stack (TraceLayer → RequestBodyLimitLayer → HandleErrorLayer → ConcurrencyLimitLayer → TimeoutLayer); order critical for error handling + observability. Bench validated p50/p99 latency under load.
- [x] UniFFI proc-macros in Bazel sandbox: `setup_scaffolding!()` + `#[derive(uniffi::Record)]` read `Cargo.toml` via file I/O — blocked by Bazel sandbox. Feature-gated to cargo-only path (Phase 11). Pattern applies to any proc-macro that does file I/O outside declared srcs.
- [x] Cargo workspace + crate_universe: migrated from single-manifest to workspace root Cargo.toml; `MODULE.bazel` crate.from_cargo updated to use `//:Cargo.lock`. multi-crate path dep resolution works cleanly.
- [x] UniFFI `uniffi_testing` / `cli` feature isolation: `uniffi_testing` crate uses `env!("CARGO")` (Cargo binary path at compile time) — fails in Bazel. Moved bindgen binary to standalone workspace in `tools/uniffi-bindgen/` to avoid dep graph pollution.

## What Worked

- **`@Observable` + `@MainActor` iOS state pattern.** Automatic main-thread hop on mutations + SwiftUI observation is ergonomic and catches threading bugs at compile time. 4-case State enum with stale-while-revalidate (keep cached `.loaded` on refresh failure, surface error via `lastLoadError` alert) was the cleanest invariant to express and test.
- **DTO wire-format layer.** Decoupling serde DTOs from proto schema lets the wire format evolve independently. Hand-written DTOs for 4 messages (~50 lines) cheaper than fighting `pbjson` custom aspects. Symmetric with Swift hand-Codables — same tradeoff accepted on both sides.
- **NotesStore trait behind Arc.** Sync trait (no `async-trait`) for in-memory ops avoids `Pin<Box<Fut>>` complexity at trait boundary. Swap to native `async fn in traits` (Rust 1.75+) when SQLx lands — straightforward upgrade, zero caller changes.
- **Tower middleware composition.** Four-layer stack (TraceLayer outermost, TimeoutLayer innermost) is self-documenting. Order matters (trace sees all errors, timeout measures handler work not queueing); the structure forces clarity. ServiceBuilder API makes the composition obvious vs ad-hoc wrapping.
- **Integration test over unit test.** 24 tests via `tower::ServiceExt::oneshot` hit real handlers + store without a socket. Covers validation, routing, concurrency, Unicode, size limits in one suite. No separate unit-test target needed; every code path reachable from HTTP is exercised.
- **Reviewer swarm cadence (Phases 5.5, 7.5, 8.5).** 3-agent triage (2–3 personas per checkpoint) surfaced unused dead code (Phase 5.5 `From<proto>` impls), accessibility gaps (Phase 7.5 NoteRow), and concurrency bugs (Phase 6 lock-scope). Swarm findings were high-signal; false positives rare.
- **Punch-list audits (Phase 9).** Three Explore agents in parallel surfaced every stale line in docs. Cost: ~30s, zero risk. Triggered test coverage expansion (APIClient `.decoding`, ViewModel non-APIError, NotesListView state, Rust 413 boundary).

## What to Change

- **platform-core hand-mirror also doesn't scale.** `platform_core::Note` mirrors proto exactly. Past ~10 messages maintaining 3 parallel mirrors (proto + Swift Codable + Rust model) is error-prone. A golden-JSON fixture asserting Rust encode → Kotlin decode would catch skew earlier. Long-term: unify via a code-generator wrapper or accept proto in the FFI surface with a prost-JNA bridge.

- **Swift hand-Codables don't scale.** 50 lines for 4 messages is fine; past ~10–15 messages, maintaining a manual mirror alongside proto is error-prone. Revisit swift-protobuf chain (rules_proto_grpc_swift on Bazel <9.0) or invest in a code-generation wrapper that's less fragile than hand-mirroring.
- **`.transport` error path hard to unit-test.** URLSession network failures are context-dependent (timeout config, network state, DNS). Mocking at the URLProtocol level is brittle. Consider: integration tests with a real URLSession against a local stub HTTP server, or a higher-level test double that doesn't require stubs at all.
- **Middleware behavior assertions missing.** 413/503/408 return codes are wired (RequestBodyLimitLayer, ConcurrencyLimitLayer, TimeoutLayer) but not explicitly asserted in tests. Plan Phase 10+ bench upgrade to include failure-path latency (e.g., p99 under concurrency limit) + explicit status-code coverage for limit/timeout branches.
- **ViewInspector snapshot tests valuable for SwiftUI.** NotesListView's 5-case state switch, alert bindings, draft-preservation logic, and NoteRow formatting are untested. Snapshots would catch view regressions without brittle frame/coordinate assertions. Worth the setup cost for Phase 11+ UI work.
- **Golden-file JSON schema round-trip.** Swift Codables mirror proto fields via CodingKeys. If Swift's `created_at_ms` drifts from Rust's snake_case, apps break silently at runtime. A golden JSON fixture asserted from both sides (Rust encode → Swift decode) would catch schema skew.
