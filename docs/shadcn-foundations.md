# Badge, Card, Alert, Separator et Collapsible

Cinq widgets Argui publics supplémentaires, avec une page par composant dans
la galerie. Référence fonctionnelle : documentation officielle shadcn/ui consultée
le 10 septembre 2026. Le portage couvre les contrats ci-dessous ; les variantes
React et la composition DOM ne font pas partie de cette API Rust.

| Widget | Contrat livré | Référence |
| --- | --- | --- |
| `Badge` | Cinq variantes visuelles, icônes avant/après, une seule annonce accessible. Composant non interactif. | [Badge](https://ui.shadcn.com/docs/components/badge) |
| `Card` | Contenu libre, titre, description, action et footer optionnels. Titre et description reliés au groupe accessible ; niveau de titre configurable. | [Card](https://ui.shadcn.com/docs/components/card) |
| `Alert` | Message inline standard/destructive, description et icône optionnelles. Annonce assertive par défaut, configurable avec `live`. | [Alert](https://ui.shadcn.com/docs/components/alert) |
| `Separator` | Une API pour ligne horizontale, verticale et texte centré entre deux traits. Décorative par défaut, rôle et orientation accessibles avec `decorative(false)`. | [Separator](https://ui.shadcn.com/docs/components/separator) |
| `Collapsible` | Ouverture contrôlée, trigger personnalisable, indicateur optionnel, désactivation, activation par clic/Entrée/Espace. Contenu démonté quand fermé. | [Collapsible](https://ui.shadcn.com/docs/components/collapsible) |

Chaque widget possède sa feature dans `argui-widgets` (`badge`, `card`, `alert`,
`separator`, `collapsible`) et sa feature `widget-*` dans la façade `argui`.
`all` / `widgets-all` les incluent. Aucune dépendance ajoutée ; seul `Collapsible`
active aussi `button`, dont il réutilise le chemin d'activation.

```rust
use argui::widgets::{Badge, BadgeVariant, Card, Collapsible, Separator};
use argui::ui::Element;

let status = Badge::new("status", "Ready")
    .variant(BadgeVariant::Secondary)
    .build(theme);
let card = Card::new("project", Element::text("Project content"))
    .title("My project")
    .description("A shared component library")
    .action(status)
    .footer(Separator::new("footer-divider").build(theme))
    .build(theme);
let disclosure = Collapsible::new("details", "Project details", open, card);
// Dans le gestionnaire d'événements : appliquer la valeur à l'état puis notifier.
if let Some(next_open) = disclosure.action(event) {
    open = next_open;
}
let element = disclosure.build(theme);
```

Les clés doivent rester uniques dans leur scope. `Collapsible::trigger_key()`
et `content_key()` permettent de cibler ses parties. Les relations `controls`
et `labelled_by` ne pointent que vers du contenu monté. L'application conserve
les données du contenu et décide de la restauration du focus si elle ferme
programmatiquement une section dont un enfant est focalisé.

`Separator::new(key)` est horizontal par défaut ;
`.orientation(Orientation::Vertical)` nécessite une hauteur définie sur son
parent. `.label("Ou continuer avec")` centre le texte entre deux traits égaux.
Sans rôle explicite, seul le texte est annoncé ; avec `decorative(false)`, le
séparateur possède ce nom accessible et ses enfants ne le répètent pas.
Pour un message persistant déjà présent à l'ouverture d'une page, la galerie
utilise `Alert::live(LiveRegion::Off)` ; les annonces dynamiques restent au choix
du consommateur. Les badges et les icônes de disclosure restent décoratifs
lorsqu'ils sont contenus dans un contrôle ayant déjà son nom accessible.

Ces widgets n'ajoutent ni timer ni abonnement à des frames d'animation. Le moteur
de rendu reste inchangé. Les transitions usuelles des boutons utilisent les
états retenus existants, et le contenu fermé de Collapsible ne participe pas au
layout ni au parcours clavier. L'ouverture est immédiate, sans animation de
hauteur.

La vérification dans le navigateur a aussi révélé que Winit transmet Espace
comme `NamedKey::Space`. La traduction runtime la convertit désormais en
`Key::Character(" ")`, le contrat d'activation déjà utilisé par les boutons.
Les touches nommées et les caractères suivent ainsi le même comportement.

Les tests de comportement vivent dans `crates/argui-widgets/tests/` : relations
accessibles, sections optionnelles, retour à la ligne en largeur réduite,
épaisseur des séparateurs, contenu démonté, activation clavier et état désactivé.
Les tests de galerie couvrent les cinq pages, la navigation depuis la carte
et la conservation de l'ouverture de Collapsible entre deux visites.

Validation du 10 septembre 2026 : 350 tests ciblés réussis, un test natif ignoré,
Clippy ciblé sans avertissement et build WebAssembly réussi. Les cinq pages ont
été inspectées dans Chromium sur une session Wayland isolée, en clair/sombre à
1220 × 780 et en fenêtre de 800 × 720. Le clic, Entrée, Espace et l'action de la
carte ont été exercés dans le navigateur sans erreur JavaScript.

```sh
cargo nextest run -p argui-widgets -p argui-widget-gallery -p argui-runtime --all-features
```
