/** Creates the small host-side theme fixture used by gallery transaction tests. */
export function createThemeBridge() {
  let sequence = 0
  const sessions = new Map()
  const resolved = (session) => session.variant === 'system'
    ? session.definition.systemVariants?.[session.systemScheme] ?? null
    : session.variant
  const values = (session) => Object.fromEntries(Object.entries(session.definition.tokens).map(
    ([name, token]) => [name, session.overrides[name]
      ?? session.definition.variants?.[resolved(session)]?.[name] ?? token.default],
  ))
  const snapshot = (session, change) => ({
    revision: session.revision, variant: session.variant, resolvedVariant: resolved(session),
    systemScheme: session.systemScheme, values: session.values,
    tokenRevisions: session.tokenRevisions, change,
  })
  return {
    create(definition) {
      const id = ++sequence
      const session = {
        definition, variant: definition.initialVariant ?? null, systemScheme: 'light',
        overrides: {}, revision: 0, values: {}, tokenRevisions: {}, listeners: new Set(),
      }
      session.values = values(session)
      session.tokenRevisions = Object.fromEntries(Object.keys(session.values).map((name) => [name, 0]))
      sessions.set(id, session)
      return { id, snapshot: snapshot(session) }
    },
    update(id, patch) {
      const session = sessions.get(id)
      if (!session) throw new Error(`Unknown theme session ${id}`)
      if ('variant' in patch) session.variant = patch.variant
      if (patch.systemScheme) session.systemScheme = patch.systemScheme
      Object.assign(session.overrides, patch.overrides)
      for (const name of patch.removeOverrides ?? []) delete session.overrides[name]
      const next = values(session)
      const changed = Object.keys(next).filter((name) => !Object.is(next[name], session.values[name]))
      if (changed.length) {
        session.revision++
        session.values = next
        for (const name of changed) session.tokenRevisions[name] = session.revision
      }
      const current = snapshot(session, { tokens: changed, impact: null })
      for (const listener of session.listeners) listener(current)
      return current
    },
    subscribe(id, listener) {
      const session = sessions.get(id)
      if (!session) throw new Error(`Unknown theme session ${id}`)
      session.listeners.add(listener)
      return () => session.listeners.delete(listener)
    },
    dispose(id) { sessions.delete(id) },
  }
}
