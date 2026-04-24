import Foundation

public enum APIError: Error, Equatable {
    case validation(String)
    case server(code: String, message: String, status: Int)
    case decoding(String)
    case transport(String)

    public var userMessage: String {
        switch self {
        case .validation:
            return "Note body can't be empty."
        case .server(_, let message, _):
            return message
        case .transport:
            return "Can't reach the server. Check your connection."
        case .decoding:
            return "Received an unexpected response."
        }
    }
}

struct APIErrorEnvelope: Decodable {
    let error: Body

    struct Body: Decodable {
        let code: String
        let message: String
    }
}
