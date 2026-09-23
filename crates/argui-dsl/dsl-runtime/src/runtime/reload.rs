//! Transactional live-package migration and commit.

use std::collections::{HashMap, HashSet};

use crate::LivePackage;

use super::theme::{evaluate_theme_defaults, evaluate_theme_mode};
use super::{
    LiveRuntime, PreparedReload, ReloadOutcome, RuntimeError, effect, initialize_instance,
    prepare_assets, validate_native_abi,
};

impl LiveRuntime {
    /// Prepares an all-or-nothing reload and state migration without changing live state.
    ///
    /// `package` supplies the next accepted generation. Returns migrated instances
    /// whose component definitions still exist; removed private children are dropped.
    ///
    /// # Errors
    ///
    /// Returns for a changed public ABI, a missing mounted root, or invalid defaults
    /// or assets. On failure, the current generation and its state remain usable.
    pub fn prepare_reload(&self, package: LivePackage) -> Result<PreparedReload, RuntimeError> {
        validate_native_abi(&package, &self.schema)?;
        if package.public_api_hash != self.package.public_api_hash {
            return Err(RuntimeError::RestartRequired {
                previous: self.package.public_api_hash,
                next: package.public_api_hash,
            });
        }
        let (tokens, active_theme_mode) = match self.active_theme_mode {
            Some(mode) => match evaluate_theme_mode(&package, mode) {
                Ok(values) => (values, Some(mode)),
                Err(_) => (evaluate_theme_defaults(&package)?, None),
            },
            None => (evaluate_theme_defaults(&package)?, None),
        };
        let mut instances = HashMap::new();
        let components = package
            .ir
            .components
            .iter()
            .map(|component| component.id)
            .collect::<HashSet<_>>();
        for (id, previous) in &self.instances {
            if !components.contains(&previous.component) && Some(*id) != self.root {
                continue;
            }
            let mut replacement = initialize_instance(&package, previous.component, *id, &tokens)?;
            previous.clone().migrate_into(&mut replacement);
            instances.insert(*id, replacement);
        }
        let assets = prepare_assets(&package, Some(&self.assets))?;
        Ok(PreparedReload {
            package,
            instances,
            tokens,
            active_theme_mode,
            assets,
        })
    }

    /// Atomically exposes a previously prepared package and migrated instances.
    #[must_use]
    pub fn commit_reload(&mut self, prepared: PreparedReload) -> ReloadOutcome {
        let previous_generation = self.package.generation;
        let migrated_instances = prepared.instances.len();
        self.effect_revisions = effect::revisions(&prepared.package, Some(&self.effect_revisions));
        self.package = prepared.package;
        self.instances = prepared.instances;
        self.tokens = prepared.tokens;
        self.active_theme_mode = prepared.active_theme_mode;
        self.assets = prepared.assets;
        self.render_error = None;
        self.event_error = None;
        self.pending_focus = None;
        self.pending_scroll = None;
        self.last_valid_element = None;
        self.token_revision = self.token_revision.wrapping_add(1).max(1);
        ReloadOutcome {
            previous_generation,
            generation: self.package.generation,
            migrated_instances,
        }
    }
}
