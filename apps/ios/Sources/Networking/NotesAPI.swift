import NotesSchema

public protocol NotesAPI: Sendable {
    func listNotes() async throws -> [Note]
    func createNote(body: String) async throws -> Note
}
