use crate::{
    Button, ButtonBehavior, ChoiceMode, Input, Toggle, Typography, TypographyVariant, WidgetTheme,
};
use argui_ui::{Element, EventHandler, EventType, UiEvent, UiEventKind, ValueHandler};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuestionChoice {
    pub id: String,
    pub label: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Question {
    pub id: String,
    pub prompt: String,
    pub description: String,
    pub choices: Vec<QuestionChoice>,
    pub mode: ChoiceMode,
    pub required: bool,
    pub freeform: bool,
}

impl Question {
    /// Creates a required single-choice question with freeform input enabled.
    /// `id` is its stable identity and `prompt` is displayed to the participant.
    #[must_use]
    pub fn new(id: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            prompt: prompt.into(),
            description: String::new(),
            choices: Vec::new(),
            mode: ChoiceMode::Single,
            required: true,
            freeform: true,
        }
    }

    fn accepts(&self, answer: &QuestionAnswer) -> bool {
        if answer.skipped {
            return !self.required && answer.choices.is_empty() && answer.text.is_empty();
        }
        (!self.required || !answer.choices.is_empty() || !answer.text.trim().is_empty())
            && (self.mode == ChoiceMode::Multiple || answer.choices.len() <= 1)
            && (self.freeform || answer.text.is_empty())
            && answer
                .choices
                .iter()
                .all(|id| self.choices.iter().any(|choice| choice.id == *id))
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct QuestionAnswer {
    pub choices: BTreeSet<String>,
    pub text: String,
    pub skipped: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct QuestionnaireState {
    pub active: usize,
    pub answers: BTreeMap<String, QuestionAnswer>,
    pub invalid: BTreeSet<String>,
    pub submitted: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QuestionnaireAction {
    Answer { id: String, answer: QuestionAnswer },
    Navigate(usize),
    Skip { id: String, next: usize },
    Invalid(BTreeSet<String>),
    Submit(BTreeMap<String, QuestionAnswer>),
}

impl QuestionnaireState {
    /// Applies a generated action to the retained answers and navigation state.
    pub fn apply(&mut self, action: &QuestionnaireAction) {
        self.submitted = false;
        match action {
            QuestionnaireAction::Answer { id, answer } => {
                self.invalid.remove(id);
                self.answers.insert(id.clone(), answer.clone());
            }
            QuestionnaireAction::Navigate(index) => self.active = *index,
            QuestionnaireAction::Skip { id, next } => {
                self.answers.insert(
                    id.clone(),
                    QuestionAnswer {
                        skipped: true,
                        ..Default::default()
                    },
                );
                self.invalid.remove(id);
                self.active = *next;
            }
            QuestionnaireAction::Invalid(ids) => self.invalid.clone_from(ids),
            QuestionnaireAction::Submit(_) => self.submitted = true,
        }
    }
}

/// Controlled multi-step questions. Submission returns values to the caller without sending them.
#[derive(Clone, Debug)]
pub struct Questionnaire {
    pub key: String,
    pub questions: Vec<Question>,
    pub state: QuestionnaireState,
    pub previous_label: String,
    pub next_label: String,
    pub skip_label: String,
    pub submit_label: String,
    pub answer_label: String,
    pub error_label: String,
    input_handlers: Vec<ValueHandler<String>>,
    choice_handlers: Vec<ValueHandler<Vec<String>>>,
    navigate_handlers: Vec<ValueHandler<usize>>,
    submit_handlers: Vec<EventHandler>,
}

impl Questionnaire {
    /// Creates a multi-step questionnaire from questions in display order.
    /// `key` scopes the generated question and control identities.
    #[must_use]
    pub fn new(key: impl Into<String>, questions: impl IntoIterator<Item = Question>) -> Self {
        Self {
            key: key.into(),
            questions: questions.into_iter().collect(),
            state: QuestionnaireState::default(),
            previous_label: "Previous".into(),
            next_label: "Next".into(),
            skip_label: "Skip".into(),
            submit_label: "Submit".into(),
            answer_label: "Your answer".into(),
            error_label: "Choose or enter a valid answer.".into(),
            input_handlers: Vec::new(),
            choice_handlers: Vec::new(),
            navigate_handlers: Vec::new(),
            submit_handlers: Vec::new(),
        }
    }

    /// Adds a handler receiving the active question's edited freeform answer.
    #[must_use]
    pub fn on_input(mut self, handler: ValueHandler<String>) -> Self {
        self.input_handlers.push(handler);
        self
    }

    /// Adds a handler receiving `[question_id, choice_id, selected]` for a choice change.
    #[must_use]
    pub fn on_choice(mut self, handler: ValueHandler<Vec<String>>) -> Self {
        self.choice_handlers.push(handler);
        self
    }

    /// Adds a handler receiving the requested controlled question index.
    #[must_use]
    pub fn on_navigate(mut self, handler: ValueHandler<usize>) -> Self {
        self.navigate_handlers.push(handler);
        self
    }

    /// Adds a handler invoked when a valid final submission is requested.
    ///
    /// The callback reads the controlled answers from application state.
    #[must_use]
    pub fn on_submit(mut self, handler: EventHandler) -> Self {
        self.submit_handlers.push(handler);
        self
    }

    /// Reject duplicate identifiers and invalid retained answers before submission.
    #[must_use]
    /// Returns question identities that prevent submission due to invalid answers or duplicate ids.
    pub fn invalid_questions(&self) -> BTreeSet<String> {
        let mut seen = BTreeSet::new();
        self.questions
            .iter()
            .filter(|question| {
                let mut choices = BTreeSet::new();
                !seen.insert(&question.id)
                    || question
                        .choices
                        .iter()
                        .any(|choice| !choices.insert(&choice.id))
                    || !question.accepts(
                        self.state
                            .answers
                            .get(&question.id)
                            .unwrap_or(&QuestionAnswer::default()),
                    )
            })
            .map(|question| question.id.clone())
            .collect()
    }

    #[must_use]
    /// Interprets `event` as an answer, navigation, skip or submission action.
    pub fn action(&self, event: &UiEvent) -> Option<QuestionnaireAction> {
        let index = self.state.active.min(self.questions.len().checked_sub(1)?);
        let question = &self.questions[index];
        let key = event.target_key()?;
        let prefix = format!("{}::{}", self.key, question.id);
        let mut answer = self
            .state
            .answers
            .get(&question.id)
            .cloned()
            .unwrap_or_default();
        if key == format!("{prefix}::input")
            && question.freeform
            && let UiEventKind::TextChanged(text) = &event.kind
        {
            answer.text.clone_from(text);
            answer.skipped = false;
            return Some(QuestionnaireAction::Answer {
                id: question.id.clone(),
                answer,
            });
        }
        for choice in &question.choices {
            let toggle = Toggle::new(
                format!("{prefix}::choice::{}", choice.id),
                &choice.label,
                answer.choices.contains(&choice.id),
            );
            if let Some(pressed) = toggle.action(event) {
                if question.mode == ChoiceMode::Single {
                    answer.choices.clear();
                }
                if pressed {
                    answer.choices.insert(choice.id.clone());
                } else {
                    answer.choices.remove(&choice.id);
                }
                answer.skipped = false;
                return Some(QuestionnaireAction::Answer {
                    id: question.id.clone(),
                    answer,
                });
            }
        }
        let clicked = |part: &str| {
            ButtonBehavior::new(format!("{}::{part}", self.key), part)
                .action(event)
                .is_some()
        };
        if clicked("previous") && index > 0 {
            return Some(QuestionnaireAction::Navigate(index - 1));
        }
        if clicked("skip") && !question.required {
            return Some(QuestionnaireAction::Skip {
                id: question.id.clone(),
                next: (index + 1).min(self.questions.len() - 1),
            });
        }
        if clicked("next") && index + 1 < self.questions.len() {
            return Some(if question.accepts(&answer) {
                QuestionnaireAction::Navigate(index + 1)
            } else {
                QuestionnaireAction::Invalid([question.id.clone()].into())
            });
        }
        if clicked("submit") && index + 1 == self.questions.len() {
            let invalid = self.invalid_questions();
            return Some(if invalid.is_empty() {
                QuestionnaireAction::Submit(
                    self.questions
                        .iter()
                        .map(|question| {
                            (
                                question.id.clone(),
                                self.state
                                    .answers
                                    .get(&question.id)
                                    .cloned()
                                    .unwrap_or_default(),
                            )
                        })
                        .collect(),
                )
            } else {
                QuestionnaireAction::Invalid(invalid)
            });
        }
        None
    }

    #[must_use]
    /// Builds the active questionnaire step and controls using `theme` for styling.
    pub fn build(&self, theme: &WidgetTheme) -> Element {
        let index = self
            .state
            .active
            .min(self.questions.len().saturating_sub(1));
        let Some(question) = self.questions.get(index) else {
            return Element::column([]).keyed(&self.key);
        };
        let prefix = format!("{}::{}", self.key, question.id);
        let answer = self
            .state
            .answers
            .get(&question.id)
            .cloned()
            .unwrap_or_default();
        let title_key = format!("{prefix}::title");
        let error_key = format!("{prefix}::error");
        let mut children = vec![
            Typography::new(
                format!("{} / {}", index + 1, self.questions.len()),
                TypographyVariant::Muted,
            )
            .build(theme),
            Typography::new(&question.prompt, TypographyVariant::Heading(3))
                .build(theme)
                .keyed(&title_key),
            Typography::new(&question.description, TypographyVariant::Muted).build(theme),
        ];
        children.extend(question.choices.iter().map(|choice| {
            let mut toggle = Toggle::new(
                format!("{prefix}::choice::{}", choice.id),
                &choice.label,
                answer.choices.contains(&choice.id),
            );
            toggle.outline = true;
            let mut element = toggle.build(theme);
            for handler in &self.choice_handlers {
                element = element.on(handler.direct_listener_value(
                    EventType::Click,
                    vec![
                        question.id.clone(),
                        choice.id.clone(),
                        (!answer.choices.contains(&choice.id)).to_string(),
                    ],
                ));
            }
            element
        }));
        if question.freeform {
            let mut builder = Input::new(
                format!("{prefix}::input"),
                &answer.text,
                &self.answer_label,
                theme.input(),
            )
            .label(&self.answer_label)
            .invalid(self.state.invalid.contains(&question.id));
            for handler in &self.input_handlers {
                builder = builder.on_input(*handler);
            }
            let mut input = builder.build().labelled_by([title_key.clone()]);
            if self.state.invalid.contains(&question.id) {
                input = input.described_by([error_key.clone()]);
            }
            children.push(input);
        }
        if self.state.invalid.contains(&question.id) {
            children.push(
                Typography::new(&self.error_label, TypographyVariant::Small)
                    .build(theme)
                    .keyed(error_key)
                    .semantics(
                        argui_ui::Semantics::new(argui_ui::Role::Alert)
                            .label(&self.error_label)
                            .live(argui_ui::LiveRegion::Polite),
                    ),
            );
        }
        let mut previous = Button::new(
            format!("{}::previous", self.key),
            &self.previous_label,
            theme.outline_button(),
        )
        .enabled(index > 0)
        .build();
        if index > 0 {
            for handler in &self.navigate_handlers {
                previous = previous.on(handler.direct_listener_value(EventType::Click, index - 1));
            }
        }
        let next_index = (index + 1).min(self.questions.len().saturating_sub(1));
        let mut skip = Button::new(
            format!("{}::skip", self.key),
            &self.skip_label,
            theme.ghost_button(),
        )
        .enabled(!question.required)
        .build();
        if !question.required {
            for handler in &self.navigate_handlers {
                skip = skip.on(handler.direct_listener_value(EventType::Click, next_index));
            }
        }
        let next = if index + 1 < self.questions.len() {
            let mut next = Button::new(
                format!("{}::next", self.key),
                &self.next_label,
                theme.button(),
            )
            .build();
            if question.accepts(&answer) {
                for handler in &self.navigate_handlers {
                    next = next.on(handler.direct_listener_value(EventType::Click, index + 1));
                }
            }
            next
        } else {
            let mut submit = Button::new(
                format!("{}::submit", self.key),
                &self.submit_label,
                theme.button(),
            )
            .build();
            if self.invalid_questions().is_empty() {
                for handler in &self.submit_handlers {
                    submit = submit.on(handler.direct_listener(EventType::Click));
                }
            }
            submit
        };
        children.push(
            Element::row([previous, skip, next])
                .gap(8.0)
                .flex_wrap(argui_ui::FlexWrap::Wrap),
        );
        Element::column(children)
            .keyed(&self.key)
            .gap(12.0)
            .semantics(argui_ui::Semantics::new(argui_ui::Role::Group))
            .labelled_by([title_key])
    }
}
