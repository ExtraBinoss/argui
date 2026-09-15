use crate::{WebViewError, WebViewPolicy, WebViewSource};

/// Browser-only origin handling. Wry already preserves the loaded site's origin.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum WebCompatibility {
    /// Keep browser content isolated from the host page's origin.
    #[default]
    Isolated,
    /// A trusted, separately hosted Argui relay, configured with an origin allowlist.
    Compatible { relay: url::Url },
}

impl WebCompatibility {
    /// Allows the configured trusted relay origin for browser-backed sessions.
    /// `relay` must be an HTTP(S) origin without credentials, query, or fragment.
    ///
    /// # Errors
    /// Returns an error if the relay is not a valid HTTP(S) origin or contains credentials, query, or fragment data.
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
    /// Reject popup and new-window requests.
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
    /// Creates restrictive defaults for an external web page.
    #[must_use]
    pub fn webpage() -> Self {
        Self {
            policy: WebViewPolicy::Browser,
            compatibility: WebCompatibility::Isolated,
            popups: PopupPolicy::Block,
            downloads: false,
        }
    }
    /// Creates restrictive defaults for a sanitized email or other untrusted HTML document.
    #[must_use]
    pub fn email() -> Self {
        Self {
            policy: WebViewPolicy::RestrictedHtml,
            ..Self::webpage()
        }
    }
    #[must_use]
    /// Selects browser origin compatibility behavior.
    /// `mode` is the requested isolation policy.
    pub fn compatibility(mut self, mode: WebCompatibility) -> Self {
        self.compatibility = mode;
        self
    }
    #[must_use]
    /// Sets how popup requests are handled.
    /// `policy` selects whether requests are blocked or opened.
    pub fn popups(mut self, policy: PopupPolicy) -> Self {
        self.popups = policy;
        self
    }
    #[must_use]
    /// Sets whether this session may download files.
    /// `allow` enables or disables downloads.
    pub fn allow_downloads(mut self, allow: bool) -> Self {
        self.downloads = allow;
        self
    }
    /// Returns the configured browser compatibility mode.
    pub fn compatibility_mode(&self) -> &WebCompatibility {
        &self.compatibility
    }
    /// Returns the configured popup policy.
    pub fn popup_policy(&self) -> PopupPolicy {
        self.popups
    }
    /// Returns whether downloads are allowed.
    pub fn downloads_allowed(&self) -> bool {
        self.downloads
    }

    /// Checks that these permissions are compatible with `source`.
    ///
    /// # Errors
    /// Returns an error if source policy differs or restricted HTML permissions were relaxed.
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
    ///
    /// # Errors
    /// Returns an error if the requested popup policy cannot be enforced by the native backend.
    pub fn validate_native(&self) -> Result<(), WebViewError> {
        if self.popups != PopupPolicy::Block {
            return Err(WebViewError::UnsupportedOptions(
                "native popup policy inheritance is unavailable".into(),
            ));
        }
        Ok(())
    }

    /// Builds the iframe sandbox token list corresponding to these options.
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
