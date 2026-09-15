use std::sync::Arc;

use crate::WebViewError;

/// Construction-time policy. Browser and untrusted HTML instances never mix.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WebViewPolicy {
    Browser,
    RestrictedHtml,
}

impl WebViewPolicy {
    /// Checks whether a navigation target is allowed by this policy.
    /// `value` is the URL requested by the document.
    #[must_use]
    pub fn allows_navigation(self, value: &str) -> bool {
        if self == Self::Browser {
            return WebViewSource::url(value).is_ok();
        }
        let Ok(url) = url::Url::parse(value) else {
            return false;
        };
        url.path() == "/document"
            && url.username().is_empty()
            && url.password().is_none()
            && url.port().is_none()
            && ((url.scheme() == "argui-content" && url.host_str() == Some("localhost"))
                || (cfg!(target_os = "windows")
                    && url.scheme() == "http"
                    && url.host_str() == Some("argui-content.localhost")))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum WebViewSource {
    /// External HTTP or HTTPS page.
    Url(url::Url),
    /// Untrusted markup rendered under the restricted HTML policy.
    Html(Arc<str>),
}

impl WebViewSource {
    /// Parses an HTTP(S) URL suitable for an external web page.
    /// `value` is the URL string to parse.
    ///
    /// # Errors
    /// Returns an error if the URL is invalid, has no host, or uses another scheme.
    pub fn url(value: &str) -> Result<Self, WebViewError> {
        let url = url::Url::parse(value).map_err(|_| WebViewError::InvalidUrl)?;
        if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
            return Err(WebViewError::InvalidUrl);
        }
        Ok(Self::Url(url))
    }

    #[must_use]
    /// Creates a restricted-HTML source from markup.
    /// `value` is the untrusted HTML document text.
    pub fn html(value: impl Into<Arc<str>>) -> Self {
        Self::Html(value.into())
    }

    #[must_use]
    /// Returns the security policy implied by this source.
    pub fn policy(&self) -> WebViewPolicy {
        match self {
            Self::Url(_) => WebViewPolicy::Browser,
            Self::Html(_) => WebViewPolicy::RestrictedHtml,
        }
    }

    /// Sanitizes this source's markup; returns `None` for URL sources.
    pub fn sanitized_html(&self) -> Option<String> {
        let Self::Html(html) = self else { return None };
        Some(ammonia::clean(html))
    }
}
