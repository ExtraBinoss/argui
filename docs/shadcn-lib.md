# Catalogue de composants Argui / shadcn/ui

Relevé du 8 septembre 2026. Les **64 entrées** ci-dessous reprennent le catalogue
[officiel shadcn/ui](https://ui.shadcn.com/docs/components), consulté à cette date.
Les statuts Argui viennent des API de ce dépôt, pas d'une équivalence supposée
avec React. Les variantes Base UI, Radix UI et React Aria ne sont pas comptées
plusieurs fois. Les blocks, recettes applicatives et registres communautaires
ne sont pas des composants supplémentaires de cette liste.

Une case cochée signifie qu'un widget public réutilisable couvre le besoin de
base. Elle ne certifie pas toutes les variantes shadcn, ni un audit sur lecteurs
d'écran réels. **Partiel** reste décoché : une primitive moteur, un comportement
interne ou une page de démonstration ne suffisent pas à livrer un widget.

Mises à jour Argui du 10 septembre 2026 : [les cinq premiers composants](shadcn-foundations.md),
puis [Avatar, Empty, Kbd, Progress, AspectRatio et le Separator enrichi](shadcn-display-widgets.md).
Ajout du 11 septembre 2026 : [Label, Skeleton, Breadcrumb et Pagination](shadcn-navigation-widgets.md).

Dans la galerie, taper du texte depuis le fond de page ou un bouton démarre
une recherche et place le curseur après le premier caractère dans le champ.
Entrée ouvre le résultat sélectionné ; Échap efface la recherche et rend le
focus à la galerie. Input, Textarea, éditeurs intégrés, menus et sélecteurs
conservent leur saisie et leurs raccourcis. Ctrl/Cmd+K reste disponible.
La couleur des libellés de Button suit désormais la transition de leur style,
y compris lors d'un changement de page et d'une inversion rapide. Le survol
reste immédiat et le mouvement réduit termine les transitions sans animation.

## Checklist complète

- [ ] **Accordion** — absent ; sections contrôlées, navigation clavier et animation à exposer.
- [x] **Alert** — `Alert` ; présentation inline standard/destructive, icône et politique d’annonce explicite.
- [ ] **Alert Dialog** — partiel ; `Dialog` existe, mais pas de contrat de confirmation dédié.
- [x] **Aspect Ratio** — `AspectRatio` ; réserve la hauteur suivant la largeur et un ratio positif, contenu ajusté au cadre.
- [ ] **Attachment** — absent ; définir présentation, état de transfert et actions, sans imposer un client mail.
- [x] **Avatar** — image chargée ou fallback contrôlé, masque circulaire, taille configurable et nom accessible unique. Groupe non livré.
- [x] **Badge** — `Badge` ; variantes primary/secondary/destructive/outline/ghost, icônes avant/après et nom accessible unique.
- [x] **Breadcrumb** — ancêtres activables, identifiants stables, séparateurs personnalisés et page courante décrite ; groupe accessible, sans landmark Navigation.
- [ ] **Bubble** — absent ; présentation réutilisable à définir.
- [x] **Button** — [`Button`](../crates/argui-widgets/src/button.rs) ; variantes via le thème, icônes, chargement et activation accessible.
- [ ] **Button Group** — absent ; disposer des boutons en ligne ne constitue pas encore une API de groupe.
- [x] **Calendar** — `Calendar` et `CalendarState` ; locale, limites, sélection simple/multiple/plage et navigation clavier.
- [x] **Card** — `Card` ; titre, description, action, contenu et pied de carte optionnels, avec relations accessibles.
- [ ] **Carousel** — absent ; défilement disponible, pagination, gestes et annonces à coordonner.
- [ ] **Chart** — absent ; rendu vectoriel disponible, échelles, séries et interactions à concevoir.
- [x] **Checkbox** — [`Checkbox`](../crates/argui-widgets/src/selection.rs) ; `CheckedState` expose Unchecked, Checked et Mixed ; activation du mode Mixed vers Checked.
- [x] **Collapsible** — ouverture contrôlée, trigger personnalisable, Entrée/Espace, état désactivé et contenu démonté une fois fermé.
- [ ] **Combobox** — partiel ; `Select` et recherche existent séparément, sans combobox publique dédiée.
- [x] **Command** — [`CommandPalette`](../crates/argui-widgets/src/command_palette.rs) ; recherche et invocation d'actions. Groupes et variantes avancées restent à examiner.
- [x] **Context Menu** — `ContextMenu` partage les entrées de `Menu`, avec ancrage au pointeur ou au clavier.
- [x] **Data Table** — modèle typé, filtres, tri multiple stable, pagination, colonnes visibles, virtualisation et édition contrôlée ; en-têtes alignés au défilement.
- [x] **Date Picker** — saisie localisable, brouillon contrôlé, validation et calendrier réutilisé.
- [x] **Dialog** — [`Dialog`](../crates/argui-widgets/src/dialog.rs) ; modal, fermeture et restauration du focus.
- [ ] **Direction** — partiel ; texte bidi et alignement logique disponibles, sans fournisseur de direction commun aux widgets.
- [ ] **Drawer** — absent ; panneau gestuel, seuils de fermeture et focus à implémenter.
- [x] **Dropdown Menu** — [`Menu`](../crates/argui-widgets/src/menu.rs) ; identifiants stables, sous-menus, groupes, entrées checkbox/radio, indicateurs et navigation clavier.
- [x] **Empty** — état vide centré, titre, description, média décoratif et actions libres ; bordure optionnelle.
- [ ] **Field** — partiel ; labels et erreurs existent sur `Input`, sans composition générique label/aide/erreur/contrôle.
- [ ] **Hover Card** — absent ; overlay disponible, délais d'ouverture/fermeture et maintien au survol à ajouter.
- [x] **Input** — [`Input`](../crates/argui-widgets/src/input.rs) ; texte, recherche, password et état contrôlé.
- [ ] **Input Group** — partiel ; décorations possibles sur `Input`, sans groupe générique de contrôles et boutons associés.
- [ ] **Input OTP** — absent ; cellules, collage, navigation et saisie unique accessible à concevoir.
- [ ] **Item** — partiel ; `List` accepte du contenu libre, sans API de parties titre/description/média/actions.
- [x] **Kbd** — touche ou combinaison, une annonce accessible personnalisable ; n'enregistre pas de raccourci.
- [x] **Label** — label visible associé via `labelled_by`, cible de focus sur clic et état désactivé ; sans arrêt Tab supplémentaire.
- [ ] **Marker** — absent ; composant dédié à définir.
- [x] **Menubar** — focus entre déclencheurs, ouverture des menus, sous-menus et navigation RTL.
- [ ] **Message** — absent ; composant de message réutilisable à définir, sans mini-application imposée.
- [ ] **Message Scroller** — partiel ; `VList` virtualise, mais suivi du bas, chargement antérieur et compteur de nouveautés restent applicatifs.
- [ ] **Native Select** — absent ; `Select` est rendu par Argui. Pour ce catalogue, reproduire son usage avec un composant Argui ; aucun contrôle système requis.
- [ ] **Navigation Menu** — absent ; navigation de site/application avec panneaux, focus et état courant à fournir.
- [x] **Pagination** — navigation contrôlée, bornes, ellipses, désactivation et annonces traduisibles ; groupe accessible, page courante décrite.
- [x] **Popover** — [`Popover`](../crates/argui-widgets/src/popover.rs) ; ancrage, collisions, fermeture et options de focus.
- [x] **Progress** — pourcentage contrôlé, état indéterminé animé quand monté comme entité, mouvement réduit et valeur accessible.
- [ ] **Questionnaire** — absent ; contrat de questions, réponses et validation à définir après les contrôles de base.
- [x] **Radio Group** — [`RadioGroup`](../crates/argui-widgets/src/selection.rs) ; sélection exclusive et orientation.
- [x] **Resizable** — [`SplitPane`](../crates/argui-widgets/src/split_pane.rs) ; séparateur, limites, gestes et clavier. Groupes imbriqués à composer explicitement.
- [ ] **Scroll Area** — partiel ; [`ScrollConfig`](../crates/argui-ui/src/scroll.rs) et scrollbars dans le moteur, sans widget général `ScrollArea`.
- [x] **Select** — [`Select`](../crates/argui-widgets/src/select.rs) ; sélection simple contrôlée, overlay et clavier.
- [x] **Separator** — `Separator::new` ; orientation configurable et texte centré entre deux traits, décoratif par défaut ou rôle accessible explicite.
- [ ] **Sheet** — absent ; `Dialog` fournit une base, sans panneau latéral dédié.
- [ ] **Sidebar** — absent ; la sidebar de la galerie n'est pas un widget public.
- [x] **Skeleton** — formes décoratives dimensionnables, pulsation liée au montage et mouvement réduit.
- [x] **Slider** — [`Slider`](../crates/argui-widgets/src/slider.rs) ; valeur, bornes, pas, gestes et clavier. Multi-poignées non livré.
- [x] **Spinner** — [`Spinner`](../crates/argui-widgets/src/spinner.rs) ; animation liée au montage et mouvement réduit.
- [x] **Switch** — [`Switch`](../crates/argui-widgets/src/selection.rs) ; booléen animé et activation accessible.
- [x] **Table** — [`Table`](../crates/argui-widgets/src/table.rs) ; en-têtes, cellules, largeurs communes, sélection et navigation par ligne. Pas de datagrid complet.
- [x] **Tabs** — [`Tabs`](../crates/argui-widgets/src/tabs.rs) ; sélection, navigation et montage du panneau actif.
- [x] **Textarea** — [`TextArea`](../crates/argui-widgets/src/input.rs) ; édition multiligne et scroll. Le redimensionnement se compose avec les gestes publics.
- [x] **Toast** — file contrôlée, durée, pause au survol/focus, annonces et actions.
- [ ] **Toggle** — partiel ; comportement booléen interne, sans bouton toggle public.
- [ ] **Toggle Group** — absent ; groupe de toggles avec navigation et sélection simple/multiple à livrer.
- [ ] **Tooltip** — absent ; délais, ancrage et description accessible à coordonner.
- [ ] **Typography** — partiel ; texte riche, sélection, alignement et décoration présents, sans jeu public de composants typographiques.

## Capacités Argui complémentaires

Ces composants ne sont pas des entrées distinctes du catalogue shadcn ci-dessus.

- [x] **List** — liste non virtuelle, sélection contrôlée simple/multiple et clavier.
- [x] **VList** — lignes fixes ou mesurées, fenêtre virtualisée et sélection commune à `List`.
- [x] **TreeView** — arborescence virtualisée et cache de lignes.
- [x] **WebView** — intégration optionnelle, à conserver comme démonstration système.

Voir [listes et tables](lists-tables.md) pour l'API, la conservation des mesures
et les responsabilités du consommateur. Le moteur et le runtime ne dépendent
pas d'un DSL ni de shadcn.

## Où concentrer le travail

1. **Composition réutilisable** : Item et Button Group, puis les primitives manquantes de navigation accessible. Label, Skeleton, Breadcrumb et Pagination sont maintenant livrés.
2. **Interactions courantes** : Tooltip, Accordion, Toggle/Toggle Group,
   puis Field/Input Group/Combobox. Vérifier clavier, focus et sémantique avant
   d'élargir les variantes visuelles.
3. **Panneaux** : Sheet, Alert Dialog et Hover Card ; réutiliser les comportements
   d'overlay et de focus validés.
4. **Chantiers plus coûteux** : Chart (géométrie/interactions), Drawer/Carousel
   (gestes), Native Select (équivalent Argui).

Aucun composant purement visuel de cette liste n'est déclaré impossible.
Les bibliothèques React sous-jacentes ne sont pas directement réutilisables dans
le renderer Rust : porter le contrat utile, pas l'implémentation DOM. Dans ce chantier, même Native Select désigne un équivalent dessiné et piloté
par Argui ; intégrer le contrôle du système ne fait pas partie du périmètre. Les recettes
React Hook Form, TanStack Form et Formisch relèvent d'une future API de formulaire,
pas de trois widgets Rust à créer.


## Suffisance de l'API Argui

Le catalogue sert de référence fonctionnelle pour des composants implémentés
entièrement dans Argui. Aucune WebView ni bibliothèque React n'est nécessaire
pour cette démarche.

L'architecture permet de poursuivre ce catalogue, mais l'API actuelle ne couvre
pas encore tous ses contrats d'interaction et d'accessibilité. Les primitives de
layout, dessin, texte, gestes, overlays, modèles et éléments personnalisés sont
présentes. Les extensions suivantes sont maintenant disponibles :

- **Relations accessibles** : références par clés locales pour les labels,
  descriptions, contrôles et descendants actifs ; résolution par scope/montage,
  diagnostics et traduction native/DOM.
- **État indéterminé** : `CheckedState::{Unchecked, Checked, Mixed}` remplace la
  valeur booléenne des checkboxes.
- **Focus des collections** : `FocusPolicy::{None, Programmatic, TabStop}`
  distingue le focus explicite du parcours Tab. Les collections conservent une cible de focus
  stable et montent l'option active même hors écran.
- **Clavier** : PageUp/PageDown, touches de fonction et touche de menu contextuel
  ont une représentation dans `Key` et une traduction native.

Ces extensions sont implémentées et la campagne complète passe 958 tests. Les
quatre métriques de couverture dépassent 85 % globalement et dans chaque crate
modifié. Voir [le bilan de validation](widget-api-completion.md). Cela ne remplace
pas un audit manuel des lecteurs d'écran ou de toutes les variantes du catalogue.

Sources : [sémantiques](../crates/argui-accessibility/src/schema.rs),
[interaction](../crates/argui-ui/src/interaction.rs),
[clavier](../crates/argui-core/src/keyboard.rs),
[éléments personnalisés](../crates/argui-ui/src/custom.rs).

Les sous-menus, délais de survol, recherches conservées, notifications, dates et
opérations sur les colonnes possèdent désormais des modèles publics dans les
widgets. Leurs pages de galerie restent indépendantes. Les détails des contrats
et de la validation sont suivis dans [widget-api-completion.md](widget-api-completion.md).

Aucun composant du catalogue n'est identifié comme impossible à reproduire dans
Argui. Cela constitue une appréciation de faisabilité architecturale, pas une
preuve que tous peuvent être livrés sans aucune évolution de l'API actuelle.
