use argui_dsl_parser::lexer::lex;
use argui_dsl_syntax::SyntaxKind;

#[test]
fn lexer_retains_comments_whitespace_units_unicode_and_longest_operators() {
    let source = "/* outer /* nested */ ok */\nélément { width: 12px; ratio: 50%; value <=> model } // tail\n";
    let lexed = lex(source);
    assert!(lexed.diagnostics.is_empty());
    let kinds = lexed
        .tokens
        .iter()
        .map(|token| token.kind)
        .collect::<Vec<_>>();
    assert!(kinds.contains(&SyntaxKind::BlockComment));
    assert!(kinds.contains(&SyntaxKind::LineComment));
    assert!(kinds.contains(&SyntaxKind::Number));
    assert!(kinds.contains(&SyntaxKind::TwoWay));
    let reconstructed = lexed
        .tokens
        .iter()
        .map(|token| &source[std::ops::Range::<usize>::from(token.range)])
        .collect::<String>();
    assert_eq!(reconstructed, source);
}

#[test]
fn lexer_reports_invalid_and_unterminated_input_without_dropping_it() {
    let source = "§ /* open";
    let lexed = lex(source);
    assert_eq!(lexed.diagnostics.len(), 2);
    let reconstructed = lexed
        .tokens
        .iter()
        .map(|token| &source[std::ops::Range::<usize>::from(token.range)])
        .collect::<String>();
    assert_eq!(reconstructed, source);
}

#[test]
fn lexer_recognizes_keywords_numbers_units_and_every_compound_operator() {
    let source = concat!(
        "import from as export struct enum component theme style effect for in key ",
        "if else property out in-out private callback slot on states when animate ",
        "parameter shader true false null let return ",
        "1_000 .5 2e-3 4ms 5s 6deg 7rad 8px 9% ",
        "<=> -> => == != <= >= && || += -= *= /= ",
        "{}()[],:;.#@+-*/%!<>=?"
    );
    let lexed = lex(source);
    assert!(lexed.diagnostics.is_empty(), "{:#?}", lexed.diagnostics);
    let kinds = lexed
        .tokens
        .iter()
        .map(|token| token.kind)
        .collect::<Vec<_>>();
    for kind in [
        SyntaxKind::ImportKw,
        SyntaxKind::FromKw,
        SyntaxKind::AsKw,
        SyntaxKind::ExportKw,
        SyntaxKind::StructKw,
        SyntaxKind::EnumKw,
        SyntaxKind::ComponentKw,
        SyntaxKind::ThemeKw,
        SyntaxKind::StyleKw,
        SyntaxKind::EffectKw,
        SyntaxKind::ForKw,
        SyntaxKind::InKw,
        SyntaxKind::KeyKw,
        SyntaxKind::IfKw,
        SyntaxKind::ElseKw,
        SyntaxKind::PropertyKw,
        SyntaxKind::OutKw,
        SyntaxKind::InOutKw,
        SyntaxKind::PrivateKw,
        SyntaxKind::CallbackKw,
        SyntaxKind::SlotKw,
        SyntaxKind::OnKw,
        SyntaxKind::StatesKw,
        SyntaxKind::WhenKw,
        SyntaxKind::AnimateKw,
        SyntaxKind::ParameterKw,
        SyntaxKind::ShaderKw,
        SyntaxKind::TrueKw,
        SyntaxKind::FalseKw,
        SyntaxKind::NullKw,
        SyntaxKind::LetKw,
        SyntaxKind::ReturnKw,
        SyntaxKind::Number,
        SyntaxKind::TwoWay,
        SyntaxKind::Arrow,
        SyntaxKind::FatArrow,
        SyntaxKind::EqEq,
        SyntaxKind::BangEq,
        SyntaxKind::LtEq,
        SyntaxKind::GtEq,
        SyntaxKind::AndAnd,
        SyntaxKind::OrOr,
        SyntaxKind::PlusEq,
        SyntaxKind::MinusEq,
        SyntaxKind::StarEq,
        SyntaxKind::SlashEq,
    ] {
        assert!(kinds.contains(&kind), "missing token kind {kind:?}");
    }
    let reconstructed = lexed
        .tokens
        .iter()
        .map(|token| &source[std::ops::Range::<usize>::from(token.range)])
        .collect::<String>();
    assert_eq!(reconstructed, source);
}
