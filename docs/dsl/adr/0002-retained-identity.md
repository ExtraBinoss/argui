# ADR 0002: Private retained source identity

## Decision

`Element` carries an optional `RetainedIdentity` separate from public keys,
focus keys, accessibility IDs, and runtime node IDs. Reconciliation matches a
retained identity before positional fallback, within the same owning
component/repeater scope.

The live and AOT backends derive identity from component instance, stable
`SiteId`, and an optional typed repeater key. Branch sites are distinct.
Identity is opaque to application code and is never derived from line numbers.

## Preservation

When kind and ownership remain compatible, a match keeps node identity and its
focus, selection, editor, scroll, interaction, overlay, and transition state.
Duplicate sibling identities are rejected deterministically in development.
