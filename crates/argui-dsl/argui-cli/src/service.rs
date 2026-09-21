use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use argui_dsl_compiler::{CompilerError, CompilerSession};
use argui_dsl_protocol::{
    AssetBytes, DiagnosticMessage, LiveMessage, LivePackageEnvelope, PackageHeader, Severity,
};

use crate::ProjectFiles;

/// Successful package or diagnostics produced by one monotonic compile attempt.
#[derive(Clone, Debug, PartialEq)]
pub struct CompileAttempt {
    pub generation: u64,
    pub message: LiveMessage,
}

/// Long-lived incremental compiler and asset revision tracker.
pub struct DevCompilerService {
    root: PathBuf,
    stdlib_root: Option<PathBuf>,
    session: CompilerSession,
    modules: HashSet<String>,
    stdlib_modules: HashSet<String>,
    generation: u64,
    assets: HashMap<argui_dsl_ir::AssetId, (u64, u64)>,
}

impl DevCompilerService {
    /// Loads an initial project snapshot for the external development host.
    ///
    /// # Errors
    ///
    /// Returns file traversal, schema, or compiler initialization errors.
    pub fn open(root: impl Into<PathBuf>, entry: impl Into<String>) -> Result<Self, ServiceError> {
        let root = root.into();
        let mut session = CompilerSession::new(entry.into())?;
        let stdlib_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../stdlib/ui")
            .canonicalize()
            .ok();
        let project = ProjectFiles::read(&root)?;
        let modules = project
            .modules
            .iter()
            .map(|module| module.path.clone())
            .collect();
        for module in project.modules {
            session.update_module(module);
        }
        let mut service = Self {
            root,
            stdlib_root,
            session,
            modules,
            stdlib_modules: HashSet::new(),
            generation: 0,
            assets: HashMap::new(),
        };
        service.refresh_stdlib_sources()?;
        Ok(service)
    }

    /// Returns the local standard-library source directory when running from a checkout.
    ///
    /// Packaged CLI binaries keep their embedded standard library and return `None`.
    #[must_use]
    pub fn stdlib_root(&self) -> Option<&Path> {
        self.stdlib_root.as_deref()
    }

    /// Synchronizes changed, added, and removed source modules incrementally.
    ///
    /// # Errors
    ///
    /// Returns a project traversal or source read failure.
    pub fn refresh_sources(&mut self) -> Result<(), ServiceError> {
        let project = ProjectFiles::read(&self.root)?;
        let next = project
            .modules
            .iter()
            .map(|module| module.path.clone())
            .collect::<HashSet<_>>();
        for removed in self.modules.difference(&next) {
            self.session.remove_module(removed);
        }
        for module in project.modules {
            self.session.update_module(module);
        }
        self.modules = next;
        self.refresh_stdlib_sources()?;
        Ok(())
    }

    /// Synchronizes checkout standard-library modules without rebuilding the CLI binary.
    ///
    /// # Errors
    ///
    /// Returns directory traversal or UTF-8 source-read failures.
    fn refresh_stdlib_sources(&mut self) -> Result<(), ServiceError> {
        let Some(root) = &self.stdlib_root else {
            return Ok(());
        };
        let mut next = HashSet::new();
        for entry in fs::read_dir(root)? {
            let entry = entry?;
            let path = entry.path();
            if !entry.file_type()?.is_file()
                || path.extension().and_then(|extension| extension.to_str()) != Some("argui")
            {
                continue;
            }
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            let module = format!("@argui/ui/{name}");
            let source = fs::read_to_string(&path)?;
            self.session
                .update_module(argui_dsl_compiler::SourceModule::new(&module, source));
            next.insert(module);
        }
        for removed in self.stdlib_modules.difference(&next) {
            self.session.remove_module(removed);
        }
        self.stdlib_modules = next;
        Ok(())
    }

    /// Compiles one generation; errors become diagnostics and never mutate clients.
    #[must_use]
    pub fn compile(&mut self) -> CompileAttempt {
        self.generation = self.generation.wrapping_add(1).max(1);
        let root = self.root.clone();
        let result = self
            .session
            .compile(|path| read_asset(&root, path).map_err(|error| error.to_string()));
        let message = match result {
            Ok(mut compiled) => match self.package(&mut compiled) {
                Ok(package) => LiveMessage::Package(Box::new(package)),
                Err(error) => rejection(self.generation, &error.to_string()),
            },
            Err(error) => compiler_diagnostics(self.generation, error, &self.session),
        };
        CompileAttempt {
            generation: self.generation,
            message,
        }
    }

