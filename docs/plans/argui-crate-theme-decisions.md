# Plan global : distribution, CLI, crates et thème d’Argui

Statut : décisions et plan de mise en œuvre pour une prochaine tâche. **Aucun changement de code n’est demandé par ce document.** Il remplace [l’ancien plan de distribution](argui-standalone-distribution.md), notamment sa proposition « une seule crate Rust » ; les exigences CLI et composants pertinentes y sont reprises ici.

## Périmètre

- Argui garde **plusieurs crates Rust**, avec des frontières qui servent réellement l’architecture. Les applications utilisent Rust pur ou TSX avec Solid/React. **Il n’y a pas de DSL et il n’y en aura pas.**
- **Une même application peut cibler plusieurs plateformes** : un projet TSX Solid ou React garde les mêmes sources UI et peut produire à la fois une application native et une application WebAssembly/Web. Les hôtes et bundles de sortie sont adaptés à chaque plateforme ; on ne duplique pas l’application ou ses pages.
- `argui init` produit une application autonome dans le dossier choisi, sans cloner le dépôt Argui ni ajouter ses sources au dépôt de l’utilisateur. Le manifeste de l’application doit rester simple même si le moteur comporte plusieurs crates publiées.
- Le thème est une capacité normale de l’application, disponible par défaut et modifiable pendant l’exécution. L’inspection et les métriques de développement sont activées explicitement et absentes d’un build release ordinaire.
- Ne pas confondre le nombre de crates publiées avec le nombre de dépendances écrites dans le `Cargo.toml` d’une application. Le point d’entrée public unique pour les applications reste à concevoir sans imposer une fusion du moteur entier.

## État constaté au 25 septembre 2026

- Le workspace a **25 crates publiables**. [`scripts/release.py`](../../scripts/release.py) publie tous les membres publiables à chaque nouvelle version commune.
- [`argui-theme`](../../crates/argui-theme/) expose `Theme<T>`, des valeurs et schémas de tokens typés, des variantes, des overrides, `ThemeRuntime` et `ThemeChange`. Le runtime stocke/expose surtout `ThemeOverrides` et `ThemeSource` ; aucune application du dépôt n’instancie `ThemeRuntime`. Son comportement avancé est vérifié par des tests isolés.
- [`argui-reactive`](../../crates/argui-reactive/) fournit des propriétés observables, calculs, transactions et liens génériques. Son seul consommateur de production dans le dépôt est `ThemeRuntime`. Les apps Rust disposent déjà de leurs modèles et notifications ; Solid/React ont leur propre état réactif.
- La galerie utilise actuellement [`palette()`](../../packages/widgets/src/shared/theme.ts) et [`theme-variables.ts`](../../apps/gallery/src/theme-variables.ts), avec des signaux Solid ou l’état React. Ces chemins ne passent pas par `ThemeRuntime`.
- [`argui-inspect`](../../crates/argui-inspect/) contient les instantanés d’arbre, données de frames/CPU/GPU, historique et traces JSON. [`argui-runtime`](../../crates/argui-runtime/Cargo.toml) en dépend aujourd’hui sans feature optionnelle ; l’automation en consomme les données.
- [`argui-shader`](../../crates/argui-shader/) fait environ 252 lignes Rust et n’a qu’un consommateur de production local : `argui-render`. [`argui-android`](../../crates/argui-android/) et [`argui-ios`](../../crates/argui-ios/) font respectivement environ 97 et 38 lignes et enveloppent principalement les entrées mobiles du runtime.
- Le [CLI actuel](../../crates/argui-cli/src/lib.rs) initialise seulement des apps TSX, clone le dépôt hors checkout, exige `argui add <solid|react> <nom>...`, et ne connaît ni `list components` ni menu interactif. [`Project::target`](../../crates/argui-cli/src/project.rs) est aujourd’hui un choix exclusif `Native` **ou** `Web` ; `argui.json` ne stocke pas encore un ensemble de cibles ni la sélection de capacités. Le [registre](../../components/registry.json) connaît cinq composants et leurs chemins, sans version ni URL source par composant.

## Décisions de consolidation

