# Layout Argui v2 : comparaison Web, Iced et Slint

Statut : **recherche ayant guidé l'implémentation v2**, 26 septembre 2026.
Ce texte conserve les hypothèses et les comparaisons initiales ; le contrat
effectivement livré est décrit dans [le guide de layout](../ui/layout.md).
Les exemples de recherche ci-dessous peuvent différer des signatures finales.

## Verdict

Le calcul de layout ne demande pas une réécriture sur les seules preuves
actuelles. Argui utilise [Taffy 0.14](../../Cargo.toml) et possède des chemins
et tests pour flex, grid, texte mesuré, scroll, sticky et overlays. Le travail
prioritaire est le **contrat public** : ce que signifient taille, espace
restant, contrainte, placement et viewport dans Rust pur, Solid et React.
L'API de layout ne doit être figée qu'après des scènes comparatives sur les
trois surfaces. Les tests existants ont été lus, pas exécutés pour cette étude.

**Décision confirmée par l'auteur :** dans un flex, `width`/`height` suivent
le modèle Web/Taffy. Ce sont des tailles préférées qui peuvent rétrécir sous
les contraintes du parent. Les noms publics TSX restent en `camelCase` ; les
méthodes et champs internes Rust restent idiomatiques en `snake_case`.

## Distance réelle par rapport au Web

| Capacité | Moteur/Rust Argui | Contrat TSX actuel | Conséquence |
| --- | --- | --- | --- |
| Block, flex, grid | Les algorithmes Taffy sont appelés sur un arbre retenu avec cache. `row`, `column`, `grid` existent. | Les trois tags existent. | Base solide pour la composition usuelle ; vérifier les défauts et les scènes réelles. |
| Tailles et contraintes | `LayoutStyle` contient auto, %, tailles intrinsèques Taffy, min/max, basis, grow/shrink. | Le wire accepte nombre, `auto`/`fit`, `fill`, % et px ; min/max sont limités à des nombres. | Une partie de la capacité existe déjà, mais l'auteur TSX ne peut pas la demander clairement. |
| Grid avancé | Les types Rust exposent pistes, répétitions, zones, placement et auto flow de Taffy. | `grid_rows`/`grid_columns` actuels sont une chaîne à grammaire restreinte ; pas de `repeat(auto-fit, minmax(...))`. | L'adaptation continue sans breakpoints demande une composition manuelle ou du JS ; le seul breakpoint grid TSX couvre moins de cas. |
| Réactivité à la taille du conteneur | Rust a des conditions de largeur/hauteur/orientation et des combinaisons booléennes ; le moteur borne les passes de convergence à quatre. | Le schéma n'offre qu'un couple `query_min_width`/`query_columns` pour les pistes grid. | Il manque un contrat général, typé, pour adapter un sous-arbre sans callback JS de mesure. |
| Texte, scroll et overlays | Mesure textuelle native, scroll, sticky, ancrage et peinture séparée du layout. | `scroll_y` sur conteneur et `Flickable` divergent ; `x`/`y` peuvent se combiner avec `position`/insets. | Le moteur existe, la surface publique doit choisir un chemin sans ambiguïté. |
| Direction et côtés | Direction d'écriture et styles Taffy existent. | Espacements et insets exposent surtout left/right physiques. | Les composants RTL doivent encore permuter des côtés à la main. |

Sources du dépôt : [algorithmes Taffy](../../crates/argui-layout/src/layout_tree/algorithms.rs),
[style Rust](../../crates/argui-ui/src/style.rs),
[dimensions wire](../../crates/argui-runtime/src/native_host/wire.rs),
[grid et requête TSX](../../crates/argui-schema/src/builtin/layout.rs),
[requêtes Rust](../../crates/argui-ui/src/responsive.rs),
[convergence](../../crates/argui-layout/src/engine/compute.rs),
[scroll conteneur](../../crates/argui-schema/src/builtin/container.rs) et
[viewport Flickable](../../crates/argui-schema/src/builtin/flickable.rs).

