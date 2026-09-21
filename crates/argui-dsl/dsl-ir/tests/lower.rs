use argui_dsl_ir::{
    AssetKind, IrElementTarget, IrExpressionKind, IrNode, IrType, PropertyTargetId, lower,
};
use argui_dsl_semantic::CompilerDatabase;

fn compile(source: &str) -> argui_dsl_ir::IrProject {
    let mut database = CompilerDatabase::with_builtins().unwrap();
    database.set_file("ui/main.argui", source);
    let project = database.check();
    assert!(
        project.is_valid(),
        "unexpected semantic diagnostics: {:#?}",
        project.diagnostics
    );
    let schema = argui_schema::builtin::registry().unwrap();
    lower(&project, &schema).unwrap()
}

fn source(extra_child: &str) -> String {
    format!(
        r#"import {{ Column, Row, Text }} from "@argui/ui"

export struct Item {{ id: string label: string }}

export theme AppTheme {{
    --accent: color = #336699
    --label: string = "Items"
    dark {{ --accent: #ffffff }}
}}

export style Spaced for Row {{
    gap: 8.0
    hover {{ gap: 12.0 }}
}}

export effect Frost {{
    shader: "effects/frost.wgsl"
    parameter amount: float = 0.5
}}

export component Action {{
    callback pressed()
    Text {{ content: "Action" }}
}}

export component Dashboard {{
    in property items: model<Item>
    private property visible: bool = true
    callback activate()
    Row #root {{
        gap: 8.0
        {extra_child}
        Text {{ content: "Heading" }}
        for item in items key item.id {{
            Text {{ content: item.label }}
        }}
        if visible {{ Text {{ content: "Shown" }} }} else {{ Text {{ content: "Hidden" }} }}
        Action {{ on pressed {{ activate() }} }}
        states {{ compact when visible {{ gap: 4.0 }} }}
        animate gap {{ stiffness: 180.0 damping: 20.0 }}
    }}
}}
"#
    )
}

#[test]
fn lowering_resolves_every_runtime_reference_to_stable_ids() {
    let project = compile(&source(""));
    assert_eq!(project.structs.len(), 1);
    assert_eq!(project.enums.len(), 0);
    assert_eq!(project.themes.len(), 1);
    assert_eq!(project.themes[0].modes.len(), 1);
    assert_eq!(project.styles.len(), 1);
    assert_eq!(project.styles[0].states.len(), 1);
    assert_eq!(project.effects.len(), 1);
    assert_eq!(project.effects[0].parameters.len(), 1);
    assert_eq!(project.assets.len(), 1);
    assert_eq!(project.assets[0].kind, AssetKind::Shader);

    let dashboard = project.components.last().unwrap();
    assert_eq!(dashboard.states.len(), 1);
    assert_eq!(dashboard.animations.len(), 1);
    let IrNode::Element {
        target,
        properties,
        children,
        ..
    } = &dashboard.body[0]
    else {
        panic!("dashboard root should be an element");
    };
    assert!(matches!(target, IrElementTarget::Native(_)));
    assert!(matches!(properties[0].target, PropertyTargetId::Native(_)));
    let repeater = children
        .iter()
        .find(|node| {
            matches!(
                node,
                IrNode::Repeater { key, .. }
                    if matches!(key.kind, IrExpressionKind::FieldRead { .. })
            )
        })
        .unwrap();
    let IrNode::Repeater {
        key, body, local, ..
    } = repeater
    else {
        unreachable!()
    };
    assert!(matches!(key.kind, IrExpressionKind::FieldRead { .. }));
    let IrNode::Element { properties, .. } = &body[0] else {
        panic!("repeater body should contain Text");
    };
    let IrExpressionKind::FieldRead { base, .. } = &properties[0].value.kind else {
        panic!("text binding should be a resolved field read");
    };
    assert_eq!(base.kind, IrExpressionKind::LocalRead(*local));
}

#[test]
fn unrelated_sibling_insertions_preserve_existing_source_site_ids() {
    let before = compile(&source(""));
    let after = compile(&source("Column { gap: 2.0 }"));
    let before_root = &before.components.last().unwrap().body[0];
    let after_root = &after.components.last().unwrap().body[0];
    let (
        IrNode::Element {
            site: before_site,
            children: before_children,
            ..
        },
        IrNode::Element {
            site: after_site,
            children: after_children,
            ..
        },
    ) = (before_root, after_root)
    else {
        panic!("roots should be elements");
    };
    assert_eq!(before_site, after_site);
    let before_heading = before_children
        .iter()
        .find(|node| matches!(node, IrNode::Element { target: IrElementTarget::Native(id), .. } if *id == argui_schema::builtin::TEXT))
        .unwrap();
    let after_heading = after_children
        .iter()
        .find(|node| matches!(node, IrNode::Element { target: IrElementTarget::Native(id), .. } if *id == argui_schema::builtin::TEXT))
        .unwrap();
    let (
        IrNode::Element {
            site: before_heading,
            ..
        },
        IrNode::Element {
            site: after_heading,
            ..
        },
    ) = (before_heading, after_heading)
    else {
        unreachable!()
    };
    assert_eq!(before_heading, after_heading);
}

