use argui_animation::{DecayConfig, Inertia, InertiaConfig, InertiaState, Spring, SpringConfig};
use argui_paint::{Border, Color, CornerRadii};
use argui_text::TextWrap;
use argui_theme::WidgetTheme;
use argui_ui::{Edges, Element, Length, Wrap, property};

use super::{StateShowcase, button, text_style};

#[derive(Clone, Copy, Default)]
pub(super) enum PhysicsMode {
    #[default]
    Spring,
    Inertia,
}

#[derive(Clone, Copy)]
pub(super) enum PhysicsCommand {
    RetargetSpring,
    LaunchInertia,
}

impl StateShowcase {
    pub(super) fn apply_physics_command(&mut self) -> bool {
        let Some(command) = self.physics_command.take() else {
            return false;
        };
        match command {
            PhysicsCommand::RetargetSpring => {
                self.inertia_held = false;
                if !matches!(self.physics_mode, PhysicsMode::Spring) {
                    self.spring = Spring::new(
                        self.physics_value,
                        self.physics_value,
                        self.inertia.velocity(),
                        showcase_spring_config(),
                    )
                    .expect("the showcase spring configuration is valid");
                }
                let target = if self.spring.target() < 0.5 { 1.0 } else { 0.0 };
                self.spring.retarget(target);
                self.physics_mode = PhysicsMode::Spring;
            }
            PhysicsCommand::LaunchInertia => {
                let velocity = if matches!(self.physics_mode, PhysicsMode::Inertia)
                    && self.inertia.velocity().abs() > 0.002
                {
                    self.inertia.velocity()
                } else if self.physics_value < 0.5 {
                    4.5
                } else {
                    -4.5
                };
                self.inertia.launch(self.physics_value, velocity);
                self.physics_mode = PhysicsMode::Inertia;
            }
        }
        true
    }

    pub(super) fn hold_inertia(&mut self) {
        if self.inertia_held {
            return;
        }
        self.inertia_hold_velocity = if self.physics_value < 0.5 { 1.6 } else { -1.6 };
        self.inertia
            .launch(self.physics_value, self.inertia_hold_velocity);
        self.physics_mode = PhysicsMode::Inertia;
        self.inertia_held = true;
    }

    pub(super) fn release_inertia(&mut self) {
        self.inertia_held = false;
    }

    pub(super) fn advance_held_inertia(&mut self, elapsed: argui_animation::Duration) -> bool {
        let previous = self.physics_value;
        let mut next = previous + self.inertia_hold_velocity * elapsed.as_secs_f64() as f32;
        if next >= 1.0 {
            next = 1.0;
            self.inertia_hold_velocity = -self.inertia_hold_velocity.abs();
        } else if next <= 0.0 {
            next = 0.0;
            self.inertia_hold_velocity = self.inertia_hold_velocity.abs();
        }
        self.inertia.launch(next, self.inertia_hold_velocity);
        next != previous
    }

    pub(super) fn physics_demo(&self, widgets: &WidgetTheme) -> Element {
        let normalized = self.physics_value.clamp(0.0, 1.0);
        let color = physics_color(normalized);
        let state = self.physics_status();
        Element::column([
            Element::text(format!("Physics · {state}")).text_style(text_style(
                17.0,
                widgets.foreground,
                650,
                TextWrap::None,
            )),
            Element::container([])
                .keyed("physics-visual")
                .height(Length::Px(68.0))
                .width(Length::Percent(1.0))
                .background(color)
                .bind(property::BackgroundColor, self.physics_color.clone())
                .border(Border::all(1.0, widgets.border))
                .radius(CornerRadii::all(10.0 + normalized * 24.0))
                .bind(property::CornerRadii, self.physics_radii.clone()),
            Element::row([
                button("physics-spring", "Spring retarget", widgets, false),
                button("physics-inertia", "Launch inertia", widgets, false),
            ])
            .wrap(Wrap::Wrap)
            .gap(10.0),
        ])
        .gap(12.0)
        .padding(Edges::all(16.0))
        .background(widgets.card)
        .border(Border::all(1.0, widgets.border))
        .radius(CornerRadii::all(10.0))
    }

