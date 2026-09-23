use argui_dsl_ir::IrType;
use argui_dsl_semantic::Type;

use crate::{CompilerError, codegen::Context};

impl Context<'_> {
    /// Converts a semantic DSL type to its generated Rust representation.
    pub(super) fn rust_type(&self, value: &Type) -> Result<String, CompilerError> {
        Ok(match value {
            Type::Unknown => return Err(CompilerError::Codegen("unknown public type".into())),
            Type::Void => "()".into(),
            Type::Bool => "bool".into(),
            Type::Int => "i64".into(),
            Type::Float
            | Type::Length
            | Type::Percentage
            | Type::Duration
            | Type::Angle
            | Type::FontSize
            | Type::LineHeight => "f32".into(),
            Type::String | Type::FontFamily | Type::FontWeight => "String".into(),
            Type::Color => "::argui::core::Color".into(),
            Type::Brush => "::argui::paint::Fill".into(),
            Type::Dimension => "::argui::ui::Dimension".into(),
            Type::Radii => "::argui::paint::CornerRadii".into(),
            Type::Insets => "::argui::core::Insets".into(),
            Type::Border => "::argui::paint::Border".into(),
            Type::Shadow => "::argui::paint::Shadow".into(),
            Type::Transform => "::argui::core::Transform2D".into(),
            Type::Asset => "::argui::schema::AssetHandle".into(),
            Type::Struct(symbol) | Type::Enum(symbol) => self
                .definitions
                .get(symbol)
                .map(|definition| type_name(&definition.name))
                .ok_or(CompilerError::Codegen(format!(
                    "unknown user type {symbol}"
                )))?,
            Type::Optional(inner) => format!("Option<{}>", self.rust_type(inner)?),
            Type::Array(inner) => format!("Vec<{}>", self.rust_type(inner)?),
            Type::Model(inner) => format!("::argui::reactive::Model<{}>", self.rust_type(inner)?),
            Type::Callback { parameters, result } => format!(
                "Rc<dyn Fn({}) -> {}>",
                parameters
                    .iter()
                    .map(|parameter| self.rust_type(parameter))
                    .collect::<Result<Vec<_>, _>>()?
                    .join(", "),
                self.rust_type(result)?
            ),
        })
    }

    /// Converts an IR type to generated Rust.
    pub(super) fn ir_rust_type(&self, value: &IrType) -> Result<String, CompilerError> {
        let semantic = match value {
            IrType::Unknown => Type::Unknown,
            IrType::Void => Type::Void,
            IrType::Bool => Type::Bool,
            IrType::Int => Type::Int,
            IrType::Float => Type::Float,
            IrType::String => Type::String,
            IrType::Color => Type::Color,
            IrType::Brush => Type::Brush,
            IrType::Length => Type::Length,
            IrType::Dimension => Type::Dimension,
            IrType::Percentage => Type::Percentage,
            IrType::Duration => Type::Duration,
            IrType::Angle => Type::Angle,
            IrType::Radii => Type::Radii,
            IrType::Insets => Type::Insets,
            IrType::Border => Type::Border,
            IrType::Shadow => Type::Shadow,
            IrType::FontFamily => Type::FontFamily,
            IrType::FontWeight => Type::FontWeight,
            IrType::FontSize => Type::FontSize,
            IrType::LineHeight => Type::LineHeight,
            IrType::Transform => Type::Transform,
            IrType::Asset => Type::Asset,
            IrType::Struct { symbol, .. } => Type::Struct(*symbol),
            IrType::Enum(symbol) => Type::Enum(*symbol),
            IrType::Optional(inner) => {
                return Ok(format!("Option<{}>", self.ir_rust_type(inner)?));
            }
            IrType::Array(inner) => {
                return Ok(format!("Vec<{}>", self.ir_rust_type(inner)?));
            }
            IrType::Model(inner) => {
                return Ok(format!(
                    "::argui::reactive::Model<{}>",
                    self.ir_rust_type(inner)?
                ));
            }
            IrType::Callback { parameters, result } => {
                return Ok(format!(
                    "Rc<dyn Fn({}) -> {}>",
                    parameters
                        .iter()
                        .map(|parameter| self.ir_rust_type(parameter))
                        .collect::<Result<Vec<_>, _>>()?
                        .join(", "),
                    self.ir_rust_type(result)?
                ));
            }
        };
        self.rust_type(&semantic)
    }

    /// Emits the boxed callback-cell type used by generated event bridges.
    pub(super) fn callback_type(
        &self,
        callback: &argui_dsl_semantic::CallbackDefinition,
    ) -> Result<String, CompilerError> {
        let parameters = callback
            .parameters
            .iter()
            .map(|parameter| self.rust_type(&parameter.value_type))
            .collect::<Result<Vec<_>, _>>()?
            .join(", ");
        let result = if callback.result == Type::Void {
            String::new()
        } else {
            format!(" -> {}", self.rust_type(&callback.result)?)
        };
        Ok(format!(
            "Rc<RefCell<Option<Box<dyn Fn({parameters}){result}>>>>"
        ))
    }

    /// Returns a total default expression for an internal generated property.
    pub(super) fn default_value(&self, value: &Type) -> Result<String, CompilerError> {
        Ok(match value {
            Type::Bool => "false".into(),
            Type::Int => "0_i64".into(),
            Type::Float
            | Type::Length
            | Type::Percentage
            | Type::Duration
            | Type::Angle
            | Type::FontSize
            | Type::LineHeight => "0.0_f32".into(),
            Type::String | Type::FontFamily | Type::FontWeight => "String::new()".into(),
            Type::Color => "::argui::core::Color::TRANSPARENT".into(),
            Type::Asset => {
                "::argui::schema::AssetHandle::Image(::argui::paint::ImageId::fresh())".into()
            }
            Type::Array(_) => "Vec::new()".into(),
            Type::Model(_) => "::argui::reactive::Model::default()".into(),
            Type::Optional(_) => "None".into(),
            Type::Void => "()".into(),
            Type::Unknown | Type::Callback { .. } => {
                return Err(CompilerError::Codegen(format!(
                    "type `{value}` requires an explicit initializer"
                )));
            }
            _ => "Default::default()".into(),
        })
    }
}