#[test]
fn invalid_semantic_projects_never_produce_partial_ir() {
    let mut database = CompilerDatabase::with_builtins().unwrap();
    database.set_file(
        "ui/broken.argui",
        "export component Broken { Missing { unknown: false } }",
    );
    let project = database.check();
    let schema = argui_schema::builtin::registry().unwrap();
    let errors = lower(&project, &schema).unwrap_err();
    assert!(!errors.is_empty());
}

#[test]
fn named_member_ids_survive_unrelated_insertions_and_change_on_rename() {
    fn member_ids(source: &str) -> (u64, u64, u64, u64) {
        let project = compile(source);
        let structure = &project.structs[0];
        let component = project
            .components
            .iter()
            .find(|component| {
                !component.properties.is_empty()
                    && !component.callbacks.is_empty()
                    && !component.slots.is_empty()
            })
            .unwrap();
        (
            structure.fields.last().unwrap().id.raw(),
            component.properties.last().unwrap().id.raw(),
            component.callbacks.last().unwrap().id.raw(),
            component.slots.last().unwrap().raw(),
        )
    }

    let before = member_ids(
        r#"import { Text } from "@argui/ui"
export struct Data { value: string }
export component Stable {
    private property value: string = "value"
    callback submit()
    slot content
    Text { content: value }
}"#,
    );
    let inserted = member_ids(
        r#"import { Text } from "@argui/ui"
export struct Data { inserted: int value: string }
export component Stable {
    private property inserted: int = 1
    private property value: string = "value"
    callback inserted_callback()
    callback submit()
    slot inserted_slot
    slot content
    Text { content: value }
}"#,
    );
    assert_eq!(before, inserted);

    let renamed = member_ids(
        r#"import { Text } from "@argui/ui"
export struct Data { renamed: string }
export component Stable {
    private property renamed_property: string = "value"
    callback renamed_callback()
    slot renamed_slot
    Text { content: renamed_property }
}"#,
    );
    assert_ne!(before, renamed);
}

