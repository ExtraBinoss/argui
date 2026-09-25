# Plan : parité des widgets Argui avec shadcn/ui

Statut : plan pour une **autre tâche d'implémentation**. Ce document ne demande aucun changement de code dans la tâche qui le rédige. Il complète le [plan global Argui](argui-crate-theme-decisions.md), en particulier le thème commun, les composants copiables par `argui add` et les cibles native + Web des mêmes sources TSX.

## Objectif et frontière

- Fournir un catalogue Argui comparable à shadcn/ui, avec des **sources modifiables** installées par `argui add` et une démonstration claire dans les deux galeries Solid et React.
- Pour chaque widget : mêmes usages et interactions attendues dans les deux frameworks, aspect cohérent avec les tokens Argui, états contrôlés et non contrôlés si pertinents, clavier, focus, accessibilité et modes clair/sombre. Le même TSX doit pouvoir cibler les hôtes Argui natif et Web prévus par le plan global.
- Prendre les TSX shadcn comme **référence de composition, API, états et apparence**, pas comme un copier-coller de `div`, CSS Tailwind, Radix ou bibliothèques DOM dans le renderer natif. Traduire vers les primitives, services et tokens Argui. Si une capacité native manque, la noter explicitement et corriger la primitive partagée avant de multiplier les contournements dans les widgets.
- Les exemples des capacités Argui optionnelles sont des pages de galerie, séparées du registre des widgets. Un widget ne doit pas forcer l'activation d'`i18n`, Media, Effects, WebView, Updater, etc. dans une application qui ne les utilise pas.
- Pas de nouvelle automation de GUI pour ce chantier. L'utilisateur veut tester la gallery lui-même à la fin. Les vérifications de build/type/API et la passe de qualité prescrite par `AGENTS.md` restent nécessaires avant les commits d'implémentation.

## Références figées et reproductibles

