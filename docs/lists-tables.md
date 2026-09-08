# Listes et tables

Activer `widget-list`, `widget-vlist` ou `widget-table` sur la façade `argui`.
`widgets-all` les active aussi. La galerie expose trois pages indépendantes :
List, VList et Table, avec des données neutres.

## Liste simple

```rust
use argui::ui::Element;
use argui::widgets::{List, ListState};

// Conserver cet état dans le composant propriétaire.
let selection = ListState::default();
let list = List::new("items", 8)
    .label("Items")
    .selection(&selection, true);
let element = list.build(theme, |index| Element::text(format!("Item {index}")));
```

Comme les autres widgets contrôlés, `List::action(&event)` renvoie le nouvel état
à conserver ; attacher les listeners Click et Key à la racine. Lorsqu'une action
est reconnue, empêcher le traitement clavier par défaut, demander le focus sur
`list.row_key(active)` et notifier le composant. Les descendants interactifs avec
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
let element = list.build_list(10_000, &selection, true, theme, |index| {
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
`heights.scroll_to(active, VirtualAlignment::Nearest, offset)`, reconstruire la
fenêtre puis envoyer `ScrollRequest::offset` et la demande de focus. Pour un
redimensionnement, `heights = heights.with_viewport(new_height)` conserve les
mesures. Les effets et la propagation du scroll restent configurables.

Les indices désignent l'ordre actuel. Lors d'une insertion/suppression, appeler
`ListState::insert/remove` **et** `VirtualList::insert/remove` sur les mêmes plages.
Pour conserver l'ancrage lors d'une mutation, retenir l'indice de la première
ligne visible et l'écart à `offset_of(index)`, remapper cet indice puis reconstruire
l'offset avec le nouvel `offset_of`. Un tri nécessite un remappage applicatif des
identifiants vers les indices ; le widget ne devine pas l'identité des données.

## Table

```rust
use argui::widgets::{Table, TableColumn};

let table = Table::new("values", [
    TableColumn::new("Name", 180.0),
    TableColumn::new("Value", 120.0),
], 8).label("Values").selection(&selection, true);
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