| Sujet | Décision |
| --- | --- |
| `argui-reactive` | **Retirer la crate** du futur graphe publié. Ne pas copier ses API génériques dans `argui-theme`. Remplacer uniquement ce que le thème utilise par un état spécialisé, avec valeurs, révisions et changements groupés. Les versions déjà publiées restent sur crates.io ; documenter la migration des imports publics. |
| `argui-theme` | **Garder et raccorder réellement la crate**, par défaut, aux apps Rust et TSX. Ne pas la fusionner dans `runtime` avant d’avoir établi cette frontière commune. Elle doit être un moteur de thème utilisé, pas une bibliothèque testée uniquement. |
| `argui-shader` | Intégrer la validation et les diagnostics WGSL dans **un sous-dossier `crates/argui-render/src/shader/`** (`mod.rs` et fichiers séparés par responsabilité), puis retirer la crate séparée après migration des API et tests. Ne pas regrouper le code dans `render/src/lib.rs` ou dans un unique gros fichier. |
| `argui-android`, `argui-ios` | Intégrer les deux entrées dans **un même sous-dossier** de `argui-runtime`, par exemple `src/mobile/android.rs` et `src/mobile/ios.rs`, avec `mod.rs`. Supprimer les deux crates séparées après migration de la galerie et de la documentation mobile. |
| `argui-inspect` | Garder la crate comme contrat de données entre runtime et automation, mais rendre sa dépendance au runtime **optionnelle** via une feature d’inspection. L’automation et le CLI de test l’activent explicitement ; une application release normale ne l’active pas. |
| Autres crates | Garder les frontières principales `core`, `paint`, `text`, `animation`, `accessibility`, `ui`, `layout`, `render`, `platform` et `runtime`, ainsi que les capacités spécialisées tant qu’elles justifient leur isolation. |

Ces quatre retraits (`reactive`, `shader`, `android`, `ios`) feraient passer le workspace de **25 à 21 crates**, sans compter d’éventuels changements futurs au point d’entrée public. Ce nombre est une conséquence des décisions, pas un quota qui justifierait des fusions artificielles.

## Contrat du thème à rendre réel

1. **Une source de vérité par application/fenêtre**, sans singleton global : schéma de tokens définis par l’application, valeurs typées, valeurs par défaut, variantes clair/sombre et personnalisées, et overrides. Préserver le suivi des préférences système. Définir et documenter la priorité des couches (défaut, variante, override d’application/fenêtre, override local si pris en charge).
2. **Mise à jour atomique et prévisible** : une opération qui modifie plusieurs tokens produit un seul changement cohérent. Une valeur identique ne déclenche aucune mise à jour. `ThemeChange` rapporte les tokens effectivement modifiés et leur impact pour éviter les recalculs inutiles de layout/paint.
3. **API Rust pur** : l’application définit et modifie son thème pendant l’exécution ; les vues lisent les valeurs résolues par l’environnement/contexte Argui. Brancher les changements aux mécanismes de notification et de réconciliation existants. Aucun composant Rust ne doit dépendre d’un adaptateur JS.
4. **API TSX** : le bridge expose lecture, abonnement aux révisions et mutation du même thème, avec des adaptateurs Solid et React idiomatiques. Transmettre des instantanés ou diffs cohérents par révision, sans appel natif pour chaque lecture de token. Les composants reçoivent des valeurs résolues par le thème commun ; ils restent copiables et personnalisables avec `argui add`.
5. **Galerie** : migrer les pages Solid **et** React, leurs presets, le basculement clair/sombre, les accents et l’éditeur JSON vers ce thème commun. Garder les variables libres propres à l’application, mais les enregistrer comme tokens typés ; ne plus laisser `palette()` ou `theme-variables.ts` former un second moteur de résolution indépendant. Vérifier que les changements en direct affectent couleurs, texte, espacements, rayons, ombres et effets comme aujourd’hui. Couvrir aussi l’hôte Web si cette galerie y est exécutée.
6. **Capacité de base** : le thème n’est pas derrière une feature. Une app Rust ou TSX doit pouvoir l’utiliser sans déclaration d’une crate `argui-theme` supplémentaire dans son manifeste d’application, dès que le point d’entrée public sera finalisé.

Ne pas supprimer `argui-reactive` avant que le thème réel n’ait remplacé les quelques usages `Property`/`transaction` de `ThemeRuntime`. Ne pas conserver `Computed`, `Model`, `TwoWayLink` et autres API génériques à l’intérieur du thème sans besoin concret.

