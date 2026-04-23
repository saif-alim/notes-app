---
name: unleash
description: Spawn full agent swarm (11 specialists) in parallel to review target code. Use /unleash [path|feature|branch] or /unleash for current changes.
---

Unleash agent swarm on: $ARGUMENTS

No target = review all uncommitted changes (`git diff --name-only`).

## Execute

Create agent team. 11 teammates, each using project subagent type:

1. `ux-reviewer` — UI/UX, accessibility, interaction patterns
2. `naive-tester` — "Can my mom use this?" Simplicity, confusion, ease of use
3. `user-flow-auditor` — End-to-end journeys, routes, transitions, guards, edge paths, flow docs
4. `qa-engineer` — Test gaps, edge cases, write failing tests (TDD)
5. `staff-engineer` — Architecture, quality, abstractions, debt
6. `security-reviewer` — Auth, data exposure, injection, permissions
7. `perf-engineer` — Renders, memory, jank, async, network
8. `api-designer` — Schema, queries, data layer, API design
9. `devops-engineer` — CI/CD, builds, deploy configs, infra
10. `junior-dev` — Readability, findability, clarity. Flags cleverness + tribal knowledge
11. `reuse-auditor` — DRY/SSOT, design-system adherence, duplicate/near-duplicate widgets. Recommends reuse-existing vs extract-shared with exact paths.

Each agent:
- Works independently in parallel
- Targets files/feature from $ARGUMENTS
- Reports in their defined output format
- TDD mandate: every issue includes test strategy

## Synthesize

After all 9 report, merge into single prioritized list:

```
## 🔴 Must fix before merge
## 🟡 Should fix (ticket-worthy)
## 🟢 Future improvements
## ✅ Passing (no issues found)
```

Use worktree isolation for write-capable agents (qa-engineer). Read-only agents share context.