/// Produces a legal snake-case Rust identifier without preserving DSL punctuation.
pub(super) fn rust_identifier(value: &str) -> String {
    let mut result = String::new();
    for (index, character) in value.chars().enumerate() {
        let valid = character.is_alphanumeric() || character == '_';
        if index == 0 && character.is_ascii_digit() {
            result.push('_');
        }
        result.push(if valid { character } else { '_' });
    }
    if result.is_empty() {
        result.push('_');
    }
    if is_keyword(&result) {
        result.push('_');
    }
    result
}

/// Produces a PascalCase Rust type or variant name.
pub(super) fn type_name(value: &str) -> String {
    let mut result = String::new();
    let mut uppercase = true;
    for character in value.chars() {
        if character.is_alphanumeric() {
            if uppercase {
                result.extend(character.to_uppercase());
                uppercase = false;
            } else {
                result.push(character);
            }
        } else {
            uppercase = true;
        }
    }
    if result.is_empty() {
        result.push_str("Generated");
    }
    if result.starts_with(|character: char| character.is_ascii_digit()) {
        result.insert(0, '_');
    }
    result
}

/// Maps a DSL effect parameter type to renderer metadata syntax.
pub(super) fn effect_parameter_type(value: &Type) -> Result<&'static str, CompilerError> {
    Ok(match value {
        Type::Float => "::argui::render::EffectParameterType::F32",
        Type::Int => "::argui::render::EffectParameterType::I32",
        Type::Bool => "::argui::render::EffectParameterType::Bool",
        Type::Color => "::argui::render::EffectParameterType::Color",
        Type::Length => "::argui::render::EffectParameterType::LogicalPixels",
        _ => {
            return Err(CompilerError::Codegen(format!(
                "unsupported WGSL parameter type `{value}`"
            )));
        }
    })
}

/// Returns whether a normalized identifier is reserved by Rust.
fn is_keyword(value: &str) -> bool {
    matches!(
        value,
        "as" | "break"
            | "const"
            | "continue"
            | "crate"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "async"
            | "await"
            | "dyn"
    )
}