## Inspection et release légère

- Ajouter une feature explicite, par exemple `inspect`, à `argui-runtime` ; placer les types, champs, calculs et exports d’inspection derrière elle. Garder le chemin sans inspection simple et sans coût de collecte permanent.
- L’automation active cette feature et les features de métriques déjà présentes dans `argui-core` et `argui-layout`. Les builds ordinaires ne doivent pas les activer par accident via l’unification des features Cargo.
- Vérifier sur le graphe réel d’une application release que `argui-inspect`, l’automation, `sysinfo` et les métriques de développement ne sont pas présents sans demande explicite. Comparer taille du binaire et temps de compilation avant/après ; ne pas inférer le résultat de la seule présence d’un `cfg`.

## Entrées mobiles dans le runtime

- Utiliser `src/mobile/` dans `argui-runtime` pour les deux plateformes, avec des modules et APIs séparés à l’intérieur de ce dossier. Exposer des features d’entrée optionnelles pour Android et iOS ; combiner chaque feature avec le `cfg(target_os)` correspondant. Les cibles non mobiles n’incluent pas leurs points d’entrée.
- Dans `argui init`, **Mobile est un seul choix groupé Android + iOS** : sélectionner Mobile ajoute les deux plateformes à l’ensemble des cibles du projet, jamais une seule en silence. Il peut coexister avec Desktop natif et Web. Les features internes restent séparées et gardées par cible pour ne pas charger une plateforme sur l’autre.
- L’entrée Android conserve le contrat `NativeActivity`, son macro d’export et les fonctions de lancement. L’entrée iOS conserve son macro C/Objective-C et ses réexports de lancement. Vérifier les chemins d’appel des macros et les symboles exportés après déplacement.
- Migrer l’hôte QuickJS de la galerie, les manifestes consommateurs, les exemples, tests et documents mobiles. Compiler les cibles mobiles déjà couvertes par la CI avant de supprimer les anciennes crates.
- La [documentation mobile actuelle](../native-mobile.md) décrit Android comme packagé et iOS comme une entrée compilée sans shell ni packaging complet. Ne pas présenter l’option groupée comme installable sur les deux plateformes tant que le générateur et les builds iOS ne le sont pas ; l’interface doit annoncer clairement l’indisponibilité ou achever ce support avant de permettre la sélection.

## CLI : création d’une application autonome

### Commande et emplacement

```text
argui init
argui init --dir .
argui init --dir ./mon-app
argui init solid --dir ./mon-app --targets native,web
argui init react --dir ./mon-app --targets native,web
argui init rust --dir ./mon-app
argui build release                 # toutes les cibles sélectionnées
argui build release --target web    # une seule sortie
argui dev --target native           # hôte natif de la même app TSX
argui dev --target web              # hôte WASM de la même app TSX
```

- `--dir` désigne **le dossier final de l’application**, relatif au dossier courant ou absolu ; sa valeur par défaut est `.`. Le nom de l’app vient du dossier final, avec `--name` si nécessaire. Aucun `apps/<nom>` ni dépôt Argui ne sont générés autour de cette app.
- Ajouter réellement le chemin **Rust pur** au générateur ; aujourd’hui `Framework` ne représente que Solid/React. Choisir Rust pur, Solid ou React au début du parcours. Remplacer le `Target` exclusif actuel par un **ensemble de cibles** : Desktop natif et Web peuvent être sélectionnés ensemble, puis Mobile ajoute Android et iOS ensemble. Pour une app TSX, Desktop + Web sont cochés par défaut dans le menu ; l’utilisateur peut garder l’un, l’autre ou les deux. Valider les combinaisons avant toute écriture.
- Pour Solid/React, générer **un seul `src/` TSX** et des hôtes/bundles natif et Web/WASM à partir de ce code commun. Les différences de services propres à une plateforme sont isolées dans les adaptateurs ; ne pas générer deux copies divergentes des pages ou des composants. Un projet Rust pur peut aussi sélectionner plusieurs cibles lorsque son hôte et ses dépendances sont compatibles.
- Stocker la liste des cibles dans `argui.json`. `argui check` valide le code commun et les cibles sélectionnées ; `argui build [dev|release]` construit **toutes** les sorties sélectionnées par défaut, avec `--target <cible>` pour n’en construire qu’une. `argui dev --target native|web` lance l’hôte demandé ; sans flag, choisir Desktop si présent, sinon Web. `test` et `screenshot` précisent leur cible quand elle ne peut pas être déduite. Les artefacts natifs et Web vont dans des dossiers de sortie distincts.
- Générer les manifestes et hôtes propres à l’application, avec des versions Argui épinglées et **sans `workspace:*` ni chemin vers le monorepo**. Conserver les dépendances Rust internes transitives derrière un point d’entrée simple pour l’app ; décider après prototype si l’entrée publique réutilise `argui` ou un autre paquet, sans fusionner physiquement tout le workspace.
- Préparer et vérifier tous les fichiers et prérequis avant mutation. Accepter un dossier existant seulement si aucun fichier à générer n’y serait écrasé ; laisser un résultat complet ou nettoyer les fichiers créés si une erreur survient. `check`, `dev`, `build`, `test` et `screenshot` doivent fonctionner depuis le projet autonome.
- Le bridge, le JSX runtime et les types Solid/React restent nécessaires sans `argui add`. Valider leur distribution versionnée hors monorepo (SDK de release GitHub ou petits adaptateurs npm si la résolution JS rend le SDK fragile). Les composants personnalisables restent copiés par `add`.

