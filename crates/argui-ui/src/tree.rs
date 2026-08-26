#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ElementKind {
    Container,
    Text(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Element {
    pub key: Option<String>,
    pub kind: ElementKind,
    pub children: Vec<Self>,
}

impl Element {
    #[must_use]
    pub fn container(children: impl IntoIterator<Item = Self>) -> Self {
        Self {
            key: None,
            kind: ElementKind::Container,
            children: children.into_iter().collect(),
        }
    }

    #[must_use]
    pub fn text(value: impl Into<String>) -> Self {
        Self {
            key: None,
            kind: ElementKind::Text(value.into()),
            children: Vec::new(),
        }
    }

    #[must_use]
    pub fn keyed(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }
}
