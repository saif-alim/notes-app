# Engineering Take-Home — Plan

## Guiding Principle

**Foundation > features.** The rubric rewards thoughtful decisions over completeness. One clean vertical slice that clearly round-trips is worth more than multiple half-baked features.

Docs are **first-class artifacts written alongside code** — not a final pass. Each phase's definition of done includes updating the relevant doc. The retrospective is a living document edited continuously from phase 1.

---

## Repo Layout

```
/
├── MODULE.bazel              # Bzlmod (modern Bazel dep mgmt)
├── .bazelrc
├── README.md
├── CLAUDE.md                 # top-level agent nav doc
├── apps/
│   ├── ios/                  # SwiftUI app + BUILD.bazel
│   └── android/              # (bonus only)
├── services/
│   └── api/                  # Rust backend + BUILD.bazel
├── libs/
│   ├── schema/               # .proto — single source of truth
│   └── platform-core/        # (bonus only) shared Rust core
├── tools/                    # scripts, codegen wrappers
└── docs/
    ├── architecture.md
    ├── test-plan.md
    └── retrospective.md
```

---

## Build Phases

| #   | Phase                                   | Definition of done                                                                                                                                |
| --- | --------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | **Scaffold**                            | Repo skeleton, `MODULE.bazel`, `.bazelrc`, stub `README.md` / `docs/*.md` / root `CLAUDE.md`                                                      |
| 2   | **Agent tooling**                       | context-mode, rtk, codebase-memory-mcp installed; doc-freshness hook wired; `agents/` → `.claude/agents/`; `/context-mode:ctx-doctor` all green   |
| 3   | **Bazel bootstrap + iOS path decision** | `bazel build //...` green on empty graph; `rules_xcodeproj` smoke test → choose Plan A or Plan B; decision recorded in `docs/architecture.md`     |
| 4   | **Shared schema**                       | `libs/schema/notes.proto`, `rules_proto` codegen emitting Rust + Swift; `libs/schema/CLAUDE.md` explains how to add a message                     |
| 5   | **Rust backend — minimal**              | Axum skeleton, `NotesStore` trait, in-memory impl, GET/POST `/notes`, one integration test; `services/api/CLAUDE.md` with routes + how to add one |
| 6   | **Rust backend — hardening**            | `tower` concurrency limits, tracing, timeouts; `oha` bench script in `tools/bench/`; `docs/test-plan.md` updated with bench recipe                |
| 7   | **iOS — minimal**                       | SwiftUI shell, Notes list view, API client, one round-trip to local backend; `apps/ios/CLAUDE.md` with structure + how to add a screen            |
| 8   | **iOS — polish**                        | Loading/error states, simple in-memory cache, one XCTest                                                                                          |
| 9   | **Docs pass**                           | README quickstart verified on fresh clone; `docs/architecture.md` has final diagram; `docs/test-plan.md` complete                                 |
| 10  | **Retrospective finalize**              | Coherence pass on the living retro doc                                                                                                            |
| 11  | **Bonus (conditional)**                 | `libs/platform-core` + `apps/android` with UniFFI bindings — **only if steps 1–10 are fully done**                                                |
| 12  | **Final verification**                  | Fresh-clone smoke test; `bazel build //... && bazel test //...` green; git history coherent                                                       |

### Scope cut checkpoints

- After step 3: if Bazel-iOS is still red → commit to Plan B (xcodebuild wrapper) and move on.
- After step 6: if backend is shaky → skip step 8 polish, go straight to iOS minimal.
- Before step 11: if any of steps 1–10 feels thin → do not start the bonus.

---

## Decisions