### Menu interactif par défaut

- Sur un terminal interactif, `argui init` affiche d’abord langage/framework, puis **une sélection multiple des cibles** (Desktop natif, Web et le groupe Mobile), puis une **liste déroulante** de capacités. `↑`/`↓` déplacent le curseur, `Espace` coche ou décoche, `Entrée` valide, `Échap` annule. Afficher la position et une description courte de l’option sélectionnée ; le défilement doit fonctionner quand la liste dépasse le terminal.
- Séparer visuellement **Required** (présent, non désactivable) et **Optional** (cases à cocher). Afficher des noms utiles à un utilisateur, puis les crates/features correspondantes dans les détails. Le menu ne doit pas exposer 21 crates comme 21 décisions techniques.
- Les choix optionnels sont initialement sobres et prévisibles. Pour une app multi-cibles, une capacité propre à Desktop reste activable **pour Desktop seulement** sans casser le build Web ; montrer les cibles concernées dans sa description. Désactiver avec explication les choix incompatibles avec toutes les cibles sélectionnées ou non pris en charge. Afficher un résumé avant génération. Les choix retenus sont enregistrés dans `argui.json` et reflétés dans les manifestes/flags de chaque build ; une case cochée qui ne change aucun build est un bug.
- En environnement sans TTY, ne jamais attendre des touches : accepter des flags explicites de framework, cible et capacités, avec une forme déterministe pour scripts/CI (`--feature` répétable ou équivalent, et option d’acceptation des défauts). Les erreurs indiquent la combinaison invalide et la commande corrigée.

| Required affiché en lecture seule | Rôle |
| --- | --- |
| `argui-core`, `argui-paint`, `argui-text`, `argui-animation`, `argui-accessibility` | Types, dessin neutre, texte, animation et sémantique du moteur. |
| `argui-ui`, `argui-layout`, `argui-render`, `argui-platform`, `argui-runtime` | Arbre retenu, géométrie, GPU, système de fenêtres et cycle de vie. |
| `argui-theme` | Tokens, variantes et overrides utilisables en direct dans toute app. |
| `argui-schema` et `argui-host` pour Solid/React seulement | Contrat et hôte des transactions TSX ; ne pas les imposer inutilement à Rust pur. |

| Optional dans le menu | Description utilisateur et effet attendu |
| --- | --- |
| **Automation & metrics** | Tests d’interaction sans fenêtre, captures ciblées et profil CPU/GPU/mémoire. Active `argui-automation`, `argui-inspect` et les features de métriques pour les builds de test ; absent d’une release normale sauf demande explicite. |
| **Internationalization** | Catalogues et choix de langue via `argui-i18n` ; pour TSX, inclure l’adaptateur JS nécessaire. |
| **Media** | Décodage image/SVG et assets via `argui-media/media` ; ne pas activer les décodeurs lourds sans sélection. |
| **Effects** | Presets visuels choisis dans `argui-effects` ; présenter les familles disponibles sans forcer tous les shaders. |
| **WebView** | Contenu web intégré via `argui-runtime/webview`, avec vérification des prérequis natifs. |
| **Updater** | Vérification et installation de mises à jour signées via `argui-updater/native`. |
| **Tasks** | Tâches asynchrones du runtime. |
| **Desktop integrations** | Sous-choix décrits pour file picker, tray, raccourcis globaux, popups natifs et backdrop de bureau ; n’afficher que ceux compatibles avec la cible. |
| **Mobile (Android + iOS)** | Groupe de cibles ajouté en bloc au projet, y compris si Desktop et Web sont déjà choisis. Disponible seulement quand `init` et les builds générés peuvent réellement prendre en charge les deux ; sinon expliquer ce qui manque. |

