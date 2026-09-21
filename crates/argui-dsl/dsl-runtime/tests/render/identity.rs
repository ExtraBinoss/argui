//! Retained visual identity when live vector assets change revision.

use std::collections::HashMap;

use argui_dsl_compiler::{Compiler, SourceModule};
use argui_dsl_runtime::{AssetPayload, LivePackage, LiveRuntime};
use argui_ui::ElementKind;

const SOURCE: &str = r#"import { Svg } from "@argui/native"
export component Main { Svg { source: asset("icon.svg") width: 20px height: 20px } }"#;
const FIRST: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20"><path d="M0 0h20v20H0z"/></svg>"#;
const SECOND: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20"><circle cx="10" cy="10" r="8"/></svg>"#;

fn package(generation: u64, bytes: &[u8]) -> (LivePackage, argui_dsl_ir::ComponentId) {
    let compiled = Compiler::compile(
        [SourceModule::new("ui/main.argui", SOURCE)],
        "ui/main.argui",
        |_| Ok(bytes.to_vec()),
    )
    .unwrap();
    let id = compiled.ir.assets[0].id;
    let root = compiled.roots[0];
    let package = LivePackage::prepare(
        generation,
        compiled.public_api_hash,
        compiled.ir,
        HashMap::from([(id, AssetPayload::new(generation, bytes.to_vec()))]),
    )
    .unwrap();
    (package, root)
}

#[test]
fn live_svg_renders_and_hot_reload_retains_handle_with_new_bytes() {
    let (initial, root) = package(1, FIRST);
    let mut runtime = LiveRuntime::new(initial).unwrap();
    runtime.mount(root, []).unwrap();
    let first = runtime.render().unwrap();
    let ElementKind::Vector {
        vector: first_id, ..
    } = first.kind
    else {
        panic!("expected vector element");
    };
    let (updated, _) = package(2, SECOND);
    let prepared = runtime.prepare_reload(updated).unwrap();
    let outcome = runtime.commit_reload(prepared);
    assert_eq!(outcome.generation, 2);
    let second = runtime.render().unwrap();
    let ElementKind::Vector {
        vector: second_id, ..
    } = second.kind
    else {
        panic!("expected vector element after reload");
    };
    assert_eq!(first_id, second_id);
    let record = runtime.assets().records().next().unwrap();
    assert_eq!(record.revision().get(), 2);
}
