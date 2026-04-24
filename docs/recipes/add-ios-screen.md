# Add a New iOS Screen

**Scope:** Add a new SwiftUI view + viewmodel to the iOS app.

## Steps

1. **Create ViewModel in `apps/ios/Sources/Features/<Feature>/`**
   ```swift
   @MainActor
   @Observable
   public final class <Feature>ViewModel {
       public enum State: Equatable {
           case idle, loading, loaded(<Data>), error(APIError)
       }
       public private(set) var state: State = .idle
       private let api: any NotesAPI   // or a feature-specific protocol

       public init(api: any NotesAPI) { self.api = api }

       public func load() async {
           state = .loading
           do {
               let data = try await api.someMethod()
               state = .loaded(data)
           } catch let e as APIError {
               state = .error(e)
           } catch {
               state = .error(.transport(error.localizedDescription))
           }
       }
   }
   ```

2. **Create SwiftUI View in same directory**
   ```swift
   public struct <Feature>View: View {
       @State private var viewModel: <Feature>ViewModel

       public init(viewModel: <Feature>ViewModel) {
           _viewModel = State(wrappedValue: viewModel)
       }

       public var body: some View {
           // switch on viewModel.state for loading/error/loaded
       }
   }
   ```

3. **Wire into `NotesApp.swift`** (or a future router).

4. **Add XCTest in `apps/ios/Tests/`**

   Create a `Fake<Feature>API` in `TestDoubles.swift`:
   ```swift
   final class FakeNotesAPI: NotesAPI, @unchecked Sendable {
       var stubbedNotes: [Note] = []
       func listNotes() async throws -> [Note] { stubbedNotes }
       func createNote(body: String) async throws -> Note { /* stub */ }
   }
   ```

   Write state-transition tests:
   ```swift
   @MainActor
   final class <Feature>ViewModelTests: XCTestCase {
       func test_load_fromIdle_transitionsToLoaded() async {
           let api = FakeNotesAPI()
           let vm = <Feature>ViewModel(api: api)
           await vm.load()
           XCTAssertEqual(vm.state, .loaded(...))
       }
   }
   ```

   Run: `bazel test //apps/ios:NotesTests`

5. **Update `apps/ios/CLAUDE.md`** — add row to Screens table.

See `apps/ios/CLAUDE.md` for detailed patterns (state, networking, a11y, cache semantics).