| Topic             | Choice                                                                                               | Rationale                                                                                                             |
| ----------------- | ---------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------- |
| Product           | Notes app                                                                                            | 2 endpoints (GET/POST `/notes`), trivial schema, trivial UI; leaves caching/optimistic-updates as crit talking points |
| iOS framework     | SwiftUI + MVVM                                                                                       | Fastest, least boilerplate for a single screen                                                                        |
| Backend framework | Axum + Tokio                                                                                         | Best ergonomics + ecosystem; non-blocking I/O; tower middleware                                                       |
| Storage           | In-memory `RwLock<HashMap>` behind `NotesStore` trait                                                | Zero setup; allows pluggable swap to SQLite/Postgres; crit talking point                                              |
| Schema            | `.proto` → `prost` (Rust) + `swift-protobuf` (iOS)                                                   | Single source of truth; machine-readable for agents                                                                   |
| Bazel-iOS         | Smoke-test `rules_xcodeproj` at hour 2; fall back to `sh_binary` wrapping `xcodebuild` if it resists | "Bazel meaningfully involved" is the bar, not "Bazel exclusively"                                                     |
| Bonus             | Only if core + docs are solid at ~hour 16                                                            | Partial bonus that breaks the core build is a net negative                                                            |

---

## Stack Details

### Rust backend (`services/api/`)

```
src/
├── main.rs
├── routes/
├── store/          # NotesStore trait + in-memory impl
├── models.rs
└── telemetry.rs
tests/              # integration tests hitting the router
```

Low-latency story to document: Tokio async runtime, `tower` middleware (timeouts, concurrency limits), `serde` zero-copy JSON, keep-alive / HTTP/2 via `hyper`, structured logging with `tracing`, p50/p99 via `oha` bench.

### iOS app (`apps/ios/`)

```
Sources/
├── App/                  # @main entry
├── Features/Notes/       # View + ViewModel
├── Networking/           # thin API client
└── Models/               # DTOs (generated or hand-written)
Tests/
```

### Bazel rulesets

- `rules_rust` — solid, well-maintained
- `rules_apple` + `rules_swift` + `rules_xcodeproj` — works, fiddly; have Plan B ready
- `rules_proto` + `rules_buf` — for shared schema codegen
- Use Bzlmod (`MODULE.bazel`) over legacy `WORKSPACE`

---

## Documentation

| File                    | Contents                                                                                                                                                                                     |
| ----------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `README.md`             | What this is; quickstart (`bazel build //...`, `bazel run //services/api`, open Xcode, run simulator); repo map; test commands; known limitations                                            |
| `docs/architecture.md`  | Component diagram (ASCII/mermaid); request path iOS → Axum → store; schema story; tradeoffs (in-memory vs SQLite, REST vs gRPC, Bazel-native vs xcodebuild wrap); perf notes + bench results |
| `docs/test-plan.md`     | Unit tests (Rust via Bazel, Swift via XCTest); integration tests; manual E2E; `oha` load smoke; failure modes                                                                                |
| `docs/retrospective.md` | Key decisions + rationale; what was prioritized; alternatives considered; what worked; what you'd change; unfamiliar territory                                                               |
| `CLAUDE.md` (root)      | One-page nav: where is X, how to run Y, how to add Z; points to per-area CLAUDE.mds                                                                                                          |
| Per-area `CLAUDE.md`    | `apps/ios/`, `services/api/`, `libs/schema/` — local conventions, test commands, file layout, gotchas                                                                                        |
| `docs/conventions.md`   | Naming, error handling, commit style                                                                                                                                                         |
| `docs/recipes/`         | `add-endpoint.md`, `add-ios-screen.md` — step-by-step playbooks                                                                                                                              |

### Documentation freshness rule

**Layer 1 — prose rule in root `CLAUDE.md`:**

> After any non-trivial code change, before declaring the task done, check whether `README.md`, `docs/architecture.md`, `docs/test-plan.md`, `docs/retrospective.md`, and the relevant area `CLAUDE.md` still describe reality. If stale, update them in the same commit. If genuinely unaffected, state so in the commit body.

**Layer 2 — Stop hook (`.claude/settings.json`):**

