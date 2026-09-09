# Listes et tables

Activer `widget-list`, `widget-vlist` ou `widget-table` sur la façade `argui`.
`widgets-all` les active aussi. La galerie expose trois pages indépendantes :
List, VList et Table, avec des données neutres.

## Liste simple

```rust
use argui::ui::Element;
use argui::widgets::{Collection, CollectionItem, List, ListState};

// Conserver cet état dans le composant propriétaire.
let selection = ListState::default();
let items = Collection::new((0..8).map(|i| CollectionItem::new(i.to_string(), format!("Item {i}")))).unwrap();
let list = List::new("items", &items)
    .label("Items")
    .selection(&selection, true);
let element = list.build(theme, |index| Element::text(format!("Item {index}")));
```

Comme les autres widgets contrôlés, `List::action(&event)` renvoie le nouvel état
à conserver ; attacher les listeners Click et Key à la racine. Lorsqu'une action
est reconnue, empêcher le traitement clavier par défaut, demander le focus sur
`"items"` (le conteneur) et notifier le composant. Les descendants interactifs avec
leur propre clé restent indépendants : leurs événements ne sélectionnent pas la
ligne. Les labels explicites du contenu sont conservés ; les lignes de texte
simples reçoivent automatiquement leur nom accessible.

Clic sélectionne une ligne, Ctrl/Cmd-clic la bascule, Shift étend depuis l'ancre,
Ctrl/Cmd-Shift ajoute une plage. Flèches et Home/End déplacent la ligne active ;
Shift étend la sélection et Ctrl/Cmd seul déplace sans sélectionner. Entrée ou Espace active
la ligne courante ; Ctrl/Cmd-A sélectionne tout en mode multiple.

## Hauteurs variables

```rust
use argui::ui::VirtualList;
use argui::widgets::VList;

// Créer une fois, puis conserver : recréer à chaque rendu perd les mesures.
let heights = VirtualList::variable(10_000, 40.0, 320.0);
let list = VList::variable("items", &heights, offset);
let element = list.build_list(&items, &selection, true, theme, |index| {
    Element::text(format!("Item {index}"))
});
```

Le layout mesure les lignes réellement montées ; les clones du modèle partagent
ces mesures. Le moteur corrige l'ancrage lors d'une mesure au-dessus du viewport.
La hauteur fournie au départ est une estimation, pas une hauteur imposée.
`VList::new` conserve le chemin à hauteur fixe. `build` reste disponible sans
sélection. Le nombre de lignes passé au widget variable doit correspondre au
modèle conservé ; une divergence est rejetée immédiatement.

Conserver l'offset fourni par les événements Scroll. Pour une navigation vers une
ligne non montée, calculer l'offset avec
`heights.scroll_to(items.index_of(active).unwrap(), VirtualAlignment::Nearest, offset)`, reconstruire la
fenêtre puis envoyer `ScrollRequest::offset` et la demande de focus. Pour un
redimensionnement, `heights = heights.with_viewport(new_height)` conserve les
mesures. Les effets et la propagation du scroll restent configurables.

`Collection` conserve les identifiants stables et construit son index une seule
fois lors des changements de données. La sélection et l'ancre restent associées
aux identifiants après un tri. `ListState::reconcile` supprime les identifiants
retirés du jeu complet, avant filtrage. Les mesures du moteur restent indexées :
adapter `VirtualList::insert/remove` aux mutations de données.

Le focus reste sur le conteneur ; `active_descendant` désigne l'option active,
qui reste montée même hors écran. Le travail de rendu reste limité à la fenêtre
et à cette option. `List::page_size` active PageUp/PageDown selon le nombre de
lignes visibles. `List::search` utilise un `Typeahead` conservé et une durée
monotone fournie par l'appelant, avec un matcher personnalisable.

## Table

```rust
use argui::widgets::{Table, TableColumn};

let table = Table::new("values", [
    TableColumn::new("Name", 180.0),
    TableColumn::new("Value", 120.0),
], &items).label("Values").selection(&selection, true);
let element = table.build(theme, |row, column| {
    Element::text(format!("{row}:{column}"))
});
```

`Table::action` suit le même protocole que `List`. Largeurs explicites positives,
en-têtes communs, sémantique Grid/Row/ColumnHeader/Cell (table interactive) et sélection par ligne.
Styliser le texte des cellules et le contenu des lignes avec le thème comme pour
les autres primitives `Element` ; le widget stylise ses en-têtes et les états de
ligne. Pour une table plus large que son conteneur, placer la table dans un
viewport horizontal.

La table monte toutes ses lignes : tri, pagination, virtualisation de table,
édition et navigation par cellule ne font pas partie de cette première API.
Les sémantiques sont testées sans écran ; la validation avec lecteurs d'écran
natifs/Web réels reste à faire.

Implémentation de l'intégration contrôlée :
[page de composants](../crates/argui-widget-gallery/src/pages/data.rs).
Tests : [sélection](../crates/argui-widgets/tests/list.rs),
[virtualisation](../crates/argui-widgets/tests/vlist.rs),
[table](../crates/argui-widgets/tests/table.rs),
[galerie](../crates/argui-widget-gallery/tests/pages/data.rs).

## DataTable

`widget-data-table` expose `DataTableModel<R>`, `DataColumn<R>` et `DataTable`.
Les colonnes reçoivent des callbacks typés pour la valeur, la comparaison, le
filtre, le rendu, la validation et l'éditeur. Le modèle recalcule son ordre lors
d'un changement explicite : filtres, tri stable multiple, puis pagination.
Le widget emprunte ce modèle et une configuration `VirtualList` correspondant
à sa page courante. Les colonnes visibles partagent leurs largeurs avec l'en-tête.

L'éditeur conserve un brouillon. `commit_edit` ou `apply(CommitEdit)` renvoie
un `CellCommit` que le propriétaire applique à ses données ; aucune sauvegarde
n'est implicite. Une erreur garde l'éditeur ouvert. Échap annule ; Tab valide
puis déplace la cellule active. La suppression, le filtrage ou une modification
externe de la valeur d'origine annulent l'édition. Conserver une copie de `model.collection()` avant une transformation, puis
appeler `model.collection().remap_heights(&previous, &mut heights)` : les mesures
suivent les identifiants conservés. Cette copie partage le snapshot et ne parcourt
pas les lignes. Les poignées de colonnes utilisent les gestes et le clavier de
`SplitPane` ; traiter également les événements Gesture et appliquer l'action
`ResizeColumn`.


Le DataTable contient son propre viewport horizontal : l'en-tête et les lignes
partagent le même déplacement, y compris après un redimensionnement. Le viewport
vertical reste limité aux lignes et conserve leur virtualisation.

## TreeView : focus virtualisé

Une seule ligne participe au parcours Tab : la sélection visible, ou la première
ligne lorsque la sélection est absente. Les autres lignes acceptent le focus
programmatique. La ligne active reste montée hors de la fenêtre virtuelle ; le
cache conserve au plus les lignes de cette fenêtre et cette ligne supplémentaire.
Tab quitte l'arbre ; PageUp/PageDown déplacent la sélection d'une fenêtre visible.
Le propriétaire applique `TreeAction` et demande le focus sur la nouvelle clé.
