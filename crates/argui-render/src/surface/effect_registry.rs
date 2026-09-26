use super::*;

#[cfg_attr(coverage_nightly, coverage(off))]
impl SurfaceRenderer {
    /// Adds or transactionally replaces a live custom-effect definition.
    ///
    /// Every WGSL pass and the complete parameter layout are prepared before the
    /// active registry and GPU pipelines are swapped. A rejected edit therefore
    /// leaves the prior working revision active. Existing definitions require a
    /// strictly greater revision; new identifiers begin at any non-zero revision.
    ///
    /// * `definition` — owned, revisioned effect schema and WGSL passes.
    ///
    /// # Errors
    ///
    /// Returns an error if validation fails or parameter storage exceeds the
    /// active adapter limit.
    pub fn update_effect_definition(
        &mut self,
        definition: crate::EffectDefinition,
    ) -> Result<(), RendererError> {
        let registry = if self.renderer_config.effects.get(&definition.id).is_some() {
            self.renderer_config
                .effects
                .clone()
                .with_replacement(definition)?
        } else {
            self.renderer_config
                .effects
                .clone()
                .with_definition(definition)?
        };
        self.install_effect_registry(registry)
    }

    /// Removes a live effect definition and releases its pipelines.
    ///
    /// Parameter storage retains its high-water capacity so repeated schema edits
    /// never cause shrink/grow churn.
    ///
    /// * `id` — effect identifier to remove.
    ///
    /// # Errors
    ///
    /// Returns an error if rebuilding the remaining GPU effect set fails.
    pub fn remove_effect_definition(
        &mut self,
        id: &argui_paint::EffectId,
    ) -> Result<(), RendererError> {
        let registry = self.renderer_config.effects.clone().without_definition(id);
        self.install_effect_registry(registry)
    }

    /// Replaces the complete GPU effect registry after preparing every shader.
    ///
    /// `registry` contains the definitions to use for subsequent frames. The
    /// current pipelines remain active if any definition cannot be prepared.
    ///
    /// # Errors
    ///
    /// Returns an error if the registry exceeds adapter limits or a WGSL pass
    /// fails to compile.
    pub fn replace_effect_registry(
        &mut self,
        registry: crate::EffectRegistry,
    ) -> Result<(), RendererError> {
        self.install_effect_registry(registry)
    }

    /// Prepares a complete GPU effect generation and commits it atomically.
    ///
    /// * `registry` — already validated next registry generation.
    ///
    /// # Errors
    ///
    /// Returns an error if storage limits or a shader pass are invalid.
    fn install_effect_registry(
        &mut self,
        registry: crate::EffectRegistry,
    ) -> Result<(), RendererError> {
        let parameter_words = registry
            .maximum_parameter_words()
            .max(self.effect.parameter_word_capacity());
        let maximum = self.device.limits().max_storage_buffer_binding_size as usize;
        let provided = parameter_words.saturating_mul(std::mem::size_of::<u32>());
        if provided > maximum {
            return Err(RendererError::EffectParametersTooLarge { provided, maximum });
        }
        let mut prepared =
            crate::effect::EffectGpu::new(&self.device, self.target_format, parameter_words);
        for definition in registry.definitions() {
            for (pass_index, pass) in definition.passes.iter().enumerate() {
                prepared.register(
                    &self.device,
                    definition.id.clone(),
                    definition.revision,
                    pass_index,
                    &pass.wgsl,
                )?;
            }
        }
        self.renderer_config.effects = registry;
        self.effect = prepared;
        self.layer_cache.clear();
        self.scene_snapshot = None;
        self.effect_root = None;
        self.damage.invalidate();
        Ok(())
    }
}
