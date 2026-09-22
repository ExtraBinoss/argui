//! AOT emission of only the rows in a fixed-extent virtual window.

use std::fmt::Write;

use argui_dsl_ir::{IrExpressionKind, IrNode, IrPropertyBinding, PropertyTargetId};

use crate::{
    CompilerError,
    codegen::{Context, expression::Scope},
};

/// Borrowed inputs that describe one native virtual-list node during emission.
pub(super) struct VirtualListNode<'a> {
    /// The single keyed repeater or caller-owned template slot.
    pub children: &'a [IrNode],
    /// Native properties that configure the visible row window.
    pub properties: &'a [IrPropertyBinding],
    /// The retained identity used for measured viewport lookup.
    pub identity: &'a str,
}

impl Context<'_> {
    /// Emits the mounted rows for one VirtualWindow and its compiler-owned window metadata.
    ///
    /// `output` receives generated Rust, `node` supplies the repeater, native
    /// properties, and retained identity, `destination` names the row vector,
    /// `depth` controls indentation, and `scope` resolves surrounding bindings.
    /// The mounted row count and start index are emitted beside the row vector.
    ///
    /// # Errors
    ///
    /// Returns a code-generation error for an invalid VirtualWindow body or missing dimensions.
    pub(super) fn emit_virtual_children(
        &self,
        output: &mut String,
        node: VirtualListNode<'_>,
        destination: &str,
        depth: usize,
        scope: &Scope,
    ) -> Result<(), CompilerError> {
        let row_height = self.virtual_property(
            node.properties,
            argui_schema::builtin::ROW_HEIGHT,
            scope,
            None,
        )?;
        let viewport = self.virtual_property(
            node.properties,
            argui_schema::builtin::VIEWPORT_HEIGHT,
            scope,
            Some("0.0"),
        )?;
        let offset = self.virtual_property(
            node.properties,
            argui_schema::builtin::SCROLL_OFFSET,
            scope,
            Some("0.0"),
        )?;
        let overscan = self.virtual_property(
            node.properties,
            argui_schema::builtin::OVERSCAN,
            scope,
            Some("3"),
        )?;
        let pad = "    ".repeat(depth);
        let identity = node.identity;
        writeln!(output, "{pad}let {destination}_viewport = if ({viewport}) > 0.0 {{ {viewport} }} else {{ virtual_viewports.get(&{identity}).copied().unwrap_or(0.0) }};").unwrap();
        let (repeater, rows_scope) = match node.children {
            [repeater @ IrNode::Repeater { .. }] => (repeater, scope),
            [IrNode::Slot { slot, .. }] => {
                let template = scope.template.as_ref().ok_or(CompilerError::Codegen(
                    "VirtualWindow template slot is outside its call site".into(),
                ))?;
                if template.slot != *slot {
                    return Err(CompilerError::Codegen(
                        "VirtualWindow template slot identity does not match its call site".into(),
                    ));
                }
                (&template.repeater, template.caller.as_ref())
            }
            _ => {
                return Err(CompilerError::Codegen(
                    "VirtualWindow requires exactly one keyed `for` repeater".into(),
                ));
            }
        };
        self.emit_virtual_rows(
            output,
            repeater,
            destination,
            depth,
            rows_scope,
            &row_height,
            &format!("{destination}_viewport"),
            &offset,
            &overscan,
        )
    }

    /// Emits the caller's row template after the VirtualWindow configuration is resolved.
    ///
    /// `repeater` is the original keyed loop and `scope` is its lexical owner;
    /// `destination` receives only mounted rows. The remaining expressions are
    /// the effective row height, viewport, offset, and overscan for this window.
    ///
    /// # Errors
    ///
    /// Returns when the repeater is malformed or its caller values cannot be emitted.
    #[allow(clippy::too_many_arguments)]
    fn emit_virtual_rows(
        &self,
        output: &mut String,
        repeater: &IrNode,
        destination: &str,
        depth: usize,
        scope: &Scope,
        row_height: &str,
        viewport: &str,
        offset: &str,
        overscan: &str,
    ) -> Result<(), CompilerError> {
        let IrNode::Repeater {
            local,
            model,
            key,
            body,
            ..
        } = repeater
        else {
            return Err(CompilerError::Codegen(
                "VirtualWindow template requires one keyed `for` repeater".into(),
            ));
        };
        let pad = "    ".repeat(depth);
        let items = format!("{destination}_items");
        let window = format!("{destination}_window");
        let index = format!("{destination}_index");
        let mounted = format!("{destination}_mounted");
        let local_name = format!("local_{}", local.raw());
        let row = format!("{destination}_row");
        if let IrExpressionKind::PropertyRead(property) = &model.kind {
            let source = scope
                .properties
                .get(property)
                .ok_or(CompilerError::Codegen(
                    "VirtualWindow model property is outside its scope".into(),
                ))?;
            writeln!(output, "{pad}let ({destination}_count, {destination}_start, {mounted}) = {source}.with(|{items}| {{").unwrap();
            writeln!(output, "{pad}    let {window} = ::argui::ui::VirtualList::fixed({items}.len(), {row_height}, {viewport}).overscan(usize::try_from({overscan}).unwrap_or(0)).window({offset});").unwrap();
            writeln!(output, "{pad}    let start = {window}.range.start;").unwrap();
            writeln!(output, "{pad}    let {mounted} = {window}.range.map(|{index}| {items}[{index}].clone()).collect::<Vec<_>>();").unwrap();
            writeln!(output, "{pad}    ({items}.len(), start, {mounted})").unwrap();
            writeln!(output, "{pad}}});").unwrap();
        } else {
            writeln!(
                output,
                "{pad}let {items} = {};",
                self.expression(model, scope)?
            )
            .unwrap();
            writeln!(output, "{pad}let {destination}_count = {items}.len();").unwrap();
            writeln!(output, "{pad}let {window} = ::argui::ui::VirtualList::fixed({destination}_count, {row_height}, {viewport}).overscan(usize::try_from({overscan}).unwrap_or(0)).window({offset});").unwrap();
            writeln!(
                output,
                "{pad}let {destination}_start = {window}.range.start;"
            )
            .unwrap();
            writeln!(output, "{pad}let {mounted} = {window}.range.map(|{index}| {items}[{index}].clone()).collect::<Vec<_>>();").unwrap();
        }
        writeln!(output, "{pad}let mut {destination}: Vec<::argui::ui::Element> = Vec::with_capacity({mounted}.len());").unwrap();
        writeln!(output, "{pad}for {local_name} in {mounted} {{").unwrap();
        writeln!(
            output,
            "{pad}    let mut {row}: Vec<::argui::ui::Element> = Vec::new();"
        )
        .unwrap();
        let mut nested = scope.clone();
        nested.locals.insert(*local, local_name);
        self.emit_nodes(output, body, &row, depth + 1, &nested, Some(key))?;
        writeln!(output, "{pad}    {destination}.push(if {row}.len() == 1 {{ {row}.into_iter().next().unwrap() }} else {{ ::argui::ui::Element::column({row}) }});").unwrap();
        writeln!(output, "{pad}}}").unwrap();
        Ok(())
    }

    /// Resolves one VirtualWindow property as a typed Rust expression.
    ///
    /// `properties` are the lowered assignments, `id` selects one property,
    /// `scope` resolves its expression, and `fallback` is used only for optional properties.
    ///
    /// # Errors
    ///
    /// Returns an error for a missing required property or expression emission failure.
    fn virtual_property(
        &self,
        properties: &[IrPropertyBinding],
        id: argui_schema::PropertyId,
        scope: &Scope,
        fallback: Option<&str>,
    ) -> Result<String, CompilerError> {
        let binding = properties
            .iter()
            .find(|binding| binding.target == PropertyTargetId::Native(id));
        match binding {
            Some(binding) => self.expression(&binding.value, scope),
            None => fallback
                .map(str::to_owned)
                .ok_or(CompilerError::Codegen(format!(
                    "VirtualWindow requires property {}",
                    id.raw()
                ))),
        }
    }
}
