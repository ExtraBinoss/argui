use argui::{
    accessibility::{Role, SemanticAction, Semantics},
    animation::{Duration, Frame, Spring, SpringConfig},
    core::{Color, Key, KeyState, Point, Transform2D, TransformOrigin},
    paint::{Border, CornerRadii, ImageFit, ImageId},
    runtime::{Context, Render},
    text::{TextStyle, TextWrap},
    ui::{
        AlignItems, CursorIcon, Element, EventType, FocusPolicy, GestureCapture, GestureDelivery,
        GestureKind, GesturePhase, GestureSet, Interaction, PanGesture, Sides, UiEventKind,
        UserSelect, length, percent,
    },
    widgets::{WidgetTheme, shadcn},
};

const ROW_SPAN: f32 = 88.0;

#[derive(Clone, Copy, Debug, PartialEq)]
struct BoardItem {
    id: u8,
    title: &'static str,
    detail: &'static str,
    color: Color,
}

pub(crate) struct DragDropDemo {
    image: ImageId,
    items: Vec<BoardItem>,
    original_items: Vec<BoardItem>,
    dragged: Option<u8>,
    drag_from: usize,
    drag_total: Point,
    velocity: Point,
    deformation: [f32; 2],
    target_deformation: [f32; 2],
    settling: Option<u8>,
    settle: Spring<[f32; 2]>,
}

impl DragDropDemo {
    /// Creates the drag-and-drop demo with one embedded preview image per item.
    pub(crate) fn new(image: ImageId) -> Self {
        let items = vec![
            BoardItem {
                id: 0,
                title: "Opening titles",
                detail: "00:00 · 5 seconds",
                color: Color::from_srgb8(37, 99, 235),
            },
            BoardItem {
                id: 1,
                title: "Product close-up",
                detail: "00:05 · 7 seconds",
                color: Color::from_srgb8(124, 58, 237),
            },
            BoardItem {
                id: 2,
                title: "Customer story",
                detail: "00:12 · 9 seconds",
                color: Color::from_srgb8(219, 39, 119),
            },
            BoardItem {
                id: 3,
                title: "Final call to action",
                detail: "00:21 · 4 seconds",
                color: Color::from_srgb8(5, 150, 105),
            },
        ];
        Self {
            image,
            original_items: items.clone(),
            items,
            dragged: None,
            drag_from: 0,
            drag_total: Point::default(),
            velocity: Point::default(),
            deformation: [0.0, 0.0],
            target_deformation: [0.0, 0.0],
            settling: None,
            settle: settle_spring([0.0, 0.0], [0.0, 0.0]),
        }
    }

    /// Returns a stable item id parsed from a retained drag target key.
    fn target_id(key: Option<&str>) -> Option<u8> {
        key?.strip_prefix("drag-item-")?.parse().ok()
    }

    /// Moves one item to a bounded list position while preserving its identity.
    fn move_to(&mut self, id: u8, target: usize) {
        let Some(current) = self.items.iter().position(|item| item.id == id) else {
            return;
        };
        let target = target.min(self.items.len() - 1);
        if current == target {
            return;
        }
        let item = self.items.remove(current);
        self.items.insert(target, item);
    }

    /// Reorders one item by a signed keyboard or accessibility step.
    fn move_by(&mut self, id: u8, delta: isize) {
        let Some(current) = self.items.iter().position(|item| item.id == id) else {
            return;
        };
        let target = (current as isize + delta).clamp(0, self.items.len() as isize - 1) as usize;
        self.move_to(id, target);
    }

