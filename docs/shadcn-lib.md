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

## Checklist complète

- [ ] **Accordion** — absent ; sections contrôlées, navigation clavier et animation à exposer.
- [ ] **Alert** — absent ; le rôle sémantique existe, sans widget de présentation.
- [ ] **Alert Dialog** — partiel ; `Dialog` existe, mais pas de contrat de confirmation dédié.
- [ ] **Aspect Ratio** — partiel ; contrainte de layout disponible, sans composant dédié.
- [ ] **Attachment** — absent ; définir présentation, état de transfert et actions, sans imposer un client mail.
- [ ] **Avatar** — absent ; images disponibles, fallback et groupe à encapsuler.
- [ ] **Badge** — absent ; petit composant de présentation à créer.
- [ ] **Breadcrumb** — absent ; liens, séparateurs et élément courant à formaliser.
- [ ] **Bubble** — absent ; présentation réutilisable à définir.
- [x] **Button** — [`Button`](../crates/argui-widgets/src/button.rs) ; variantes via le thème, icônes, chargement et activation accessible.
- [ ] **Button Group** — absent ; disposer des boutons en ligne ne constitue pas encore une API de groupe.
- [ ] **Calendar** — absent ; modèle de date, locale, limites et navigation par jour/mois nécessaires.
- [ ] **Card** — absent ; conteneurs stylables disponibles, API de sections à livrer.
- [ ] **Carousel** — absent ; défilement disponible, pagination, gestes et annonces à coordonner.
- [ ] **Chart** — absent ; rendu vectoriel disponible, échelles, séries et interactions à concevoir.
- [x] **Checkbox** — [`Checkbox`](../crates/argui-widgets/src/selection.rs) ; contrôle booléen. État indéterminé à ajouter séparément.
- [ ] **Collapsible** — absent ; API d'ouverture, trigger et contenu à créer.
- [ ] **Combobox** — partiel ; `Select` et recherche existent séparément, sans combobox publique dédiée.
- [x] **Command** — [`CommandPalette`](../crates/argui-widgets/src/command_palette.rs) ; recherche et invocation d'actions. Groupes et variantes avancées restent à examiner.
- [ ] **Context Menu** — partiel ; menu contextuel de sélection de texte, sans menu générique public à coordonnées libres.
- [ ] **Data Table** — partiel ; `Table` et sélection de lignes existent. Tri, filtres, pagination, visibilité des colonnes et virtualisation de table ne sont pas intégrés.
- [ ] **Date Picker** — absent ; dépend du calendrier et d'un contrat de saisie/localisation des dates.
- [x] **Dialog** — [`Dialog`](../crates/argui-widgets/src/dialog.rs) ; modal, fermeture et restauration du focus.
- [ ] **Direction** — partiel ; texte bidi et alignement logique disponibles, sans fournisseur de direction commun aux widgets.
- [ ] **Drawer** — absent ; panneau gestuel, seuils de fermeture et focus à implémenter.
- [ ] **Dropdown Menu** — partiel ; [`Menu`](../crates/argui-widgets/src/menu.rs) à un niveau. Sous-menus, groupes et entrées checkbox/radio manquent.
- [ ] **Empty** — absent ; présentation d'état vide avec contenu/actions à exposer.
- [ ] **Field** — partiel ; labels et erreurs existent sur `Input`, sans composition générique label/aide/erreur/contrôle.
- [ ] **Hover Card** — absent ; overlay disponible, délais d'ouverture/fermeture et maintien au survol à ajouter.
- [x] **Input** — [`Input`](../crates/argui-widgets/src/input.rs) ; texte, recherche, password et état contrôlé.
- [ ] **Input Group** — partiel ; décorations possibles sur `Input`, sans groupe générique de contrôles et boutons associés.
- [ ] **Input OTP** — absent ; cellules, collage, navigation et saisie unique accessible à concevoir.
- [ ] **Item** — partiel ; `List` accepte du contenu libre, sans API de parties titre/description/média/actions.
- [ ] **Kbd** — absent ; composant typographique pour raccourcis à livrer.
- [ ] **Label** — partiel ; nom accessible sur les contrôles, sans label public associé à une cible de focus.
- [ ] **Marker** — absent ; composant dédié à définir.
- [ ] **Menubar** — absent ; actions disponibles, navigation entre menus et sous-menus à coordonner.
- [ ] **Message** — absent ; composant de message réutilisable à définir, sans mini-application imposée.
- [ ] **Message Scroller** — partiel ; `VList` virtualise, mais suivi du bas, chargement antérieur et compteur de nouveautés restent applicatifs.
- [ ] **Native Select** — absent ; `Select` est rendu par Argui. Pour ce catalogue, reproduire son usage avec un composant Argui ; aucun contrôle système requis.
- [ ] **Navigation Menu** — absent ; navigation de site/application avec panneaux, focus et état courant à fournir.
- [ ] **Pagination** — absent ; page courante, bornes et navigation accessible à encapsuler.
- [x] **Popover** — [`Popover`](../crates/argui-widgets/src/popover.rs) ; ancrage, collisions, fermeture et options de focus.
- [ ] **Progress** — partiel ; rôle sémantique présent, sans widget déterminé/indéterminé.
- [ ] **Questionnaire** — absent ; contrat de questions, réponses et validation à définir après les contrôles de base.
- [x] **Radio Group** — [`RadioGroup`](../crates/argui-widgets/src/selection.rs) ; sélection exclusive et orientation.
- [x] **Resizable** — [`SplitPane`](../crates/argui-widgets/src/split_pane.rs) ; séparateur, limites, gestes et clavier. Groupes imbriqués à composer explicitement.
- [ ] **Scroll Area** — partiel ; [`ScrollConfig`](../crates/argui-ui/src/scroll.rs) et scrollbars dans le moteur, sans widget général `ScrollArea`.
- [x] **Select** — [`Select`](../crates/argui-widgets/src/select.rs) ; sélection simple contrôlée, overlay et clavier.
- [ ] **Separator** — partiel ; primitive et séparateur redimensionnable, sans widget décoratif/sémantique générique.
- [ ] **Sheet** — absent ; `Dialog` fournit une base, sans panneau latéral dédié.
- [ ] **Sidebar** — absent ; la sidebar de la galerie n'est pas un widget public.
- [ ] **Skeleton** — absent ; formes et animations disponibles, sans composant d'attente.
- [x] **Slider** — [`Slider`](../crates/argui-widgets/src/slider.rs) ; valeur, bornes, pas, gestes et clavier. Multi-poignées non livré.
- [x] **Spinner** — [`Spinner`](../crates/argui-widgets/src/spinner.rs) ; animation liée au montage et mouvement réduit.
- [x] **Switch** — [`Switch`](../crates/argui-widgets/src/selection.rs) ; booléen animé et activation accessible.
- [x] **Table** — [`Table`](../crates/argui-widgets/src/table.rs) ; en-têtes, cellules, largeurs communes, sélection et navigation par ligne. Pas de datagrid complet.
- [x] **Tabs** — [`Tabs`](../crates/argui-widgets/src/tabs.rs) ; sélection, navigation et montage du panneau actif.
- [x] **Textarea** — [`TextArea`](../crates/argui-widgets/src/input.rs) ; édition multiligne et scroll. Le redimensionnement se compose avec les gestes publics.
- [ ] **Toast** — absent ; file de notifications, durée, annonces et actions à ajouter.
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

