use std::{collections::HashMap, sync::Arc};

use argui_dsl_parser::Parse;
use argui_dsl_syntax::FileId;

use crate::{SemanticProject, check, lower::InputModule, path};

#[derive(Clone)]
struct SourceFile {
    path: String,
    source: Arc<str>,
    revision: u64,
}

#[derive(Clone)]
struct CachedParse {
    revision: u64,
    parse: Parse,
}

/// Observable query counters used to verify incremental behavior.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct QueryStats {
    pub parse_executions: u64,
    pub semantic_executions: u64,
}

/// Shared incremental source, parse, module, and semantic query database.
pub struct CompilerDatabase {
    schema: argui_schema::SchemaRegistry,
    files: HashMap<FileId, SourceFile>,
    paths: HashMap<String, FileId>,
    parse_cache: HashMap<FileId, CachedParse>,
    project_cache: Option<(u64, Arc<SemanticProject>)>,
    generation: u64,
    next_file: u32,
    stats: QueryStats,
}

impl CompilerDatabase {
    /// Creates a compiler database backed by a canonical native schema registry.
    ///
    /// * `schema` — native primitives/components shared with runtime and tooling.
    #[must_use]
    pub fn new(schema: argui_schema::SchemaRegistry) -> Self {
        Self {
            schema,
            files: HashMap::new(),
            paths: HashMap::new(),
            parse_cache: HashMap::new(),
            project_cache: None,
            generation: 1,
            next_file: 1,
            stats: QueryStats::default(),
        }
    }

    /// Creates a compiler database with Argui's built-in native primitive schema.
    ///
    /// # Errors
    ///
    /// Returns a schema error if built-in metadata violates registry invariants.
    pub fn with_builtins() -> Result<Self, argui_schema::SchemaError> {
        let mut database = Self::new(argui_schema::builtin::registry()?);
        for (path, source) in argui_dsl_stdlib::UI_MODULES {
            database.set_file(path, *source);
        }
        Ok(database)
    }

    /// Adds or replaces a source file while preserving its stable file ID.
    ///
    /// Semantically identical source leaves the generation and caches unchanged.
    ///
    /// * `module_path` — canonicalizable project-relative module path.
    /// * `source` — current UTF-8 source contents.
    ///
    /// # Panics
    ///
    /// Panics only if the process-local file ID or revision space is exhausted.
    pub fn set_file(&mut self, module_path: &str, source: impl Into<Arc<str>>) -> FileId {
        let canonical = path::canonical(module_path).unwrap_or_else(|| module_path.into());
        let source = source.into();
        if let Some(id) = self.paths.get(&canonical).copied() {
            let file = self
                .files
                .get_mut(&id)
                .expect("path index points to a file");
            if file.source != source {
                file.source = source;
                file.revision = file
                    .revision
                    .checked_add(1)
                    .expect("source revision space exhausted");
                self.bump_generation();
            }
            return id;
        }
        let id = FileId::from_raw(self.next_file);
        self.next_file = self
            .next_file
            .checked_add(1)
            .expect("file ID space exhausted");
        self.paths.insert(canonical.clone(), id);
        self.files.insert(
            id,
            SourceFile {
                path: canonical,
                source,
                revision: 1,
            },
        );
        self.bump_generation();
        id
    }

    /// Removes a source file by stable ID.
    ///
    /// * `file` — file to remove.
    ///
    /// Returns whether a file existed.
    pub fn remove_file(&mut self, file: FileId) -> bool {
        let Some(source) = self.files.remove(&file) else {
            return false;
        };
        self.paths.remove(&source.path);
        self.parse_cache.remove(&file);
        self.bump_generation();
        true
    }

    /// Removes a source file by canonical module path.
    ///
    /// * `module_path` — project-relative path previously passed to [`Self::set_file`].
    ///
    /// Returns whether a matching file existed.
    pub fn remove_path(&mut self, module_path: &str) -> bool {
        let canonical = path::canonical(module_path).unwrap_or_else(|| module_path.into());
        self.paths
            .get(&canonical)
            .copied()
            .is_some_and(|file| self.remove_file(file))
    }

