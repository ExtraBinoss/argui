# API TSX v2 : contrat simple avant reconstruction des widgets

Statut : **plan de migration v2 en cours d'implémentation** au 26 septembre
2026, à partir de `fd8d28d2`. Ce document fixe la direction demandée et les
cinq premiers widgets ; les questions historiques restent visibles. Il remplace les conventions TSX
et le périmètre de migration de la galerie des plans précédents en cas de
conflit. Les décisions sur les crates, les cibles et la distribution restent
dans [le plan global](argui-crate-theme-decisions.md), avec les précisions
ci-dessous. L'implémentation a été explicitement demandée après la rédaction
de ce plan.

## Objectif et périmètre

Une même application TSX doit être naturelle à écrire en Solid et en React,
avec le moteur natif Argui, son arbre retenu, son schéma et ses transactions.
Les deux adaptateurs offrent les mêmes noms de propriétés et les mêmes contrats
de composants. Leur gestion de l'état reste idiomatique à chaque framework.
Les applications Rust pur et les cibles native et Web restent prises en charge.
Il n'y a pas de DSL supplémentaire.

La v2 est une **rupture nette**. Retirer des chemins actifs les anciennes
propriétés, les anciens composants et leurs appelants ; seule l'archive
`OLD_API/` les conserve comme référence. Ne pas ajouter d'alias de migration,
de couche de compatibilité ou d'export déprécié. Prévoir une note de migration
et une version de release qui signale la rupture.

## Contrat public à stabiliser

### Précondition : le layout public n'est pas encore figé

Le moteur sait déjà calculer des arbres flex et grid, mesurer le texte, gérer
défilement et positionnement. Il n'y a pas de décision de le refaire. En
revanche, son **API publique Rust/TSX doit être conçue et éprouvée avant le gel
de la v2**. La [comparaison Web, Iced et Slint](layout-api-research.md) motive
la proposition et ses scénarios de validation. Les cas suivants sont encore
des décisions de contrat :

| Sujet | Situation actuelle | Décision à prendre avant le gel |
| --- | --- | --- |
| Taille et remplissage | Dans le wire TSX, `width="fill"` devient `100%` du parent, tandis que l'espace restant d'un flex se demande par `grow`. `fit` devient `auto`. | Nommer distinctement taille relative, taille intrinsèque et croissance flex ; définir les unités et les parents à taille automatique. |
| Contraintes et rétrécissement | Les bornes `min_width`/`max_width` TSX sont des nombres en pixels ; Rust accepte des longueurs et pourcentages typés. Certains composants ajoutent `min_width={0}` pour pouvoir rétrécir. | Définir `minWidth`/`maxWidth`, `flexBasis`, `grow` et `shrink`, leurs valeurs par défaut et les cas de texte long ou viewport étroit. |
| Placement des widgets | Les props actuelles de `Button` et `InputField` n'exposent pas les propriétés de layout du conteneur ; `Select` a seulement un `width` spécifique. | Choisir un petit contrat de layout commun appliqué à la racine du widget (`width`, `grow`, `alignSelf`, etc.) ou une règle de wrapper explicite ; vérifier qu'un formulaire simple reste court à écrire. |
| Alignement et espacement | `row`/`column` existent, mais les alignements TSX sont des chaînes libres décodées en Rust. `gap`, `row_gap`, `column_gap` et les côtés de padding se chevauchent. | Choisir les noms, options fermées, unités, priorité des props composées et comportement RTL ; rendre la composition ordinaire courte. |
| Grid | Rust expose les pistes et placements Taffy typés ; TSX accepte une petite grammaire textuelle pour `grid_rows`/`grid_columns`, validée au runtime. | Choisir une forme typée simple qui couvre les pistes et placements réellement nécessaires, avec une voie avancée claire. |
| Positionnement | `x`/`y` imposent `absolute`, mais `position` et les `inset_*` peuvent aussi être fournis. Les transforms changent le rendu visuel, pas l'espace de layout. | Définir une seule règle pour `absolute`, `sticky`, insets, ancres et transforms ; rejeter ou résoudre explicitement les combinaisons contradictoires. |
| Défilement | `scroll_y` sur un conteneur ajoute une scrollbar prescrite ; `Flickable` fournit un viewport neutre avec un autre contrat. | Choisir le chemin recommandé pour le scroll, les axes, la taille du viewport, les scrollbars et l'imbrication. |
| API Rust | `LayoutStyle` et beaucoup de builders exposent directement des types Taffy ; TSX n'en présente qu'une partie. | Décider quelles sémantiques sont publiques et communes, et quelle surface avancée Rust peut rester liée à Taffy. |