La proximité concerne donc les **algorithmes centraux**, pas l'ensemble du
Web. Argui n'a pas besoin d'une cascade CSS, de sélecteurs et de toutes les
unités du navigateur pour devenir agréable. Il lui faut une description
cohérente et vérifiable des comportements qu'il possède déjà.

## Ce que font les autres

| Modèle | Idée utile | Limite ou piège pour Argui |
| --- | --- | --- |
| Web CSS | Flex distingue largeur/hauteur, base, croissance et rétrécissement ; Grid sait répéter des pistes adaptatives. `box-sizing: border-box` rend les tailles plus prévisibles. | `width: 100%` ne signifie pas « reste disponible » ; `min-content` peut empêcher un flex item de rétrécir. Copier toute la grammaire CSS créerait une surface trop grande. |
| Iced | `row`, `column`, `container` et `scrollable` rendent la composition lisible. `Length` nomme les stratégies `Fit`, `Fill`, `FillPortion` et `Fixed`. L'espacement courant passe par `spacing` et `padding`. | Son `Fill` signifie « espace restant », à l'inverse du `fill` actuel d'Argui. Sa surface volontairement resserrée ne couvre pas à elle seule toute la composition grid/positionnement d'Argui. |
| Slint | Sépare taille préférée, bornes min/max et facteur d'étirement ; les contraintes s'appliquent aux éléments visibles. `HorizontalLayout`, `VerticalLayout`, `GridLayout` rendent le rôle du parent évident. | Ses règles de taille et de stretch diffèrent de CSS ; les recopier au-dessus de Taffy sans contrat explicite surprendrait les auteurs TSX. |
| Taffy | Fournit déjà les algorithmes flex/grid/block et des dimensions typées, y compris tailles intrinsèques. Slint s'en sert aussi pour son `FlexboxLayout`. | Son `Style` public est une grande surface technique ; le réexporter comme seule API Argui rend la stabilité dépendante des choix de Taffy. |

