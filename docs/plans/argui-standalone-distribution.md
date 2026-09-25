# Distribution autonome d’Argui et composants copiables

> **Archive : plan remplacé.** Suivre le [plan global Argui](argui-crate-theme-decisions.md). La proposition ci-dessous d’une seule crate publiée n’est plus retenue ; le nouveau plan reprend aussi les décisions CLI, thème, composants et mobiles.

Statut : plan pour une prochaine tâche. Aucune étape ci-dessous n’est encore implémentée.

## Décision et résultat attendu

- Publier **un seul paquet Rust `argui` sur crates.io**, contenant le moteur organisé en modules et des fonctionnalités optionnelles. Le CLI reste dans le même dépôt et porte la même version ; ses binaires sont distribués par les releases GitHub. Son paquet Cargo peut rester privé (`publish = false`).
- `argui init` crée l’application directement dans `.` ou dans le chemin demandé. Le projet généré dépend de `argui` par version sur crates.io et ne contient aucune copie du dépôt Argui.
- Le code indispensable aux adaptateurs Solid/React (bridge, JSX runtime, types) est fourni par un petit SDK JavaScript associé à la version d’Argui, sans imposer la publication de packages npm Argui. Le CLI le récupère depuis une archive de release GitHub et le place dans un dossier ignoré par Git. La faisabilité avec Vite, TypeScript, Bun et les éditeurs doit être vérifiée avant de figer cette voie ; publier les adaptateurs sur npm reste le repli si cette intégration crée trop de friction.
- `argui add` copie **uniquement les composants choisis** dans `ui/react-components/` ou `ui/solid-components/`. Ces fichiers appartiennent ensuite à l’application : l’utilisbo ateur les commit et les personnalise.

Le nombre de crates internes n’est pas la cause des milliers de lignes visibles dans le dépôt d’une application : c’est le clone actuel effectué par `argui init`. Une dépendance crates.io vit dans le cache de Cargo, hors du dépôt de l’utilisateur. Le choix d’un seul paquet publié simplifie en plus la publication, mais son effet sur le temps de compilation doit être mesuré.

## État du dépôt à reprendre