- Dépôt officiel : [shadcn-ui/ui](https://github.com/shadcn-ui/ui), commit `98a1fe67b439324ddc857f47fbdce056600a4329` (21 septembre 2026). Les widgets React TSX sont dans [`apps/v4/registry/new-york-v4/ui`](https://github.com/shadcn-ui/ui/tree/98a1fe67b439324ddc857f47fbdce056600a4329/apps/v4/registry/new-york-v4/ui), leur catalogue dans [`_registry.ts`](https://github.com/shadcn-ui/ui/blob/98a1fe67b439324ddc857f47fbdce056600a4329/apps/v4/registry/new-york-v4/ui/_registry.ts), et 239 exemples TSX dans [`examples`](https://github.com/shadcn-ui/ui/tree/98a1fe67b439324ddc857f47fbdce056600a4329/apps/v4/registry/new-york-v4/examples).
- **Les références sont committées avec ce plan** dans [`docs/references/shadcn-ui/`](../references/shadcn-ui/) : sources et exemples du commit amont, puis les 61 composants, `components.json`, CSS et utilitaire d'un projet Vite généré par la CLI officielle `shadcn` **4.21.0** (`init` puis `add --all`). Le prochain worktree les trouve directement dans Git. Les copies sous `/tmp` ne servent plus que de zone de préparation et peuvent disparaître.
- Reproduire si nécessaire, depuis `/tmp`, avec la [CLI officielle](https://ui.shadcn.com/docs/cli) :

  ```sh
  npx -y shadcn@4.21.0 init --template vite --base radix --preset nova --name argui-shadcn-cli-reference-20260925 --cwd /tmp --yes --no-monorepo
  npx -y shadcn@4.21.0 add --all --yes --cwd /tmp/argui-shadcn-cli-reference-20260925
  git clone --depth 1 --filter=blob:none --sparse https://github.com/shadcn-ui/ui.git /tmp/argui-shadcn-upstream-20260925
  git -C /tmp/argui-shadcn-upstream-20260925 sparse-checkout set apps/v4/registry/new-york-v4/ui apps/v4/registry/new-york-v4/examples
  git -C /tmp/argui-shadcn-upstream-20260925 checkout 98a1fe67b439324ddc857f47fbdce056600a4329
  ```

  Cette reproduction est facultative puisque les fichiers sont committés. Le clone à commit fixe peut exiger un `git fetch` ciblé si HEAD de `main` a changé depuis la rédaction du plan. Lire aussi la [licence MIT locale](../references/shadcn-ui/LICENSE.md) et conserver l'attribution requise si du code amont est repris substantiellement.

- Les deux inventaires ne sont pas identiques : le registre amont compte 62 entrées, la CLI génère 61 fichiers ; `form` et `toast` figurent dans le registre amont mais pas dans ce `add --all`, tandis que `questionnaire` est généré par la CLI mais n'a pas de fichier dans le dossier TSX du commit figé. La checklist prend **l'union de 63 noms** et marque ces écarts. Les pages officielles [Data Table](https://ui.shadcn.com/docs/components/data-table), [Date Picker](https://ui.shadcn.com/docs/components/date-picker) et [Typography](https://ui.shadcn.com/docs/components/typography) sont en plus des compositions documentées, pas des fichiers du lot CLI.

## Point de départ Argui

- [`components/registry.json`](../../components/registry.json) déclare cinq composants Solid et React : `button`, `input-field`, `select`, `popover`, `dialog`. `input-field` est l'équivalent le plus proche d'`input`, mais son API et ses états doivent être comparés à la référence. **Présent ne veut pas dire parité terminée.**
- Les sources vivent dans [`packages/widgets/src/solid`](../../packages/widgets/src/solid/) et [`packages/widgets/src/react`](../../packages/widgets/src/react/), avec types/helpers partagés. Le registre ne porte aujourd'hui ni version ni source versionnée par widget ; sa migration suit le plan CLI global.
- La [navigation de galerie](../../apps/gallery/src/gallery-pages.ts) expose cinq pages Components et des pages Examples (Media, Services, Theming, Internationalization, Accessibility, Overlay, Animation Lab, Damage Control et WGSL Lab). Les nouvelles entrées doivent couvrir les états et interactions significatifs dans **les deux** pages framework, sans gonfler systématiquement la page d'accueil.
- Garder les widgets faciles à copier. Mutualiser seulement les types, les algorithmes neutres du framework, les tokens et les primitives dont plusieurs composants ont un vrai besoin. Éviter une seconde grosse bibliothèque de comportement ou de CSS à installer chez l'utilisateur.

## Checklist de parité

`[~]` signifie présent dans Argui mais à auditer/compléter ; `[ ]` signifie absent du registre Argui. Une case ne devient `[x]` qu'après les sources Solid **et** React, la démo de gallery, le registre versionné et la vérification finale. Les familles aident à choisir des lots dépendants ; elles ne changent pas le nom public du composant.

### Déjà présents : audit de parité

- [~] `button` : variantes, tailles, icône, état busy, désactivation, focus et thème.
- [~] `input` ↔ `input-field` : noms/alias CLI, édition, validation, password, labels et états.
- [~] `select` : listes, groupes, scroll, clavier, popup et valeur contrôlée.
- [~] `popover` : ancrage, placement, fermeture, focus et contenu personnalisable.
- [~] `dialog` : composition header/body/footer, fermeture, focus modal, tailles et scrim.

### Fondations visuelles et formulaires

- [ ] `alert`
- [ ] `aspect-ratio`
- [ ] `avatar`
- [ ] `badge`
- [ ] `breadcrumb`
- [ ] `button-group`
- [ ] `card`
- [ ] `checkbox`
- [ ] `empty`
- [ ] `field`
- [ ] `form` — entrée du registre amont, absente de `add --all` ; définir l'API Solid/React de validation sans imposer React Hook Form aux deux.
- [ ] `input-group`
- [ ] `input-otp`
- [ ] `item`
- [ ] `kbd`
- [ ] `label`
- [ ] `native-select`
- [ ] `progress`
- [ ] `radio-group`
- [ ] `separator`
- [ ] `skeleton`
- [ ] `slider`
- [ ] `spinner`
- [ ] `switch`
- [ ] `textarea`
- [ ] `toggle`
- [ ] `toggle-group`

### Surfaces, navigation et overlays

- [ ] `accordion`
- [ ] `alert-dialog`
- [ ] `collapsible`
- [ ] `combobox`
- [ ] `command`
- [ ] `context-menu`
- [ ] `direction`
- [ ] `drawer`
- [ ] `dropdown-menu`
- [ ] `hover-card`
- [ ] `menubar`
- [ ] `navigation-menu`
- [ ] `pagination`
- [ ] `resizable`
- [ ] `scroll-area`
- [ ] `sheet`
- [ ] `sidebar`
- [ ] `tabs`
- [ ] `tooltip`

### Données, contenu et capacités avancées

- [ ] `attachment`
- [ ] `bubble`
- [ ] `calendar`
- [ ] `carousel`
- [ ] `chart`
- [ ] `marker`
- [ ] `message`
- [ ] `message-scroller`
- [ ] `questionnaire` — présent dans `add --all` 4.21.0, absent du dossier amont figé ; consulter le TSX temporaire et sa provenance/version avant adaptation.
- [ ] `sonner` — adapter en système de toast natif réutilisable ; éviter une dépendance DOM/`next-themes` dans le widget Argui.
- [ ] `table`
- [ ] `toast` — entrée amont sans fichier généré par `add --all` ; comparer à `sonner` et choisir une API publique unique ou documenter clairement deux comportements.

### Compositions des pages officielles à démontrer

- [ ] `data-table` : scénario tri, filtres, sélection, pagination, défilement et grands jeux de données ; composer `table` et la liste virtuelle Argui quand cela apporte une vraie valeur.
- [ ] `date-picker` : composer `calendar` et `popover`, avec saisie, formatage localisé et clavier.
- [ ] `typography` : hiérarchie et styles de texte pilotés par le thème, dans la gallery et dans les exemples de composition ; peut rester une recette plutôt qu'un widget du registre si aucun fichier copiable n'est nécessaire.

Le contrat de parité se juge sur les **fonctionnalités** et les exemples, pas sur le nombre brut de fichiers. `toast`, `sonner`, `form` et les compositions peuvent être des recettes ou API regroupées si le coordinateur documente pourquoi la couverture utilisateur reste complète. Ne pas déclarer une case finie avec un widget statique qui n'a pas les interactions annoncées.

## Exemples pour capacités optionnelles

- [ ] Montrer une page/démo Solid **et** React pour chaque capacité activable du futur `argui init` qui a un effet visible : Internationalization, Media, Effects, WebView, Updater, Tasks et intégrations Desktop. Réutiliser les pages déjà présentes lorsqu'elles couvrent réellement la capacité ; ajouter un état explicatif pour plateforme indisponible.
- [ ] Ajouter les démos Mobile Android + iOS quand le générateur et les deux hôtes sont réellement utilisables. La gallery doit annoncer les différences de plateforme sans simuler une fonctionnalité absente.
- [ ] Faire du thème commun `argui-theme` la source des tokens du catalogue, en cohérence avec le plan global ; vérifier en direct clair/sombre, accents, tailles, rayons, contrastes et éventuels styles locaux. Les widgets copiés restent personnalisables dans le projet utilisateur.
- [ ] Structurer la navigation de la gallery pour parcourir vite les 60+ widgets : catégories, recherche, variantes et états sur une page de composant. Conserver une version Solid et une React avec les mêmes cas. Privilégier le chargement à la demande pour garder démarrage, scroll et release légers.

## Exécution dans la nouvelle tâche

1. **Coordinateur : Sol 6, effort `xhigh`.** Lire `AGENTS.md`, le présent plan et le plan global ; ouvrir les sources shadcn temporaires ou les reconstruire au commit/version indiqués. Cartographier les primitives Argui manquantes, les dépendances entre widgets et le plan du registre. Maintenir la checklist et les décisions de compatibilité.
2. **Implémentation : Luna MAX** (`gpt-6-luna`, effort `max`) sous coordination du Sol. Donner à chaque Luna un lot borné de **trois composants à la suite**, avec fichiers propriétaires explicites et les références TSX/exemples correspondantes. Le lot comprend Solid, React, styles/tokens, démos et métadonnées de registre de ses trois composants. Dès qu'un lot est intégré, lui confier trois autres composants. Le Sol prend les primitives partagées, conflits de fichiers et choix d'API ; plusieurs Luna n'éditent jamais simultanément `registry.json`, `gallery-pages.ts` ou les mêmes index. Il peut séquencer leur intégration.
3. Choisir les lots par dépendances : petits composants de base, contrôles de formulaire, overlays/navigation, puis calendriers, tableaux, graphes et compositions. Auditer les cinq widgets existants tôt pour stabiliser les API, sans bloquer les lots indépendants. Les éventuels changements de moteur restent génériques et testables ; ils ne doivent pas encoder une règle spéciale pour un seul widget.
4. **Tests en fin de chantier** : lorsque toutes les cases fonctionnelles sont implémentées, compiler/vérifier les deux bundles Solid/React, les cibles réellement prises en charge et les chemins `argui add`/`list components`; inspecter les exports et les sources copiées. Ne pas créer de suite d'automation GUI dédiée à ce plan. Faire ensuite la passe de qualité finale prescrite par `AGENTS.md` une fois, juste avant les commits finaux. L'utilisateur fera la revue manuelle de la gallery. Si une vérification échoue, corriger puis rejouer seulement ce qui est nécessaire conformément aux règles du dépôt.
5. Ajouter proprement chaque composant **livrable** dans [`components/registry.json`](../../components/registry.json), avec source/version/checksum et dépendances de fichiers selon le nouveau contrat du plan global. Ne pas publier une entrée avant que les variantes Solid/React et leurs dépendances soient complètes. Vérifier `argui add` de plusieurs noms avec les deux frameworks et l'ouverture de leurs démos après copie.
6. Intégrer les commits du chantier sur **`codex/dsl-gallery-live`**, puis pousser cette branche sur `origin/codex/dsl-gallery-live` une fois l'implémentation et la qualité finale terminées. Résoudre les divergences de la branche avant le push ; vérifier que le commit distant contient widgets, gallery, registre et références nécessaires.

## Critères de sortie

- Les 63 noms de l'union des deux catalogues sont soit couverts dans Solid et React, soit explicitement mappés à une recette/API équivalente avec justification vérifiable ; les trois compositions documentées ont une démo utilisable.
- Chaque composant interactif expose un comportement natif cohérent (souris/tactile, clavier, focus, sémantique), des états utiles et un thème modifiable en direct. La gallery reste fluide malgré l'augmentation du catalogue.
- `registry.json`, `argui add`, les exports et les démos concordent : aucun composant n'est affiché comme installable sans fichiers complets. Les exemples optionnels n'imposent pas leurs features aux apps qui n'en ont pas besoin.
- La compilation/typecheck et la qualité finale passent ; l'utilisateur peut lancer la gallery Solid et React pour la vérifier manuellement.
