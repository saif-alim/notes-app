import Foundation
import NotesSchema
import NotesLib

// MARK: - FakeNotesAPI

// @unchecked Sendable: mutable state is only accessed from @MainActor tests — no real races.
final class FakeNotesAPI: NotesAPI, @unchecked Sendable {
    var stubbedNotes: [Note] = []
    var stubbedCreateResult: Result<Note, Error> = .success(Note(id: "1", body: "ok", createdAtMs: 0))
    var listCallCount = 0
    var createCallCount = 0
    var lastCreatedBody: String?

    func listNotes() async throws -> [Note] {
        listCallCount += 1
        return stubbedNotes
    }

    func createNote(body: String) async throws -> Note {
        createCallCount += 1
        lastCreatedBody = body
        return try stubbedCreateResult.get()
    }
}

// MARK: - FailingNotesAPI

// @unchecked Sendable: immutable after init; safe to call from any context.
final class FailingNotesAPI: NotesAPI, @unchecked Sendable {
    let error: APIError
    init(_ error: APIError) { self.error = error }

    func listNotes() async throws -> [Note] { throw error }
    func createNote(body: String) async throws -> Note { throw error }
}

// MARK: - StubURLProtocol

final class StubURLProtocol: URLProtocol {
    typealias Handler = (URLRequest) -> (Data, HTTPURLResponse)
    // nonisolated(unsafe): URLProtocol callbacks arrive on URLSession's internal thread,
    // not on any actor. Tests run serially so access is safe in practice.
    nonisolated(unsafe) static var handler: Handler?

    override class func canInit(with request: URLRequest) -> Bool { true }
    override class func canonicalRequest(for request: URLRequest) -> URLRequest { request }

    override func startLoading() {
        guard let handler = StubURLProtocol.handler else {
            client?.urlProtocol(self, didFailWithError: URLError(.unknown))
            return
        }
        let (data, response) = handler(request)
        client?.urlProtocol(self, didReceive: response, cacheStoragePolicy: .notAllowed)
        client?.urlProtocol(self, didLoad: data)
        client?.urlProtocolDidFinishLoading(self)
    }

    override func stopLoading() {}
}

// MARK: - SucceedOnceThenFailNotesAPI

// Succeeds on the first listNotes() call, then throws — tests the stale-while-revalidate path.
// @unchecked Sendable: callCount mutated only from @MainActor tests — no real races.
final class SucceedOnceThenFailNotesAPI: NotesAPI, @unchecked Sendable {
    private let firstNotes: [Note]
    private let failError: APIError
    private var callCount = 0

    init(first: [Note], thenFail: APIError) {
        firstNotes = first
        failError = thenFail
    }

    func listNotes() async throws -> [Note] {
        callCount += 1
        if callCount == 1 { return firstNotes }
        throw failError
    }

    func createNote(body: String) async throws -> Note {
        throw failError
    }
}

// MARK: - Helpers

func makeSession() -> URLSession {
    let config = URLSessionConfiguration.ephemeral
    config.protocolClasses = [StubURLProtocol.self]
    return URLSession(configuration: config)
}

func stubResponse(statusCode: Int, body: String, for url: URL) -> (Data, HTTPURLResponse) {
    let data = body.data(using: .utf8)!
    let response = HTTPURLResponse(url: url, statusCode: statusCode, httpVersion: nil, headerFields: nil)!
    return (data, response)
}
