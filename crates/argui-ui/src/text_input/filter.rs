#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TextInputFilter {
    #[default]
    Any,
    Decimal,
    Arithmetic,
}

impl TextInputFilter {
    pub(super) fn accepts(self, value: &str) -> bool {
        match self {
            Self::Any => true,
            Self::Decimal => decimal(value),
            Self::Arithmetic => value.chars().all(|character| {
                character.is_ascii_digit()
                    || matches!(
                        character,
                        '.' | '+' | '-' | '*' | '/' | '%' | '(' | ')' | ' ' | '\t'
                    )
            }),
        }
    }
}

fn decimal(value: &str) -> bool {
    let value = value
        .strip_prefix('-')
        .or_else(|| value.strip_prefix('+'))
        .unwrap_or(value);
    let mut point = false;
    value.chars().all(|character| {
        if character == '.' && !point {
            point = true;
            true
        } else {
            character.is_ascii_digit()
        }
    })
}
