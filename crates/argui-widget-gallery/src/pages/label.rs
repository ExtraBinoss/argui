use argui::{
    runtime::Context,
    ui::{Element, EventType, length},
    widgets::{Input, Label, WidgetTheme},
};

use crate::{WidgetGallery, app::text};

pub(super) fn render(
    gallery: &WidgetGallery,
    theme: &WidgetTheme,
    cx: &mut Context<WidgetGallery>,
) -> Element {
    let label = Label::new("profile-name-label", "Display name", "name");
    let workspace = Label::new("profile-id-label", "Workspace ID", "workspace-id");
    let name =
        label.associate(Input::new("name", &gallery.name, "Your name", theme.input()).build());
    let id = workspace.associate(
        Input::new(
            "workspace-id",
            &gallery.workspace_id,
            "Workspace ID",
            theme.input(),
        )
        .build(),
    );
    super::preview(
        "Make every field clear",
        "Click a label to focus its field. Use Tab to move between controls.",
        Element::column([
            Element::column([
                label.build(theme),
                name,
                text(
                    "This is how your name appears to your team.",
                    13.0,
                    theme.muted_foreground,
                    400,
                ),
            ])
            .gap(8.0),
            Element::column([workspace.build(theme), id]).gap(8.0),
        ])
        .gap(28.0)
        .max_width(length(420.0)),
        theme,
    )
    .on(cx.listener(EventType::Click, move |_, event, cx| {
        if let Some(target) = label
            .focus_target(event)
            .or_else(|| workspace.focus_target(event))
        {
            cx.request_focus(target);
            event.stop_propagation();
        }
    }))
}
