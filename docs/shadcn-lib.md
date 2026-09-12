# Catalogue Argui / shadcn/ui

État du **12 septembre 2026**, comparé aux **64 entrées** du
[catalogue officiel shadcn/ui](https://ui.shadcn.com/docs/components).
Les variantes Base UI, Radix UI et React Aria ne sont comptées qu’une fois.
Les blocks, recettes de formulaires et registres communautaires sont hors inventaire.

**Les 64 entrées disposent d’une API publique Argui couvrant leur usage principal.**
Ce relevé décrit le périmètre livré, pas une équivalence de toutes les variantes
React ni une certification par des lecteurs d’écran réels. Les limites figurent
explicitement dans la dernière colonne.

## Changements depuis le relevé du 8 septembre

Les fondations, widgets d’affichage et de navigation ont été livrés les 10–11
septembre. Tooltip, les effets des overlays, les popups natifs, le sélecteur de
fichiers et les réglages de fond de fenêtre existaient aussi ; l’ancien inventaire
n’avait pas suivi, notamment pour Tooltip.

Ce chantier ajoute les **27 API restantes**, **26 pages de galerie** et les
exemples de Typography dans sa page existante. Le moteur gagne l’état accessible
`pressed`, le rôle `Navigation`, la direction héritée, le filtre de longueur/chiffres
des éditeurs et le défilement clavier des viewports focalisés.

Les comptes de tests des anciens bilans restent datés dans leurs documents.
Ils ne constituent pas la validation de cet inventaire.

## API et périmètre

Chaque lien ouvre l’implémentation. La feature indiquée s’active dans
[`argui-widgets`](../crates/argui-widgets/Cargo.toml), ou avec le préfixe `widget-`
dans la façade [`argui`](../crates/argui/Cargo.toml). `all` (ou `widgets-all`
dans la façade) les active ensemble ; les 27 ajouts
n’introduisent aucune dépendance externe.

| shadcn/ui | API Argui | Feature | Périmètre livré et limites |
| --- | --- | --- | --- |
| Accordion | [Accordion](../crates/argui-widgets/src/accordion.rs) | `accordion` | Sections contrôlées simples/multiples, flèches/Home/End et désactivation ; ouverture sans animation. |
| Alert | [Alert](../crates/argui-widgets/src/alert.rs) | `alert` | présentation inline standard/destructive, icône et politique d’annonce explicite. |
| Alert Dialog | [AlertDialog](../crates/argui-widgets/src/alert_dialog.rs) | `alert-dialog` | Confirmation modale, focus initial sur Annuler, clic extérieur ignoré et action de confirmation distincte. |
| Aspect Ratio | [AspectRatio](../crates/argui-widgets/src/aspect_ratio.rs) | `aspect-ratio` | réserve la hauteur suivant la largeur et un ratio positif, contenu ajusté au cadre. |
| Attachment | [Attachment](../crates/argui-widgets/src/attachment.rs) | `attachment` | Fichier, média, actions, cinq états de transfert et pourcentage ; transfert fourni par l’application. |
| Avatar | [Avatar](../crates/argui-widgets/src/avatar.rs) | `avatar` | image chargée ou fallback contrôlé, masque circulaire, taille configurable et nom accessible unique. Groupe non livré. |
| Badge | [Badge](../crates/argui-widgets/src/badge.rs) | `badge` | variantes primary/secondary/destructive/outline/ghost, icônes avant/après et nom accessible unique. |
| Breadcrumb | [Breadcrumb](../crates/argui-widgets/src/breadcrumb.rs) | `breadcrumb` | ancêtres activables, identifiants stables, séparateurs personnalisés et page courante décrite ; groupe accessible, sans landmark Navigation. |
| Bubble | [Bubble](../crates/argui-widgets/src/bubble.rs) | `bubble` | Sept variantes, alignement et réactions ; contenu libre, couleurs de texte à choisir selon le fond. |
| Button | [Button](../crates/argui-widgets/src/button.rs) | `button` | variantes via le thème, icônes, chargement et activation accessible. |
| Button Group | [ButtonGroup](../crates/argui-widgets/src/button_group.rs) | `button-group` | Groupe nommé horizontal/vertical ; chaque bouton conserve son arrêt Tab. |
| Calendar | [Calendar](../crates/argui-widgets/src/calendar.rs) | `calendar` | `Calendar` et `CalendarState` ; locale, limites, sélection simple/multiple/plage et navigation clavier. |
| Card | [Card](../crates/argui-widgets/src/card.rs) | `card` | titre, description, action, contenu et pied de carte optionnels, avec relations accessibles. |
| Carousel | [Carousel](../crates/argui-widgets/src/carousel.rs) | `carousel` | Diapositive contrôlée, boutons, clavier, balayage, boucle optionnelle et annonce ; sans lecture automatique. |
| Chart | [Chart](../crates/argui-widgets/src/chart.rs) | `chart` | Barres et lignes multi-séries, échelle incluant zéro, légende et points accessibles activables ; autres tracés non fournis. |
| Checkbox | [Checkbox](../crates/argui-widgets/src/selection.rs) | `checkbox` | `CheckedState` expose Unchecked, Checked et Mixed ; activation du mode Mixed vers Checked. |
| Collapsible | [Collapsible](../crates/argui-widgets/src/collapsible.rs) | `collapsible` | ouverture contrôlée, trigger personnalisable, Entrée/Espace, état désactivé et contenu démonté une fois fermé. |
| Combobox | [Combobox](../crates/argui-widgets/src/combobox.rs) | `combobox` | Recherche éditable, filtrage, options désactivées, descendant actif et sélection clavier ; sélection simple. |
| Command | [CommandPalette](../crates/argui-widgets/src/command_palette.rs) | `command-palette` | recherche et invocation d'actions. Groupes et variantes avancées restent à examiner. |
| Context Menu | [ContextMenu](../crates/argui-widgets/src/context_menu.rs) | `context-menu` | `ContextMenu` partage les entrées de `Menu`, avec ancrage au pointeur ou au clavier. |
| Data Table | [DataTable](../crates/argui-widgets/src/data_table.rs) | `data-table` | modèle typé, filtres, tri multiple stable, pagination, colonnes visibles, virtualisation et édition contrôlée ; en-têtes alignés au défilement. |
| Date Picker | [DatePicker](../crates/argui-widgets/src/date_picker.rs) | `date-picker` | saisie localisable, brouillon contrôlé, validation et calendrier réutilisé. |
| Dialog | [Dialog](../crates/argui-widgets/src/dialog.rs) | `dialog` | Modal, piège et restauration du focus, focus initial configurable, placement et contenu défilable. |
| Direction | [Direction](../crates/argui-widgets/src/direction.rs) | `direction` | Direction de layout héritée avec scopes imbriqués ; paramètre rtl des contrôleurs clavier explicite. |
| Drawer | [Drawer](../crates/argui-widgets/src/drawer.rs) | `drawer` | Panneau bas modal, poignée gestuelle, seuil de distance/vitesse et annulation ; sans positions intermédiaires. |
| Dropdown Menu | [Menu](../crates/argui-widgets/src/menu.rs) | `menu` | identifiants stables, sous-menus, groupes, entrées checkbox/radio, indicateurs et navigation clavier. |
| Empty | [Empty](../crates/argui-widgets/src/empty.rs) | `empty` | état vide centré, titre, description, média décoratif et actions libres ; bordure optionnelle. |
| Field | [Field](../crates/argui-widgets/src/field.rs) | `field` | Label, aide, erreur, requis et désactivation associés à la clé du contrôle ; validation applicative. |
| Hover Card | [HoverCard](../crates/argui-widgets/src/hover_card.rs) | `hover-card` | Aperçu interactif non modal, délais, maintien au survol entre déclencheur et contenu, focus et fermeture. |
| Input | [Input](../crates/argui-widgets/src/input.rs) | `input` | texte, recherche, password et état contrôlé. |
| Input Group | [InputGroup](../crates/argui-widgets/src/input_group.rs) | `input-group` | Surface commune avec éditeur, décorations et actions avant/après. |
| Input OTP | [InputOtp](../crates/argui-widgets/src/input_otp.rs) | `input-otp` | Un éditeur de 1 à 16 chiffres ASCII espacés, filtre avant mutation et complétion ; pas de cases indépendantes ni récupération SMS. |
| Item | [Item](../crates/argui-widgets/src/item.rs) | `item` | Titre, description, média et actions sur une ligne réutilisable. |
| Kbd | [Kbd](../crates/argui-widgets/src/kbd.rs) | `kbd` | touche ou combinaison, une annonce accessible personnalisable ; n'enregistre pas de raccourci. |
| Label | [Label](../crates/argui-widgets/src/label.rs) | `label` | label visible associé via `labelled_by`, cible de focus sur clic et état désactivé ; sans arrêt Tab supplémentaire. |
| Marker | [Marker](../crates/argui-widgets/src/marker.rs) | `marker` | Note inline, bordée ou séparatrice, icône et annonce opt-in. |
| Menubar | [Menubar](../crates/argui-widgets/src/menubar.rs) | `menubar` | focus entre déclencheurs, ouverture des menus, sous-menus et navigation RTL. |
| Message | [Message](../crates/argui-widgets/src/message.rs) | `message` | Auteur, avatar, en-tête, contenu et pied ; alignement configurable. |
| Message Scroller | [MessageScroller](../crates/argui-widgets/src/message_scroller.rs) | `message-scroller` | Suivi du bas, pause pendant la lecture, compteur, retour au dernier et conservation du décalage lors d’un ajout en tête. |
| Native Select | [NativeSelect](../crates/argui-widgets/src/native_select.rs) | `native-select` | Sélecteur compact dessiné par Argui, requis/désactivé, clavier et recherche ; aucun contrôle OS. |
| Navigation Menu | [NavigationMenu](../crates/argui-widgets/src/navigation_menu.rs) | `navigation-menu` | Landmark Navigation, actions, panneaux, page courante et navigation clavier. |
| Pagination | [Pagination](../crates/argui-widgets/src/pagination.rs) | `pagination` | navigation contrôlée, bornes, ellipses, désactivation et annonces traduisibles ; groupe accessible, page courante décrite. |
| Popover | [Popover](../crates/argui-widgets/src/popover.rs) | `popover` | ancrage, collisions, fermeture et options de focus. |
| Progress | [Progress](../crates/argui-widgets/src/progress.rs) | `progress` | pourcentage contrôlé, état indéterminé animé quand monté comme entité, mouvement réduit et valeur accessible. |
| Questionnaire | [Questionnaire](../crates/argui-widgets/src/questionnaire.rs) | `questionnaire` | Étapes, choix simples/multiples, texte libre, facultatif, validation et réponses retournées au consommateur. |
| Radio Group | [RadioGroup](../crates/argui-widgets/src/selection.rs) | `radio-group` | sélection exclusive et orientation. |
| Resizable | [SplitPane](../crates/argui-widgets/src/split_pane.rs) | `split-pane` | séparateur, limites, gestes et clavier. Groupes imbriqués à composer explicitement. |
| Scroll Area | [ScrollArea](../crates/argui-widgets/src/scroll_area.rs) | `scroll-area` | Viewport horizontal/vertical, roue, gestes, scrollbar et défilement clavier focalisé. |
| Select | [Select](../crates/argui-widgets/src/select.rs) | `select` | sélection simple contrôlée, overlay et clavier. |
| Separator | [Separator](../crates/argui-widgets/src/separator.rs) | `separator` | orientation configurable et texte centré entre deux traits, décoratif par défaut ou rôle accessible explicite. |
| Sheet | [Sheet](../crates/argui-widgets/src/sheet.rs) | `sheet` | Panneau modal attaché à l’un des quatre côtés, focus restauré et contenu défilable. |
| Sidebar | [Sidebar](../crates/argui-widgets/src/sidebar.rs) | `sidebar` | Navigation étendue, rail replié ou Sheet mobile ; breakpoint choisi par l’application. |
| Skeleton | [Skeleton](../crates/argui-widgets/src/skeleton.rs) | `skeleton` | formes décoratives dimensionnables, pulsation liée au montage et mouvement réduit. |
| Slider | [Slider](../crates/argui-widgets/src/slider.rs) | `slider` | valeur, bornes, pas, gestes et clavier. Multi-poignées non livré. |
| Spinner | [Spinner](../crates/argui-widgets/src/spinner.rs) | `spinner` | animation liée au montage et mouvement réduit. |
| Switch | [Switch](../crates/argui-widgets/src/selection.rs) | `switch` | booléen animé et activation accessible. |
| Table | [Table](../crates/argui-widgets/src/table.rs) | `table` | en-têtes, cellules, largeurs communes, sélection et navigation par ligne. Pas de datagrid complet. |
| Tabs | [Tabs](../crates/argui-widgets/src/tabs.rs) | `tabs` | sélection, navigation et montage du panneau actif. |
| Textarea | [TextArea](../crates/argui-widgets/src/input.rs) | `textarea` | édition multiligne et scroll. Le redimensionnement se compose avec les gestes publics. |
| Toast | [Toast](../crates/argui-widgets/src/toast.rs) | `toast` | file contrôlée, durée, pause au survol/focus, annonces et actions. |
| Toggle | [Toggle](../crates/argui-widgets/src/toggle.rs) | `toggle` | Bouton à état persistant pressed, désactivation et variantes visuelles. |
| Toggle Group | [ToggleGroup](../crates/argui-widgets/src/toggle_group.rs) | `toggle-group` | Sélection simple/multiple, orientation, RTL et focus itinérant. |
| Tooltip | [Tooltip](../crates/argui-widgets/src/tooltip.rs) | `tooltip` | Délais, focus/survol, relation descriptive, contenu survolable, effets et présentation native optionnelle. |
| Typography | [Typography](../crates/argui-widgets/src/typography.rs) | `typography` | Titres h1–h6, paragraphe, lead, large, small, muted, code et citation ; texte riche disponible dans le moteur. |

## Intégration

Les nouveaux widgets suivent le contrat existant : construire la vue avec le
thème, transmettre les événements à `action`, appliquer l’action à l’état retenu
et notifier l’entité. Transmettre les demandes de focus au runtime. Les modèles
ne font ni requêtes réseau, ni transferts, ni envois de réponses.

Exemples complets, répartis par responsabilité :

- [Formulaires](../crates/argui-widget-gallery/src/pages/catalogue/forms.rs) :
  Field, Input Group, OTP, Combobox, Native Select et Questionnaire.
- [Navigation](../crates/argui-widget-gallery/src/pages/catalogue/navigation.rs) :
  Accordion, toggles, Button Group, Navigation Menu, Sidebar et Direction.
- [Surfaces](../crates/argui-widget-gallery/src/pages/catalogue/surfaces.rs) :
  Sheet, Alert Dialog, Drawer, Carousel, Chart et Hover Card.
- [Conversation et contenu](../crates/argui-widget-gallery/src/pages/catalogue/conversation.rs) :
  Attachment, Bubble, Item, Marker, Message, Message Scroller et Scroll Area.
- [Typography](../crates/argui-widget-gallery/src/pages/typography.rs) : styles
  publics et primitives de texte riche/sélection.

Pour Hover Card, conserver `HoverCardState`, transmettre Focus/Blur/Key en capture
sur le panneau et planifier `advance` à `next_deadline` avec un `TaskSlot`.
Aucun polling permanent n’est nécessaire. Pour Message Scroller, fournir les
mesures de layout à `appended`/`prepended`, transmettre les événements de lecture
à `observe`, puis appliquer les `ScrollRequest` retournées. La virtualisation,
les ancres de tours de conversation et le streaming réseau restent à composer.

Direction hérite dans le **layout** et autorise un scope imbriqué contraire.
Les contrôleurs de collection conservent leur paramètre `rtl` pour interpréter
les flèches : l’application doit le régler selon la même direction. Le moteur
reste indépendant des widgets, de shadcn et de tout futur DSL.

Dans la galerie, rechercher par le nom affiché ; un nom exact passe devant une
correspondance partielle (Select avant Native Select). Ctrl/Cmd+K focalise la
recherche. La saisie depuis le fond ou un bouton la démarre directement ; les
éditeurs et collections conservent leurs propres raccourcis.

## Validation reproductible

Les [tests de widgets](../crates/argui-widgets/tests/) couvrent actions, limites,
états désactivés, relations accessibles, gestes et layout. Les
[tests de galerie](../crates/argui-widget-gallery/tests/pages/catalogue.rs)
parcourent les nouvelles pages en clair/sombre à deux tailles, contrôlent les
relations et exercent les états retenus.

Le scénario [catalogue.mjs](../crates/argui-widget-gallery/tests/pages/catalogue.mjs)
exerce le canvas WebGPU à 1220 × 780 et 800 × 720 dans les deux thèmes : captures
des pages, boutons, navigation clavier, champs, OTP, sélecteurs, modales,
restauration du focus, Hover Card et scroll. Les PNG sont enregistrés dans
`target/catalogue-interactions/` et doivent être inspectés. Une capture blanche
échoue. Voir [la procédure Linux](linux_testing.md) pour le build et le display
privé ; le navigateur ne doit jamais utiliser le bureau personnel.

Le contrôle d’acceptation est celui de [code_quality.md](code_quality.md), avec
85 % minimum sur chaque métrique globale et par crate, et les tests natifs activés.
Les artefacts de couverture dans `target/` portent les résultats de l’exécution,
sans recopier un ancien total de tests comme preuve actuelle.

## Historique et capacités complémentaires

Les bilans suivants documentent les étapes antérieures et leurs validations :
[API du moteur](widget-api-completion.md), [fondations](shadcn-foundations.md),
[affichage](shadcn-display-widgets.md), [navigation](shadcn-navigation-widgets.md),
[overlays](shadcn-overlays.md), [popups natifs](native-popovers.md),
[sélecteur de fichiers](file-picker.md) et [fonds de bureau](desktop-backdrops.md).

List, VList, TreeView, FilePicker, WebView et les effets de verre ne sont pas des
entrées supplémentaires du catalogue shadcn. Ils restent disponibles dans Argui.
Voir [listes et tables](lists-tables.md) pour la virtualisation et ses mesures.
