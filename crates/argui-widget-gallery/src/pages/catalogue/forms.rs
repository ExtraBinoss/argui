use super::*;
use crate::app::text;
use argui::{
    ui::{UiEventKind, percent},
    widgets::*,
};

impl CatalogueDemo {
    fn field(&self, theme: &WidgetTheme) -> Field {
        let mut field = Field::new(
            "email-field",
            "Email address",
            "email-control",
            Input::new(
                "email-control",
                &self.text,
                "you@example.com",
                theme.input(),
            )
            .build(),
        );
        field.description = Some("Use your work email to receive the invitation.".into());
        if !self.text.is_empty() && !self.text.contains('@') {
            field.error = Some("Include an @ in the email address.".into());
        }
        field.required = true;
        field
    }

    fn combobox(&self) -> Combobox {
        let mut combo = Combobox::new(
            "framework",
            "Search frameworks",
            ["Argui", "React", "Svelte", "Vue", "Solid"].map(SelectOption::new),
        );
        combo.query.clone_from(&self.text);
        combo.open = self.open;
        combo.highlighted = self.highlighted;
        combo.selected = Some(self.selected);
        combo
    }

    fn native_select(&self) -> NativeSelect {
        let mut select = NativeSelect::new(
            "language",
            "Language",
            ["English", "Français", "Deutsch"].map(SelectOption::new),
            Some(self.selected),
        );
        select.open = self.open;
        select.highlighted = self.highlighted.unwrap_or(self.selected);
        select.required = true;
        select
    }

    fn questionnaire_widget(&self) -> Questionnaire {
        let mut first = Question::new("project", "What would you like to build?");
        first.description = "Choose a starting point, or describe your idea.".into();
        first.choices = ["Desktop app", "Dashboard", "Design tool"]
            .into_iter()
            .enumerate()
            .map(|(index, label)| QuestionChoice {
                id: index.to_string(),
                label: label.into(),
            })
            .collect();
        let mut second = Question::new("features", "Which features matter to you?");
        second.mode = ChoiceMode::Multiple;
        second.required = false;
        second.freeform = false;
        second.choices = ["Keyboard support", "Dark mode", "Live data"]
            .into_iter()
            .enumerate()
            .map(|(index, label)| QuestionChoice {
                id: index.to_string(),
                label: label.into(),
            })
            .collect();
        let mut widget = Questionnaire::new("survey", [first, second]);
        widget.state = self.questionnaire.clone();
        widget
    }

    pub(super) fn forms_view(&self, theme: &WidgetTheme) -> Element {
        let view = match self.page {
            Page::Field => self.field(theme).build(theme),
            Page::InputGroup => {
                let mut group = InputGroup::new(
                    "site",
                    "Website",
                    Input::new("domain", &self.text, "example.com", theme.input())
                        .label("Domain")
                        .build(),
                );
                group.leading = Some(text("https://", 14.0, theme.muted_foreground, 400));
                group.trailing =
                    Some(Button::new("visit", "Preview", theme.outline_button()).build());
                group.build(theme)
            }
            Page::InputOtp => Element::column([
                InputOtp::new("code", "Verification code", &self.text, 6).build(theme),
                text(
                    "Enter six digits. You can paste the whole code.",
                    13.0,
                    theme.muted_foreground,
                    400,
                ),
            ])
            .gap(10.0),
            Page::Combobox => self.combobox().build(theme),
            Page::NativeSelect => self.native_select().build(theme),
            Page::Questionnaire => self.questionnaire_widget().build(theme),
            _ => unreachable!("form page"),
        };
        view.width(percent(1.0)).max_width(argui::ui::length(520.0))
    }

    pub(super) fn forms_event(
        &mut self,
        event: &UiEvent,
        theme: &WidgetTheme,
        cx: &mut Context<Self>,
    ) -> bool {
        match self.page {
            Page::Field => {
                if let Some(target) = self.field(theme).focus_target(event) {
                    cx.request_focus(target);
                    return true;
                }
                if event.target_key() == Some("email-control")
                    && let UiEventKind::TextChanged(value) = &event.kind
                {
                    self.text.clone_from(value);
                    return true;
                }
            }
            Page::InputGroup => {
                if event.target_key() == Some("domain")
                    && let UiEventKind::TextChanged(value) = &event.kind
                {
                    self.text.clone_from(value);
                    return true;
                }
                if ButtonBehavior::new("visit", "Preview")
                    .action(event)
                    .is_some()
                {
                    self.status = format!("Preview: https://{}", self.text);
                    return true;
                }
            }
            Page::InputOtp => {
                if let Some(value) =
                    InputOtp::new("code", "Verification code", &self.text, 6).action(event)
                {
                    self.text = value;
                    self.status = if self.text.len() == 6 {
                        "Code complete"
                    } else {
                        "Keep typing"
                    }
                    .into();
                    return true;
                }
            }
            Page::Combobox => {
                if let Some(action) = self.combobox().action(event) {
                    match action {
                        ComboboxAction::Query(value) => {
                            self.text = value;
                            self.open = true;
                            self.highlighted = None;
                        }
                        ComboboxAction::Open => self.open = true,
                        ComboboxAction::Close => self.open = false,
                        ComboboxAction::Highlight(index) => {
                            self.highlighted = Some(index);
                            cx.scroll(argui::ui::ScrollRequest::reveal(
                                self.combobox().option_key(index),
                            ));
                        }
                        ComboboxAction::Select(index) => {
                            self.selected = index;
                            self.text = self.combobox().options[index].label.clone();
                            self.status = format!("Selected {}", self.text);
                            self.open = false;
                            cx.request_focus("framework");
                        }
                    }
                    return true;
                }
            }
            Page::NativeSelect => {
                if let Some(action) = self.native_select().action(event) {
                    match action {
                        SelectAction::Toggle => self.open = !self.open,
                        SelectAction::Close => self.open = false,
                        SelectAction::Highlight(index) => {
                            self.open = true;
                            self.highlighted = Some(index);
                            cx.request_focus(format!("language::option::{index}"));
                        }
                        SelectAction::Select(index) => {
                            self.selected = index;
                            self.open = false;
                            self.status =
                                format!("Selected {}", self.native_select().options[index].label);
                            cx.request_focus("language");
                        }
                    }
                    return true;
                }
            }
            Page::Questionnaire => {
                if let Some(action) = self.questionnaire_widget().action(event) {
                    self.questionnaire.apply(&action);
                    if matches!(action, QuestionnaireAction::Submit(_)) {
                        self.status = "Your answers are ready.".into();
                    }
                    return true;
                }
            }
            _ => unreachable!("form page"),
        }
        false
    }
}
