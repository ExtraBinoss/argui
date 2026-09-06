use std::sync::Arc;

use crate::WebViewError;

/// Construction-time policy. Browser and untrusted HTML instances never mix.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WebViewPolicy {
    Browser,
    RestrictedHtml,
}

impl WebViewPolicy {
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
    Url(url::Url),
    Html(Arc<str>),
}

impl WebViewSource {
    pub fn url(value: &str) -> Result<Self, WebViewError> {
        let url = url::Url::parse(value).map_err(|_| WebViewError::InvalidUrl)?;
        if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
            return Err(WebViewError::InvalidUrl);
        }
        Ok(Self::Url(url))
    }

    #[must_use]
    pub fn html(value: impl Into<Arc<str>>) -> Self {
        Self::Html(value.into())
    }

    #[must_use]
    pub fn policy(&self) -> WebViewPolicy {
        match self {
            Self::Url(_) => WebViewPolicy::Browser,
            Self::Html(_) => WebViewPolicy::RestrictedHtml,
        }
    }

    pub fn sanitized_html(&self) -> Option<String> {
        let Self::Html(html) = self else { return None };
        Some(ammonia::clean(html))
    }
}