Le tableau est un **catalogue cible**, pas la promesse que toutes ces cases fonctionnent aujourd’hui : la recherche Luna Max a confirmé qu’`init` ne génère actuellement aucun `Cargo.toml` d’app et construit l’hôte QuickJS partagé. Chaque option doit être reliée au manifeste et au build avant d’apparaître comme sélectionnable.

## CLI : composants copiables

```text
argui add button input dialog
argui add --solid button input-field
argui add button --react select --project ./mon-app
argui list components
argui list components --solid --project ./mon-app
argui list components --json
```

- `argui add` accepte **autant de noms que permet la ligne de commande du système**. Dédupliquer les noms et fichiers partagés. Accepter `--solid`, `--react` et `--project <dir>` avant, entre ou après les noms ; rejeter les flags inconnus ou contradictoires. `input` peut être un alias documenté de `input-field`.
- Détecter le framework depuis `argui.json` à partir du dossier courant ou d’un parent ; `--project` choisit explicitement l’app. Un flag de framework l’emporte, y compris pour ajouter une variante Solid à une app React ou l’inverse, en installant alors ses dépendances JS. Si la détection est absente ou ambiguë, produire une erreur utile plutôt qu’un choix silencieux.
- Copier les sources modifiables dans `ui/solid-components/` ou `ui/react-components/`, et les aides communes dans `ui/shared/`. Suivre chaque composant/fichier, sa version et son empreinte dans `argui.json`. Ne jamais écraser silencieusement un fichier modifié ou non suivi.
- Valider **tout le lot avant la première écriture** : noms, variantes, dépendances, version du registre, source, taille, chemins, symlinks, collisions, checksums et état des fichiers déjà installés. Mettre fichiers et manifeste en staging, puis appliquer avec rollback si un échec d’écriture survient ; aucun ajout partiel ne doit subsister.
- Obtenir le registre et les sources du **tag exact** de la version Argui de l’app, avec cache vérifié ; le projet ne requiert pas un checkout Argui. Un composant installé appartient ensuite au dépôt de l’utilisateur, comme avec shadcn.
- `argui list components` fonctionne aussi hors projet avec le catalogue de la version du CLI. Afficher au minimum **Name** et **Source**, puis **Framework**, **Version** et **Installed** quand disponibles. `Source` est un lien GitHub vers le fichier ou dossier au tag immuable, pas vers `main`. `--json` expose les mêmes données et les chemins/dépendances de fichiers pour les outils.
- Étendre le registre : distinguer `registry.version` (format), version Argui de release, éventuelle version de composant, URLs sources par variante, fichiers et sommes de contrôle. Le catalogue doit être cohérent avec l’archive utilisée par `add` et avec `argui.json`.

## Organisation des agents pour la mise en œuvre ultérieure

- **Recherche : Luna Max** (`gpt-6-luna`, effort `max`). Faire l’inventaire en lecture seule des API, dépendances, limites des cibles et parcours utilisateur avant de déplacer du code. La recherche CLI initiale a constaté les écarts cités plus haut ; compléter seulement les questions restées ouvertes, notamment l’entrée publique Rust, la distribution du SDK JS et le support mobile réel.
- **Implémentation : sous-agents Sol 6** (`gpt-6-sol`, effort `high` ou `xhigh` selon le risque). Leur attribuer des sous-tâches bornées et des fichiers propriétaires explicites : (1) thème + galerie ; (2) consolidation renderer/mobile/inspection ; (3) `init` autonome et menu ; (4) registre, `add` et `list components`. Échelonner les sous-tâches qui touchent `argui-runtime`, `crates/argui-cli/src/lib.rs`, `Cargo.toml` ou le release script ; un seul agent intègre ces fichiers à la fois.
- L’agent principal conserve l’intégration du graphe Cargo, du dispatch CLI, de la documentation et des vérifications de bout en bout. Ne pas lancer plusieurs quality gates concurrents ; suivre `AGENTS.md` pour la passe finale unique avant commit.
- **Ce tour est documentaire** : ne pas lancer les sous-agents d’implémentation ni coder sur la base de ce plan sans nouvelle instruction.

