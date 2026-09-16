import ActivityKit
import SwiftUI
import WidgetKit

private enum ArguiActivityPalette {
    static let accent = Color(red: 0.16, green: 0.47, blue: 0.96)
    static let accentSoft = Color(red: 0.16, green: 0.47, blue: 0.96).opacity(0.18)
    static let background = Color(red: 0.055, green: 0.067, blue: 0.09)
    static let foreground = Color.white
    static let secondary = Color.white.opacity(0.7)
}

private extension ArguiActivityAttributes.ContentState {
    var boundedProgress: Double {
        min(max(progress, 0), 1)
    }

    var percentage: Int {
        Int((boundedProgress * 100).rounded())
    }

    var isComplete: Bool {
        boundedProgress >= 1
    }
}

private struct ArguiActivityMark: View {
    let compact: Bool

    var body: some View {
        ZStack {
            RoundedRectangle(cornerRadius: compact ? 7 : 10, style: .continuous)
                .fill(ArguiActivityPalette.accentSoft)
            Image(systemName: "square.stack.3d.up.fill")
                .font(compact ? .caption2 : .subheadline)
                .foregroundStyle(ArguiActivityPalette.accent)
        }
        .frame(width: compact ? 24 : 36, height: compact ? 24 : 36)
        .accessibilityHidden(true)
    }
}

private struct ArguiProgressBadge: View {
    let state: ArguiActivityAttributes.ContentState

    var body: some View {
        Text(state.isComplete ? "Terminé" : "\(state.percentage)%")
            .font(.caption.weight(.semibold).monospacedDigit())
            .foregroundStyle(state.isComplete ? Color.green : ArguiActivityPalette.accent)
            .padding(.horizontal, 9)
            .padding(.vertical, 5)
            .background(
                Capsule(style: .continuous)
                    .fill((state.isComplete ? Color.green : ArguiActivityPalette.accent).opacity(0.16))
            )
            .accessibilityLabel(state.isComplete ? "Activité terminée" : "Progression \(state.percentage) pour cent")
    }
}

private struct ArguiLockScreenActivityView: View {
    let context: ActivityViewContext<ArguiActivityAttributes>

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack(spacing: 10) {
                ArguiActivityMark(compact: false)
                VStack(alignment: .leading, spacing: 2) {
                    Text(context.attributes.title)
                        .font(.headline.weight(.semibold))
                        .foregroundStyle(ArguiActivityPalette.foreground)
                        .lineLimit(1)
                    Text(context.state.message)
                        .font(.subheadline)
                        .foregroundStyle(ArguiActivityPalette.secondary)
                        .lineLimit(2)
                }
                Spacer(minLength: 8)
                ArguiProgressBadge(state: context.state)
            }

            ProgressView(value: context.state.boundedProgress)
                .progressViewStyle(.linear)
                .tint(context.state.isComplete ? .green : ArguiActivityPalette.accent)
                .accessibilityLabel("Progression de l’activité")
                .accessibilityValue("\(context.state.percentage) pour cent")

            HStack(spacing: 5) {
                Image(systemName: context.state.isComplete ? "checkmark.circle.fill" : "bolt.horizontal.circle.fill")
                Text(context.state.isComplete ? "Synchronisation terminée" : "Argui continue en arrière-plan")
            }
            .font(.caption2.weight(.medium))
            .foregroundStyle(ArguiActivityPalette.secondary)
        }
        .padding(16)
        .activityBackgroundTint(ArguiActivityPalette.background)
        .activitySystemActionForegroundColor(ArguiActivityPalette.accent)
    }
}

@main
struct ArguiActivityWidget: Widget {
    var body: some WidgetConfiguration {
        ActivityConfiguration(for: ArguiActivityAttributes.self) { context in
            ArguiLockScreenActivityView(context: context)
        } dynamicIsland: { context in
            DynamicIsland {
                DynamicIslandExpandedRegion(.leading) {
                    HStack(spacing: 7) {
                        ArguiActivityMark(compact: true)
                        Text("ARGUI")
                            .font(.caption2.weight(.bold))
                            .foregroundStyle(ArguiActivityPalette.secondary)
                    }
                }
                DynamicIslandExpandedRegion(.trailing) {
                    ArguiProgressBadge(state: context.state)
                }
                DynamicIslandExpandedRegion(.center) {
                    Text(context.attributes.title)
                        .font(.headline.weight(.semibold))
                        .lineLimit(1)
                }
                DynamicIslandExpandedRegion(.bottom) {
                    VStack(alignment: .leading, spacing: 8) {
                        Text(context.state.message)
                            .font(.subheadline)
                            .foregroundStyle(ArguiActivityPalette.secondary)
                            .lineLimit(2)
                        ProgressView(value: context.state.boundedProgress)
                            .progressViewStyle(.linear)
                            .tint(context.state.isComplete ? .green : ArguiActivityPalette.accent)
                    }
                    .padding(.top, 2)
                }
            } compactLeading: {
                Image(systemName: context.state.isComplete ? "checkmark.circle.fill" : "square.stack.3d.up.fill")
                    .foregroundStyle(context.state.isComplete ? .green : ArguiActivityPalette.accent)
            } compactTrailing: {
                Text("\(context.state.percentage)%")
                    .font(.caption2.weight(.semibold).monospacedDigit())
                    .foregroundStyle(context.state.isComplete ? .green : ArguiActivityPalette.accent)
            } minimal: {
                ZStack {
                    Circle()
                        .stroke(ArguiActivityPalette.secondary.opacity(0.35), lineWidth: 2)
                    Circle()
                        .trim(from: 0, to: context.state.boundedProgress)
                        .stroke(
                            context.state.isComplete ? Color.green : ArguiActivityPalette.accent,
                            style: StrokeStyle(lineWidth: 2.5, lineCap: .round)
                        )
                        .rotationEffect(.degrees(-90))
                    Image(systemName: context.state.isComplete ? "checkmark" : "arrow.up")
                        .font(.system(size: 8, weight: .bold))
                }
                .padding(4)
                .accessibilityLabel("Progression \(context.state.percentage) pour cent")
            }
            .keylineTint(ArguiActivityPalette.accent)
        }
    }
}
