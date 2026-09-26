//! Parsing of ordered CSS-style backdrop filters into renderer-independent paint filters.

use argui_core::Color;
use argui_paint::{
    EffectArgument, EffectId, EffectInstance, EffectValue, Filter, Refraction, Shadow,
};

/// Parses `value` as a space-separated CSS-style filter function list.
///
/// Returns filters in source order. `none`, `initial`, and `unset` return an empty list.
/// SVG `url()` filters and cascade-dependent global keywords cannot be resolved here.
///
/// # Errors
///
/// Returns a descriptive error for malformed input, unsupported units or colors, SVG URLs,
/// and cascade-dependent keywords.
pub fn parse(value: &str) -> Result<Vec<Filter>, String> {
    let value = value.trim();
    if matches!(value, "none" | "initial" | "unset") {
        return Ok(Vec::new());
    }
    if matches!(value, "inherit" | "revert" | "revert-layer") {
        return Err(format!("{value} requires CSS cascade context"));
    }
    if value.is_empty() {
        return Err("expected a backdrop-filter function or none".into());
    }
    let mut filters = Vec::new();
    let mut rest = value;
    while !rest.trim_start().is_empty() {
        rest = rest.trim_start();
        let name_end = rest
            .bytes()
            .take_while(|byte| byte.is_ascii_alphabetic() || *byte == b'-')
            .count();
        if name_end == 0 {
            return Err(format!("expected filter function near `{rest}`"));
        }
        let name = &rest[..name_end];
        rest = &rest[name_end..];
        if !rest.starts_with('(') {
            return Err(format!("expected `(` after {name}"));
        }
        let (arguments, tail) = function_arguments(rest, name)?;
        filters.push(parse_function(name, arguments.trim())?);
        if !tail.is_empty() && !tail.starts_with(char::is_whitespace) {
            return Err(format!("expected whitespace after {name}()"));
        }
        rest = tail;
    }
    Ok(filters)
}

/// Returns the arguments and following text for a balanced function call in `text`.
/// `name` identifies the function in an error. Quoted strings keep internal parentheses literal.
///
/// # Errors
///
/// Returns an error for an unterminated call or quoted string.
fn function_arguments<'a>(text: &'a str, name: &str) -> Result<(&'a str, &'a str), String> {
    let mut depth = 0_u32;
    let mut quote = None;
    for (index, character) in text.char_indices() {
        if let Some(delimiter) = quote {
            if character == delimiter {
                quote = None;
            }
            continue;
        }
        match character {
            '\'' | '"' => quote = Some(character),
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Ok((&text[1..index], &text[index + 1..]));
                }
            }
            _ => {}
        }
    }
    Err(format!("unterminated {name}()"))
}

/// Converts one named function and its `arguments` to an engine filter.
///
/// # Errors
///
/// Returns an error when a function, unit, value or argument count is unsupported.
fn parse_function(name: &str, arguments: &str) -> Result<Filter, String> {
    match name {
        "blur" => Ok(Filter::Blur(length(arguments, false)?)),
        "brightness" => Ok(Filter::Brightness(ratio(arguments, false)?)),
        "contrast" => Ok(Filter::Contrast(ratio(arguments, false)?)),
        "saturate" => Ok(Filter::Saturation(ratio(arguments, false)?)),
        "opacity" => Ok(Filter::Opacity(ratio(arguments, true)?)),
        "hue-rotate" => Ok(Filter::HueRotate(angle(arguments)?)),
        "grayscale" => Ok(Filter::ColorMatrix(saturation(
            1.0 - ratio(arguments, true)?,
        ))),
        "invert" => Ok(Filter::ColorMatrix(invert(ratio(arguments, true)?))),
        "sepia" => Ok(Filter::ColorMatrix(sepia(ratio(arguments, true)?))),
        "drop-shadow" => Ok(Filter::DropShadow(drop_shadow(arguments)?)),
        "refraction" => Ok(Filter::Refraction(refraction(arguments)?)),
        "effect" => Ok(Filter::Effect(custom_effect(arguments)?)),
        "color-matrix" => Ok(Filter::ColorMatrix(color_matrix(arguments)?)),
        "url" => Err("SVG url() backdrop filters are unsupported".into()),
        _ => Err(format!("unsupported backdrop-filter function `{name}`")),
    }
}

