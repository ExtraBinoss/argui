use argui_dsl_parser::{format_source, parse};

#[test]
fn formatter_is_comment_preserving_sorted_and_idempotent() {
    let source = r#"import { Text } from "@argui/ui"
import { Card } from "./z.argui"
// retained component documentation
export component Main { private property title:string="Hello" Card { Text{text:title} } }
"#;

    let formatted = format_source(source);

    assert!(formatted.contains("// retained component documentation"));
    assert!(formatted.find("./z.argui").unwrap() < formatted.find("@argui/ui").unwrap());
    assert!(parse(&formatted).is_valid(), "{formatted}");
    assert_eq!(format_source(&formatted), formatted);
}

#[test]
fn formatter_leaves_recovering_source_byte_exact() {
    let source = "export component Broken { Text { text: }";
    assert_eq!(format_source(source), source);
}

#[test]
fn formatter_handles_nested_blocks_semicolons_and_else_branches() {
    let source = "import { B, A } from \"@argui/ui\";\nexport enum Mode { off,active, }\nexport component Main { property ok: bool = true Column { gap: 1.0; if ok { Text { text: \"yes\"; } } else { Text { text: \"no\"; } } } }\n// keep this comment\n";
    let formatted = format_source(source);
    assert!(formatted.contains("// keep this comment"));
    assert!(formatted.contains("} else {"));
    assert!(formatted.contains("import {"));
    assert!(parse(&formatted).is_valid(), "{formatted}");
    assert_eq!(format_source(&formatted), formatted);
}

#[test]
fn formatter_preserves_empty_and_whitespace_only_sources() {
    assert_eq!(format_source(""), "\n");
    assert_eq!(format_source("  \n"), "\n");
}
