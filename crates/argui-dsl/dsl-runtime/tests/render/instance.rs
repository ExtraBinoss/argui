//! Live authored paths and shared generated asset bytes.

use std::collections::HashMap;

use argui_dsl_compiler::{CompiledProject, Compiler, SourceModule};
use argui_dsl_protocol::{LivePackageEnvelope, PackageHeader};
use argui_dsl_runtime::{DslValue, LivePackage, LiveRuntime};
use argui_runtime::Render;
use argui_ui::ElementKind;

/// Compiles one geometry revision and returns both backend products.
fn compile(endpoint: f32) -> CompiledProject {
    Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            format!(
                r#"import {{ Path }} from "@argui/native"
export component Main {{
    private property selected: bool = false
    Path #shape {{
        source: path(20.0, 20.0, [move_to(1.0, 1.0), line_to({endpoint}, 1.0), line_to(19.0, 19.0), close_path()], true, 0.0, false)
        width: 20px
        height: 20px
        color: selected ? #ff0000 : #0000ff
    }}
}}"#
            ),
        )],
        "ui/main.argui",
        |_| Err("path tests do not load external assets".into()),
    )
    .unwrap()
}

/// Inline IR bytes reach the live vector registry without a filesystem asset.
#[test]
fn inline_path_asset_and_dynamic_color_render_in_live_runtime() {
    let compiled = compile(19.0);
    let generated = compiled
        .ir
        .assets
        .iter()
        .find(|asset| asset.inline_bytes.is_some())
        .unwrap();
    let svg = generated.inline_bytes.as_ref().unwrap().clone();
    let root = compiled.roots[0];
    let selected = compiled
        .ir
        .components
        .iter()
        .find(|part| part.id == root)
        .unwrap()
        .properties[0]
        .id;
    let package =
        LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
    let mut runtime = LiveRuntime::new(package).unwrap();
    let instance = runtime.mount(root, []).unwrap();
    let first = runtime.render().unwrap();
    let ElementKind::Vector { color: before, .. } = first.kind else {
        panic!("generic Path must render a vector element");
    };
    assert_eq!(runtime.vector_assets().len(), 1);
    assert_eq!(runtime.vector_assets()[0].svg.as_ref(), svg.as_slice());
    runtime
        .set_property(instance, selected, DslValue::Bool(true))
        .unwrap();
    let second = runtime.render().unwrap();
    let ElementKind::Vector { color: after, .. } = second.kind else {
        panic!("Path remains vector after a color binding update");
    };
    assert_ne!(before, after);
}

/// An empty wire asset list still consumes identical bytes embedded in typed IR.
#[test]
fn inline_path_survives_wire_package_and_geometry_reload() {
    let first = compile(19.0);
    let root = first.roots[0];
    let envelope = LivePackageEnvelope {
        header: PackageHeader::current(first.public_api_hash, 1),
        roots: first.roots,
        ir: first.ir,
        assets: Vec::new(),
    };
    let package = LivePackage::from_envelope(envelope).unwrap();
    let mut runtime = LiveRuntime::new(package).unwrap();
    runtime.mount(root, []).unwrap();
    let first_svg = runtime.vector_assets()[0].svg.clone();
    let next = compile(12.0);
    let next_package =
        LivePackage::prepare(2, next.public_api_hash, next.ir, HashMap::new()).unwrap();
    let prepared = runtime.prepare_reload(next_package).unwrap();
    let _ = runtime.commit_reload(prepared);
    runtime.render().unwrap();
    let second_svg = runtime.vector_assets()[0].svg.clone();
    assert_ne!(first_svg, second_svg);
    assert_eq!(runtime.generation(), 2);
}