Sources officielles : [MDN flex](https://developer.mozilla.org/en-US/docs/Web/CSS/Guides/Flexible_box_layout/Basic_concepts),
[MDN grid responsive](https://developer.mozilla.org/en-US/docs/Learn_web_development/Core/CSS_layout/Grids),
[MDN box sizing](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/box-sizing),
[Iced `Length`](https://docs.iced.rs/iced/enum.Length.html),
[Iced `Row`](https://docs.iced.rs/iced/widget/row/struct.Row.html),
[Iced `Scrollable`](https://docs.iced.rs/iced/widget/scrollable/struct.Scrollable.html),
[Slint positionnement et contraintes](https://docs.slint.dev/latest/docs/slint/guide/language/coding/positioning-and-layouts/),
[Slint `FlexboxLayout`](https://docs.slint.dev/latest/docs/slint/reference/layouts/flexboxlayout/),
[Taffy `Dimension`](https://docs.rs/taffy/0.14.0/taffy/style/struct.Dimension.html).

### La leçon la plus importante sur `fill`

Dans [Iced](https://docs.iced.rs/iced/enum.Length.html), `Fill` consomme le
reste de l'espace sur un axe, éventuellement en parts pondérées. Dans le
[wire Argui actuel](../../crates/argui-runtime/src/native_host/wire.rs),
`"fill"` devient `Dimension::percent(1.0)` : 100 % du parent. Sur une ligne
avec un panneau fixe et un panneau `width="fill"`, cette taille ne représente
pas le reste après le premier panneau. Le chemin Taffy pour distribuer le
reste est `grow`. Conserver le mot `fill` avec son sens actuel provoquerait
des erreurs de composition récurrentes.

### La leçon sur les contraintes

Le Web explique séparément
[flex basis, grow et shrink](https://developer.mozilla.org/en-US/docs/Web/CSS/Guides/Flexible_box_layout/Controlling_flex_item_ratios)
et signale les minimums intrinsèques. Slint demande de penser en
[taille préférée, minimum, maximum et stretch](https://docs.slint.dev/latest/docs/slint/reference/layouts/overview/).
Dans Argui, [`LayoutStyle`](../../crates/argui-ui/src/style.rs) contient déjà
taille, min/max, aspect ratio, `flex_basis`, `flex_grow` et `flex_shrink`.
Le [schéma TSX](../../crates/argui-schema/src/builtin/layout.rs) ne leur donne
pas encore un vocabulaire également typé : `min_width`/`max_width` sont des
pixels numériques et plusieurs autres valeurs arrivent en chaînes libres.
Les composants doivent parfois mettre `min_width={0}` pour rétrécir dans une
ligne. Il faut établir les défauts et les attentes sur du texte long avant de
les exposer comme définitifs.

## Contrat Argui proposé pour discussion

### 1. Un parent choisit le mode de composition

Garder `row`, `column` et `grid` pour les arrangements ordinaires. Le parent
porte `gap`, `padding`, `alignItems`, `justifyContent`, `wrap` et les pistes
grid. L'enfant porte sa taille, ses bornes, `grow`/`shrink` et `alignSelf`.
`container` reste une boîte neutre pour décor et contraintes. Le scroll
demande un viewport explicite. Les overlays ancrés utilisent le mécanisme
d'ancrage, avec une couche distincte du flux normal.

Cela reprend la lisibilité d'[Iced](https://docs.iced.rs/iced/) et de
[Slint](https://docs.slint.dev/latest/docs/slint/guide/language/coding/positioning-and-layouts/)
en gardant les capacités flex/grid du moteur existant. La même règle mentale
s'applique en Rust pur, Solid et React ; seuls les noms de méthodes Rust
restent en `snake_case`.

### 2. Chaque taille a un sens unique

Proposition de base : `width={240}`/`height={40}` = longueur en pixels
logiques ; `"50%"` = part du conteneur ; `"auto"` = taille déterminée par le
contenu et les règles du parent ; `grow={1}` = part de l'espace flex restant.
Les tailles intrinsèques plus précises (`minContent`, `maxContent`,
`fitContent`) restent disponibles dans une forme typée quand leur comportement
est vérifié dans Argui. Garder `minWidth`, `maxWidth`, `minHeight`, `maxHeight`
et `aspectRatio` cohérents entre Rust et TSX. **Décision prise :** le `width`
d'un item flex est une taille préférée soumise aux contraintes et au shrink,
comme sur le [Web](https://developer.mozilla.org/en-US/docs/Web/CSS/Guides/Flexible_box_layout/Basic_concepts).
Pour une largeur strictement fixe, définir une expression explicite et courte.
Éviter le raccourci nu `fill` ; utiliser `"100%"` ou `grow` selon le besoin.

### 3. Les widgets participent directement au layout

Un `Button`, `InputField` ou `Select` doit pouvoir prendre un petit ensemble
commun de props pour sa **racine** : au minimum taille, bornes, `grow`,
`shrink` et `alignSelf`. Ces props ne pilotent pas la géométrie de ses détails
internes. L'ancien [`InputFieldProps`](../../packages/widgets/src/shared/types.ts)
n'a aucune prop de placement, tandis que `SelectProps` possède un `width`
spécifique. Une ligne de formulaire ne devrait pas demander un wrapper par
champ pour exprimer « celui-ci prend le reste ».

### 4. Espacement, RTL et positionnement explicites

Le chemin courant utilise `gap` entre enfants et `padding` dans le parent ;
`margin` reste un outil avancé pour un décalage local. Un `padding` ou `inset`
structuré peut accepter des côtés physiques et des côtés logiques, avec une
règle de conflit explicite. Les côtés logiques `start`/`end` évitent de
réécrire les écrans RTL, comme les
[propriétés logiques du Web](https://developer.mozilla.org/en-US/docs/Web/CSS/Guides/Logical_properties_and_values).
Pour une position hors flux, préférer `position="absolute"` avec un `inset`
typé ; retirer le raccourci `x`/`y` s'il impose implicitement `absolute`.
Documenter qu'une transform déplace le rendu mais ne réserve pas d'espace de
layout. Les popovers utilisent leur ancre, pas des coordonnées calculées dans
JS.

### 5. Scroll et grid ont une voie courante et une voie avancée

Un viewport de scroll dédié, par exemple `scrollView`, rend axes et politique
de scrollbar explicites. Il remplace dans le chemin recommandé le doublon
entre `scroll_y` sur les conteneurs et `Flickable`. Le viewport doit avoir une
taille ou une contrainte résoluble ; décrire ce qui arrive dans une colonne à
hauteur automatique. Le scroll reste natif, sans callback JS par frame.

Grid a besoin d'un cas simple court et d'une expression avancée typée. Le
[Web](https://developer.mozilla.org/en-US/docs/Learn_web_development/Core/CSS_layout/Grids)
offre `repeat(auto-fit, minmax(...))`, tandis que le
[parseur Argui actuel](../../crates/argui-schema/src/builtin/layout.rs)
n'accepte que `auto`, px, %, fr et `minmax(px,fr)` séparés par espaces. Pour la
v2, comparer des pistes structurées générables avec un parseur CSS plus
complet. La forme retenue doit couvrir un nombre fixe de colonnes, des parts,
un minimum de carte avec nombre adaptatif de colonnes et les placements.
Éviter de promettre à TypeScript qu'il valide une grammaire de chaîne libre
qu'il ne peut pas contrôler.

### 6. Deux niveaux Rust, une même sémantique

Les builders usuels `Element::row/column/grid`, `.gap`, `.padding`, `.width`,
`.grow` sont le chemin à documenter d'abord. Garder une voie avancée typée
pour tout ce que Taffy sait faire, après avoir décidé si les types Taffy
réexportés font partie de la stabilité publique garantie. Le moteur continue
de faire le calcul ; les adaptateurs TSX sérialisent une description typée.
Une API simple ne doit pas dupliquer le moteur en JavaScript.

## Esquisses à éprouver

Formulaire simple, avec `grow` appliqué à la racine du widget :

```tsx
<column gap={12} padding={16}>
  <row gap={8} alignItems="center">
    <text>Nom</text>
    <InputField grow={1} accessibleName="Nom" />
  </row>
</column>
```

Deux panneaux :

```tsx
<row gap={12}>
  <column width={240}>Navigation</column>
  <column grow={1} minWidth={0}>Contenu</column>
</row>
```

Ici, `minWidth={0}` documente explicitement le cas à valider : peut-on le
rendre implicite pour les surfaces qui grandissent sans faire déborder ou
couper le contenu important ? La forme d'un grid adaptatif et d'un scroll
contraint sera choisie après comparaison de leurs géométries.

## Matrice avant gel

Exécuter les mêmes scènes en Rust pur, Solid et React, sur hôtes natif et Web
quand disponibles, à plusieurs largeurs et directions. Vérifier des relations
géométriques et les interactions, pas seulement la compilation :

1. Ligne à panneau fixe + panneau flexible ; plusieurs enfants flexibles
   pondérés, texte long, redimensionnement étroit.
2. Colonne à en-tête fixe + zone scrollable ; viewport imbriqué, scrollbar,
   sticky et conservation de l'offset après mise à jour.
3. Grille de cartes adaptative, spans, gaps et limites de largeur ; bascule
   autour d'un seuil.
4. Texte intrinsèque, police ou langue changée, images à ratio, padding et
   border-box, DPI différent.
5. Popup ancré dans un scroller et élément absolument placé ; déplacement de
   l'ancre, retournement au bord de fenêtre, démontage.
6. RTL avec côtés logiques ; liste virtuelle longue dans un conteneur borné.

Comparer le nombre de props et de wrappers, les erreurs TypeScript avant
lancement, les transactions natives et les coûts de layout. Fixer ensuite les
valeurs par défaut et documenter chaque mot (`auto`, pourcentage, `grow`,
minimum, scroll). Les tests et captures correspondants seront réalisés lors
de l'implémentation suivant les règles de contribution du dépôt.

## Décisions retenues dans la v2

1. `width` et `height` sont des tailles préférées. `shrink={0}` conserve la
   taille dans un flex ; des bornes min/max égales expriment une contrainte
   rigide quand elle est nécessaire. `grow` distribue le reste.
2. `gridColumns` et `gridRows` acceptent des pistes structurées, notamment
   `fr`, `minmax` et `repeat` avec `autoFit`/`autoFill`. Les règles de conteneur
   sont des valeurs typées, résolues dans le moteur.
3. `padding`, `margin` et `inset` acceptent `start`/`end` selon la direction
   héritée. La frontière TSX rejette le mélange de côtés physiques et logiques
   horizontaux dans un même objet. Un côté d'`inset` absent reste `auto`.
4. `scrollView` est le viewport public recommandé, avec `scrollX`/`scrollY` et
   une hauteur ou borne résoluble. L'ancien vocabulaire `Flickable` n'est plus
   dans le contrat actif.

Les scènes Rust de [l'engine](../../crates/argui-layout/tests/engine/compute.rs)
vérifient taille préférée, pourcentage, espace restant, redimensionnement et
insets LTR/RTL. Les scènes Solid et React de la galerie couvrent la même
surface ; les captures visuelles sont laissées à l'auteur comme demandé.

## Améliorations prioritaires au-delà de la parité Web

1. **Types et diagnostics.** Une taille ou un alignement invalide échoue au
   typage ou à `argui check`. En développement, signaler une taille en % sous
   parent indéfini, des contraintes contradictoires, un viewport sans borne,
   ou un couple `position`/inset incompatible avec l'intention exprimée. Le
   navigateur accepte souvent ces cas puis laisse l'auteur inspecter le rendu.
2. **Une source de vérité pour chaque concept.** Un seul viewport de scroll
   recommandé, un seul type public de dimension, une seule règle de priorité
   pour `padding` et ses côtés. Les widgets prennent les mêmes props de
   placement que les primitives ordinaires à leur racine.
3. **Responsive sans mesure en JS.** Exposer les capacités Rust de container
   queries comme styles conditionnels typés, puis définir la portée et le
   comportement des cycles. Le Web utilise le
   [containment](https://developer.mozilla.org/en-US/docs/Web/CSS/Guides/Containment/Container_size_and_style_queries)
   pour éviter qu'une règle dépendant de la taille modifie indéfiniment son
   propre conteneur ; Argui borne déjà les passes, mais doit expliquer l'erreur
   et empêcher les motifs problématiques quand c'est possible.
4. **Composition courte avec accès avancé.** Les écrans courants tiennent en
   `row`/`column`/`grid`, `gap`, `padding`, `width` et `grow`. Grid adaptatif,
   tailles intrinsèques, styles conditionnels et positionnement restent
   disponibles sous des formes typées, sans reproduire toute la grammaire CSS.
5. **Inspection qui explique la géométrie.** En mode développement, étendre
   les bornes et styles déjà capturés par
   [`argui-inspect`](../../crates/argui-inspect/src/records.rs) avec les
   contraintes entrantes, la taille préférée, la taille calculée, le
   parent de layout et la cause d'un débordement. Un humain ou un agent doit
   pouvoir répondre à « pourquoi ce panneau vaut 100 % ? » sans déduire la
   réponse d'une capture seule.

Les gains de performance restent à mesurer. L'arbre retenu, les caches Taffy
et le scroll natif offrent une base ; une nouvelle prop ou un nouvel alias ne
prouve aucun gain à elle seule.
