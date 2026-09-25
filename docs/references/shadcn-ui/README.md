# Référence shadcn/ui pour le plan de parité Argui

Ces fichiers sont une **référence de lecture**, pas une dépendance du moteur Argui ni des composants prêts à distribuer. Ils permettent à un autre worktree de consulter les TSX, les classes Tailwind et les exemples sans dépendre d'un dossier temporaire ou de la disponibilité du réseau.

| Dossier | Origine |
| --- | --- |
| `upstream/ui/` | `apps/v4/registry/new-york-v4/ui/` du dépôt `shadcn-ui/ui`, commit `98a1fe67b439324ddc857f47fbdce056600a4329`. Inclut `_registry.ts`. |
| `upstream/examples/` | `apps/v4/registry/new-york-v4/examples/` du même commit ; 239 fichiers TSX et leurs fichiers d'accompagnement. |
| `cli/components/` | `src/components/ui/` d'un projet Vite généré par `shadcn` 4.21.0, `--base radix --preset nova`, puis `add --all` ; 61 fichiers TSX. |
| `cli/components.json`, `cli/index.css`, `cli/utils.ts` | Configuration, variables de thème et utilitaire de classes du même projet généré. |

Source : [dépôt officiel](https://github.com/shadcn-ui/ui/tree/98a1fe67b439324ddc857f47fbdce056600a4329). CLI : [documentation officielle](https://ui.shadcn.com/docs/cli). Licence : [MIT](LICENSE.md), à conserver avec toute reprise substantielle. Les nouveaux widgets Argui doivent être écrits pour les primitives Solid/React d'Argui et son thème ; ces références React/DOM ne sont pas intégrées directement au runtime natif.

Le [plan des widgets](../../plans/argui-shadcn-widget-parity.md) contient l'inventaire, les écarts entre les catalogues et les critères d'implémentation.
