import SwiftUI
import NotesSchema

public struct NotesListView: View {
    @State private var viewModel: NotesViewModel
    @State private var draft: String = ""

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
                            .onSubmit(submit)
                        Button("Add", action: submit)
                            .disabled(draft.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
                    }
                }
                Section("Notes") {
                    notesContent
                }
            }
            .navigationTitle("Notes")
            .task { await viewModel.load() }
            .refreshable { await viewModel.load() }
        }
    }

    @ViewBuilder
    private var notesContent: some View {
        switch viewModel.state {
        case .idle:
            Text("Loading…").foregroundStyle(.secondary)
        case .loaded(let notes) where notes.isEmpty:
            Text("No notes yet").foregroundStyle(.secondary)
        case .loaded(let notes):
            ForEach(notes, id: \.id) { note in
                NoteRow(note: note)
            }
        }
    }

    private func submit() {
        let body = draft
        draft = ""
        Task { await viewModel.create(body: body) }
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
            Text(Self.formatter.localizedString(
                for: Date(timeIntervalSince1970: Double(note.createdAtMs) / 1000),
                relativeTo: Date()
            ))
            .font(.caption)
            .foregroundStyle(.secondary)
        }
    }
}
