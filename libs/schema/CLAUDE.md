# libs/schema — Shared Proto Schema

**Single source of truth for types crossing the iOS ↔ Rust boundary.** Edit
`notes.proto`, regenerate Rust, hand-update Swift, test.

## Layout

```
libs/schema/
├── BUILD.bazel                          # proto_library + rust_prost_library + swift_library
├── notes.proto                          # ← single source of truth
├── Sources/NotesSchema/
│   └── Notes.swift                      # hand-written Codables mirroring notes.proto
└── CLAUDE.md                            # this file
```

## Generated targets

| Target | What | Consumed by |
|--------|------|-------------|
| `//libs/schema:notes_proto` | `proto_library` — canonical descriptor | other proto_library's |
| `//libs/schema:notes_rust_proto` | `rust_prost_library` — generates `pub mod notes::v1 { Note, ListNotesResponse, CreateNoteRequest, CreateNoteResponse }` | `services/api` (Phase 5) |
| `//libs/schema:notes_swift_proto` | `swift_library` exposing `NotesSchema` module with `Note`, `ListNotesResponse`, `CreateNoteRequest`, `CreateNoteResponse` (Codable, snake_case JSON) | `apps/ios` (Phase 7) |

## How to add a new message

1. **Edit `notes.proto`** — add the message under `package notes.v1;` with numbered fields.
2. **Rebuild Rust** — `bazel build //libs/schema:notes_rust_proto`. Prost regenerates automatically; new struct appears under `notes::v1::`.
3. **Mirror in Swift** — add a `public struct MyNewMessage: Codable` to `Sources/NotesSchema/Notes.swift`. Match field names **exactly** (snake_case → camelCase via `CodingKeys`). Example:
   ```swift
   public struct Foo: Codable, Equatable {
       public let myField: String
       enum CodingKeys: String, CodingKey {
           case myField = "my_field"
       }
   }
   ```
4. **Rebuild Swift** — `bazel build //libs/schema:notes_swift_proto`.
5. **Commit both `notes.proto` and the Swift mirror in the same commit** — keeps the two in lockstep.

## Why hand-written Swift?

`rules_proto_grpc_swift` 5.6.0 (and its `rules_swift_package_manager` dep) use the old `CcInfo` symbol that was removed in Bazel 9.x. Overriding transitively broke further. Rather than downgrade Bazel or fight a tooling rabbit hole, Swift mirrors the proto by hand — acceptable for one `.proto` file with four messages. Reassess if schema grows past ~10 messages.

See `docs/retrospective.md` Phase 4 entry for full context.

## Gotchas

- Field names use `snake_case` in proto (e.g. `created_at_unix`) to match prost's default and the JSON wire format. Swift uses `camelCase` + `CodingKeys` to map back.
- The Rust struct has `pub note: Option<Note>` for `CreateNoteResponse.note` because proto3 message-typed fields are optional. Swift mirrors this as `public let note: Note` (non-optional) because the server always populates it — if that changes, make Swift match.
- `i64` in prost ↔ `Int64` in Swift. JSON numbers > 2^53 won't round-trip through JavaScript intermediaries, but there aren't any in this stack.
