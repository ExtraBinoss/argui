export function createThemeBridge() {
  let snapshot
  let definition
  let systemScheme = 'light'
  return {
    create(nextDefinition) {
      definition = nextDefinition
      const variant = nextDefinition.initialVariant ?? 'system'
      snapshot = makeSnapshot(1, variant)
      return { id: 1, snapshot }
    },
    update(_id, patch) {
      if (patch.systemScheme) systemScheme = patch.systemScheme
      const variant = Object.hasOwn(patch, 'variant') ? patch.variant : snapshot.variant
      snapshot = makeSnapshot(snapshot.revision + 1, variant, patch.overrides)
      return snapshot
    },
    subscribe() { return () => {} },
    dispose() {},
  }

  function makeSnapshot(revision, variant, overrides = {}) {
    const resolvedVariant = variant === 'system'
      ? definition.systemVariants?.[systemScheme] ?? null
      : variant
    const values = {
      ...Object.fromEntries(Object.entries(definition.tokens).map(([name, token]) => [name, token.default])),
      ...(resolvedVariant ? definition.variants?.[resolvedVariant] : {}),
      ...overrides,
    }
    return {
      revision,
      variant,
      resolvedVariant,
      systemScheme,
      values,
      tokenRevisions: Object.fromEntries(Object.keys(values).map((name) => [name, revision])),
      change: { tokens: Object.keys(values), impact: 'Paint' },
    }
  }
}