## Ordre de mise en œuvre et vérification

1. Lire `AGENTS.md` et [`code-quality.md`](../contributing/code-quality.md). Fixer les API publiées et la matrice Rust pur/Solid/React × **ensembles de cibles** Desktop/Web/Mobile, y compris Desktop + Web pour une seule app TSX. Décider le paquet d’entrée public simple pour les apps, tout en conservant plusieurs crates physiques. Aucun DSL n’est à préserver ou préparer.
2. Rendre le thème réellement observable dans **une app Rust pur, une page Solid et une page React** avec le même schéma et les mêmes règles de résolution. Tester changement en direct, override, retour au défaut et préférence système ; migrer ensuite toute la galerie et ses composants sans perdre la personnalisation du code copié.
3. Retirer la dépendance `argui-reactive`, puis sa crate, ses entrées workspace/release et ses références documentaires. Rendre `argui-inspect` optionnel pour le runtime et vérifier le graphe d’un build release sans instrumentation ainsi que le mode automation avec métriques.
4. Déplacer `shader` dans `render/src/shader/`, puis Android/iOS dans `runtime/src/mobile/`. Conserver des modules séparés et déplacer les tests dans les dossiers `tests/` correspondants ; migrer les usages, docs et manifestes. Vérifier les cibles mobiles déjà couvertes et combler le manque iOS avant d’activer l’option groupée du menu.
5. Construire un projet autonome directement dans `--dir` pour Rust pur, Solid et React. Faire compiler **le même projet TSX en natif et en Web/WASM en une commande**, sans dupliquer les sources. Faire fonctionner `check`, `dev`, `build`, `test` et `screenshot` hors monorepo, puis ajouter le menu interactif et son équivalent sans TTY. Prouver que chaque case optionnelle modifie effectivement les manifestes et les builds concernés.
6. Étendre le catalogue versionné, puis implémenter `list components` et `add` multi-composants atomique avec détection/override du framework. Vérifier l’installation de sources modifiables et la reconstruction depuis un checkout Git frais de l’app générée.
7. Pendant l’implémentation, utiliser les tests ciblés et mesurer taille/temps de build. Exécuter la qualité finale prescrite par `AGENTS.md` à la fin et avant tout commit. Les tests d’automation sans fenêtre utilisent directement `argui test` ; les contrôles GUI natifs Linux suivent [`linux-testing.md`](../contributing/linux-testing.md).

## Critères de sortie

- Galerie Solid/React alimentée par `argui-theme`, thème dynamique fonctionnel en Rust pur, aucune dépendance à `argui-reactive` ; composants copiés toujours personnalisables.
- Inspection et métriques absentes du graphe release ordinaire, accessibles en test ou sur demande explicite.
- Entrées mobiles sous `runtime/src/mobile/`, validation WGSL sous `render/src/shader/`, publication cohérente avec le nouveau graphe multi-crates.
- `argui init --dir .` et `--dir <chemin>` créent l’app directement dans ce dossier sans clone ; le menu permet de cocher **Desktop natif + Web pour la même app TSX**, puis les capacités Required/Optional. Le mode sans TTY est utilisable en CI.
- `argui add` accepte plusieurs composants et les flags de framework à toute position, valide tout le lot et ne laisse aucun ajout partiel ; `argui list components` affiche nom et source versionnée, avec variantes et état d’installation.
- Applications Rust pur, Solid et React construites hors du monorepo avec des dépendances publiées/versionnées, sans `workspace:*` ou chemin local vers Argui. Une app Solid et une React de test produisent chacune, depuis leurs **mêmes sources TSX**, les sorties native et Web/WASM en un `argui build release`. Le choix Mobile annoncé dans l’interface construit réellement Android **et** iOS.
