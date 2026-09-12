use argui_core::{Color, ColorScheme};
use argui_ui::{ClickEvent, Element, UiEvent, UiEventKind, UiTree};
use argui_widgets::{Sidebar, SidebarAction, shadcn};

#[test]
fn sidebar_switches_between_desktop_rail_and_mobile_modal_without_dead_controls() {
    let mut sidebar = Sidebar::new("nav", "Navigation", Element::text("Content").keyed("full"));
    sidebar.header = Some(Element::text("Header"));
    sidebar.footer = Some(Element::text("Footer"));
    let node = UiTree::new(Element::container([])).node_ids()[0];
    for mobile in [false, true] {
        for collapsed in [false, true] {
            for open in [false, true] {
                sidebar.mobile = mobile;
                sidebar.collapsed = collapsed;
                sidebar.open = open;
                let tree = UiTree::new(
                    sidebar
                        .clone()
                        .build(shadcn(Color::BLACK).resolve(ColorScheme::Light)),
                );
                assert!(tree.semantic_diagnostics().is_empty());
                assert_eq!(
                    tree.node_ids()
                        .iter()
                        .any(|node| tree.key(*node) == Some("full")),
                    if mobile { open } else { !collapsed }
                );
                let key = if mobile {
                    "nav::trigger"
                } else {
                    "nav::toggle"
                };
                assert_eq!(
                    sidebar.action(&UiEvent::new(
                        node,
                        Some(key.into()),
                        UiEventKind::Click(ClickEvent::accessibility())
                    )),
                    Some(if mobile {
                        SidebarAction::SetOpen(true)
                    } else {
                        SidebarAction::SetCollapsed(!collapsed)
                    })
                );
            }
        }
    }
    sidebar.mobile = false;
    sidebar.collapsed = true;
    sidebar.rail = Some(Element::text("Rail").keyed("rail"));
    let tree = UiTree::new(sidebar.build(shadcn(Color::BLACK).resolve(ColorScheme::Dark)));
    assert!(
        tree.node_ids()
            .iter()
            .any(|node| tree.key(*node) == Some("rail"))
    );
}
