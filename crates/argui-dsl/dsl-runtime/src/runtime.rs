use std::collections::{HashMap, HashSet};

use argui_dsl_ir::{CallbackId, ComponentId, LocalId, PropertyId, SiteId, ThemeModeId, TokenId};

use crate::{
    ComponentInstance, DslValue, DynamicProperty, EvaluationContext, InstanceId, LivePackage,
    RuntimeError,
};

mod effect;
mod host_effect;
mod initialize;
mod text_edit;
mod theme;

pub(crate) use initialize::initialize_instance;
use theme::{evaluate_theme_defaults, evaluate_theme_mode};

type TranslationResolver = dyn Fn(&str) -> Option<String>;

/// Result of atomically committing an accepted generation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReloadOutcome {
    pub previous_generation: u64,
    pub generation: u64,
    pub migrated_instances: usize,
}

/// Fully validated next generation and migrated state, not yet visible to rendering.
pub struct PreparedReload {
    package: LivePackage,
    instances: HashMap<InstanceId, ComponentInstance>,
    tokens: HashMap<TokenId, DslValue>,
    pub(crate) active_theme_mode: Option<ThemeModeId>,
    assets: argui_assets::AssetRegistry,
}

/// Live package, component state, theme state, and retained animation ownership.
pub struct LiveRuntime {
    pub(crate) package: LivePackage,
    pub(crate) schema: std::sync::Arc<argui_schema::SchemaRegistry>,
    pub(crate) instances: HashMap<InstanceId, ComponentInstance>,
    pub(crate) tokens: HashMap<TokenId, DslValue>,
    pub(crate) property_motions: argui_schema::PropertyMotionStore,
    pub(crate) native_cache: argui_schema::NativeElementCache,
    pub(crate) virtual_viewports: argui_schema::VirtualViewportStore,
    root: Option<InstanceId>,
    next_instance: u64,
    pub(crate) token_revision: u64,
    pub(crate) translator: Option<std::rc::Rc<TranslationResolver>>,
    pub(crate) active_theme_mode: Option<ThemeModeId>,
    assets: argui_assets::AssetRegistry,
    pub(crate) rendered_instances: HashSet<InstanceId>,
    pub(crate) event_error: Option<RuntimeError>,
    pub(crate) pending_focus: Option<argui_ui::FocusRequest>,
    pub(crate) pending_scroll: Option<argui_ui::ScrollRequest>,
    pub(crate) pending_prevent_default: bool,
    pub(crate) pending_stop_propagation: bool,
    effect_revisions: HashMap<argui_dsl_ir::EffectId, (u64, u64)>,
    pub(crate) render_error: Option<RuntimeError>,
    pub(crate) last_valid_element: Option<argui_ui::Element>,
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) live_client: Option<std::sync::Arc<std::sync::Mutex<crate::LiveClient>>>,
    pub(crate) live_task: Option<argui_runtime::tasks::TaskHandle>,
    pub(crate) last_client_event: Option<crate::ClientEvent>,
    #[cfg(target_arch = "wasm32")]
    pub(crate) web_client: Option<std::rc::Rc<crate::web_client::WebLiveClient>>,
}

impl LiveRuntime {
    /// Creates a runtime from a fully prepared first package.
    ///
    /// # Errors
    ///
    /// Returns if native schema construction or theme initialization fails.
    pub fn new(package: LivePackage) -> Result<Self, RuntimeError> {
        let schema = argui_schema::builtin::registry()
            .map_err(|error| RuntimeError::Schema(error.to_string()))?;
        let tokens = evaluate_theme_defaults(&package)?;
        let assets = prepare_assets(&package, None)?;
        let effect_revisions = effect::revisions(&package, None);
        Ok(Self {
            package,
            schema: std::sync::Arc::new(schema),
            instances: HashMap::new(),
            tokens,
            property_motions: argui_schema::PropertyMotionStore::new(),
            native_cache: argui_schema::NativeElementCache::new(),
            virtual_viewports: argui_schema::VirtualViewportStore::new(),
            root: None,
            next_instance: 1,
            token_revision: 1,
            translator: None,
            active_theme_mode: None,
            assets,
            rendered_instances: HashSet::new(),
            event_error: None,
            pending_focus: None,
            pending_scroll: None,
            pending_prevent_default: false,
            pending_stop_propagation: false,
            effect_revisions,
            render_error: None,
            last_valid_element: None,
            #[cfg(not(target_arch = "wasm32"))]
            live_client: None,
            live_task: None,
            last_client_event: None,
            #[cfg(target_arch = "wasm32")]
            web_client: None,
        })
    }

