import XCTest
import NotesSchema
import NotesLib

final class APIClientTests: XCTestCase {

    private let baseURL = URL(string: "http://127.0.0.1:3000")!

    // MARK: URL composition

    func test_resolvesURL_withLeadingSlash() async throws {
        StubURLProtocol.handler = { request in
            XCTAssertEqual(request.url?.absoluteString, "http://127.0.0.1:3000/notes")
            let json = #"{"notes":[]}"#
            return stubResponse(statusCode: 200, body: json, for: request.url!)
        }
        let client = APIClient(baseURL: baseURL, session: makeSession())
        _ = try await client.listNotes()
    }

    // MARK: Successful decode

    func test_listNotes_decodesResponseArray() async throws {
        StubURLProtocol.handler = { request in
            let json = #"{"notes":[{"id":"abc","body":"hi","created_at_ms":1234}]}"#
            return stubResponse(statusCode: 200, body: json, for: request.url!)
        }
        let client = APIClient(baseURL: baseURL, session: makeSession())
        let notes = try await client.listNotes()
        XCTAssertEqual(notes.count, 1)
        XCTAssertEqual(notes[0].body, "hi")
        XCTAssertEqual(notes[0].createdAtMs, 1234)
    }

    func test_createNote_decodesCreatedNote() async throws {
        StubURLProtocol.handler = { request in
            let json = #"{"note":{"id":"z","body":"world","created_at_ms":9}}"#
            return stubResponse(statusCode: 201, body: json, for: request.url!)
        }
        let client = APIClient(baseURL: baseURL, session: makeSession())
        let note = try await client.createNote(body: "world")
        XCTAssertEqual(note.id, "z")
        XCTAssertEqual(note.body, "world")
    }

    // MARK: Server error envelope

    func test_serverErrorEnvelope_decodesIntoAPIError() async throws {
        StubURLProtocol.handler = { request in
            let json = #"{"error":{"code":"VALIDATION_ERROR","message":"body must not be empty"}}"#
            return stubResponse(statusCode: 400, body: json, for: request.url!)
        }
        let client = APIClient(baseURL: baseURL, session: makeSession())
        do {
            _ = try await client.listNotes()
            XCTFail("Expected throw")
        } catch let error as APIError {
            if case .server(let code, let message, let status) = error {
                XCTAssertEqual(code, "VALIDATION_ERROR")
                XCTAssertEqual(message, "body must not be empty")
                XCTAssertEqual(status, 400)
            } else {
                XCTFail("Expected .server, got \(error)")
            }
        }
    }

    func test_unknownServerError_fallsBackToHTTPStatus() async throws {
        StubURLProtocol.handler = { request in
            return stubResponse(statusCode: 500, body: "internal", for: request.url!)
        }
        let client = APIClient(baseURL: baseURL, session: makeSession())
        do {
            _ = try await client.listNotes()
            XCTFail("Expected throw")
        } catch let error as APIError {
            if case .server(let code, _, let status) = error {
                XCTAssertEqual(code, "UNKNOWN")
                XCTAssertEqual(status, 500)
            } else {
                XCTFail("Expected .server, got \(error)")
            }
        }
    }

    // MARK: Decoding errors

    func test_decodingError_onMalformedJSON() async throws {
        StubURLProtocol.handler = { request in
            let malformed = #"{"notes": [malformed json]}"#
            return stubResponse(statusCode: 200, body: malformed, for: request.url!)
        }
        let client = APIClient(baseURL: baseURL, session: makeSession())
        do {
            _ = try await client.listNotes()
            XCTFail("Expected .decoding throw")
        } catch let error as APIError {
            if case .decoding = error {
                // Expected
            } else {
                XCTFail("Expected .decoding, got \(error)")
            }
        }
    }
}

// MARK: - APIError.userMessage

final class APIErrorUserMessageTests: XCTestCase {

    func test_userMessage_mapsAllCases() {
        XCTAssertEqual(APIError.validation("x").userMessage, "Note body can't be empty.")
        XCTAssertEqual(
            APIError.server(code: "C", message: "backend said so", status: 400).userMessage,
            "backend said so"
        )
        XCTAssertEqual(APIError.transport("x").userMessage, "Can't reach the server. Check your connection.")
        XCTAssertEqual(APIError.decoding("x").userMessage, "Received an unexpected response.")
    }
}