    /// Applies a pan sample, including live reordering and velocity deformation.
    fn drag(
        &mut self,
        id: u8,
        phase: GesturePhase,
        total: Point,
        velocity: Point,
        reduced_motion: bool,
    ) {
        if phase == GesturePhase::Started {
            let Some(index) = self.items.iter().position(|item| item.id == id) else {
                return;
            };
            self.dragged = Some(id);
            self.drag_from = index;
            self.original_items.clone_from(&self.items);
            self.settling = None;
            self.deformation = [0.0, 0.0];
            self.target_deformation = [0.0, 0.0];
        }
        if phase == GesturePhase::Cancelled {
            self.items.clone_from(&self.original_items);
            self.dragged = None;
            self.drag_total = Point::default();
            self.velocity = Point::default();
            self.deformation = [0.0, 0.0];
            self.target_deformation = [0.0, 0.0];
            return;
        }
        self.drag_total = total;
        self.velocity = velocity;
        self.target_deformation = normalized_velocity(velocity);
        let target = (self.drag_from as f32 + total.y / ROW_SPAN)
            .round()
            .clamp(0.0, self.items.len() as f32 - 1.0) as usize;
        self.move_to(id, target);
        if phase == GesturePhase::Ended {
            self.dragged = None;
            self.drag_total = Point::default();
            self.velocity = Point::default();
            if reduced_motion {
                self.settling = None;
                self.deformation = [0.0, 0.0];
                self.target_deformation = [0.0, 0.0];
            } else {
                let release = normalized_velocity(velocity);
                self.deformation = [
                    self.deformation[0] * 0.7 + release[0] * 0.3,
                    self.deformation[1] * 0.7 + release[1] * 0.3,
                ];
                self.target_deformation = [0.0, 0.0];
                self.settling = Some(id);
                self.settle =
                    settle_spring(self.deformation, [release[0] * 0.28, release[1] * 0.28]);
            }
        }
    }

    /// Builds one retained card, including its held and momentum-settling transform.
    fn card(&self, item: BoardItem, index: usize, theme: &WidgetTheme) -> Element {
        let held = self.dragged == Some(item.id);
        let settling = self.settling == Some(item.id);
        let deformation = if held || settling {
            self.deformation
        } else {
            [0.0, 0.0]
        };
        let current = self
            .items
            .iter()
            .position(|candidate| candidate.id == item.id)
            .unwrap_or(index);
        let translation = if held {
            Point::new(
                self.drag_total.x,
                self.drag_total.y - (current as f32 - self.drag_from as f32) * ROW_SPAN,
            )
        } else {
            Point::default()
        };
        let stretch_x =
            (1.0 + deformation[0].abs() * 0.08 - deformation[1].abs() * 0.035).clamp(0.94, 1.1);
        let stretch_y =
            (1.0 + deformation[1].abs() * 0.08 - deformation[0].abs() * 0.035).clamp(0.94, 1.1);
        let transform = Transform2D::IDENTITY
            .translate(translation.x, translation.y)
            .rotate(deformation[0] * 0.055)
            .scale(stretch_x, stretch_y);
        let thumbnail = Element::image(self.image)
            .image_fit(ImageFit::Contain)
            .width(length(58.0))
            .height(length(58.0))
            .background(item.color.with_alpha(0.16))
            .clip(CornerRadii::all(10.0));
        let copy = Element::column([
            Element::text(item.title).text_style(TextStyle {
                color: theme.foreground,
                font_size: 15.0,
                line_height: 20.0,
                weight: 680,
                wrap: TextWrap::None,
                ..TextStyle::default()
            }),
            Element::text(item.detail).text_style(TextStyle {
                color: theme.muted_foreground,
                font_size: 12.0,
                line_height: 17.0,
                weight: 450,
                wrap: TextWrap::None,
                ..TextStyle::default()
            }),
        ])
        .gap(3.0)
        .grow(1.0);
        let grip = Element::text(if held { "HOLDING" } else { "DRAG" }).text_style(TextStyle {
            color: if held {
                theme.primary
            } else {
                theme.muted_foreground
            },
            font_size: 10.0,
            line_height: 14.0,
            weight: 750,
            wrap: TextWrap::None,
            ..TextStyle::default()
        });
        let surface = Element::row([thumbnail, copy, grip])
            .width(percent(1.0))
            .height(percent(1.0))
            .padding(Sides::length(10.0))
            .gap(12.0)
            .align_items(AlignItems::CENTER)
            .background(if held { theme.secondary } else { theme.card })
            .border(Border::all(
                if held { 2.0 } else { 1.0 },
                if held { theme.primary } else { theme.border },
            ))
            .radius(CornerRadii::all(14.0))
            .z_index(1);
        let mut layers = Vec::with_capacity(if held { 3 } else { 1 });
        if held {
            layers.push(flat_shadow(10.0, 0.08, 0.975));
            layers.push(flat_shadow(5.0, 0.13, 0.99));
        }
        layers.push(surface);
        Element::container(layers)
            .keyed(format!("drag-item-{}", item.id))
            .width(percent(1.0))
            .height(length(78.0))
            .transform(transform)
            .transform_origin(TransformOrigin::CENTER)
            .z_index(if held { 20 } else { 0 })
            .interaction(
                Interaction::default()
                    .focus_policy(FocusPolicy::TabStop)
                    .cursor(if held {
                        CursorIcon::Grabbing
                    } else {
                        CursorIcon::Grab
                    })
                    .gestures(
                        GestureSet::EMPTY.pan(
                            PanGesture::default()
                                .immediate()
                                .capture(GestureCapture::OnPress)
                                .delivery(GestureDelivery::FrameCoalesced),
                        ),
                    ),
            )
            .semantics(
                Semantics::new(Role::ListItem)
                    .label(format!("{} at position {}", item.title, index + 1))
                    .description("Drag to reorder, or use Up and Down")
                    .position_in_set(index as u32 + 1, self.items.len() as u32)
                    .action(SemanticAction::Focus)
                    .action(SemanticAction::Increment)
                    .action(SemanticAction::Decrement),
            )
            .user_select(UserSelect::None)
    }
}

