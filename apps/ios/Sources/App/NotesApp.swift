import SwiftUI

@main
struct NotesApp: App {
    @State private var viewModel = NotesViewModel(api: APIClient())

    var body: some Scene {
        WindowGroup {
            NotesListView(viewModel: viewModel)
        }
    }
}