    /// Mounts a root component and applies typed input values by stable property ID.
    ///
    /// * `component` — exported component definition ID.
    /// * `inputs` — initial Rust/business values already converted to DSL values.
    ///
    /// # Errors
    ///
    /// Returns for missing components, invalid defaults, or incompatible inputs.
    pub fn mount(
        &mut self,
        component: ComponentId,
        inputs: impl IntoIterator<Item = (PropertyId, DslValue)>,
    ) -> Result<InstanceId, RuntimeError> {
        let id = InstanceId::from_raw(self.next_instance);
        self.next_instance = self.next_instance.wrapping_add(1).max(1);
        let mut instance = initialize_instance(&self.package, component, id, &self.tokens)?;
        for (property, value) in inputs {
            instance
                .properties
                .get_mut(&property)
                .ok_or(RuntimeError::MissingProperty(property.raw()))?
                .set(value)?;
        }
        self.instances.insert(id, instance);
        self.root = Some(id);
        Ok(id)
    }

    /// Returns the currently mounted root instance.
    #[must_use]
    pub const fn root(&self) -> Option<InstanceId> {
        self.root
    }

    /// Returns the currently committed compiler generation.
    #[must_use]
    pub const fn generation(&self) -> u64 {
        self.package.generation
    }

    /// Returns the currently committed public component ABI hash.
    #[must_use]
    pub const fn public_api_hash(&self) -> u64 {
        self.package.public_api_hash
    }

    /// Returns the immutable typed IR used by the current generation.
    #[must_use]
    pub fn ir(&self) -> &argui_dsl_ir::IrProject {
        &self.package.ir
    }

    /// Returns decoded live image/vector assets with stable renderer handles.
    #[must_use]
    pub const fn assets(&self) -> &argui_assets::AssetRegistry {
        &self.assets
    }

    /// Resolves a compiled asset ID to its stable decoded renderer handle.
    ///
    /// * `id` — asset expression ID from the current IR generation.
    ///
    /// # Errors
    ///
    /// Returns an asset error if the ID has no decoded image or SVG record.
    pub(crate) fn asset_handle(
        &self,
        id: argui_dsl_ir::AssetId,
    ) -> Result<argui_assets::AssetHandle, RuntimeError> {
        let asset = self
            .package
            .ir
            .assets
            .iter()
            .find(|asset| asset.id == id)
            .ok_or(RuntimeError::MissingAsset(id.raw()))?;
        let key = argui_assets::AssetKey::new(&asset.path)
            .map_err(|error| RuntimeError::Asset(error.to_string()))?;
        self.assets
            .get(&key)
            .map(argui_assets::AssetRecord::handle)
            .ok_or(RuntimeError::MissingAsset(id.raw()))
    }

    /// Returns the most recent compiler, commit, restart, or disconnect event.
    #[must_use]
    pub fn last_client_event(&self) -> Option<&crate::ClientEvent> {
        self.last_client_event.as_ref()
    }

    /// Updates one dynamic property transactionally.
    pub fn set_property(
        &mut self,
        instance: InstanceId,
        property: PropertyId,
        value: DslValue,
    ) -> Result<bool, RuntimeError> {
        self.set_property_inner(instance, property, value, &mut HashSet::new())
    }

    /// Binds a Rust/business callback to a root/component callback ID.
    pub fn bind_callback(
        &mut self,
        instance: InstanceId,
        callback: CallbackId,
        handler: impl FnMut(Vec<DslValue>) -> DslValue + 'static,
    ) -> Result<(), RuntimeError> {
        self.instances
            .get_mut(&instance)
            .ok_or(RuntimeError::MissingComponent(instance.raw()))?
            .bind_callback(callback, handler);
        Ok(())
    }

