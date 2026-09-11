# Avatar, Empty, Kbd, Progress et AspectRatio

Cinq widgets publics supplémentaires, chacun avec sa page de galerie. Le
Separator utilise désormais un constructeur commun aux deux orientations et
peut placer un texte au milieu. Références fonctionnelles : documentation
officielle shadcn/ui consultée le 10 septembre 2026.

| Widget | Contrat livré | Référence |
| --- | --- | --- |
| `Avatar` | Image déjà chargée ou fallback contrôlé, tailles libres, découpe circulaire et un seul nom accessible. | [Avatar](https://ui.shadcn.com/docs/components/avatar) |
| `Empty` | Titre, description et média décoratif optionnels, contenu libre pour les actions, bordure optionnelle. | [Empty](https://ui.shadcn.com/docs/components/empty) |
| `Kbd` | Touches individuelles et combinaisons, description vocale personnalisable, sans enregistrer de raccourci. | [Kbd](https://ui.shadcn.com/docs/components/kbd) |
| `Progress` | Pourcentage de 0 à 100 ou état indéterminé, valeur accessible, animation respectant le mouvement réduit. | [Progress](https://ui.shadcn.com/docs/components/progress) |
| `AspectRatio` | Cadre proportionnel à sa largeur, contenu ajusté au cadre, place conservée dans le flux du layout. | [Aspect Ratio](https://ui.shadcn.com/docs/components/aspect-ratio) |

Les features `avatar`, `empty`, `kbd`, `progress`, `aspect-ratio` sont incluses
dans `argui-widgets/all` et exposées par les features `widget-*` de la façade.
Aucune nouvelle dépendance n'est ajoutée ; `progress` active la dépendance
optionnelle existante vers `argui-runtime`, comme `spinner`.

```rust
use argui::ui::{Element, Orientation};
use argui::widgets::{AspectRatio, Avatar, Empty, Kbd, Progress, Separator};

let horizontal = Separator::new("divider").build(theme);
let vertical = Separator::new("column-divider")
    .orientation(Orientation::Vertical)
    .build(theme); // Placer dans un parent de hauteur définie.
let labeled = Separator::new("login-divider")
    .label("Ou continuer avec")
    .build(theme);
let avatar = Avatar::new("profile", "Ada Lovelace", "AL")
    .image(loaded_image) // Option<ImageId> ; None pendant le chargement ou en erreur.
    .size(40.0)
    .build(theme);
let empty = Empty::new("projects", "Aucun projet")
    .description("Créez votre premier projet pour commencer.")
    .content(create_button)
    .build(theme);
let shortcut = Kbd::new("search-shortcut", ["Ctrl", "K"]).build(theme);
let frame = AspectRatio::new("cover", 16.0 / 9.0, Element::image(image_id)).build();
let progress = Progress::new("upload", "Envoi des fichiers", Some(45.0)).build(theme);
```

`Avatar` ne charge pas d'URL : l'application conserve le résultat du chargement
et choisit l'image ou le fallback. `Empty` relie son titre et sa description à
son groupe accessible ; ses actions conservent leur focus. Son média est
décoratif. `Kbd::label` permet de donner un nom lisible aux symboles tels que
`⌘` ou `↵`. `AspectRatio` exige un ratio fini strictement positif ; appliquer
un clip au cadre si le contenu doit être découpé.

Pour animer un Progress indéterminé, le monter comme `Entity<Progress>` via
`cx.entity(&progress)`. Mettre à jour la valeur avec `set_value` et `cx.notify()`
dans `Entity::update`. `None` et les valeurs non finies produisent l'état
indéterminé ; les valeurs finies sont bornées à 0–100. Un appel direct à `build`
produit une vue statique.

Seul le Progress indéterminé monté demande des frames d'animation. Une valeur
déterminée, le mouvement réduit ou le démontage arrêtent ces demandes. La version
web du runtime lit la préférence de mouvement réduit au lancement :
recharger la page après un changement de cette préférence système.
La galerie crée son modèle à la première visite et le conserve entre les visites.
Les autres composants ne créent ni timer, ni abonnement, ni tâche de fond.
Les démonstrations d'images réutilisent le logo déjà chargé par la galerie.

Les tests ciblés vérifient le centrage des séparateurs, les annonces accessibles,
les états et tailles des avatars, les actions d'Empty, les raccourcis décrits
une seule fois, les proportions après redimensionnement et les transitions de
Progress, dont le mouvement réduit.

```sh
cargo nextest run -p argui-widgets -p argui-widget-gallery --all-features
cargo clippy -p argui-widgets -p argui-widget-gallery -p argui --all-targets --all-features -- -D warnings
```

Validation visuelle du 10 septembre 2026 : les six pages ont été exercées dans
Chromium sur un compositeur Wayland isolé, sans fenêtre sur le bureau principal,
en clair/sombre à 1220 × 780 et à 800 × 720. Les contrôles de Progress et l'action
d'Empty fonctionnent, sans erreur JavaScript. Sur des fenêtres de mesure de
650 ms après stabilisation, le navigateur a exécuté 37 callbacks d'animation en
mode indéterminé, et zéro en mode déterminé, après navigation hors de la page ou
avec le mouvement réduit actif au lancement. Ce contrôle du cycle d'animation
ne constitue pas une mesure du CPU ou de la mémoire de l'application native.

Le contrôle `./scripts/quality.sh` a été exécuté une fois après l'implémentation :
structure, formatage, Clippy, compilation WASM, contrats Rustdoc et les 997 tests
de comportement passent (deux tests natifs optionnels ignorés). Le build natif
release passe aussi. La couverture mesurée est :

| Périmètre | Branches | Fonctions | Lignes | Régions |
| --- | ---: | ---: | ---: | ---: |
| Workspace | 86,48 % | 89,90 % | 90,84 % | 90,42 % |
| Widgets | 89,25 % | 95,27 % | 96,13 % | 95,40 % |
| Galerie | 85,25 % | 93,07 % | 95,72 % | 95,51 % |
| Renderer | 78,79 % | 60,00 % | 59,36 % | 65,97 % |
| Runtime | 69,22 % | 69,86 % | 69,98 % | 68,17 % |

Le gate global échoue sur le renderer et le runtime, déjà sous le seuil de
85 % avant cette série de composants. Les modifications de ces crates issues
du chantier précédent sont toujours présentes dans le workspace. Aucun seuil
ni exclusion de couverture n'a été modifié. Après communication de ce résultat,
l'utilisateur a explicitement demandé de committer l'ensemble des changements.

Suite du catalogue : [Label, Skeleton, Breadcrumb et Pagination](shadcn-navigation-widgets.md).
Pour les prochains contrôles graphiques, utiliser [la procédure Linux invisible](linux_testing.md).
