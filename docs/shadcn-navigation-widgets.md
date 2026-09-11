# Label, Skeleton, Breadcrumb et Pagination

Quatre widgets publics et quatre pages de galerie supplémentaires. Références
fonctionnelles : documentation officielle shadcn/ui consultée le 11 septembre 2026.

| Widget | Contrat Argui | Référence |
| --- | --- | --- |
| `Label` | Texte associé à un contrôle, cible de focus sur clic, état désactivé, sans arrêt Tab supplémentaire. | [Label](https://ui.shadcn.com/docs/components/base/label) |
| `Skeleton` | Forme décorative dimensionnable, rayon libre, pulsation liée au montage, mouvement réduit. | [Skeleton](https://ui.shadcn.com/docs/components/base/skeleton) |
| `Breadcrumb` | Ancêtres avec identifiants stables, page courante non interactive, séparateur personnalisable et liens activables avec Entrée. | [Breadcrumb](https://ui.shadcn.com/docs/components/base/breadcrumb) |
| `Pagination` | Pages à partir de 1, précédent/suivant, ellipses, bornes, état désactivé et libellés traduisibles. | [Pagination](https://ui.shadcn.com/docs/components/base/pagination) |

Features `label`, `skeleton`, `breadcrumb`, `pagination` dans `argui-widgets/all`,
et `widget-*` dans la façade `argui`. Aucune nouvelle dépendance : Skeleton
active le runtime optionnel existant, Breadcrumb et Pagination réutilisent Button.

```rust
use argui::widgets::{Breadcrumb, BreadcrumbLink, Input, Label, Pagination, Skeleton};
use argui::ui::{length, percent};

let label = Label::new("name-label", "Nom", "name");
let control = label.associate(Input::new("name", "Ada", "", theme.input()).build());
let caption = label.build(theme);
// Monter caption et control dans le même scope accessible.
// Dans le listener Click :
if let Some(target) = label.focus_target(event) {
    cx.request_focus(target);
}

let path = Breadcrumb::new("path", [BreadcrumbLink::new("home", "Accueil")], "Projet");
let pages = Pagination::new("results", current_page, page_count);
let loading = Skeleton::new("cover").size(percent(1.0), length(160.0));
```

`Label::associate` conserve les propriétés du contrôle, assigne sa clé et
remplace son nom direct par la relation `labelled_by`. Le consommateur conserve
la même valeur `enabled` sur le label et le contrôle. Le clic demande le focus ;
il ne modifie pas la valeur d'une checkbox. Les actions et validations restent
la responsabilité du contrôle.

`Breadcrumb::action` renvoie l'identifiant d'un ancêtre ; l'application décide
de la navigation. Aucun navigateur ni URL n'est ouvert. Les identifiants en
double sont refusés. Les textes longs passent à la ligne. Les menus repliés
du catalogue shadcn ne font pas partie de cette version.

`Pagination::action` renvoie une nouvelle page uniquement pour une destination
visible et active. Les pages invalides sont bornées ; un total nul donne une
page courante égale à zéro et deux boutons désactivés. Le rendu contient au
plus sept entrées entre précédent et suivant, y compris avec `usize::MAX` pages.
La page active garde son focus et ne déclenche pas de changement. Les boutons
utilisent Tab, Entrée et Espace. `PaginationLabels` permet de traduire toutes
les annonces et les libellés de navigation.

Breadcrumb et Pagination exposent actuellement un **groupe nommé** dans les
adaptateurs accessibles d'Argui, et une description pour la page courante.
Le schéma moteur ne possède pas encore de landmark Navigation ni d'état
`aria-current` : ces deux sémantiques shadcn ne sont donc pas revendiquées.

`Skeleton::build` est statique à la phase courante ; monter `Entity<Skeleton>`
pour animer. `set_animated(false)`, le mouvement réduit et le démontage arrêtent
les demandes de frames. Les formes sont ignorées par l'accessibilité ; annoncer
le chargement sur leur groupe parent, une seule fois. La galerie conserve ses
modèles entre les visites, tout en démontant les formes quand le contenu est prêt.

L'inspection visuelle a également révélé une largeur de texte intrinsèque
arrondie vers le bas par le layout : un dernier caractère ou mot pouvait passer
sur une ligne hors de sa boîte. Le moteur texte arrondit maintenant cette largeur
vers le haut. Des tests avec la police Noto Sans de la galerie vérifient les
glyphes complets des liens et des numéros de page, en plus des bounds du layout.

Les commandes et les limites des vérifications graphiques sont conservées dans
[linux_testing.md](linux_testing.md). Les captures locales se trouvent dans
`target/shadcn-navigation/`. Le contrôle de couverture utilise le commit de départ
`6ac75a2`, sans abaisser les quatre seuils de 85 %.

Validation du 11 septembre 2026 : 343 tests ciblés réussis (un test optionnel
ignoré), Clippy ciblé sans avertissement, builds natif et WebAssembly réussis.
Les 19 captures Chromium/Vulkan ont été produites sur le compositeur privé,
avec rejet automatique des images blanches ou trop petites. Les quatre pages
ont été inspectées en clair/sombre à 1220 × 780 et 800 × 720. Clic, saisie,
Tab, Entrée, Espace, dernière page et état chargé ont été exercés sans erreur
JavaScript. Sur 650 ms après stabilisation : 37 callbacks d'animation pendant
le chargement, zéro après chargement, démontage ou mouvement réduit. Le lancement
natif a été limité volontairement à huit secondes ; il ne remplace pas ces
vérifications visuelles ni un audit des lecteurs d'écran.