    /// Invokes `callback` on `instance`, normalizing typed `arguments` and its result.
    /// Returns the routed or host-produced value using the declared result type.
    ///
    /// # Errors
    /// Returns for an absent instance, callback, invalid arguments, or handler failure.
    pub fn invoke_callback(
        &mut self,
        instance: InstanceId,
        callback: CallbackId,
        arguments: Vec<DslValue>,
    ) -> Result<DslValue, RuntimeError> {
        let mounted = self
            .instances
            .get(&instance)
            .ok_or(RuntimeError::MissingComponent(instance.raw()))?;
        let signature = self
            .package
            .ir
            .components
            .iter()
            .find(|component| component.id == mounted.component)
            .and_then(|component| {
                component
                    .callbacks
                    .iter()
                    .find(|candidate| candidate.id == callback)
            })
            .ok_or_else(|| RuntimeError::InvalidBytecode("unknown callback signature".into()))?
            .clone();
        if arguments.len() != signature.parameters.len()
            || arguments
                .iter()
                .zip(&signature.parameters)
                .any(|(value, expected)| !value.compatible_with(expected))
        {
            return Err(RuntimeError::TypeMismatch {
                expected: format!("{:?}", signature.parameters),
                actual: format!("{arguments:?}"),
            });
        }
        let arguments = arguments
            .into_iter()
            .zip(&signature.parameters)
            .map(|(value, expected)| value.coerce(expected))
            .collect::<Vec<_>>();
        if let Some(route) = mounted.route(callback) {
            let mut locals = route.locals;
            for (parameter, value) in route.parameters.into_iter().zip(arguments) {
                locals.insert(parameter, value);
            }
            return self
                .execute_statements(route.parent, &route.statements, locals)
                .map(|value| value.coerce(&signature.result));
        }
        let handler = mounted.callback(callback).ok_or_else(|| {
            RuntimeError::InvalidBytecode(format!(
                "callback {} is not bound on instance {}",
                callback.raw(),
                instance.raw()
            ))
        })?;
        Ok((handler.borrow_mut())(arguments).coerce(&signature.result))
    }

    /// Applies every theme sharing a resolved mode identity.
    ///
    /// * `mode` — stable ID derived from the requested mode name.
    ///
    /// # Errors
    ///
    /// Returns an unknown-mode or expression error without changing runtime state.
    pub fn set_theme_mode(&mut self, mode: ThemeModeId) -> Result<(), RuntimeError> {
        self.tokens = evaluate_theme_mode(&self.package, mode)?;
        self.active_theme_mode = Some(mode);
        self.token_revision = self.token_revision.wrapping_add(1).max(1);
        Ok(())
    }

    /// Installs the host localization resolver used by DSL `tr()` expressions.
    ///
    /// * `translator` — callback returning localized text for a Fluent message ID.
    ///   Returning `None` keeps the message ID as the deterministic fallback.
    pub fn set_translator(&mut self, translator: impl Fn(&str) -> Option<String> + 'static) {
        self.translator = Some(std::rc::Rc::new(translator));
        for instance in self.instances.values_mut() {
            instance.clear_expression_cache();
        }
    }

    /// Removes the localization resolver so `tr()` returns source message IDs.
    pub fn clear_translator(&mut self) {
        self.translator = None;
        for instance in self.instances.values_mut() {
            instance.clear_expression_cache();
        }
    }

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

    /// Returns an immutable instance for inspection.
    #[must_use]
    pub fn instance(&self, id: InstanceId) -> Option<&ComponentInstance> {
        self.instances.get(&id)
    }

    /// Applies one property update and follows explicit two-way links without cycles.
    fn set_property_inner(
        &mut self,
        instance: InstanceId,
        property: PropertyId,
        value: DslValue,
        visited: &mut HashSet<(InstanceId, PropertyId)>,
    ) -> Result<bool, RuntimeError> {
        if !visited.insert((instance, property)) {
            return Ok(false);
        }
        let mounted = self
            .instances
            .get_mut(&instance)
            .ok_or(RuntimeError::MissingComponent(instance.raw()))?;
        let changed = mounted
            .properties
            .get_mut(&property)
            .ok_or(RuntimeError::MissingProperty(property.raw()))?
            .set(value.clone())?;
        let link = mounted.link(property);
        if let Some(link) = link {
            return self
                .set_property_inner(link.instance, link.property, value, visited)
                .map(|linked| changed || linked);
        }
        Ok(changed)
    }
}

