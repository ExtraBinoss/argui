pub(crate) fn evaluate(source: &str) -> Option<f32> {
    let mut parser = Parser {
        source: source.as_bytes(),
        cursor: 0,
    };
    let value = parser.expression()?;
    parser.whitespace();
    (parser.cursor == parser.source.len() && value.is_finite()).then_some(value)
}

struct Parser<'a> {
    source: &'a [u8],
    cursor: usize,
}

impl Parser<'_> {
    fn expression(&mut self) -> Option<f32> {
        let mut value = self.term()?;
        loop {
            if self.consume(b'+') {
                value += self.term()?;
            } else if self.consume(b'-') {
                value -= self.term()?;
            } else {
                return Some(value);
            }
        }
    }

    fn term(&mut self) -> Option<f32> {
        let mut value = self.factor()?;
        loop {
            if self.consume(b'*') {
                value *= self.factor()?;
            } else if self.consume(b'/') {
                value /= self.factor()?;
            } else if self.consume(b'%') {
                value %= self.factor()?;
            } else {
                return Some(value);
            }
        }
    }

    fn factor(&mut self) -> Option<f32> {
        if self.consume(b'+') {
            return self.factor();
        }
        if self.consume(b'-') {
            return Some(-self.factor()?);
        }
        if self.consume(b'(') {
            let value = self.expression()?;
            return self.consume(b')').then_some(value);
        }
        self.number()
    }

    fn number(&mut self) -> Option<f32> {
        self.whitespace();
        let start = self.cursor;
        let mut point = false;
        while let Some(byte) = self.source.get(self.cursor) {
            if byte.is_ascii_digit() {
                self.cursor += 1;
            } else if *byte == b'.' && !point {
                point = true;
                self.cursor += 1;
            } else {
                break;
            }
        }
        (self.cursor > start).then(|| {
            std::str::from_utf8(&self.source[start..self.cursor])
                .ok()?
                .parse()
                .ok()
        })?
    }

    fn consume(&mut self, expected: u8) -> bool {
        self.whitespace();
        if self.source.get(self.cursor) == Some(&expected) {
            self.cursor += 1;
            true
        } else {
            false
        }
    }

    fn whitespace(&mut self) {
        while self
            .source
            .get(self.cursor)
            .is_some_and(u8::is_ascii_whitespace)
        {
            self.cursor += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::evaluate;

    #[test]
    fn evaluates_precedence_parentheses_and_unary_signs() {
        assert_eq!(evaluate("50 + 10 * 2"), Some(70.0));
        assert_eq!(evaluate("-(50 + 10) / 2"), Some(-30.0));
        assert_eq!(evaluate("11 % 4"), Some(3.0));
        assert_eq!(evaluate("2 / 0"), None);
        assert_eq!(evaluate("2 + nope"), None);
    }
}
