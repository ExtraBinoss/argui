use argui_dsl_ir::{ComponentId, PropertyId, ThemeModeId};

use crate::{DslValue, InstanceId, LiveRuntime};

/// Immutable development snapshot of one mounted DSL component instance.
#[derive(Clone, Debug, PartialEq)]
pub struct InstanceInspection {
    pub id: InstanceId,
    pub component: ComponentId,
    pub properties: Vec<(PropertyId, DslValue, u64)>,
    pub callback_count: usize,
}

/// Immutable, transport-friendly summary of the committed live generation.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeInspection {
    pub generation: u64,
    pub public_api_hash: u64,
    pub root: Option<InstanceId>,
    pub theme_mode: Option<ThemeModeId>,
    pub instances: Vec<InstanceInspection>,
    pub decoded_assets: usize,
    pub animations: usize,
}

impl LiveRuntime {
    /// Captures stable IDs, values, revisions, and resource counts for DevTools.
    #[must_use]
    pub fn inspect(&self) -> RuntimeInspection {
        let mut instances = self
            .instances
            .values()
            .map(|instance| {
                let mut properties = instance
                    .properties
                    .values()
                    .map(|property| (property.id, property.get().clone(), property.revision()))
                    .collect::<Vec<_>>();
                properties.sort_unstable_by_key(|(id, _, _)| *id);
                InstanceInspection {
                    id: instance.id,
                    component: instance.component,
                    properties,
                    callback_count: instance.callbacks.len(),
                }
            })
            .collect::<Vec<_>>();
        instances.sort_unstable_by_key(|instance| instance.id);
        RuntimeInspection {
            generation: self.generation(),
            public_api_hash: self.public_api_hash(),
            root: self.root(),
            theme_mode: self.active_theme_mode,
            instances,
            decoded_assets: self.assets().len(),
            animations: self.animations.len(),
        }
    }
}
