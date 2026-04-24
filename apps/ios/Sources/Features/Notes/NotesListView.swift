import SwiftUI
import NotesSchema

public struct NotesListView: View {
    @State private var viewModel: NotesViewModel
    @State private var draft: String = ""
    @FocusState private var fieldFocused: Bool

    public init(viewModel: NotesViewModel) {
        _viewModel = State(wrappedValue: viewModel)
    }

    public var body: some View {
        NavigationStack {
            List {
                Section {
                    HStack {
                        TextField("New note", text: $draft)
                            .textFieldStyle(.roundedBorder)
                            .submitLabel(.send)
                            .focused($fieldFocused)
                            .onSubmit(submit)
                        Button("Add", action: submit)
                            .disabled(draft.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
                            .accessibilityLabel("Add note")
                    }
                }
                Section("Notes") {
                    notesContent
                }
            }
            .navigationTitle("Notes")
            .task { await viewModel.load() }
            .refreshable { await viewModel.load() }
            .alert(
                "Couldn't save note",
                isPresented: Binding(
                    get: { viewModel.lastCreateError != nil },
                    set: { if !$0 { viewModel.lastCreateError = nil } }
                ),
                presenting: viewModel.lastCreateError
            ) { _ in
                Button("Retry") { submit() }
                Button("Dismiss", role: .cancel) { viewModel.lastCreateError = nil }
            } message: { error in
                Text(error.userMessage)
            }
            .alert(
                "Couldn't refresh notes",
                isPresented: Binding(
                    get: { viewModel.lastLoadError != nil },
                    set: { if !$0 { viewModel.lastLoadError = nil } }
                ),
                presenting: viewModel.lastLoadError
            ) { _ in
                Button("Retry") { Task { await viewModel.load() } }
                Button("Dismiss", role: .cancel) { viewModel.lastLoadError = nil }
            } message: { error in
                Text(error.userMessage)
            }
        }
    }

    @ViewBuilder
    private var notesContent: some View {
        switch viewModel.state {
        case .idle:
            EmptyView()
        case .loading:
            ProgressView("Loading…")
        case .loaded(let notes) where notes.isEmpty:
            Text("No notes yet").foregroundStyle(.secondary)
        case .loaded(let notes):
            ForEach(notes, id: \.id) { note in
                NoteRow(note: note)
            }
        case .error(let error):
            VStack(alignment: .leading, spacing: 8) {
                Text(error.userMessage).foregroundStyle(.secondary)
                Button("Retry") { Task { await viewModel.load() } }
            }
        }
    }

    private func submit() {
        let body = draft
        fieldFocused = false
        Task {
            await viewModel.create(body: body)
            if viewModel.lastCreateError == nil {
                draft = ""
            }
        }
    }
}

private struct NoteRow: View {
    let note: Note

    private static let formatter: RelativeDateTimeFormatter = {
        let f = RelativeDateTimeFormatter()
        f.unitsStyle = .short
        return f
    }()

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(note.body)
                .lineLimit(3)
                .truncationMode(.tail)
            let relativeDate = Self.formatter.localizedString(
                for: Date(timeIntervalSince1970: Double(note.createdAtMs) / 1000),
                relativeTo: Date()
            )
            Text(relativeDate)
                .font(.caption)
                .foregroundStyle(.secondary)
                .accessibilityLabel("Created \(relativeDate)")
        }
        .accessibilityElement(children: .combine)
    }
}
