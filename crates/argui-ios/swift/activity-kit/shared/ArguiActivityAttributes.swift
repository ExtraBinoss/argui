import ActivityKit

@available(iOS 16.1, *)
struct ArguiActivityAttributes: ActivityAttributes, Sendable {
    struct ContentState: Codable, Hashable, Sendable {
        var message: String
        var progress: Double
    }

    var title: String
}
