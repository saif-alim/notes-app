import Foundation
import NotesSchema

public actor APIClient {
    private let baseURL: URL
    private let session: URLSession
    private let decoder = JSONDecoder()
    private let encoder = JSONEncoder()

    public init(
        baseURL: URL = URL(string: "http://127.0.0.1:3000")!,
        session: URLSession = .shared
    ) {
        self.baseURL = baseURL
        self.session = session
    }

    public func listNotes() async throws -> [Note] {
        let request = makeRequest(path: "/notes", method: "GET", body: nil)
        let response: ListNotesResponse = try await send(request)
        return response.notes
    }

    public func createNote(body: String) async throws -> Note {
        let payload = CreateNoteRequest(body: body)
        let data = try encoder.encode(payload)
        let request = makeRequest(path: "/notes", method: "POST", body: data)
        let response: CreateNoteResponse = try await send(request)
        return response.note
    }

    private func makeRequest(path: String, method: String, body: Data?) -> URLRequest {
        var request = URLRequest(url: baseURL.appendingPathComponent(path))
        request.httpMethod = method
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.setValue("application/json", forHTTPHeaderField: "Accept")
        request.httpBody = body
        return request
    }

    private func send<T: Decodable>(_ request: URLRequest) async throws -> T {
        let data: Data
        let response: URLResponse
        do {
            (data, response) = try await session.data(for: request)
        } catch {
            throw APIError.transport(error.localizedDescription)
        }
        guard let http = response as? HTTPURLResponse else {
            throw APIError.transport("non-HTTP response")
        }
        guard (200..<300).contains(http.statusCode) else {
            if let envelope = try? decoder.decode(APIErrorEnvelope.self, from: data) {
                throw APIError.server(
                    code: envelope.error.code,
                    message: envelope.error.message,
                    status: http.statusCode
                )
            }
            throw APIError.server(code: "UNKNOWN", message: "HTTP \(http.statusCode)", status: http.statusCode)
        }
        do {
            return try decoder.decode(T.self, from: data)
        } catch {
            throw APIError.decoding(error.localizedDescription)
        }
    }
}
