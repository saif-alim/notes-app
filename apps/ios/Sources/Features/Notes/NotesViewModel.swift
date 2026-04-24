import Foundation
import Observation
import NotesSchema

@MainActor
@Observable
public final class NotesViewModel {
    public enum State: Equatable {
        case idle
        case loaded([Note])
    }

    public private(set) var state: State = .idle
    private let api: APIClient

    public init(api: APIClient) {
        self.api = api
    }

    public func load() async {
        do {
            let notes = try await api.listNotes()
            state = .loaded(notes)
        } catch {
            // Phase 8 adds .error state + user-facing surface.
        }
    }

    public func create(body: String) async {
        let trimmed = body.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else { return }
        do {
            _ = try await api.createNote(body: trimmed)
            await load()
        } catch {
            // Phase 8 adds .error state + user-facing surface.
        }
    }
}