#[test]
fn lowering_covers_typed_expressions_assets_and_event_statements() {
    let source = r#"import { Button, Column, Input, Row, Text } from "@argui/ui"
export struct Item { id: int label: string }
export enum Mode { idle active }

export theme Palette {
    --accent: color = #369
    --space: length = 8px
    --derived: length = var(--space)
    dark { --accent: #ffffff --space: 12px }
}

export style RowStyle for Row {
    gap: 8px
    hover { gap: 12px }
}

export effect Glow {
    shader: "effects/glow.wgsl"
    parameter amount: float = 0.5
    parameter tint: color = #ffffff
    parameter fallback: float
}

export component Child {
    in property item: Item
    property highlighted: bool = false
    callback pressed(value: string) -> bool
    slot body
    Button { text: item.label on click { pressed("clicked") } }
}

export style ChildStyle for Child {
    highlighted: true
}

export component Main {
    in property items: model<Item>
    in property selected: Item
        in-out property query: string = ""
    private property count: int = 1 + 2 - 3 * 4 / 5 % 2
    private property ratio: float = 1.5e2 + 2
    private property mixed: float = 2 + 1.5
    private property length_value: length = 4px
    private property percentage_value: percentage = 25%
    private property seconds: duration = 2s
    private property milliseconds: duration = 10ms
    private property degrees: angle = 90deg
    private property radians: angle = 1rad
    private property negated: float = -1.0
    private property positive: int = +1
    private property inverted: bool = !false
    private property logic: bool = true && false || true
    private property compared: bool = 1 < 2 && 3 <= 4 && 5 > 4 && 6 >= 5 && 1 == 1 && 1 != 2
    private property choice: int = inverted ? 1 : 2
    private property rgba: color = #1234
    private property argb: color = #11223344
    private property values: array<int> = [1, 2, 3]
    private property optional: optional<string> = null
    private property vector: asset = asset("icons/icon.svg")
    private property image: asset = asset("images/photo.PNG")
    private property data: asset = asset("data.bin")
    private property package_asset: asset = asset("@shared/icon.svg")
    private property absolute_asset: asset = asset("/shared/icon.svg")
    private property empty_values: array<int> = []
        callback activate()
        callback notify(value: string) -> bool
    slot content

    Column #root {
        gap: 8.0
        Text { content: tr("hello") }
        Text { content: selected.label }
        Input { value <=> query on submit { count += 1 count -= 1 count *= 2 count /= 1 return activate() } }
        for item in items key item.label {
            Child { item: item on pressed { notify(item.label) } }
            Child { item: item }
            for item in items key item.id { Text { content: item.label } }
        }
        for value in [1, 2] key value { Text { content: "array" } }
        if inverted { Text { content: "yes" } } else { if logic { Text { content: "maybe" } } }
        content
        states { compact when inverted { gap: 2.0 } }
        animate gap { duration: 10ms }
        animate missing { duration: 1ms }
    }
}
"#;
    let mut database = CompilerDatabase::with_builtins().unwrap();
    database.set_file("ui/rich.argui", source);
    let project = database.check();
    assert!(
        project.is_valid(),
        "unexpected semantic diagnostics: {:#?}",
        project.diagnostics
    );
    let schema = argui_schema::builtin::registry().unwrap();
    let ir = lower(&project, &schema).unwrap();

    assert_eq!(ir.structs.len(), 1);
    assert_eq!(ir.enums.len(), 1);
    assert_eq!(ir.themes[0].tokens.len(), 3);
    assert_eq!(ir.themes[0].modes.len(), 1);
    assert_eq!(ir.styles[0].states.len(), 1);
    assert!(matches!(
        ir.styles[1].target,
        argui_dsl_ir::IrElementTarget::Component(_)
    ));
    assert!(matches!(
        ir.styles[1].properties[0].target,
        PropertyTargetId::Component(_)
    ));
    assert_eq!(ir.effects[0].parameters.len(), 3);
    assert_eq!(ir.assets.len(), 6);
    assert!(
        ir.assets
            .iter()
            .any(|asset| asset.kind == AssetKind::Shader)
    );
    assert!(
        ir.assets
            .iter()
            .any(|asset| asset.kind == AssetKind::Vector)
    );
    assert!(ir.assets.iter().any(|asset| asset.kind == AssetKind::Image));
    assert!(ir.assets.iter().any(|asset| asset.kind == AssetKind::Other));

    let main = ir
        .components
        .iter()
        .find(|component| component.properties.len() > 10)
        .unwrap();
    assert_eq!(main.states.len(), 1);
    assert_eq!(main.animations.len(), 1);
    assert!(
        main.properties
            .iter()
            .any(|property| matches!(property.value_type, IrType::Array(_)))
    );
    assert!(
        main.properties
            .iter()
            .any(|property| matches!(property.value_type, IrType::Optional(_)))
    );

    let mut found_repeater = false;
    let mut found_conditional = false;
    let mut found_slot = false;
    let mut found_two_way = false;
    let mut found_event = false;
    fn inspect(
        nodes: &[IrNode],
        found_repeater: &mut bool,
        found_conditional: &mut bool,
        found_slot: &mut bool,
        found_two_way: &mut bool,
        found_event: &mut bool,
    ) {
        for node in nodes {
            match node {
                IrNode::Element {
                    properties,
                    events,
                    children,
                    ..
                } => {
                    *found_two_way |= properties.iter().any(|property| property.two_way);
                    *found_event |= !events.is_empty();
                    inspect(
                        children,
                        found_repeater,
                        found_conditional,
                        found_slot,
                        found_two_way,
                        found_event,
                    );
                }
                IrNode::Repeater { body, key, .. } => {
                    *found_repeater = true;
                    assert!(matches!(
                        key.kind,
                        IrExpressionKind::FieldRead { .. }
                            | IrExpressionKind::LocalRead(_)
                            | IrExpressionKind::Constant(_)
                    ));
                    inspect(
                        body,
                        found_repeater,
                        found_conditional,
                        found_slot,
                        found_two_way,
                        found_event,
                    );
                }
                IrNode::Conditional {
                    then_body,
                    else_body,
                    ..
                } => {
                    *found_conditional = true;
                    inspect(
                        then_body,
                        found_repeater,
                        found_conditional,
                        found_slot,
                        found_two_way,
                        found_event,
                    );
                    inspect(
                        else_body,
                        found_repeater,
                        found_conditional,
                        found_slot,
                        found_two_way,
                        found_event,
                    );
                }
                IrNode::Slot { .. } => *found_slot = true,
            }
        }
    }
    inspect(
        &main.body,
        &mut found_repeater,
        &mut found_conditional,
        &mut found_slot,
        &mut found_two_way,
        &mut found_event,
    );
    assert!(found_repeater);
    assert!(found_conditional);
    assert!(found_slot);
    assert!(found_two_way);
    assert!(found_event);
}

#[test]
fn lowering_resolves_relative_assets_from_a_root_module() {
    let mut database = CompilerDatabase::with_builtins().unwrap();
    database.set_file(
        "main.argui",
        r#"export effect RootEffect {
    shader: "effects/root.wgsl"
}"#,
    );
    let project = database.check();
    assert!(project.is_valid(), "unexpected semantic diagnostics");
    let schema = argui_schema::builtin::registry().unwrap();
    let ir = lower(&project, &schema).unwrap();

    assert_eq!(ir.assets.len(), 1);
    assert_eq!(ir.assets[0].path, "effects/root.wgsl");
    assert_eq!(ir.assets[0].kind, AssetKind::Shader);
}

#[test]
fn lowering_rejects_colliding_explicit_visual_ids() {
    let mut database = CompilerDatabase::with_builtins().unwrap();
    database.set_file(
        "ui/colliding.argui",
        r#"import { Text } from "@argui/ui"
export component Colliding {
    Text #same { content: "first" }
    Text #same { content: "second" }
}"#,
    );
    let project = database.check();
    assert!(project.is_valid(), "unexpected semantic diagnostics");
    let schema = argui_schema::builtin::registry().unwrap();
    let errors = lower(&project, &schema).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("source sites"))
    );
}