    /// Builds a pruned live package and monotonic source-asset revisions.
    fn package(
        &mut self,
        compiled: &mut argui_dsl_compiler::CompiledProject,
    ) -> Result<LivePackageEnvelope, ServiceError> {
        compiled.reachability.prune(&mut compiled.ir);
        let mut assets = Vec::new();
        for asset in &compiled.ir.assets {
            let bytes = read_asset(&self.root, &asset.path)?;
            let hash = content_hash(&bytes);
            let revision = match self.assets.get(&asset.id) {
                Some((previous, revision)) if *previous == hash => *revision,
                Some((_, revision)) => revision.checked_add(1).ok_or_else(|| {
                    ServiceError::Protocol(format!("asset {} revision exhausted", asset.id.raw()))
                })?,
                None => 1,
            };
            self.assets.insert(asset.id, (hash, revision));
            assets.push(AssetBytes {
                id: asset.id,
                revision,
                bytes,
            });
        }
        self.assets
            .retain(|id, _| compiled.ir.assets.iter().any(|asset| asset.id == *id));
        Ok(LivePackageEnvelope {
            header: PackageHeader::current(compiled.public_api_hash, self.generation),
            roots: compiled.roots.clone(),
            ir: compiled.ir.clone(),
            assets,
        })
    }
}

/// Reads a project resource or one feature-gated embedded icon SVG.
///
/// `root` identifies the project and `path` is the canonical DSL asset path.
/// The returned bytes are the exact source payload sent to live clients.
///
/// # Errors
///
/// Returns a file error or `NotFound` for an unavailable built-in icon.
fn read_asset(root: &Path, path: &str) -> std::io::Result<Vec<u8>> {
    if let Some(name) = path
        .strip_prefix("@argui/icons/")
        .and_then(|path| path.strip_suffix(".svg"))
    {
        return argui_dsl_stdlib::icon_svg(name)
            .map(String::into_bytes)
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Tabler icon `{name}` is unavailable"),
                )
            });
    }
    fs::read(root.join(path))
}

/// Dev service initialization, filesystem, compilation, or protocol failure.
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("filesystem: {0}")]
    Io(#[from] std::io::Error),
    #[error("compiler: {0}")]
    Compiler(#[from] CompilerError),
    #[error("protocol: {0}")]
    Protocol(String),
}

/// Converts compiler failures to source diagnostics suitable for remote clients.
fn compiler_diagnostics(
    generation: u64,
    error: CompilerError,
    session: &CompilerSession,
) -> LiveMessage {
    let location = |span: argui_dsl_syntax::Span| {
        (
            session.file_path(span.file).map(str::to_owned),
            Some(u32::from(span.range.start())),
            Some(u32::from(span.range.end())),
        )
    };
    let diagnostics = match error {
        CompilerError::Semantic(values) => values
            .into_iter()
            .map(|diagnostic| {
                let (path, start, end) = location(diagnostic.primary);
                DiagnosticMessage {
                    path,
                    start,
                    end,
                    severity: match diagnostic.severity {
                        argui_dsl_semantic::Severity::Error => Severity::Error,
                        argui_dsl_semantic::Severity::Warning => Severity::Warning,
                    },
                    code: format!("{:?}", diagnostic.code),
                    message: diagnostic.message,
                }
            })
            .collect(),
        CompilerError::Lower(values) => values
            .into_iter()
            .map(|error| {
                let (path, start, end) = location(error.span);
                DiagnosticMessage {
                    path,
                    start,
                    end,
                    severity: Severity::Error,
                    code: "Lowering".into(),
                    message: error.message,
                }
            })
            .collect(),
        CompilerError::Asset {
            path,
            message,
            source_span,
        } => {
            let (source_path, start, end) = source_span.map(location).unwrap_or((None, None, None));
            vec![DiagnosticMessage {
                path: source_path,
                start,
                end,
                severity: Severity::Error,
                code: "Asset".into(),
                message: format!("asset `{}`: {message}", path.display()),
            }]
        }
        error => vec![DiagnosticMessage {
            path: None,
            start: None,
            end: None,
            severity: Severity::Error,
            code: "compile".into(),
            message: error.to_string(),
        }],
    };
    LiveMessage::Diagnostics {
        generation,
        diagnostics,
    }
}

/// Creates one host-side rejection diagnostic.
fn rejection(generation: u64, message: &str) -> LiveMessage {
    LiveMessage::Diagnostics {
        generation,
        diagnostics: vec![DiagnosticMessage {
            path: None,
            start: None,
            end: None,
            severity: Severity::Error,
            code: "package".into(),
            message: message.into(),
        }],
    }
}

/// Hashes asset bytes for source revision tracking.
fn content_hash(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

/// Returns whether a watcher path can affect DSL, WGSL, or source assets.
#[must_use]
pub fn is_relevant_path(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|extension| {
            matches!(
                extension,
                "argui" | "wgsl" | "png" | "jpg" | "jpeg" | "webp" | "gif" | "svg"
            )
        })
}

/// Returns whether a watcher event represents a possible source-content change.
///
/// * `event` — filesystem notification whose kind and paths are inspected.
///
/// Read-only access and watcher metadata events are ignored so compiling a
/// source file cannot recursively schedule another generation.
#[must_use]
pub fn is_relevant_event(event: &notify::Event) -> bool {
    !matches!(
        event.kind,
        notify::EventKind::Access(_) | notify::EventKind::Other
    ) && event.paths.iter().any(|path| is_relevant_path(path))
}
