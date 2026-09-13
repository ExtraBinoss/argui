//! Static or dynamic HTTPS JSON feed. Sign artifacts with `minisign -Sm FILE`.
use crate::install::{Format, Installer};
use crate::{
    Backend, CancellationToken, DownloadEvent, Error, InstallOutcome, Progress, Release,
    ReleaseInfo, Result,
};
use minisign_verify::{PublicKey, Signature};
use reqwest::{Url, blocking::Client, redirect::Policy};
use semver::Version;
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    io::{Read, Write},
    time::Duration,
};
use tempfile::NamedTempFile;

/// Endpoint and key are application configuration, never supplied by a release manifest.
pub struct Config {
    current: Version,
    endpoint: Url,
    key: PublicKey,
    pub target: String,
    pub allow_prerelease: bool,
    pub timeout: Duration,
    pub max_download_bytes: u64,
}

impl Config {
    pub fn new(current: &str, endpoint: &str, public_key: &str) -> Result<Self> {
        let endpoint = safe_url(endpoint)?;
        Ok(Self {
            current: Version::parse(current).map_err(Error::backend)?,
            endpoint,
            key: PublicKey::decode(public_key).map_err(Error::backend)?,
            target: format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
            allow_prerelease: false,
            timeout: Duration::from_secs(300),
            max_download_bytes: 1024 * 1024 * 1024,
        })
    }
}

/// The fields are frozen at check time and cannot be replaced between download and install.
#[derive(Deserialize)]
pub struct Artifact {
    url: String,
    signature: String,
    format: Format,
}

#[derive(Deserialize)]
struct Manifest {
    version: Version,
    #[serde(default)]
    notes: String,
    platforms: BTreeMap<String, Artifact>,
}

/// Opaque, authenticated staging file, automatically removed on cancellation, failure or drop.
pub struct VerifiedPackage(NamedTempFile);

pub struct HttpBackend<I> {
    config: Config,
    client: Client,
    installer: I,
}

impl<I: Installer> HttpBackend<I> {
    pub fn new(config: Config, installer: I) -> Result<Self> {
        if config.target.is_empty() || config.timeout.is_zero() || config.max_download_bytes == 0 {
            return Err(Error::backend(
                "target, timeout and download limit must be nonzero",
            ));
        }
        let client = Client::builder()
            .user_agent(concat!("argui-updater/", env!("CARGO_PKG_VERSION")))
            .timeout(config.timeout)
            .connect_timeout(Duration::from_secs(15))
            .redirect(Policy::custom(|attempt| {
                if attempt.previous().len() >= 10 || safe_url(attempt.url().as_str()).is_err() {
                    attempt.error("unsafe update redirect or too many redirects")
                } else {
                    attempt.follow()
                }
            }))
            .build()
            .map_err(Error::backend)?;
        Ok(Self {
            config,
            client,
            installer,
        })
    }
}

impl<I: Installer> Backend for HttpBackend<I> {
    type Artifact = Artifact;
    type Package = VerifiedPackage;

    fn check(&self) -> Result<Option<Release<Artifact>>> {
        let response = self
            .client
            .get(self.config.endpoint.clone())
            .send()
            .map_err(Error::backend)?;
        if response.status() == reqwest::StatusCode::NO_CONTENT {
            return Ok(None);
        }
        let response = response.error_for_status().map_err(Error::backend)?;
        let mut bytes = Vec::new();
        response
            .take(1024 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map_err(Error::backend)?;
        if bytes.len() > 1024 * 1024 {
            return Err(Error::backend("update manifest exceeds 1 MiB"));
        }
        let mut manifest: Manifest = serde_json::from_slice(&bytes).map_err(Error::backend)?;
        if manifest
            .version
            .cmp_precedence(&self.config.current)
            .is_le()
            || (!self.config.allow_prerelease && !manifest.version.pre.is_empty())
        {
            return Ok(None);
        }
        let artifact = manifest
            .platforms
            .remove(&self.config.target)
            .ok_or_else(|| {
                Error::backend(format!("no update artifact for {}", self.config.target))
            })?;
        safe_url(&artifact.url)?;
        Signature::decode(&artifact.signature).map_err(Error::backend)?;
        self.installer.supports(artifact.format)?;
        Ok(Some(Release {
            info: ReleaseInfo {
                version: manifest.version.to_string(),
                notes: manifest.notes,
            },
            artifact,
        }))
    }

    fn download(
        &self,
        artifact: &Artifact,
        cancel: &CancellationToken,
        emit: &mut dyn FnMut(DownloadEvent),
    ) -> Result<VerifiedPackage> {
        cancel.check()?;
        let signature = Signature::decode(&artifact.signature).map_err(Error::backend)?;
        let mut verifier = self
            .config
            .key
            .verify_stream(&signature)
            .map_err(Error::backend)?;
        let mut response = self
            .client
            .get(safe_url(&artifact.url)?)
            .send()
            .map_err(Error::backend)?
            .error_for_status()
            .map_err(Error::backend)?;
        let total = response.content_length();
        if total.is_some_and(|size| size > self.config.max_download_bytes) {
            return Err(Error::backend("update exceeds the download limit"));
        }
        let mut file = NamedTempFile::new().map_err(Error::backend)?;
        let mut progress = Progress {
            downloaded: 0,
            total,
        };
        emit(DownloadEvent::Progress(progress));
        let mut buffer = [0; 64 * 1024];
        loop {
            cancel.check()?;
            let count = response.read(&mut buffer).map_err(Error::backend)?;
            if count == 0 {
                break;
            }
            progress.downloaded += count as u64;
            if progress.downloaded > self.config.max_download_bytes {
                return Err(Error::backend("update exceeds the download limit"));
            }
            file.write_all(&buffer[..count]).map_err(Error::backend)?;
            verifier.update(&buffer[..count]);
            emit(DownloadEvent::Progress(progress));
        }
        cancel.check()?;
        emit(DownloadEvent::Verifying);
        verifier.finalize().map_err(Error::backend)?;
        file.as_file().sync_all().map_err(Error::backend)?;
        Ok(VerifiedPackage(file))
    }

    fn install(&self, artifact: &Artifact, package: VerifiedPackage) -> Result<InstallOutcome> {
        self.installer.install(artifact.format, package.0.path())
    }
}

fn safe_url(value: &str) -> Result<Url> {
    let url = Url::parse(value).map_err(Error::backend)?;
    let loopback = url
        .host_str()
        .and_then(|host| {
            host.trim_matches(['[', ']'])
                .parse::<std::net::IpAddr>()
                .ok()
        })
        .is_some_and(|ip| ip.is_loopback());
    if !url.username().is_empty()
        || url.password().is_some()
        || !(url.scheme() == "https" || (url.scheme() == "http" && loopback))
    {
        return Err(Error::backend(
            "update URLs require HTTPS (HTTP is allowed only for loopback tests)",
        ));
    }
    Ok(url)
}