- [`crates/argui-cli/src/project.rs`](../../crates/argui-cli/src/project.rs) : hors du dépôt, `init` clone le tag GitHub correspondant au CLI, puis génère `apps/<nom>/` dans cette copie. `check`, `dev`, `build` et le Web cherchent ensuite le dépôt avec `find_root`.
- [`apps/gallery/quickjs-host/Cargo.toml`](../../apps/gallery/quickjs-host/Cargo.toml) : l’hôte TSX natif est une application de galerie, avec des dépendances par chemin vers les crates Argui et les assets de la galerie. Le Web possède aussi un hôte spécifique sous `apps/web-host/`.
- [`crates/argui-cli/src/components.rs`](../../crates/argui-cli/src/components.rs) : `add` valide déjà les noms et chemins, copie des sources, suit les fichiers dans `argui.json` et protège les conflits. Il lit toutefois [`components/registry.json`](../../components/registry.json) et les fichiers des widgets depuis le dépôt cloné, impose le framework du projet et écrit dans `src/argui-ui/`.
- [`crates/argui-cli/src/lib.rs`](../../crates/argui-cli/src/lib.rs) : la syntaxe actuelle est `argui add <solid|react> <composant>...` ; `argui list components` n’existe pas.
- [`scripts/release.py`](../../scripts/release.py) et [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml) publient aujourd’hui les crates du workspace en ordre de dépendance, puis les binaires CLI.
- Le nom [`argui` existe déjà sur crates.io](https://docs.rs/crate/argui/latest) comme ancienne façade publiée par ce projet ; la version 0.3.2 dépend de nombreuses crates `argui-*`. Comparer l’API et les propriétaires de cette publication avant de la remplacer par le paquet consolidé. Les anciennes versions et leurs dépendances restent disponibles ; l’objectif « une publication » concerne les prochaines versions.
- Le profil de développement actuel définit `incremental = false` dans [`Cargo.toml`](../../Cargo.toml). Rust sait réutiliser des résultats à l’intérieur d’une crate quand la compilation incrémentale est activée ; comparer les deux réglages pendant la consolidation.

## Contrat CLI proposé

```text
argui init solid .
argui init react ./mon-app --target web

argui add button input
argui add --react button input-field dialog
argui add button --solid input --project /chemin/vers/mon-app

argui list components
argui list components --react
argui list components --json --project /chemin/vers/mon-app
```

### `init`

- `.` signifie le dossier courant ; un chemin relatif ou absolu désigne exactement le dossier de l’application. Le nom du projet vient du nom du dossier si aucun nom explicite n’est donné.
- Refuser un dossier contenant des fichiers que l’init devrait remplacer. Générer à la racine de l’application `argui.json`, `Cargo.toml`, `Cargo.lock`, `package.json`, `bun.lock`, les sources, la configuration TS/Vite et `.gitignore` ; ne pas générer `apps/<nom>/` autour de l’application.
- Le `Cargo.toml` généré mentionne une seule dépendance Argui versionnée, avec les features adaptées au target. Le CLI installe ou vérifie les dépendances nécessaires et garde `argui doctor` pour diagnostiquer les prérequis système.
- `argui check`, `dev`, `build`, `test` et `screenshot` doivent fonctionner depuis la racine générée, depuis un sous-dossier par remontée vers `argui.json`, ou avec un chemin de projet explicite. Aucun ne doit requérir le dépôt Argui.

### `add`

- Accepter un nombre arbitraire de noms de composants, sous réserve de la limite de la ligne de commande du système ; supprimer les doublons. Accepter `--solid` et `--react` avant, entre ou après ces noms. Les deux flags ensemble sont une erreur claire.
- Sans flag, lire le framework d’`argui.json`. Avec flag, sélectionner explicitement cette variante même si l’application utilise l’autre par défaut ; installer ses dépendances JS nécessaires sans changer le framework par défaut du projet.
- Chercher `argui.json` dans le dossier courant puis ses parents. `--project <chemin>` permet l’appel depuis n’importe quel autre dossier ; le flag fonctionne à toute position. Si aucun projet n’est identifiable, expliquer l’usage de `--project`.
- Accepter des alias documentés tels que `input` pour le composant canonique `input-field`. Afficher les noms canoniques dans `list` et dans `argui.json`.
- Écrire les variantes sous `ui/solid-components/` et `ui/react-components/`. Placer les petits auxiliaires communs sous `ui/shared/`, avec des imports relatifs valides quand une ou deux variantes sont installées. Mettre à jour les chemins TypeScript si nécessaire.
- Les fichiers copiés sont suivis dans `argui.json` avec version source et empreinte. Une répétition identique est sans effet. Ne jamais écraser silencieusement une modification de l’utilisateur ; une mise à jour future doit afficher un diff ou demander une action explicite.
- Lire les sources depuis une archive de composants liée au **tag exact** de la version déclarée dans le projet, publiée sur GitHub avec un manifeste et des sommes de contrôle. Mettre l’archive en cache pour les appels suivants et pour le mode hors ligne. Valider les chemins et tailles avant extraction. Les liens affichés par le CLI doivent pointer vers le code source à ce tag, jamais vers `main`.

### `list components`

- Fonctionner sans application à partir du catalogue associé à la version du CLI ; dans une application, utiliser sa version Argui épinglée et indiquer les composants déjà copiés.
- Afficher un tableau stable et lisible avec `Nom`, `Variantes`, `Version`, `Installé` et `Source`. `Source` renvoie au chemin GitHub épinglé ; `--json` expose l’URL complète, la révision, les chemins sources et les dépendances de fichiers pour les agents et éditeurs.
- `--solid` et `--react` filtrent les variantes disponibles ; `--project` fournit le contexte d’une application depuis un autre dossier. Le registre distingue sa version de schéma de la version des composants publiée avec Argui.

## Architecture Rust : un paquet publié, des modules cohérents

Cargo autorise un seul paquet contenant beaucoup de modules et des dépendances optionnelles. Il n’oblige pas Argui à publier chaque sous-système séparément. En revanche, le paquet `argui` publié ne peut pas dépendre de crates internes disponibles **seulement par chemin** : elles doivent être publiées séparément ou leur code doit être intégré à `argui`.

La direction choisie est l’intégration du code du moteur dans un paquet `argui`, avec des dossiers de modules par responsabilité (`core`, `animation`, `layout`, `text`, `paint`, `render`, `platform`, `runtime`, `host`, `automation`, etc.). Garder des fichiers courts et des interfaces internes explicites ; déplacer les fichiers et tests plutôt que créer un fichier géant ou dupliquer le moteur. Exposer une API publique réduite depuis la racine de `argui` et laisser les détails en `pub(crate)`.

- Transformer l’hôte QuickJS de la galerie en API générique de `argui` : l’application générée fournit son bundle et ses services ; la galerie devient un consommateur de cette API. Le nom, les assets et les fenêtres spécifiques à la galerie restent dans `apps/gallery/`.
- Faire de même pour l’hôte Web ou l’entrée WASM : l’application possède son petit lanceur, le moteur réutilisable vit dans `argui`.
- Utiliser `cfg(target_...)` pour les plateformes et des features additives pour les capacités coûteuses (`tsx`, médias, WebView, tray, etc.). Définir des valeurs par défaut sobres. La trace de métriques et l’automation restent absentes de la release ordinaire ; `argui test` construit son hôte de test avec ces capacités.
- Garder le CLI comme paquet du workspace non publié sur crates.io, distribué en binaires GitHub à la même version que `argui`. Cela garde une seule publication Rust sans mélanger les dépendances de l’outil dans la bibliothèque. Les applications n’ont que `argui` dans leur manifeste Rust.
- Laisser les coquilles mobiles, exemples et galerie non publiés ; ils utilisent la même crate `argui`. Préserver leur comportement pendant la migration.

Mesurer la taille de l’archive `.crate`, le graphe de dépendances activé, la taille des binaires, le temps de build froid et le temps de rebuild après une petite modification. Faire cette mesure avec et sans compilation incrémentale pour le développement local. Une seule publication simplifie la CI de **publication** ; elle ne garantit pas une CI de **compilation** plus rapide.

## SDK JavaScript sans package npm Argui

`argui add` suffit pour les composants personnalisables. Le bridge, le JSX runtime et les types Solid/React doivent toutefois être résolus même dans une application qui n’ajoute aucun composant.

Premier choix à valider : l’archive SDK GitHub contient ces fichiers pour la version d’Argui ; le CLI la télécharge et la décompresse dans `.argui/sdk/`, ignoré par Git. Il configure les alias Vite et les chemins TypeScript, et `check`/`dev`/`build`/`test` matérialisent automatiquement la version requise. Le projet garde ses dépendances ordinaires `solid-js` ou `react` et ses lockfiles, sans package npm Argui ni clone du monorepo.

Avant de généraliser ce choix, réaliser une application Solid et une React **hors du dépôt** et vérifier : imports, JSX, éditeur TypeScript, hot reload, `argui add`, tests, Web, Windows/macOS/Linux et reconstruction à partir d’un checkout Git frais. Si le SDK géré par le CLI provoque des exceptions fragiles dans la résolution JS, publier les seuls adaptateurs de base sur npm ; les composants restent toujours copiés via `argui add`.

## Ordre de travail pour le prochain agent

1. Lire `AGENTS.md`, [`docs/contributing/code-quality.md`](../contributing/code-quality.md) et, avant les contrôles GUI Linux, [`docs/contributing/linux-testing.md`](../contributing/linux-testing.md). Faire un inventaire des imports entre crates et du chemin réel `init` → `check` → `dev` → `build` → `test`.
2. Comparer l’API de la façade `argui` déjà publiée avec la future API, puis prouver sur un petit prototype empaqueté localement que le paquet consolidé, ses features et les cibles native/WASM sont empaquetables. Fixer la surface API publique et les modules internes ; mesurer compilation et taille avant de déplacer tout le moteur.
3. Consolider les crates du moteur sans copie permanente de sources. Porter les tests avec leurs modules ; migrer la galerie, les exemples et les coquilles mobiles vers `argui`.
4. Extraire l’hôte TSX générique, créer une application Solid et une React autonomes hors du monorepo, puis remplacer `init` et les commandes qui appellent `find_root`.
5. Valider le SDK JavaScript versionné sans npm, puis implémenter `add` et `list components` avec les comportements ci-dessus. Mettre à jour documentation et tests CLI.
6. Adapter la publication : un seul `cargo publish` pour `argui`, puis archive SDK/composants et binaires CLI du même tag. Vérifier leurs versions et empreintes avant de présenter le CLI comme installable. Faire un test de consommation depuis un dossier vierge, avec la crate du registre et aucun chemin local.
7. Exécuter les contrôles ciblés pendant le travail puis la qualité finale prescrite par `AGENTS.md` avant commit. Les contrôles d’automation sans fenêtre utilisent directement `argui test` ; réserver l’affichage privé aux vérifications GUI natives.

## Critères de sortie

- Le dépôt d’une application générée ne contient pas le dépôt Argui. Son manifeste Rust ne nomme qu’une crate Argui et ses lockfiles sont commitables.
- Le graphe normal d’une application issue de crates.io ne contient aucune autre crate `argui-*` et aucun chemin vers le dépôt de développement. `argui` est le seul paquet Argui publié sur crates.io pour cette version.
- `argui init solid .`, `argui init react <chemin>`, `argui add` avec plusieurs composants et flags à toute position, `argui list components`, `check`, `dev`, `test` et `build release` fonctionnent hors du monorepo.
- Les fichiers ajoutés sous `ui/<framework>-components/` sont éditables et ne sont jamais remplacés silencieusement. Le catalogue affiche version, source épinglée et état d’installation.
- Le binaire release ordinaire ne contient ni moteur de test ni métriques de développement, à moins d’une feature explicite. Les temps de build et tailles avant/après sont consignés.

## Références externes consultées

- [Blinc : releases CLI avec binaires par plateforme et commande `doctor`](https://github.com/project-blinc/Blinc/releases) ; [README : plusieurs crates Rust et entrée `blinc_app`](https://github.com/project-blinc/Blinc).
- [Iced : paquet d’entrée `iced`, crates internes et features](https://github.com/iced-rs/iced/blob/master/Cargo.toml).
- [egui : plusieurs crates dans le workspace](https://github.com/emilk/egui/tree/main/crates) ; [architecture officielle](https://github.com/emilk/egui/blob/main/ARCHITECTURE.md).
- [Cargo : features et dépendances optionnelles](https://doc.rust-lang.org/cargo/reference/features.html), [publication et dépendances locales](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#local-paths-in-published-crates), [librairie et binaire dans un paquet](https://doc.rust-lang.org/cargo/reference/cargo-targets.html).