    /// Returns the stable file ID for a canonicalizable module path.
    ///
    /// * `module_path` — project-relative or package module path.
    #[must_use]
    pub fn file_id(&self, module_path: &str) -> Option<FileId> {
        let canonical = path::canonical(module_path).unwrap_or_else(|| module_path.into());
        self.paths.get(&canonical).copied()
    }

    /// Returns the current UTF-8 source for a file ID.
    ///
    /// * `file` — stable database file identity.
    #[must_use]
    pub fn source(&self, file: FileId) -> Option<&str> {
        self.files.get(&file).map(|source| source.source.as_ref())
    }

    /// Returns the canonical module path for a file ID.
    ///
    /// * `file` — stable database file identity.
    #[must_use]
    pub fn file_path(&self, file: FileId) -> Option<&str> {
        self.files.get(&file).map(|source| source.path.as_str())
    }

    /// Returns the canonical native schema shared with semantic checking.
    #[must_use]
    pub const fn schema(&self) -> &argui_schema::SchemaRegistry {
        &self.schema
    }

    /// Returns the cached or newly parsed lossless syntax for a file.
    ///
    /// * `file` — source file identity.
    #[must_use]
    pub fn parse(&mut self, file: FileId) -> Option<Parse> {
        let source = self.files.get(&file)?;
        if let Some(cached) = self.parse_cache.get(&file)
            && cached.revision == source.revision
        {
            return Some(cached.parse.clone());
        }
        let parse = argui_dsl_parser::parse(&source.source);
        self.stats.parse_executions += 1;
        self.parse_cache.insert(
            file,
            CachedParse {
                revision: source.revision,
                parse: parse.clone(),
            },
        );
        Some(parse)
    }

    /// Resolves and type-checks the current project snapshot incrementally.
    #[must_use]
    pub fn check(&mut self) -> Arc<SemanticProject> {
        if let Some((generation, project)) = &self.project_cache
            && *generation == self.generation
        {
            return project.clone();
        }
        let mut file_ids = self.files.keys().copied().collect::<Vec<_>>();
        file_ids.sort_unstable_by_key(|file| file.raw());
        let mut inputs = file_ids
            .into_iter()
            .filter_map(|file| {
                let path = self.files.get(&file)?.path.clone();
                Some(InputModule {
                    file,
                    path,
                    parse: self.parse(file)?,
                })
            })
            .collect::<Vec<_>>();
        let icon_names = inputs
            .iter()
            .flat_map(|input| {
                crate::lower::module(input.clone())
                    .imports
                    .into_iter()
                    .filter(|import| import.source == "@argui/icons")
                    .flat_map(|import| import.items.into_iter().map(|item| item.name))
            })
            .collect::<std::collections::BTreeSet<_>>();
        for name in icon_names {
            let Some(source) = argui_dsl_stdlib::icon_component_source(&name) else {
                continue;
            };
            let path = format!("@argui/icons/{name}.argui");
            let file = self.set_file(&path, source);
            let parse = self.parse(file).expect("inserted icon module is available");
            if let Some(input) = inputs.iter_mut().find(|input| input.file == file) {
                input.parse = parse;
            } else {
                inputs.push(InputModule { file, path, parse });
            }
        }
        let project = Arc::new(check::project(inputs, &self.schema));
        self.stats.semantic_executions += 1;
        self.project_cache = Some((self.generation, project.clone()));
        project
    }

    /// Returns cumulative query execution counts.
    #[must_use]
    pub const fn stats(&self) -> QueryStats {
        self.stats
    }

    /// Invalidates project-level queries after a real source graph change.
    fn bump_generation(&mut self) {
        self.generation = self
            .generation
            .checked_add(1)
            .expect("compiler generation space exhausted");
        self.project_cache = None;
    }
}
