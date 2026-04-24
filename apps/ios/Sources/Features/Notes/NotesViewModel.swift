import Foundation
import Observation
import NotesSchema

@MainActor
@Observable
public final class NotesViewModel {
    public enum State: Equatable {
        case idle
        case loading
        case loaded([Note])
        case error(APIError)
    }

    public private(set) var state: State = .idle
    public var lastLoadError: APIError? = nil
    public var lastCreateError: APIError? = nil

    private let api: any NotesAPI

    public init(api: any NotesAPI) {
        self.api = api
    }

    public func load() async {
        switch state {
        case .loaded:
            // Stale-while-revalidate: keep cached rows, surface errors via lastLoadError.
            do {
                let notes = try await api.listNotes()
                state = .loaded(notes)
                lastLoadError = nil
            } catch let error as APIError {
                lastLoadError = error
            } catch {
                lastLoadError = .transport(error.localizedDescription)
            }
        default:
            state = .loading
            do {
                let notes = try await api.listNotes()
                state = .loaded(notes)
                lastLoadError = nil
            } catch let error as APIError {
                state = .error(error)
            } catch {
                state = .error(.transport(error.localizedDescription))
            }
        }
    }

    public func create(body: String) async {
        let trimmed = body.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else { return }
        do {
            _ = try await api.createNote(body: trimmed)
            lastCreateError = nil
            await load()
        } catch let error as APIError {
            lastCreateError = error
        } catch {
            lastCreateError = .transport(error.localizedDescription)
        }
    }
}
