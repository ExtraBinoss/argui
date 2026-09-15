use std::time::Duration;

/// Retained keyboard search. The owner supplies a monotonic time; no timer or frame is scheduled.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Typeahead {
    config: TypeaheadConfig,
    query: String,
    last_input: Option<Duration>,
}

impl Typeahead {
    /// Creates an empty search state using `config`.
    pub fn new(config: TypeaheadConfig) -> Self {
        Self {
            config,
            ..Self::default()
        }
    }
    /// Clears the accumulated query and input timestamp.
    pub fn clear(&mut self) {
        self.query.clear();
        self.last_input = None;
    }

    /// Returns the currently accumulated search query.
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Search after `active`, wrapping once. Repeated characters cycle matching items.
    /// `label` returns `None` for disabled entries. Supply a matcher for locale-specific rules.
    /// `input` is the newly typed text; `now` is monotonic elapsed time; `active` is the
    /// current item index; and `count` is the number of candidate indices.
    /// `matches` defines how a query is compared with each candidate label.
    ///
    /// Returns the matching item index, if any.
    pub fn search<'a>(
        &mut self,
        input: &str,
        now: Duration,
        active: Option<usize>,
        count: usize,
        mut label: impl FnMut(usize) -> Option<&'a str>,
        matches: impl Fn(&str, &str) -> bool,
    ) -> Option<usize> {
        if input.is_empty() || input.chars().any(char::is_control) {
            return None;
        }
        if self
            .last_input
            .is_none_or(|last| now < last || now - last >= self.config.timeout)
        {
            self.query.clear();
        }
        self.last_input = Some(now);
        self.query.push_str(input);
        let first = self.query.chars().next()?;
        let cycling = self.query.chars().all(|c| c == first);
        let single = first.to_string();
        let query = if cycling {
            single.as_str()
        } else {
            self.query.as_str()
        };
        // A longer prefix first refines the active match instead of skipping it.
        let start = active
            .filter(|&index| index < count)
            .map_or(0, |index| if cycling { (index + 1) % count } else { index });
        (0..count)
            .map(|offset| (start + offset) % count)
            .find(|&index| label(index).is_some_and(|label| matches(label, query)))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Timing configuration for retained typeahead search.
pub struct TypeaheadConfig {
    /// Quiet interval after which the accumulated query is discarded.
    pub timeout: Duration,
}

impl Default for TypeaheadConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_millis(700),
        }
    }
}

/// Unicode lowercase prefix matching. Applications can supply a locale-aware matcher.
/// `label` is the candidate text and `query` is the accumulated search prefix.
pub fn unicode_prefix(label: &str, query: &str) -> bool {
    label.to_lowercase().starts_with(&query.to_lowercase())
}
