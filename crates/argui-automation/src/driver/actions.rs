//! Native input actions over the retained automation tree.

use super::*;

impl Driver {
    /// Applies one input action and queues native callback deliveries.
    /// `action` uses authored keys or coordinates; callbacks can be retrieved
    /// with [`Self::take_deliveries`] and delivered into the same JS session.
    ///
    /// # Errors
    /// Returns an error for an unknown target, invalid key, or missing root.
    pub fn act(&mut self, action: Action) -> Result<(), String> {
        let _dispatch = self.metrics.span("input.dispatch");
        match action {
            Action::Click { target, right } => {
                let point = self.point(&target)?;
                self.pointer(point, PointerPhase::Moved, None)?;
                let button = if right {
                    PointerButton::Secondary
                } else {
                    PointerButton::Primary
                };
                self.pointer(point, PointerPhase::Pressed, Some(button))?;
                if !right {
                    let regions = self.regions()?;
                    let update = self
                        .tree
                        .as_mut()
                        .expect("mounted tree")
                        .focus_pointer_default(PointerId::MOUSE, &regions);
                    self.apply(update)?;
                }
                self.pointer(point, PointerPhase::Released, Some(button))?;
                if right {
                    let node = self.hit_node(point)?;
                    let tree = self.tree.as_mut().expect("mounted tree");
                    let capabilities = tree.selection_capabilities(node);
                    let events = tree.event_deliveries(
                        node,
                        UiEventKind::ContextMenu {
                            position: point,
                            capabilities,
                        },
                    );
                    self.queue(events);
                }
            }
            Action::Scroll {
                target,
                x,
                y,
                lines,
            } => {
                if !x.is_finite() || !y.is_finite() {
                    return Err("scroll delta must be finite".into());
                }
                let point = self.point(&target)?;
                let regions = self
                    .layout
                    .as_ref()
                    .ok_or("application has no layout")?
                    .scroll_regions
                    .clone();
                let delta = if lines {
                    ScrollDelta::Lines(Point::new(-x, -y))
                } else {
                    ScrollDelta::Pixels(Point::new(-x, -y))
                };
                let wheel = self
                    .tree
                    .as_mut()
                    .expect("mounted tree")
                    .wheel_event(point, delta, &regions);
                self.apply(wheel)?;
                let update = self
                    .tree
                    .as_mut()
                    .expect("mounted tree")
                    .scroll(point, delta, &regions);
                self.apply(update)?;
            }
            Action::Fill { target, value } => {
                let node = self.find_node(&target)?;
                let tree = self.tree.as_mut().expect("mounted tree");
                if tree.text_input_value(node).is_none() {
                    return Err(format!("{target:?} is not a text input"));
                }
                let update = tree.replace_text_input(node, &value);
                self.apply(update)?;
            }
            Action::Key { value } => {
                let key = parse_key(&value)?;
                for state in [KeyState::Pressed, KeyState::Released] {
                    let input = KeyInput {
                        key: key.clone(),
                        state,
                        modifiers: Default::default(),
                        repeat: false,
                        text: match &key {
                            Key::Character(text) if state == KeyState::Pressed => {
                                Some(text.clone())
                            }
                            _ => None,
                        },
                    };
                    let regions = self.regions()?;
                    let update = self
                        .tree
                        .as_mut()
                        .expect("mounted tree")
                        .key_input(&input, &regions);
                    self.apply(update)?;
                }
            }
            Action::Drag { from, to } => {
                let start = self.point(&from)?;
                let end = self.point(&to)?;
                self.pointer(start, PointerPhase::Moved, None)?;
                self.pointer(start, PointerPhase::Pressed, Some(PointerButton::Primary))?;
                for step in 1..=5 {
                    let fraction = step as f32 / 5.0;
                    self.pointer(
                        Point::new(
                            start.x + (end.x - start.x) * fraction,
                            start.y + (end.y - start.y) * fraction,
                        ),
                        PointerPhase::Moved,
                        Some(PointerButton::Primary),
                    )?;
                }
                self.pointer(end, PointerPhase::Released, Some(PointerButton::Primary))?;
            }
        }
        Ok(())
    }
}
