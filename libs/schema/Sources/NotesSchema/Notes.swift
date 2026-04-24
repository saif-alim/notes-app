import Foundation

// Mirror of notes.proto (notes.v1). Kept in sync by hand — see
// libs/schema/CLAUDE.md. Uses Codable with snake_case CodingKeys so JSON
// wire format matches the Rust prost-generated structs (which serialize
// with the same field names via pbjson-compatible naming).

public struct Note: Codable, Equatable, Hashable {
    public let id: String
    public let body: String
    public let createdAtMs: Int64

    public init(id: String, body: String, createdAtMs: Int64) {
        self.id = id
        self.body = body
        self.createdAtMs = createdAtMs
    }

    enum CodingKeys: String, CodingKey {
        case id
        case body
        case createdAtMs = "created_at_ms"
    }
}

public struct ListNotesResponse: Codable, Equatable {
    public let notes: [Note]

    public init(notes: [Note]) {
        self.notes = notes
    }
}

public struct CreateNoteRequest: Codable, Equatable {
    public let body: String

    public init(body: String) {
        self.body = body
    }
}

public struct CreateNoteResponse: Codable, Equatable {
    public let note: Note

    public init(note: Note) {
        self.note = note
    }
}
