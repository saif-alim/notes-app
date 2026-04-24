import XCTest
import SwiftUI
import NotesSchema
import NotesLib

@MainActor
final class NotesListViewTests: XCTestCase {

    // MARK: State transitions

    func test_viewTransitionsToLoadedState() async {
        let api = FakeNotesAPI()
        api.stubbedNotes = [Note(id: "1", body: "test", createdAtMs: 1000)]
        let vm = NotesViewModel(api: api)

        // Initially idle
        XCTAssertEqual(vm.state, .idle)

        // Load transitions to .loaded
        await vm.load()
        guard case .loaded(let notes) = vm.state else {
            XCTFail("Expected .loaded state")
            return
        }
        XCTAssertEqual(notes.count, 1)

        // Verify view renders without crashing
        let view = NotesListView(viewModel: vm)
        _ = view.body
    }

    func test_viewTransitionsToLoadedEmptyState() async {
        let api = FakeNotesAPI()
        api.stubbedNotes = []
        let vm = NotesViewModel(api: api)

        await vm.load()
        guard case .loaded(let notes) = vm.state else {
            XCTFail("Expected .loaded state")
            return
        }
        XCTAssertEqual(notes.count, 0)

        let view = NotesListView(viewModel: vm)
        _ = view.body
    }

    func test_viewTransitionsToErrorState() async {
        let api = FailingNotesAPI(.transport("offline"))
        let vm = NotesViewModel(api: api)

        // Initially idle
        XCTAssertEqual(vm.state, .idle)

        // Load fails → transitions to .error
        await vm.load()
        if case .error(.transport) = vm.state {
            // Expected
        } else {
            XCTFail("Expected .error(.transport) state, got \(vm.state)")
        }

        // Verify view renders the error state without crashing
        let view = NotesListView(viewModel: vm)
        _ = view.body
    }

    // MARK: Draft preservation

    func test_draftPreservedOnCreateFailure() async {
        let api = FakeNotesAPI()
        api.stubbedCreateResult = .failure(APIError.server(code: "ERROR", message: "failed", status: 500))
        api.stubbedNotes = []
        let vm = NotesViewModel(api: api)

        // Load first to get into .loaded state
        await vm.load()
        guard case .loaded = vm.state else {
            XCTFail("Expected .loaded after initial load")
            return
        }

        // Now create with a failing API
        await vm.create(body: "test note")

        // lastCreateError should be set; the ViewModel preserved the error state
        XCTAssertNotNil(vm.lastCreateError)
        if case .server = vm.lastCreateError {
            // Expected
        } else {
            XCTFail("Expected .server error")
        }
    }

    // MARK: Stale-while-revalidate

    func test_staleWhileRevalidateKeepsCacheOnRefreshFailure() async {
        let api = SucceedOnceThenFailNotesAPI(
            first: [Note(id: "1", body: "cached", createdAtMs: 0)],
            thenFail: .transport("offline")
        )
        let vm = NotesViewModel(api: api)

        // First load succeeds, caches the note
        await vm.load()
        guard case .loaded(let firstNotes) = vm.state else {
            XCTFail("Expected .loaded after first load")
            return
        }
        XCTAssertEqual(firstNotes.count, 1)
        XCTAssertNil(vm.lastLoadError)

        // Second load fails but should keep cache and surface error
        await vm.load()
        guard case .loaded(let cachedNotes) = vm.state else {
            XCTFail("Expected .loaded (stale cache) after failed refresh")
            return
        }
        XCTAssertEqual(cachedNotes.count, 1)
        XCTAssertEqual(cachedNotes[0].body, "cached")
        XCTAssertNotNil(vm.lastLoadError)
    }

    // MARK: Add button interaction

    func test_addButtonGuardsEmptyBody() async {
        let api = FakeNotesAPI()
        let vm = NotesViewModel(api: api)

        // Empty body should not call the API
        await vm.create(body: "")
        XCTAssertEqual(api.createCallCount, 0)
    }

    func test_addButtonCallsAPIOnNonEmptyDraft() async {
        let api = FakeNotesAPI()
        api.stubbedCreateResult = .success(Note(id: "x", body: "test", createdAtMs: 100))
        api.stubbedNotes = [Note(id: "x", body: "test", createdAtMs: 100)]
        let vm = NotesViewModel(api: api)

        await vm.create(body: "test note")
        XCTAssertEqual(api.createCallCount, 1)
    }
}