```json
{
  "hooks": {
    "Stop": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": ".claude/hooks/check-docs-freshness.sh"
          }
        ]
      }
    ]
  }
}
```

`check-docs-freshness.sh` inspects `git diff --name-only HEAD`: if code changed but no `*.md` changed, it emits an advisory reminder. Non-blocking — a hard failure is too noisy when docs genuinely don't need updating.

---

## Agent Tooling

| Tool                    | Role                                                                 | Layer                        |
| ----------------------- | -------------------------------------------------------------------- | ---------------------------- |
| **codebase-memory-mcp** | Indexes repo into a queryable structural graph                       | Navigation                   |
| **context-mode**        | Sandboxes tool outputs to SQLite+FTS5; session continuity across 24h | Input — tool results         |
| **rtk**                 | Shell-command output compression (git, cargo, grep)                  | Input — shell only           |
| **caveman**             | Compresses Claude's output tokens                                    | Output                       |
| **headroom**            | Generic LLM-call input compressor                                    | Input — LLM-level (optional) |

**Safe combo:** rtk (shell) + context-mode (everything else). Add headroom only if context bloat appears — running all three simultaneously risks double-compression artifacts. caveman is orthogonal, install unconditionally.

### Install order

```bash
# 1. Session continuity
# /plugin marketplace add mksglu/context-mode
# /plugin install context-mode@context-mode
# /context-mode:ctx-doctor   ← verify all green

# 2. Shell compression
brew install rtk && rtk init -g    # restart Claude Code after
rtk gain                           # sanity check

# 3. Codebase index (run AFTER repo skeleton exists, ~hour 1)
curl -fsSL https://raw.githubusercontent.com/DeusData/codebase-memory-mcp/main/install.sh | bash
# then: "Index this project"

# 4. Headroom — OPTIONAL, only if context pressure appears
# pip install "headroom-ai[all]" && headroom init claude
```

---

## Reviewer Swarm (`.claude/agents/`)

Move during step 2:

```bash
mkdir -p .claude/agents .claude/commands
mv agents/api-designer.md         .claude/agents/api-designer.md
mv agents/junior-dev-notes.md     .claude/agents/junior-dev.md          # rename: frontmatter name: junior-dev
mv agents/naive-tester.md         .claude/agents/naive-tester.md
mv agents/perf-engineer.md        .claude/agents/perf-engineer.md
mv agents/qa-engineer.md          .claude/agents/qa-engineer.md
mv agents/reuse-auditor.md        .claude/agents/reuse-auditor.md
mv agents/security-reviewer.md    .claude/agents/security-reviewer.md
mv agents/staff-engineer-notes.md .claude/agents/staff-engineer.md      # rename: frontmatter name: staff-engineer
mv agents/user-flow-auditor.md    .claude/agents/user-flow-auditor.md
mv agents/ux-reviewer.md          .claude/agents/ux-reviewer.md
mv agents/unleash.md              .claude/commands/unleash.md
rmdir agents
```

Verify: `ls .claude/agents/ | wc -l` → 10. Test one agent before relying on `/unleash`.

| Agent               | Model   | When                                                          |
| ------------------- | ------- | ------------------------------------------------------------- |
| `api-designer`      | sonnet  | After steps 5, 6 — reviews routes, schema, error model        |
| `staff-engineer`    | opus    | After steps 5, 6, 7 — highest-signal architecture review      |
| `security-reviewer` | opus    | After step 6, before step 12 — Axum input validation, CORS    |
| `perf-engineer`     | sonnet  | After steps 6, 8 — Rust handler hot paths, SwiftUI re-renders |
| `reuse-auditor`     | default | After steps 6, 8 — uses codebase-memory-mcp directly          |
| `qa-engineer`       | sonnet  | After steps 5, 7 — **only writer**; run in a worktree         |
| `ux-reviewer`       | sonnet  | After steps 7, 8 — SwiftUI audit                              |
| `naive-tester`      | sonnet  | After step 8 — non-technical UI perspective                   |
| `user-flow-auditor` | sonnet  | After step 8 — catches dead-ends in the notes flow            |
| `junior-dev`        | sonnet  | After step 8, before step 12 — readability + findability      |
| `/unleash`          | command | Step 12 — one comprehensive final-pass swarm                  |