Proposition à discuter : garder `row`, `column` et `grid` comme modèles de
composition ; écrire directement les propriétés fréquentes du conteneur
(`gap`, `padding`, `alignItems`, `justifyContent`) et de l'enfant (`width`,
`minWidth`, `grow`, `alignSelf`). Les widgets devraient accepter le petit jeu de
propriétés nécessaire au placement de leur racine, sans exposer toutes les
propriétés de leurs détails internes. Un nombre de taille désigne des pixels
logiques ; `auto` garde la taille intrinsèque ; un pourcentage se réfère au
conteneur ; `grow` prend l'espace flex disponible. **Décision confirmée :**
`width`/`height` sont des tailles préférées à la Web/Taffy et peuvent rétrécir
dans un flex. Retirer ou renommer `fill`
si sa promesse reste ambiguë. Rust garde des méthodes idiomatiques en
`snake_case` pour ces mêmes opérations. Les détails de grid, d'insets et de
scroll restent ouverts jusqu'aux scènes comparatives.

Le contrat doit être démontré sur les mêmes scènes en Rust pur, Solid et React :
ligne à deux panneaux dont un grandit, colonne avec texte long et champ,
grille responsive, viewport à défilement imbriqué, élément sticky, popup ancré,
liste virtuelle et direction RTL. Vérifier la géométrie obtenue à des tailles
de fenêtre différentes et après changement de texte/thème. Ces scènes
décideront les défauts de largeur, d'alignement et de taille minimale ; ne pas
les déduire d'un seul exemple de galerie. Le gel de l'API publique vient après
cette matrice, puis les cinq widgets valident le contrat choisi.

### 1. Un seul vocabulaire camelCase

- Tous les noms contractuels de propriétés, événements, observations et
  paramètres exposés à TSX sont en `camelCase` : types générés Solid/React,
  schémas natifs Rust, contrat JSON, encodage/décodage du wire, hôtes natif/Web,
  exemples, documentation et tests. Exemples : `alignItems`, `fontSize`,
  `accessibleName`, `onValueChange`, `shadowOffsetY`.
- La source canonique produit ces noms. Aucun adaptateur ne maintient sa propre
  table manuelle de synonymes `snake_case`/`camelCase`.
- `PascalCase` reste réservé aux noms de types, de composants et de variantes
  Rust qui sont des types ; `kebab-case` reste possible pour les commandes CLI
  et les noms de fichiers. **Décision confirmée :** les noms publics du schéma et
  du protocole définis côté Rust sont en `camelCase`, tandis que les fonctions,
  champs et autres identifiants internes du code Rust restent idiomatiques en
  `snake_case`. La conversion se fait à la frontière publique, sans alias.
- Un contrôle de génération refuse les noms contractuels qui ne sont pas en
  `camelCase` et toute divergence entre le schéma, le SDK embarqué par le CLI
  et les hôtes. Un contrat ABI obsolète empêche la publication.

### 2. Primitives courantes faciles à lire

- Garder des primitives natives pour la composition et les cas avancés, mais
  limiter la surface recommandée au départ : disposition, texte, surface,
  défilement, liste virtuelle et portail. Décrire quand employer un widget et
  quand descendre à une primitive.
- Conserver directement les propriétés de disposition les plus utilisées
  (`width`, `height`, `gap`, `padding`, `alignItems`, etc.). Regrouper les concepts
  composés, notamment `border`, `shadow`, `radii` et `transform`, dans des valeurs
  structurées typées. Éviter un objet `style` universel qui rendrait chaque
  élément plus long à écrire et créerait deux façons de régler la même valeur.
