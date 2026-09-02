use argui::{
    core::Transform2D,
    paint::{Border, CornerRadii, PaintStyle, QuadStyle},
    ui::{
        AlignItems, ContainerQuery, ContainerScopeId, CursorIcon, Element, FlexDirection,
        GestureSet, HitShape, HitTestStyle, Interaction, JustifyContent, KeyboardActivation,
        PointerEvents, Role, SemanticAction, Semantics, Sides, StyleCondition, StylePatch,
        StyleTransition, VisualState, length, percent, property,
    },
    widgets::{Button, TablerIcon, WidgetAssets, WidgetTheme},
};

use super::preview;
use crate::{app::WidgetGallery, property_slider};

const PANEL_SCOPE: ContainerScopeId = ContainerScopeId::new("advanced-composition-panel");

pub(super) fn render(
    gallery: &WidgetGallery,
    theme: &WidgetTheme,
    assets: &WidgetAssets,
) -> Element {
    Element::column([
        preview(
            "Editable property control",
            "A range behavior, direct arithmetic input, reset action, detents and animated parts compose without a specialized renderer.",
            property_slider::render(gallery, theme, assets),
            theme,
        ),
        preview(
            "Container-aware composition",
            "Resize the window: the same retained children switch layout from their nearest named container.",
            responsive_actions(theme, assets),
            theme,
        ),
        preview(
            "Visible hit geometry",
            "Hover or click the outlined 6 px halo: it belongs to the ellipse hit slop even though the button paints only the solid circle.",
            precise_target(gallery, theme, assets),
            theme,
        ),
    ])
    .gap(28.0)
}

fn responsive_actions(theme: &WidgetTheme, assets: &WidgetAssets) -> Element {
    let title = crate::app::text("Renderer ready", 15.0, theme.foreground, 650).when(
        ContainerQuery::max_width(PANEL_SCOPE, 520.0),
        StylePatch::new().set(property::TextColor, theme.primary),
    );
    let description = crate::app::text(
        "Native WGPU and WebGPU share this tree",
        12.0,
        theme.muted_foreground,
        450,
    );
    let copy = Element::column([title, description]).gap(3.0);
    let action = Button::new(
        "composition-inspect",
        "Inspect frame",
        theme.outline_button.clone(),
    )
    .leading(assets.icon(TablerIcon::Sidebar, 16.0))
    .build()
    .when(
        StyleCondition::all([
            VisualState::Hovered.into(),
            ContainerQuery::max_width(PANEL_SCOPE, 520.0).into(),
        ]),
        StylePatch::new().set(
            property::Transform,
            Transform2D::IDENTITY.translate(4.0, 0.0),
        ),
    );
    let base = Element::row([copy, action])
        .width(percent(1.0))
        .align_items(AlignItems::CENTER)
        .justify_content(JustifyContent::SPACE_BETWEEN)
        .gap(14.0);
    let mut compact = base.style.clone();
    compact.flex_direction = FlexDirection::Column;
    compact.align_items = Some(AlignItems::STRETCH);
    let content = base.when(
        ContainerQuery::max_width(PANEL_SCOPE, 520.0),
        StylePatch::new().layout(compact),
    );

    Element::container([content])
        .container_scope(PANEL_SCOPE)
        .width(percent(1.0))
        .padding(Sides::length(18.0))
        .background(theme.card)
        .border(Border::all(1.0, theme.border))
        .radius(CornerRadii::all(10.0))
}

fn precise_target(gallery: &WidgetGallery, theme: &WidgetTheme, assets: &WidgetAssets) -> Element {
    let decoration = Element::container([])
        .absolute(Sides::length(-6.0))
        .border(Border::all(1.0, theme.border))
        .radius(CornerRadii::all(999.0))
        .hit_test(HitTestStyle::default().pointer_events(PointerEvents::None));
    let target = Element::container([decoration, assets.icon(TablerIcon::Restore, 18.0)])
        .keyed("composition-circle")
        .width(length(52.0))
        .height(length(52.0))
        .align_items(AlignItems::CENTER)
        .justify_content(JustifyContent::CENTER)
        .paint_style(PaintStyle::new(
            QuadStyle::solid(theme.primary).radius(CornerRadii::all(999.0)),
        ))
        .interaction(
            Interaction::default()
                .focusable(true)
                .cursor(CursorIcon::Pointer)
                .gestures(GestureSet::NONE.tap())
                .keyboard_activation(KeyboardActivation::EnterOrSpace),
        )
        .hit_test(
            HitTestStyle::default()
                .shape(HitShape::Ellipse)
                .slop(Sides::length(6.0)),
        )
        .semantics(
            Semantics::new(Role::Button)
                .label("Retarget composition")
                .action(SemanticAction::Click),
        )
        .when(
            VisualState::Hovered,
            StylePatch::new()
                .set(property::Opacity, 0.88)
                .set(property::Transform, Transform2D::IDENTITY.scale(1.08, 1.08)),
        )
        .when(
            VisualState::Pressed,
            StylePatch::new().set(property::Transform, Transform2D::IDENTITY.scale(0.94, 0.94)),
        )
        .transition(StyleTransition::default());

    Element::column([
        Element::row([target])
            .height(length(78.0))
            .align_items(AlignItems::CENTER)
            .justify_content(JustifyContent::CENTER),
        crate::app::text(
            format!(
                "Activations from circle or halo: {}",
                gallery.composition_hits
            ),
            12.0,
            theme.muted_foreground,
            550,
        ),
    ])
    .width(percent(1.0))
    .align_items(AlignItems::CENTER)
    .justify_content(JustifyContent::CENTER)
    .gap(4.0)
    .padding(Sides::length(10.0))
    .background(theme.muted)
    .radius(CornerRadii::all(10.0))
}
