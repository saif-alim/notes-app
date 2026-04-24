import XCTest
import NotesSchema
import NotesLib

@MainActor
final class NotesViewModelTests: XCTestCase {

    // MARK: load()

    func test_load_fromIdle_transitionsLoadingThenLoaded() async {
        let api = FakeNotesAPI()
        api.stubbedNotes = [Note(id: "1", body: "hello", createdAtMs: 1000)]
        let vm = NotesViewModel(api: api)

        XCTAssertEqual(vm.state, .idle)
        await vm.load()
        XCTAssertEqual(vm.state, .loaded([Note(id: "1", body: "hello", createdAtMs: 1000)]))
    }

    func test_load_onError_setsErrorState_whenNoCache() async {
        let api = FailingNotesAPI(.transport("offline"))
        let vm = NotesViewModel(api: api)

        await vm.load()
        XCTAssertEqual(vm.state, .error(.transport("offline")))
        XCTAssertNil(vm.lastLoadError)
    }

    func test_load_onError_keepsCache_andSurfacesLastLoadError() async {
        let api = FakeNotesAPI()
        let cached = [Note(id: "1", body: "cached", createdAtMs: 0)]
        api.stubbedNotes = cached
        let vm = NotesViewModel(api: api)
        await vm.load()
        XCTAssertEqual(vm.state, .loaded(cached))

        let switchingAPI = SucceedOnceThenFailNotesAPI(first: cached, thenFail: .transport("offline"))
        let vm3 = NotesViewModel(api: switchingAPI)
        await vm3.load()
        XCTAssertEqual(vm3.state, .loaded(cached))

        await vm3.load() // second call fails — stale-while-revalidate
        XCTAssertEqual(vm3.state, .loaded(cached)) // still showing cached
        XCTAssertEqual(vm3.lastLoadError, .transport("offline"))
    }

    // MARK: create()

    func test_create_onSuccess_clearsLastCreateError() async {
        let api = FakeNotesAPI()
        api.stubbedNotes = [Note(id: "1", body: "hi", createdAtMs: 0)]
        let vm = NotesViewModel(api: api)
        vm.lastCreateError = .transport("stale")

        await vm.create(body: "hi")
        XCTAssertNil(vm.lastCreateError)
        XCTAssertEqual(api.createCallCount, 1)
    }

    func test_create_onError_preservesLastCreateError() async {
        let api = FakeNotesAPI()
        api.stubbedCreateResult = .failure(APIError.transport("no network"))
        let vm = NotesViewModel(api: api)

        await vm.create(body: "x")
        XCTAssertEqual(vm.lastCreateError, .transport("no network"))
    }

    func test_create_emptyBody_doesNotCallAPI() async {
        let api = FakeNotesAPI()
        let vm = NotesViewModel(api: api)

        await vm.create(body: "   ")
        XCTAssertEqual(api.createCallCount, 0)
    }

    // MARK: Non-APIError catch arms

    func test_load_nonAPIError_wrapsIntoTransport() async {
        let api = GenericErrorNotesAPI()
        let vm = NotesViewModel(api: api)

        await vm.load()
        // Non-APIError throws are wrapped into .transport
        if case .error(.transport) = vm.state {
            // Expected
        } else {
            XCTFail("Expected .error(.transport), got \(vm.state)")
        }
    }

    func test_create_nonAPIError_wrapsIntoTransport() async {
        let api = GenericErrorNotesAPI()
        let vm = NotesViewModel(api: api)

        await vm.create(body: "test")
        // Non-APIError throws are wrapped into .transport
        if case .transport = vm.lastCreateError {
            // Expected
        } else {
            XCTFail("Expected .transport, got \(String(describing: vm.lastCreateError))")
        }
    }
}