- Revoir les valeurs par défaut de largeur et d'alignement à partir des pages
  réelles pour retirer les `width="fill"` répétitifs, sans rendre la disposition
  implicite ou surprenante. Toute nouvelle valeur par défaut doit avoir un
  équivalent explicite pour obtenir l'ancien comportement quand il est utile.
- Autoriser `<text>Bonjour</text>` pour une chaîne ou un contenu texte réactif.
  Garder `text={...}` comme forme explicite utile, par exemple pour une valeur
  préparée par l'application. Définir une règle exclusive pour `children` et
  `text` ; ne pas accepter silencieusement les deux.
- Les états visuels usuels (hover, press, focus) doivent être résolus par le
  moteur natif quand ils ne changent que la présentation. Aucun callback JS par
  frame pour une couleur de hover, un défilement ou une animation décorative.

### 3. Identité, références et accessibilité

- `key` appartient uniquement à la réconciliation du framework. Il n'est pas
  une clé native adressable et n'est pas utilisé pour les ancres, relations
  d'accessibilité ou sélecteurs de test.
- `id` est l'identité native adressable, avec le même sens en Solid et React.
  Il est facultatif pour une utilisation ordinaire. Les widgets génèrent un ID
  interne stable pendant leur montage et pour leurs sous-éléments. Un auteur
  donne un `id` quand un autre élément, un popup, un test ou une relation
  d'accessibilité doit le cibler. Un ID fourni reste stable tant que l'auteur
  le garde ; il n'est jamais dérivé du texte visible ni d'un index de liste.
- Définir la portée et l'unicité des ID par fenêtre/racine, le comportement
  après réordonnancement et les collisions. Les listes gardent une clé d'item
  stable distincte de l'ID adressable ; elles ne réutilisent pas un ID pour deux
  éléments simultanés.
- `ref` expose le même type de poignée publique dans les deux adaptateurs.
  `WorkNode` React et `NativeNode` Solid restent des détails internes. Définir
  comment la poignée devient invalide après démontage.
- Les noms accessibles sont typés et cohérents (`accessibleName` recommandé
  pour une primitive). Les widgets fournissent les sémantiques ordinaires ;
  l'auteur peut les préciser sans devoir reconstituer un `focusScope` entier.

### 4. Thème à la racine

- Installer une seule source de thème par application/fenêtre au montage. Les
  widgets et primitives thématiques lisent le thème hérité sans recevoir une
  prop `theme` obligatoire à chaque utilisation.
- Autoriser une surcharge locale **partielle** de tokens, résolue au-dessus du
  thème courant pour le sous-arbre ou le widget concerné. Les tokens non
  surchargés continuent de suivre les changements du thème racine. Définir la
  priorité entre défaut, variante, surcharge d'application/fenêtre et surcharge
  locale conformément au moteur `argui-theme`.
- Une mise à jour de plusieurs tokens produit un instantané cohérent. Les
  composants non concernés ne doivent pas faire de transaction native inutile ;
  mesurer les rendus Solid/React avant de choisir l'implémentation du contexte.
- Les assets nécessaires aux cinq widgets de base sont fournis au montage ou
  installés automatiquement par leur paquet. L'auteur peut les remplacer ; il
  ne devrait pas devoir renseigner manuellement un catalogue de six icônes
  pour afficher un premier bouton ou sélecteur.

### 5. Widgets et valeurs contrôlées

- `Button` prend son texte ordinaire dans `children` et utilise `variant`, pas
  `label` et `kind`. Un bouton à icône seule exige un nom accessible, vérifié
  par les types si la forme des props le permet et par validation sinon.
- Les widgets de valeur utilisent `value`, `defaultValue` et `onValueChange`.
  Les widgets ouvrables utilisent `open`, `defaultOpen` et `onOpenChange`. Le
  contrat précise la différence entre contrôlé et autonome, la valeur initiale,
  et la réaction quand la prop contrôlée change. Une valeur contrôlée sans
  callback doit être explicitement en lecture seule, pas sembler interactive
  puis refuser silencieusement la modification.
