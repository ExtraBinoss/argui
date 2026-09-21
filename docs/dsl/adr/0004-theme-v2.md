# ADR 0004: Typed Theme v2

## Decision

Themes are schemas of typed token definitions plus mode-specific overrides.
Token keys resolve once to numeric `ThemeTokenId` values; reads are observable
at token granularity. Values retain units and domain types rather than sharing
an untyped number variant.

Derived tokens form a dependency graph checked for type compatibility and
cycles before activation. A mode or override change is committed atomically
and invalidates only consumers of tokens whose resolved values changed, using
the smallest correct update classification.

Style precedence is deterministic and contains no CSS specificity.
