use crate::{WebViewError, WebViewPolicy, WebViewSource};

/// Browser-only origin handling. Wry already preserves the loaded site's origin.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum WebCompatibility {
    #[default]
    Isolated,
    /// A trusted, separately hosted Argui relay, configured with an origin allowlist.
    Compatible { relay: url::Url },
}

impl WebCompatibility {
    pub fn compatible(relay: &str) -> Result<Self, WebViewError> {
        let WebViewSource::Url(relay) = WebViewSource::url(relay)? else {
            unreachable!()
        };
        if !relay.username().is_empty()
            || relay.password().is_some()
            || relay.query().is_some()
            || relay.fragment().is_some()
        {
            return Err(WebViewError::InvalidOptions(
                "relay URL must not contain credentials, query or fragment".into(),
            ));
        }
        Ok(Self::Compatible { relay })
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PopupPolicy {
    #[default]
    Block,
    /// New windows inherit the frame sandbox.
    Sandboxed,
    /// New windows escape the sandbox, subject to browser popup protections.
    External,
}

/// Immutable session policy: changing it requires a separate retained session.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WebViewOptions {
    policy: WebViewPolicy,
    compatibility: WebCompatibility,
    popups: PopupPolicy,
    downloads: bool,
}

impl WebViewOptions {
    #[must_use]
    pub fn webpage() -> Self {
        Self {
            policy: WebViewPolicy::Browser,
            compatibility: WebCompatibility::Isolated,
            popups: PopupPolicy::Block,
            downloads: false,
        }
    }
    #[must_use]
    pub fn email() -> Self {
        Self {
            policy: WebViewPolicy::RestrictedHtml,
            ..Self::webpage()
        }
    }
    #[must_use]
    pub fn compatibility(mut self, mode: WebCompatibility) -> Self {
        self.compatibility = mode;
        self
    }
    #[must_use]
    pub fn popups(mut self, policy: PopupPolicy) -> Self {
        self.popups = policy;
        self
    }
    #[must_use]
    pub fn allow_downloads(mut self, allow: bool) -> Self {
        self.downloads = allow;
        self
    }
    pub fn compatibility_mode(&self) -> &WebCompatibility {
        &self.compatibility
    }
    pub fn popup_policy(&self) -> PopupPolicy {
        self.popups
    }
    pub fn downloads_allowed(&self) -> bool {
        self.downloads
    }

    pub fn validate(&self, source: &WebViewSource) -> Result<(), WebViewError> {
        if source.policy() != self.policy {
            return Err(WebViewError::PolicyMismatch);
        }
        if self.policy == WebViewPolicy::RestrictedHtml
            && (self.compatibility != WebCompatibility::Isolated
                || self.popups != PopupPolicy::Block
                || self.downloads)
        {
            return Err(WebViewError::InvalidOptions(
                "email permissions cannot be relaxed".into(),
            ));
        }
        if let WebCompatibility::Compatible { relay } = &self.compatibility {
            WebCompatibility::compatible(relay.as_str())?;
        }
        Ok(())
    }

    /// Wry default popup windows do not guarantee inheritance of our policies.
    pub fn validate_native(&self) -> Result<(), WebViewError> {
        if self.popups != PopupPolicy::Block {
            return Err(WebViewError::UnsupportedOptions(
                "native popup policy inheritance is unavailable".into(),
            ));
        }
        Ok(())
    }

    pub fn sandbox(&self) -> String {
        if self.policy == WebViewPolicy::RestrictedHtml {
            return "allow-same-origin".into();
        }
        let mut tokens = vec!["allow-scripts", "allow-forms"];
        if matches!(self.compatibility, WebCompatibility::Compatible { .. }) {
            tokens.push("allow-same-origin");
        }
        if self.popups != PopupPolicy::Block {
            tokens.push("allow-popups");
        }
        if self.popups == PopupPolicy::External {
            tokens.push("allow-popups-to-escape-sandbox");
        }
        if self.downloads {
            tokens.push("allow-downloads");
        }
        tokens.join(" ")
    }
}