- Les mêmes props et valeurs de variante existent en Solid et en React. Les
  types propres au framework ne changent que `children`, la référence et les
  conventions de rendu indispensables.
- Les labels visibles de champ restent possibles ; `children` ne remplace pas
  le label accessible d'un champ ni une option de liste.

### 6. Types et wire fidèles au runtime

- Générer les déclarations TSX depuis les métadonnées du schéma Rust, y compris
  la description des propriétés, les valeurs fermées autorisées, les défauts
  documentés, les payloads d'événements et les contraintes connues. Une faute
  comme `placement="bottom_strat"` doit échouer à la vérification TypeScript.
- Ne laisser `unknown` que pour une extension réellement opaque, qui oblige
  l'application à valider sa propre donnée. Les événements natifs connus ont
  une forme documentée et partagée entre les deux adaptateurs.
- Terminer le chemin `Border` et `Shadow` de bout en bout : type public,
  sérialisation JS, validation wire, décodage Rust, application au moteur et
  génération du contrat. Couvrir également les autres valeurs structurées que
  le schéma annonce (`Insets`, `Radii`, `Transform`). Si une valeur reste propre
  à Rust, elle ne figure pas comme capacité TSX fonctionnelle.
- Les primitives personnalisées reçoivent le même traitement de génération et
  de validation que les primitives intégrées ; elles ne réintroduisent pas des
  props `object` ou des chaînes libres par défaut.

## Audit des crates et traitement du code peu utilisé

La stabilisation du contrat TSX touche directement `argui-schema`,
`argui-host`, `argui-runtime` et `argui-theme`. L'audit de consolidation des
crates est détaillé dans [le plan global](argui-crate-theme-decisions.md) ; les
décisions suivantes restent applicables avec la galerie limitée à cinq widgets :

| Crate ou zone | Constat dans le dépôt | Action proposée |
| --- | --- | --- |
| `argui-theme` | La galerie résout surtout ses propres palettes ; `ThemeRuntime` n'y est pas branché. | Garder cette crate et en faire la source réelle du thème racine pour Rust, Solid et React. |
| `argui-reactive` | Son seul consommateur de production local identifié est `ThemeRuntime`. | Remplacer ce lien par un état spécialisé du thème, puis retirer la crate du futur graphe publié. |
| `argui-shader` | Petite enveloppe dont le consommateur local est `argui-render`. | Déplacer la validation WGSL dans `render/src/shader/`, puis retirer la crate distincte. |
| `argui-android` et `argui-ios` | Petites enveloppes des entrées mobiles du runtime. | Déplacer les entrées dans `runtime/src/mobile/`, en conservant les chemins et symboles publics nécessaires aux consommateurs. |
| `argui-inspect` | Le runtime en dépend aujourd'hui sans feature optionnelle. | Garder la crate ; rendre l'instrumentation du runtime optionnelle et mesurer le graphe release. |
| Crates principales du moteur | Leurs frontières correspondent à des responsabilités distinctes. | Garder ces frontières, puis juger chaque API peu utilisée sur des usages et des mesures précis. |

« Aucun usage dans ce monorepo » n'établit pas qu'une API publique est morte :
des consommateurs externes peuvent l'utiliser. Avant chaque retrait, inventorier
les exports, les références locales, les features, les cibles et les chemins de
release ; vérifier les usages connus hors dépôt et documenter la rupture. Pour
le code interne, chercher les branches jamais atteintes, les conversions et
copies redondantes, les données conservées sans lecteur et les couches qui
répliquent l'état des frameworks. Garder un dossier de preuves par suppression
avec le chemin d'appel, le remplacement et une mesure avant/après quand elle
concerne la performance ou la taille. Les types `Border` et `Shadow` du schéma
ne sont pas du code mort : leur décodage TSX manque encore et doit être achevé
avant de promettre ces valeurs aux primitives personnalisées.

