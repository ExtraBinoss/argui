use argui_core::{Point, Rect, Size};
use argui_platform::WindowKey;
use argui_runtime::{
    AppModel, Context, LayoutBounds, LayoutSnapshot, Render, SingleWindowModel, WindowEnvironment,
};
use argui_ui::{Element, RetainedIdentity, UiTree};
use std::{cell::RefCell, rc::Rc};

struct Measured {
    identity: RetainedIdentity,
    observed: Rc<RefCell<Option<Size>>>,
}
impl Render for Measured {
    /// Reads the last measured size and keeps the observed node identity on the root.
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        *self.observed.borrow_mut() = cx.observed_interaction(&self.identity).measured;
        Element::container([]).retained_identity(self.identity.clone())
    }
}

/// Layout measurements for unwatched identities cannot invalidate a retained view.
#[test]
fn interaction_refresh_ignores_unwatched_layout_measurements() {
    let identity = RetainedIdentity::new(22, 1);
    let other = RetainedIdentity::new(22, 2);
    let observed = Rc::new(RefCell::new(None));
    let app = SingleWindowModel::new(Measured {
        identity: identity.clone(),
        observed: observed.clone(),
    });
    let key = WindowKey::main();
    let tree = UiTree::new(app.view(&key, WindowEnvironment::default()).unwrap());
    let node = tree.node_ids()[0];
    let layout = LayoutSnapshot {
        viewport: Rect::default(),
        nodes: vec![
            LayoutBounds {
                node,
                key: None,
                retained_identity: Some(identity),
                bounds: Rect::new(Point::default(), Size::new(40.0, 20.0)),
            },
            LayoutBounds {
                node,
                key: None,
                retained_identity: Some(other),
                bounds: Rect::new(Point::default(), Size::new(90.0, 70.0)),
            },
        ],
    };
    assert!(app.refresh_interaction_observations(&tree, &[], &[], None, Some(&layout)));
    app.view(&key, WindowEnvironment::default()).unwrap();
    assert_eq!(*observed.borrow(), Some(Size::new(40.0, 20.0)));
    assert!(!app.refresh_interaction_observations(&tree, &[], &[], None, Some(&layout)));
}
