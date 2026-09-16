import ActivityKit
import Foundation

private enum ArguiActivityBridge {
    @MainActor private static var nextIdentifier: UInt64 = 1
    @available(iOS 16.1, *)
    @MainActor private static var activities: [UInt64: Activity<ArguiActivityAttributes>] = [:]

    static func onMain<T: Sendable>(
        _ operation: @escaping @MainActor @Sendable () -> T
    ) -> T {
        if Thread.isMainThread {
            return MainActor.assumeIsolated(operation)
        }
        return DispatchQueue.main.sync {
            MainActor.assumeIsolated(operation)
        }
    }

    @MainActor static func start(title: String, message: String) -> UInt64 {
        guard #available(iOS 16.1, *), ActivityAuthorizationInfo().areActivitiesEnabled else {
            return 0
        }

        let attributes = ArguiActivityAttributes(title: title)
        let state = ArguiActivityAttributes.ContentState(message: message, progress: 0)
        do {
            let activity = try Activity.request(
                attributes: attributes,
                contentState: state,
                pushType: nil
            )
            let identifier = nextIdentifier
            nextIdentifier = nextIdentifier == UInt64.max ? 1 : nextIdentifier + 1
            activities[identifier] = activity
            return identifier
        } catch {
            return 0
        }
    }

    @available(iOS 16.1, *)
    @MainActor static func update(
        identifier: UInt64,
        percent: UInt8,
        message: String
    ) -> Int32 {
        guard let activity = activities[identifier] else {
            return 0
        }
        let state = ArguiActivityAttributes.ContentState(
            message: message,
            progress: Double(min(percent, 100)) / 100
        )
        Task {
            await activity.update(using: state)
        }
        return 1
    }

    @available(iOS 16.1, *)
    @MainActor static func finish(identifier: UInt64) -> Int32 {
        guard let activity = activities.removeValue(forKey: identifier) else {
            return 0
        }
        Task {
            await activity.end(using: nil, dismissalPolicy: .immediate)
        }
        return 1
    }
}

@_cdecl("argui_ios_activity_start")
public func arguiIOSActivityStart(
    _ title: UnsafePointer<CChar>?,
    _ message: UnsafePointer<CChar>?
) -> UInt64 {
    guard let title, let message else {
        return 0
    }
    let titleText = String(cString: title)
    let messageText = String(cString: message)
    return ArguiActivityBridge.onMain {
        ArguiActivityBridge.start(title: titleText, message: messageText)
    }
}

@_cdecl("argui_ios_activity_update")
public func arguiIOSActivityUpdate(
    _ identifier: UInt64,
    _ percent: UInt8,
    _ message: UnsafePointer<CChar>?
) -> Int32 {
    guard let message else {
        return 0
    }
    let messageText = String(cString: message)
    return ArguiActivityBridge.onMain {
        guard #available(iOS 16.1, *) else {
            return 0
        }
        ArguiActivityBridge.update(identifier: identifier, percent: percent, message: messageText)
    }
}

@_cdecl("argui_ios_activity_finish")
public func arguiIOSActivityFinish(_ identifier: UInt64) -> Int32 {
    ArguiActivityBridge.onMain {
        guard #available(iOS 16.1, *) else {
            return 0
        }
        ArguiActivityBridge.finish(identifier: identifier)
    }
}

@_cdecl("argui_ios_activity_bridge_anchor")
public func arguiIOSActivityBridgeAnchor() {}