Cette consolidation suit [ses propres critères de sortie](argui-crate-theme-decisions.md#critères-de-sortie).
Le branchement réel du thème et l'alignement schéma/wire sont requis par l'API
v2 ; les autres déplacements de crates peuvent être réalisés après les cinq
widgets, avec leurs vérifications par cible.

## Exemple cible à discuter

La même partie visible doit pouvoir être écrite en Solid et en React, avec la
lecture de l'état propre à chaque framework. Le nom de l'API de montage n'est
pas encore fixé.

```tsx
<column gap={12} padding={16}>
  <text>Bonjour</text>
  <Button variant="default" onClick={save}>Enregistrer</Button>
  <InputField value={name} onValueChange={setName} accessibleName="Nom" />
</column>
```

Le montage fournit le thème et les assets une fois. Aucun `id` n'est requis ici.
Si un popup doit s'ancrer à ce bouton, l'auteur lui donne un `id` explicite.

## Sort de la galerie actuelle : `OLD_API`

Lors de la mise en œuvre, déplacer **toute l'application TSX de la galerie
actuelle**, pour **Solid et React**, dans un dossier racine `OLD_API/`. Cela
comprend les 63 pages de composants, les autres pages d'exemples et laboratoires
qui utilisent l'ancienne API, les deux navigations et points d'entrée TSX,
tous les widgets actuels, leurs aides partagées, le thème propre à cette ancienne
galerie et les versions du catalogue/registre qui les décrivent. Préserver leur
arborescence relative et ajouter un court manifeste indiquant le commit source
et le statut d'archive. Un des cinq nouveaux widgets portant le même nom est
réécrit ; il n'est pas repêché tel quel depuis l'archive. Les hôtes natif/Web,
le moteur et l'infrastructure de build restent actifs et sont adaptés à la
nouvelle galerie.

L'archive sert de référence de lecture. Elle ne participe ni aux exports actifs,
ni au bundle, ni au registre `argui add`, ni aux tests de la nouvelle galerie.
Déplacer ou retirer les tests qui ne portent que sur l'ancienne API ; garder les
tests moteur valables. Garder les assets encore nécessaires à la nouvelle
galerie et archiver les seuls assets obsolètes. Vérifier qu'aucun import actif
ne traverse `OLD_API/`. Le moteur Rust, les hôtes et les adaptateurs de framework
ne sont pas archivés : leur contrat est migré.

La nouvelle Widget Gallery n'affiche **que cinq composants confirmés** pendant
cette phase, avec une page Solid et une page React pour chacun :

| Widget | Ce qu'il doit prouver |
| --- | --- |
| `Button` | `children`, `variant`, thème hérité, états natifs, clavier, focus, nom accessible. |
| `InputField` | Édition native, IME, valeur contrôlée/autonome, sélection, validation d'événements. |
| `Select` | Options, valeur contrôlée/autonome, clavier, accessibilité de liste. |
| `Popover` | `open`/`defaultOpen`, ancre par `id`, focus, fermeture, contenu libre. |
| `VirtualList` | Identité des items, fenêtre bornée, défilement natif, coût de mise à jour. |

`VirtualList` est aujourd'hui dans les adaptateurs plutôt que dans
`@argui/widgets`. Sa place dans les exports reste à discuter, mais sa présence
parmi les cinq pages est décidée. Aucun des autres widgets n'est reconstruit
avant que ces cinq usages aient stabilisé le contrat commun.

## Ordre proposé pour l'implémentation ultérieure

1. **Figer les exemples et les règles.** Valider les points ouverts de ce plan,
   d'abord la matrice de layout Rust/Solid/React ci-dessus, puis les signatures
   des cinq widgets et les exemples minimaux. Écrire les critères de géométrie,
   de type, d'accessibilité et de performance avant la migration.
2. **Migrer le contrat canonique.** Renommer les noms publics en `camelCase`
   dans le schéma, le contrat, le wire, les hôtes natif/Web, les générateurs et
   les types. Achever les valeurs structurées et les payloads d'événements.
   Supprimer les anciens noms dans la même migration.
3. **Stabiliser les adaptateurs.** Implémenter ID automatique, distinction
   `key`/`id`, référence publique commune, texte en `children`, montage et
   disposition du thème à la racine. Vérifier chaque comportement dans les deux
   frameworks avant de refaire les widgets.
4. **Archiver l'ancienne galerie.** Déplacer tout le périmètre décrit dans
   `OLD_API/`, retirer ses exports/imports actifs et reconstruire une galerie
   vide mais lançable avec navigation réduite à cinq entrées.
5. **Écrire les cinq widgets neufs.** Partager seulement les types et comportements
   sans état de framework ; garder deux vues idiomatiques. Chaque widget a une
   page Solid, une page React et des vérifications d'interaction réelles.
6. **Aligner la distribution.** Régénérer le SDK embarqué par le CLI, son ABI,
   les modèles d'application, le registre `argui add` limité aux cinq widgets,
   les README et les exemples publics. Un projet créé hors monorepo doit utiliser
   la même API en natif et en Web.
7. **Mesurer puis finaliser.** Comparer le nombre de props nécessaires à un écran
   simple, les transactions natives au repos et après changement de thème, et
   le temps de commit React sur de grandes scènes. Faire les tests ciblés ; pour
   les contrôles GUI Linux, utiliser l'affichage privé et inspecter les captures.
   Exécuter `quality.sh` une seule fois à la fin, immédiatement avant un commit.

## Critères de sortie de cette phase

- Les cinq pages Solid et les cinq pages React montrent les mêmes capacités avec
  les mêmes noms publics. Une application minimale n'a besoin ni de passer
  `theme` à chaque widget ni d'inventer des ID pour le fonctionnement interne.
- Les scènes de layout convenues donnent les mêmes résultats attendus en Rust,
  Solid et React, y compris au redimensionnement et avec du texte long. Le sens
  de `width`, des pourcentages, de `grow`, des bornes et du scroll est documenté.
- Tous les noms contractuels actifs sont en `camelCase`. Les anciens noms
  n'existent plus dans les types, le contrat, le SDK ou les exemples actifs.
- Les options fermées incorrectes et les combinaisons de props invalides
  échouent au contrôle TypeScript. Aucun événement natif connu n'expose un
  payload `unknown` sans raison documentée.
- `Border`, `Shadow` et les autres valeurs structurées annoncées passent de TSX
  au moteur ou sont explicitement déclarées hors du contrat TSX.
- Un bouton à icône seule sans nom accessible est refusé. Un popup garde son
  ancre et le focus attendu après réordonnancement et démontage.
- `OLD_API/` contient l'ensemble ancien mais aucun chemin d'exécution actif.
  Le registre et la galerie actifs ne montrent que les cinq nouveaux widgets.
- Le SDK livré et les hôtes partagent la même empreinte ABI. Les coûts de rendu
  et de compilation sont mesurés ; aucune amélioration de performance n'est
  affirmée sur la seule base d'un renommage.

## Questions à trancher ensemble avant de coder

1. Pour les primitives courantes, préfère-t-on les tags natifs existants
   (`<column>`, `<text>`) avec de meilleurs défauts, ou une petite façade
   (`<Column>`, `<Text>`) ? Il faut choisir un chemin recommandé, sans doubler
   chaque option dans deux syntaxes publiques.
2. La surcharge locale du thème s'applique-t-elle à un widget seulement ou à
   tout son sous-arbre ? Proposition : sous-arbre, avec une valeur partielle.
3. Quel nom public retenir pour les sémantiques : `accessibleName` est proposé ;
   valider aussi la forme des relations entre IDs avant de générer les types.
4. `VirtualList` reste-t-elle exportée par les adaptateurs Solid/React ou devient-elle
   un composant de `@argui/widgets` ? Son API et sa page de galerie doivent être
   identiques dans les deux cas.

Après accord sur ces points, transformer ce brouillon en spécification de l'API
et en petites étapes d'implémentation vérifiables. Le skill « common pitfalls »
sera écrit sur cette API stabilisée, pas sur les conventions en cours de retrait.