    pub(super) fn physics_status(&self) -> &'static str {
        match self.physics_mode {
            PhysicsMode::Spring if self.spring.is_active() => "spring moving",
            PhysicsMode::Spring => "spring settled",
            PhysicsMode::Inertia if self.inertia_held => "inertia held",
            PhysicsMode::Inertia if self.inertia.state() == InertiaState::Decaying => "decaying",
            PhysicsMode::Inertia if self.inertia.state() == InertiaState::Bouncing => "bouncing",
            PhysicsMode::Inertia => "inertia settled",
        }
    }
}

pub(super) fn physics_color(value: f32) -> Color {
    let normalized = value.clamp(0.0, 1.0);
    Color::rgb(
        0.18 + normalized * 0.68,
        0.72 - normalized * 0.32,
        0.92 - normalized * 0.58,
    )
}

pub(super) fn physics_radii(value: f32) -> [f32; 4] {
    [10.0 + value.clamp(0.0, 1.0) * 24.0; 4]
}

pub(super) fn showcase_spring_config() -> SpringConfig {
    SpringConfig {
        stiffness: 150.0,
        damping: 12.0,
        rest_speed: 0.002,
        rest_delta: 0.002,
        ..SpringConfig::default()
    }
}

pub(super) fn showcase_spring() -> Spring<f32> {
    Spring::new(0.0, 0.0, 0.0, showcase_spring_config())
        .expect("the showcase spring configuration is valid")
}

pub(super) fn showcase_inertia() -> Inertia {
    Inertia::new(
        0.0,
        0.0,
        InertiaConfig {
            decay: DecayConfig {
                rate: 3.2,
                rest_speed: 0.002,
            },
            bounds: Some((0.0, 1.0)),
            ..InertiaConfig::default()
        },
    )
    .expect("the showcase inertia configuration is valid")
}

#[cfg(test)]
mod tests {
    use argui_ui::ElementKind;

    use super::{PhysicsCommand, PhysicsMode, showcase_inertia};
    use crate::StateShowcase;

    fn state_label(showcase: &StateShowcase) -> String {
        let themes = argui_theme::shadcn(argui_paint::Color::WHITE);
        let demo = showcase.physics_demo(themes.resolve(argui_core::ColorScheme::Dark));
        match &demo.children[0].kind {
            ElementKind::Text { content, .. } => content.clone(),
            _ => panic!("physics state is rendered as text"),
        }
    }

    #[test]
    fn physics_commands_cover_retargeting_and_both_inertia_directions() {
        let mut showcase = StateShowcase::default();
        assert!(!showcase.apply_physics_command());

        showcase.physics_command = Some(PhysicsCommand::RetargetSpring);
        assert!(showcase.apply_physics_command());
        assert_eq!(showcase.spring.target(), 1.0);

        showcase.physics_command = Some(PhysicsCommand::RetargetSpring);
        assert!(showcase.apply_physics_command());
        assert_eq!(showcase.spring.target(), 0.0);

        showcase.physics_mode = PhysicsMode::Inertia;
        showcase.physics_command = Some(PhysicsCommand::RetargetSpring);
        assert!(showcase.apply_physics_command());
        assert_eq!(showcase.spring.target(), 1.0);

        showcase.physics_value = 0.25;
        showcase.physics_command = Some(PhysicsCommand::LaunchInertia);
        assert!(showcase.apply_physics_command());
        assert!(showcase.inertia.velocity() > 0.0);
        assert!(state_label(&showcase).contains("decaying"));

        showcase.inertia.launch(-0.1, 0.0);
        assert!(state_label(&showcase).contains("bouncing"));

        showcase.inertia = showcase_inertia();
        assert!(state_label(&showcase).contains("inertia settled"));

        showcase.physics_value = 0.75;
        showcase.physics_command = Some(PhysicsCommand::LaunchInertia);
        assert!(showcase.apply_physics_command());
        assert!(showcase.inertia.velocity() < 0.0);
    }
}
