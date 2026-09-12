use argui_core::{Color, ColorScheme};
use argui_ui::{ClickEvent, Element, UiEvent, UiEventKind, UiTree};
use argui_widgets::{
    ChoiceMode, Question, QuestionAnswer, QuestionChoice, Questionnaire, QuestionnaireAction,
    shadcn,
};

fn event(key: &str, kind: UiEventKind) -> UiEvent {
    UiEvent::new(
        UiTree::new(Element::container([])).node_ids()[0],
        Some(key.into()),
        kind,
    )
}
fn click(key: &str) -> UiEvent {
    event(key, UiEventKind::Click(ClickEvent::accessibility()))
}

#[test]
fn required_answers_block_progress_optional_questions_skip_and_submit_returns_data() {
    let mut first = Question::new("project", "Project?");
    first.choices = vec![
        QuestionChoice {
            id: "desktop".into(),
            label: "Desktop".into(),
        },
        QuestionChoice {
            id: "web".into(),
            label: "Web".into(),
        },
    ];
    let mut second = Question::new("notes", "Anything else?");
    second.required = false;
    second.mode = ChoiceMode::Multiple;
    let mut survey = Questionnaire::new("s", [first, second]);
    assert_eq!(survey.invalid_questions(), ["project".into()].into());
    let action = survey.action(&click("s::next")).unwrap();
    assert!(matches!(action, QuestionnaireAction::Invalid(_)));
    survey.state.apply(&action);
    assert!(survey.state.invalid.contains("project"));
    assert!(survey.action(&click("s::previous")).is_none());
    assert!(survey.action(&click("s::skip")).is_none());
    let root = survey.build(shadcn(Color::BLACK).resolve(ColorScheme::Light));
    assert!(UiTree::new(root).semantic_diagnostics().is_empty());
    for key in ["s::project::choice::desktop", "s::project::choice::web"] {
        survey.state.apply(&survey.action(&click(key)).unwrap());
    }
    assert_eq!(
        survey.state.answers["project"].choices,
        ["web".into()].into()
    );
    survey
        .state
        .apply(&survey.action(&click("s::next")).unwrap());
    assert_eq!(survey.state.active, 1);
    survey
        .state
        .apply(&survey.action(&click("s::skip")).unwrap());
    assert!(survey.state.answers["notes"].skipped);
    let submit = survey.action(&click("s::submit")).unwrap();
    let QuestionnaireAction::Submit(answers) = &submit else {
        panic!("valid answers")
    };
    assert_eq!(answers.len(), 2);
    survey.state.apply(&submit);
    assert!(survey.state.submitted);
    survey
        .state
        .apply(&survey.action(&click("s::previous")).unwrap());
    assert_eq!(survey.state.active, 0);
    survey.state.apply(
        &survey
            .action(&event(
                "s::project::input",
                UiEventKind::TextChanged("Native editor".into()),
            ))
            .unwrap(),
    );
    assert_eq!(survey.state.answers["project"].text, "Native editor");
    assert!(!survey.state.submitted);
    assert!(
        survey
            .action(&event("unknown", UiEventKind::Focused))
            .is_none()
    );
}

#[test]
fn stale_unknown_multiple_and_duplicate_choices_cannot_bypass_validation() {
    let mut question = Question::new("q", "Choose");
    question.freeform = false;
    question.choices = vec![
        QuestionChoice {
            id: "a".into(),
            label: "A".into(),
        },
        QuestionChoice {
            id: "b".into(),
            label: "B".into(),
        },
    ];
    let mut survey = Questionnaire::new("s", [question]);
    for answer in [
        QuestionAnswer {
            choices: ["unknown".into()].into(),
            ..Default::default()
        },
        QuestionAnswer {
            choices: ["a".into(), "b".into()].into(),
            ..Default::default()
        },
        QuestionAnswer {
            text: "unexpected".into(),
            ..Default::default()
        },
        QuestionAnswer {
            skipped: true,
            ..Default::default()
        },
    ] {
        survey.state.answers.insert("q".into(), answer);
        assert!(!survey.invalid_questions().is_empty());
        assert!(matches!(
            survey.action(&click("s::submit")),
            Some(QuestionnaireAction::Invalid(_))
        ));
    }
    survey.questions[0].mode = ChoiceMode::Multiple;
    survey.state.answers.clear();
    for key in ["s::q::choice::a", "s::q::choice::b", "s::q::choice::a"] {
        survey.state.apply(&survey.action(&click(key)).unwrap());
    }
    assert_eq!(survey.state.answers["q"].choices, ["b".into()].into());
    assert!(survey.invalid_questions().is_empty());
    survey.questions[0].choices.push(QuestionChoice {
        id: "b".into(),
        label: "Duplicate".into(),
    });
    assert!(!survey.invalid_questions().is_empty());
    survey.questions[0].choices.pop();
    survey.questions.push(survey.questions[0].clone());
    assert!(!survey.invalid_questions().is_empty());
    survey.questions.clear();
    assert!(survey.action(&click("s::submit")).is_none());
    assert!(
        survey
            .build(shadcn(Color::BLACK).resolve(ColorScheme::Dark))
            .children
            .is_empty()
    );
}