1. **Petits composants réutilisables** : Separator, Label, Badge, Card, Empty, Kbd,
   Avatar, Skeleton et Progress. Un fichier de widget et une page sobre par besoin.
2. **Interactions courantes** : Tooltip, Collapsible/Accordion, Toggle/Toggle Group,
   puis Field/Input Group/Combobox. Vérifier clavier, focus et sémantique avant
   d'élargir les variantes visuelles.
3. **Menus et panneaux** : terminer Dropdown/Context Menu, puis Sheet, Alert Dialog,
   Hover Card et Toast ; mutualiser les comportements d'overlay existants.
4. **Données** : tables virtualisées, tri, filtrage et colonnes ; conserver une API
   de table simple indépendante de ces fonctionnalités avancées.
5. **Chantiers plus coûteux** : Calendar/Date Picker (dates/locales), Chart
   (géométrie/interactions), Drawer/Carousel (gestes), Native Select (plateformes).

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
présentes. Les points suivants nécessitent des extensions ciblées :

- **Relations accessibles** : `Semantics` possède un label et une description
  textuels, mais pas de relations vers les éléments qui étiquettent, décrivent ou
  contrôlent un autre élément, ni de descendant actif. À compléter pour Field,
  Combobox, Tooltip et les collections virtuelles.
- **État indéterminé** : `checked: Option<bool>` représente une valeur booléenne
  ou l'absence de valeur, pas un troisième état « mixte ». Le contrat doit évoluer
  pour une checkbox de sélection partielle.
- **Focus des collections** : `Interaction::focusable` ne distingue pas la
  possibilité de recevoir le focus et la participation au parcours Tab. Ce
  contrat doit permettre un seul arrêt Tab dans une collection, avec déplacement
  interne et conservation du focus pendant la virtualisation.
- **Clavier** : `Key` ne distingue pas PageUp/PageDown. Ajouter leur traduction
  plateforme pour les calendriers et les grandes collections.

Sources : [sémantiques](../crates/argui-accessibility/src/schema.rs),
[interaction](../crates/argui-ui/src/interaction.rs),
[clavier](../crates/argui-core/src/keyboard.rs),
[éléments personnalisés](../crates/argui-ui/src/custom.rs).

Sous-menus, délais de survol, recherche au clavier, gestion des toasts, modèle de
dates et opérations sur les colonnes relèvent principalement de comportements
et modèles à ajouter aux widgets. Leur absence ne prouve pas un manque dans le
renderer. Les contrats communs seront extraits lorsqu'un composant en démontre
le besoin, sans construire à l'avance une nouvelle infrastructure générale.

Aucun composant du catalogue n'est identifié comme impossible à reproduire dans
Argui. Cela constitue une appréciation de faisabilité architecturale, pas une
preuve que tous peuvent être livrés sans aucune évolution de l'API actuelle.
