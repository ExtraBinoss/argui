# ADR 0006: Revisioned transactional effects

## Decision

Effect definitions own names, parameters, passes, sources, and a monotonic
revision. Replacement is prepared by validating metadata and WGSL and creating
all required render resources before an atomic registry swap.

A failed preparation leaves the active revision untouched. Parameter storage
grows to the accepted maximum word count and does not eagerly shrink. Removed
revisions remain alive only while referenced by retained render resources.

The registry exposes immutable snapshots so frame rendering never observes a
partially replaced definition.
