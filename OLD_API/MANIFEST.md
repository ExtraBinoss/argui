# OLD_API archive

- Source commit: `fd8d28d21cb73c5c743cf571138a87887610442d`.
- Status: read-only reference for the pre-v2 TSX gallery and widget package.
- Archived paths: `apps/gallery/src`, `apps/gallery/tests`, the gallery README and
  Tabler generator; `packages/widgets/src` and `packages/widgets/tests`; the
  source catalog and generated registry from the source commit.
- The archive is outside workspace package roots and is excluded from builds,
  exports, registry discovery, and active tests.
- New gallery and widget implementations are maintained under `apps/gallery`
  and `packages/widgets`; no compatibility imports should target this folder.