impl Render for DragDropDemo {
    fn wants_animation_frame(&self) -> bool {
        self.dragged.is_some() || (self.settling.is_some() && self.settle.is_active())
    }

    fn animation_frame(&mut self, frame: Frame, cx: &mut Context<Self>) {
        if self.dragged.is_some() {
            let seconds = frame.elapsed.as_secs_f64().min(0.05) as f32;
            let follow = 1.0 - (-18.0 * seconds).exp();
            let decay = (-7.0 * seconds).exp();
            for axis in 0..2 {
                self.deformation[axis] +=
                    (self.target_deformation[axis] - self.deformation[axis]) * follow;
                self.target_deformation[axis] *= decay;
            }
            cx.notify();
        }
        if self.settling.is_some()
            && self
                .settle
                .advance(frame.elapsed.min(Duration::from_millis(34)))
        {
            self.deformation = self.settle.value();
            cx.notify();
        }
        if !self.settle.is_active() && self.settling.take().is_some() {
            self.deformation = [0.0, 0.0];
            cx.notify();
        }
    }

    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let themes = shadcn(cx.environment());
        let theme = themes.resolve(cx.environment().color_scheme);
        let cards = self
            .items
            .iter()
            .copied()
            .enumerate()
            .map(|(index, item)| self.card(item, index, theme));
        let status = if let Some(id) = self.dragged {
            let item = self.items.iter().find(|item| item.id == id);
            let position = self
                .items
                .iter()
                .position(|item| item.id == id)
                .unwrap_or(0)
                + 1;
            format!(
                "Holding {} · position {} of {} · velocity {:.0}, {:.0}",
                item.map_or("item", |item| item.title),
                position,
                self.items.len(),
                self.velocity.x,
                self.velocity.y
            )
        } else {
            format!(
                "Order: {}",
                self.items
                    .iter()
                    .map(|item| item.title)
                    .collect::<Vec<_>>()
                    .join(" → ")
            )
        };
        let list = Element::column(cards)
            .keyed("drag-drop-list")
            .gap(10.0)
            .semantics(Semantics::new(Role::List).label("Editable sequence"))
            .on(cx.listener(EventType::Gesture, |demo, event, cx| {
                let Some(id) = Self::target_id(event.target_key()) else {
                    return;
                };
                let UiEventKind::Gesture(gesture) = event.kind else {
                    return;
                };
                let GestureKind::Pan {
                    total, velocity, ..
                } = gesture.kind
                else {
                    return;
                };
                demo.drag(
                    id,
                    gesture.phase,
                    total,
                    velocity,
                    cx.environment().reduced_motion,
                );
                event.stop_propagation();
                cx.notify();
            }))
            .on(cx.listener(EventType::Key, |demo, event, cx| {
                let Some(id) = Self::target_id(event.target_key()) else {
                    return;
                };
                let UiEventKind::KeyInput(input) = &event.kind else {
                    return;
                };
                if input.state != KeyState::Pressed {
                    return;
                }
                let delta = match input.key {
                    Key::ArrowUp => -1,
                    Key::ArrowDown => 1,
                    _ => return,
                };
                demo.move_by(id, delta);
                event.stop_propagation();
                cx.notify();
            }))
            .on(cx.listener(EventType::SemanticAction, |demo, event, cx| {
                let Some(id) = Self::target_id(event.target_key()) else {
                    return;
                };
                let UiEventKind::SemanticAction { action, .. } = event.kind else {
                    return;
                };
                let delta = match action {
                    SemanticAction::Decrement => -1,
                    SemanticAction::Increment => 1,
                    _ => return,
                };
                demo.move_by(id, delta);
                event.stop_propagation();
                cx.notify();
            }));
        super::preview(
            "Momentum reorder board",
            "Grab a card by any point. Stable retained identities keep the held item under the pointer while the data order changes live; release velocity drives squash, stretch and spring settling.",
            Element::column([
                list,
                Element::text(status.clone())
                    .keyed("drag-drop-status")
                    .text_style(TextStyle {
                        color: theme.muted_foreground,
                        font_size: 12.0,
                        line_height: 18.0,
                        weight: 500,
                        wrap: TextWrap::Word,
                        ..TextStyle::default()
                    })
                    .semantics(Semantics::new(Role::Status).label(status)),
            ])
            .width(percent(1.0))
            .max_width(length(620.0))
            .gap(14.0),
            theme,
        )
    }
}