**Rules:**

- Don't run reviewers during the build — save for end-of-step checkpoints.
- Per checkpoint: run 2–3 relevant personas only.
- `/unleash` fires once at step 12. Treat findings as a triage queue: fix 🔴s, document 🟡s in the retrospective.
- `qa-engineer` writes tests — run with `isolation: "worktree"` so output can be reviewed before merging.
- Leave inert skill references in agent files (Flutter/React refs) — not worth editing 11 files.

---

## Per-Phase Orchestration

| Phase                  | Main thread                                                                                      | Parallel delegates                                                                                                                                                     |
| ---------------------- | ------------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1–2 Scaffold + tooling | Write all stubs; install tooling; relocate agents                                                | 3 Explore agents: `rules_rust`, `rules_proto`, `rules_apple`/`rules_xcodeproj` minimal examples                                                                        |
| 3 Bazel bootstrap      | Apply research findings; index repo via codebase-memory-mcp; run Plan agent on iOS path decision | —                                                                                                                                                                      |
| 4 Schema               | Define `notes.proto`, wire `rules_proto`                                                         | 1 Explore agent: working `prost` + `swift-protobuf` Bazel example                                                                                                      |
| 5 Backend minimal      | Implement Axum service, store, routes                                                            | **Checkpoint review:** `api-designer` + `staff-engineer` + `qa-engineer` (worktree)                                                                                    |
| 6 Backend hardening    | tower limits, tracing, timeouts, bench script                                                    | 1 agent expands `docs/test-plan.md`; **Checkpoint review:** `perf-engineer` + `security-reviewer` + `reuse-auditor`                                                    |
| 7 iOS minimal          | SwiftUI views, view models, API client                                                           | 1 agent wires Swift DTO codegen into Bazel; 1 agent drafts `apps/ios/CLAUDE.md`; **Checkpoint review:** `ux-reviewer` + `user-flow-auditor` + `qa-engineer` (worktree) |
| 8 iOS polish           | Loading/error states, cache, XCTest                                                              | **Checkpoint review:** `naive-tester` + `junior-dev` + `perf-engineer`                                                                                                 |
| 9 Docs pass            | Own the retrospective                                                                            | 3 Explore agents (one per doc): architecture, test-plan, README — each reads current code                                                                              |
| 10 Retro finalize      | Main thread only                                                                                 | —                                                                                                                                                                      |
| 11 Bonus               | —                                                                                                | 1 agent: UniFFI bindings; 1 agent: Android Gradle + Bazel integration                                                                                                  |
| 12 Final verification  | Fresh-clone smoke test; `bazel build //... && bazel test //...`; git history review              | Run `/unleash` once; run Plan agent on "what would an interviewer probe first?"                                                                                        |

**Anti-patterns:**

- Don't delegate the retrospective — it's read as evidence of your thinking.
- Don't run more than 3 parallel agents per question.
- Toggle caveman off when drafting prose docs — terse output in a README undermines the communication rubric.
- Don't run headroom if context-mode is already active — pick one input compressor.

---

## Git Etiquette

- Commit at each meaningful step with imperative present-tense messages (e.g., `add GET /notes endpoint`).
- No single giant "initial commit".
- Tag milestones if helpful (`v0.1-backend`, `v0.2-ios`).

---

## Final Verification Checklist

- [ ] `bazel build //...` from fresh clone succeeds
- [ ] `bazel test //...` passes
- [ ] `bazel run //services/api` starts the server
- [ ] iOS app builds in Xcode, simulator launches, notes round-trip to local backend
- [ ] All four docs present and answer their prompt questions
- [ ] `git log --oneline` tells a coherent build story