/// Decodes a complete asset generation against stable handles from the prior one.
fn prepare_assets(
    package: &LivePackage,
    previous: Option<&argui_assets::AssetRegistry>,
) -> Result<argui_assets::AssetRegistry, RuntimeError> {
    let mut registry = previous.cloned().unwrap_or_default();
    let next_keys = package
        .ir
        .assets
        .iter()
        .filter_map(|asset| argui_assets::AssetKey::new(&asset.path).ok())
        .collect::<Vec<_>>();
    let removed = registry
        .records()
        .map(|record| record.key().clone())
        .filter(|key| !next_keys.contains(key))
        .collect::<Vec<_>>();
    for key in removed {
        registry.remove(&key);
    }
    for asset in &package.ir.assets {
        let payload = package
            .assets
            .get(&asset.id)
            .ok_or(RuntimeError::MissingAsset(asset.id.raw()))?;
        let key = argui_assets::AssetKey::new(&asset.path)
            .map_err(|error| RuntimeError::Asset(error.to_string()))?;
        match asset.kind {
            argui_dsl_ir::AssetKind::Image => {
                registry
                    .upsert_image(key, &payload.bytes)
                    .map_err(|error| RuntimeError::Asset(error.to_string()))?;
            }
            argui_dsl_ir::AssetKind::Vector => {
                registry
                    .upsert_vector(key, &payload.bytes)
                    .map_err(|error| RuntimeError::Asset(error.to_string()))?;
            }
            argui_dsl_ir::AssetKind::Shader | argui_dsl_ir::AssetKind::Other => {}
        }
    }
    Ok(registry)
}

/// Minimal evaluation environment for defaults and theme expressions.
pub(crate) struct ValueContext<'a> {
    properties: &'a HashMap<PropertyId, DynamicProperty>,
    tokens: &'a HashMap<TokenId, DslValue>,
}

impl<'a> ValueContext<'a> {
    /// Creates an expression environment over properties and active theme tokens.
    pub(crate) fn new(
        properties: &'a HashMap<PropertyId, DynamicProperty>,
        tokens: &'a HashMap<TokenId, DslValue>,
    ) -> Self {
        Self { properties, tokens }
    }
}

impl EvaluationContext for ValueContext<'_> {
    fn property(&self, id: PropertyId) -> Option<DslValue> {
        self.properties
            .get(&id)
            .map(|property| property.get().clone())
    }

    fn local(&self, _id: LocalId) -> Option<DslValue> {
        None
    }

    fn token(&self, id: TokenId) -> Option<DslValue> {
        self.tokens.get(&id).cloned()
    }

    fn callback(&mut self, _id: CallbackId, _arguments: Vec<DslValue>) -> Option<DslValue> {
        None
    }

    fn translate(&self, _id: &str) -> Option<String> {
        None
    }
}

/// Evaluation environment that can also invoke callbacks on a mounted instance.
pub(crate) struct InstanceValueContext<'a> {
    instance: &'a mut ComponentInstance,
    locals: &'a HashMap<LocalId, DslValue>,
    tokens: &'a HashMap<TokenId, DslValue>,
    translator: Option<&'a TranslationResolver>,
    presentation: bool,
}

impl<'a> InstanceValueContext<'a> {
    /// Creates a mounted-instance expression environment.
    pub(crate) const fn new(
        instance: &'a mut ComponentInstance,
        locals: &'a HashMap<LocalId, DslValue>,
        tokens: &'a HashMap<TokenId, DslValue>,
        translator: Option<&'a TranslationResolver>,
    ) -> Self {
        Self {
            instance,
            locals,
            tokens,
            translator,
            presentation: true,
        }
    }

    /// Uses canonical property values while evaluating a child's retained input.
    ///
    /// The returned context ignores render-only presentation overlays.
    #[must_use]
    pub(crate) const fn canonical(mut self) -> Self {
        self.presentation = false;
        self
    }
}

impl EvaluationContext for InstanceValueContext<'_> {
    fn property(&self, id: PropertyId) -> Option<DslValue> {
        if self.presentation
            && let Some(value) = self.instance.presented_properties.get(&id)
        {
            return Some(value.clone());
        }
        self.instance
            .properties
            .get(&id)
            .map(|property| property.get().clone())
    }

    fn observed(
        &self,
        site: argui_dsl_ir::SiteId,
        observation: argui_dsl_ir::IrObservation,
    ) -> Option<DslValue> {
        self.instance
            .observations
            .get(&site)
            .map(|value| crate::observation::value(*value, observation))
    }

    fn child_property(&self, site: SiteId, property: PropertyId) -> Option<DslValue> {
        self.instance.child_outputs.get(&(site, property)).cloned()
    }

    fn local(&self, id: LocalId) -> Option<DslValue> {
        self.locals.get(&id).cloned()
    }

    fn token(&self, id: TokenId) -> Option<DslValue> {
        self.tokens.get(&id).cloned()
    }

    fn callback(&mut self, id: CallbackId, arguments: Vec<DslValue>) -> Option<DslValue> {
        self.instance.invoke_callback(id, arguments)
    }

    fn translate(&self, id: &str) -> Option<String> {
        self.translator.and_then(|translator| translator(id))
    }
}