/// Converts pointer velocity into bounded directional deformation.
fn normalized_velocity(velocity: Point) -> [f32; 2] {
    [
        (velocity.x / 1_200.0).clamp(-1.0, 1.0),
        (velocity.y / 1_200.0).clamp(-1.0, 1.0),
    ]
}

/// Builds one inexpensive shadow plate that shares the held card transform.
fn flat_shadow(offset: f32, alpha: f32, scale: f32) -> Element {
    Element::container([])
        .absolute(Sides::length(0.0))
        .background(Color::BLACK.with_alpha(alpha))
        .radius(CornerRadii::all(14.0))
        .transform(
            Transform2D::IDENTITY
                .translate(0.0, offset)
                .scale(scale, scale),
        )
        .transform_origin(TransformOrigin::CENTER)
        .semantic_hidden(true)
        .user_select(UserSelect::None)
}

/// Creates the spring that settles a dropped item without discarding momentum.
fn settle_spring(value: [f32; 2], velocity: [f32; 2]) -> Spring<[f32; 2]> {
    Spring::new(
        value,
        [0.0, 0.0],
        velocity,
        SpringConfig {
            stiffness: 245.0,
            damping: 15.0,
            rest_speed: 0.002,
            rest_delta: 0.002,
            ..SpringConfig::default()
        },
    )
    .expect("drag-and-drop spring parameters are physical")
}
