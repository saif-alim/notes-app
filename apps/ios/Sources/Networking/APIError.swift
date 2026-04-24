import Foundation

public enum APIError: Error, Equatable {
    case validation(String)
    case server(code: String, message: String, status: Int)
    case decoding(String)
    case transport(String)
}

struct APIErrorEnvelope: Decodable {
    let error: Body

    struct Body: Decodable {
        let code: String
        let message: String
    }
}