/// Parses a registered custom effect and its live scalar parameters.
///
/// `arguments` starts with a namespaced effect ID followed by zero or more
/// `name=value` finite f32 parameters. Returns the effect instance used by
/// the renderer, or an error for malformed names and values.
///
/// # Errors
/// Returns an error when the ID, parameter syntax, or scalar is invalid.
fn custom_effect(arguments: &str) -> Result<EffectInstance, String> {
    let mut parts = arguments.split_whitespace();
    let id = parts.next().ok_or("effect() needs an identifier")?;
    if !valid_effect_name(id) || !id.contains('.') {
        return Err("effect() needs a namespaced identifier".into());
    }
    let parameters = parts
        .map(|part| {
            let (name, value) = part
                .split_once('=')
                .ok_or("effect() parameters need name=value")?;
            if !valid_effect_name(name) {
                return Err("invalid effect() parameter name".into());
            }
            Ok(EffectArgument::from_owned(
                name.to_string(),
                EffectValue::F32(finite(value)?),
            ))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(EffectInstance::new(
        EffectId::from_owned(id.to_string()),
        parameters,
    ))
}

/// Returns whether `name` uses only identifier characters accepted by effect IDs.
///
/// `name` is an effect or parameter identifier. Returns false for empty names
/// or punctuation that would make the filter syntax ambiguous.
fn valid_effect_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

/// Parses `text` as a finite CSS length in logical pixels; `negative` permits offsets.
///
/// # Errors
///
/// Returns an error for non-pixel units, invalid numbers, or disallowed negative values.
fn length(text: &str, negative: bool) -> Result<f32, String> {
    let text = text.trim();
    if text.is_empty() {
        return Ok(0.0);
    }
    let value = if let Some(number) = text.strip_suffix("px") {
        finite(number.trim())?
    } else if text.parse::<f32>().is_ok_and(|value| value == 0.0) {
        0.0
    } else {
        return Err(format!("expected a pixel length, got `{text}`"));
    };
    if !negative && value < 0.0 {
        return Err(format!("length must be nonnegative: `{text}`"));
    }
    Ok(value)
}

/// Parses a finite CSS number or percentage from `text`, optionally clamping to one.
///
/// # Errors
///
/// Returns an error for invalid or negative values.
fn ratio(text: &str, clamp: bool) -> Result<f32, String> {
    let text = text.trim();
    if text.is_empty() {
        return Ok(1.0);
    }
    let value = if let Some(percent) = text.strip_suffix('%') {
        finite(percent.trim())? / 100.0
    } else {
        finite(text)?
    };
    if value < 0.0 {
        return Err(format!("filter amount must be nonnegative: `{text}`"));
    }
    Ok(if clamp { value.min(1.0) } else { value })
}

/// Parses a finite scalar from `text`.
///
/// # Errors
///
/// Returns an error for invalid or non-finite numbers.
fn finite(text: &str) -> Result<f32, String> {
    let value = text
        .parse::<f32>()
        .map_err(|_| format!("expected a number, got `{text}`"))?;
    if !value.is_finite() {
        return Err(format!("number must be finite: `{text}`"));
    }
    Ok(value)
}

/// Converts a CSS angle in `text` to radians for the renderer.
///
/// # Errors
///
/// Returns an error for unsupported units or a non-finite value.
fn angle(text: &str) -> Result<f32, String> {
    let text = text.trim();
    if text.is_empty() {
        return Ok(0.0);
    }
    for (unit, factor) in [
        ("deg", std::f32::consts::PI / 180.0),
        ("rad", 1.0),
        ("grad", std::f32::consts::PI / 200.0),
        ("turn", std::f32::consts::TAU),
    ] {
        if let Some(number) = text.strip_suffix(unit) {
            return Ok(finite(number.trim())? * factor);
        }
    }
    if text.parse::<f32>().is_ok_and(|value| value == 0.0) {
        return Ok(0.0);
    }
    Err(format!("expected a CSS angle, got `{text}`"))
}

/// Splits `text` on whitespace outside nested color functions and returns argument tokens.
fn tokens(text: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut start = None;
    let mut depth = 0_u32;
    for (index, character) in text.char_indices() {
        if character.is_whitespace() && depth == 0 {
            if let Some(first) = start.take() {
                result.push(&text[first..index]);
            }
        } else {
            start.get_or_insert(index);
            if character == '(' {
                depth += 1;
            } else if character == ')' {
                depth = depth.saturating_sub(1);
            }
        }
    }
    if let Some(first) = start {
        result.push(&text[first..]);
    }
    result
}

/// Parses a CSS drop shadow from `text`, accepting color before or after lengths.
///
/// # Errors
///
/// Returns an error for invalid length counts, blur, or colors.
fn drop_shadow(text: &str) -> Result<Shadow, String> {
    let mut lengths = Vec::new();
    let mut color = None;
    for token in tokens(text) {
        if token.starts_with('#')
            || token.starts_with(|character: char| character.is_ascii_alphabetic())
        {
            if color.is_some() {
                return Err("drop-shadow() accepts one color".into());
            }
            color = Some(css_color(token)?);
        } else {
            lengths.push(length(token, lengths.len() < 2)?);
        }
    }
    if !(2..=3).contains(&lengths.len()) {
        return Err("drop-shadow() needs two offsets and an optional blur radius".into());
    }
    Ok(Shadow::drop(
        [lengths[0], lengths[1]],
        lengths.get(2).copied().unwrap_or(0.0),
        color.unwrap_or(Color::BLACK),
    ))
}

/// Parses a named or core-supported CSS color in `text`.
///
/// # Errors
///
/// Returns an error when the color syntax is unsupported or invalid.
fn css_color(text: &str) -> Result<Color, String> {
    match text {
        "transparent" => return Ok(Color::TRANSPARENT),
        "black" => return Ok(Color::BLACK),
        "white" => return Ok(Color::WHITE),
        "red" => return Ok(Color::srgb(1.0, 0.0, 0.0)),
        "green" => return Ok(Color::srgb(0.0, 0.5, 0.0)),
        "blue" => return Ok(Color::srgb(0.0, 0.0, 1.0)),
        _ => {}
    }
    Color::from_literal(text).map_err(|error| error.to_string())
}

/// Parses an Argui refraction strength, optional chromatic amount, and optional edge.
///
/// # Errors
///
/// Returns an error for an invalid argument count or numeric value.
fn refraction(text: &str) -> Result<Refraction, String> {
    let values: Vec<_> = text
        .split([',', ' '])
        .filter(|part| !part.is_empty())
        .collect();
    if !(1..=3).contains(&values.len()) {
        return Err("refraction() needs strength, optional chromatic aberration and edge".into());
    }
    let strength = length(values[0], false)?;
    let chromatic = values
        .get(1)
        .map(|value| length(value, false))
        .transpose()?
        .unwrap_or(0.0);
    let edge = values
        .get(2)
        .map(|value| ratio(value, true))
        .transpose()?
        .unwrap_or(0.15);
    Ok(Refraction {
        strength,
        chromatic_aberration: chromatic,
        edge,
    })
}

/// Parses 20 finite row-major coefficients from `text` for an Argui color matrix.
///
/// # Errors
///
/// Returns an error unless exactly 20 valid numbers are supplied.
fn color_matrix(text: &str) -> Result<[f32; 20], String> {
    let values: Vec<_> = text
        .split(|character: char| character == ',' || character.is_whitespace())
        .filter(|part| !part.is_empty())
        .collect();
    if values.len() != 20 {
        return Err("color-matrix() needs exactly 20 coefficients".into());
    }
    let mut matrix = [0.0; 20];
    for (target, value) in matrix.iter_mut().zip(values) {
        *target = finite(value)?;
    }
    Ok(matrix)
}

/// Returns a saturation matrix, used to implement CSS grayscale amount `1 - value`.
fn saturation(value: f32) -> [f32; 20] {
    let inverse = 1.0 - value;
    let [red, green, blue] = [0.2126 * inverse, 0.7152 * inverse, 0.0722 * inverse];
    [
        red + value,
        green,
        blue,
        0.0,
        red,
        green + value,
        blue,
        0.0,
        red,
        green,
        blue + value,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
        0.0,
        0.0,
        0.0,
        0.0,
    ]
}

/// Returns a matrix blending the original color with its inversion by `amount`.
fn invert(amount: f32) -> [f32; 20] {
    let scale = 1.0 - 2.0 * amount;
    [
        scale, 0.0, 0.0, 0.0, 0.0, scale, 0.0, 0.0, 0.0, 0.0, scale, 0.0, 0.0, 0.0, 0.0, 1.0,
        amount, amount, amount, 0.0,
    ]
}

/// Returns a matrix blending the original color with the CSS sepia transform by `amount`.
fn sepia(amount: f32) -> [f32; 20] {
    let base = [
        0.393, 0.769, 0.189, 0.0, 0.349, 0.686, 0.168, 0.0, 0.272, 0.534, 0.131, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0, 0.0, 0.0,
    ];
    let mut matrix = base.map(|value| value * amount);
    for index in [0, 5, 10, 15] {
        matrix[index] += 1.0 - amount;
    }
    matrix
}
