# Popover et Tooltip

La galerie expose **Popover** et **Tooltip** dans sa liste alphabétique, alignée
à gauche. Les interactions suivent les références
[Popover de shadcn/ui](https://ui.shadcn.com/docs/components/base/popover) et
[Tooltip de shadcn/ui](https://ui.shadcn.com/docs/components/base/tooltip).

Popover présente trois panneaux : renommer un projet, régler le partage et
choisir un accent. Le bouton ouvre ou ferme son panneau ; le clic extérieur et
Échap le ferment. Les champs et boutons du panneau restent interactifs.

Tooltip fournit une description au survol ou au focus clavier. `TooltipState`
attend 350 ms au survol (délai configurable), annule les passages rapides,
permet de déplacer la souris dans la bulle et la ferme 100 ms après la sortie.
Le focus clavier ouvre immédiatement la bulle. Échap et l'activation du bouton
la ferment sans déplacer le focus ni consommer l'action du bouton. Le contenu
porte le rôle `Tooltip`, relié au déclencheur par `described_by`.

L'application transmet ses événements à `TooltipState::update`, y compris
Échap reçu ailleurs dans la fenêtre lorsqu'une bulle est ouverte. Elle programme
`advance` à `next_deadline`, puis reconstruit si l'état change. Aucun polling ou
runtime de tâches n'est imposé par le widget. La galerie utilise `TaskSlot` et
annule les timers périmés ; voir `src/pages/tooltip.rs` dans la crate galerie.
Appeler `reset` et annuler le timer lorsque le déclencheur est retiré ou que sa
page est masquée, pour ne pas conserver un ancien survol lors de son retour.

## Surfaces et effets

Chaque page montre une surface opaque sans filtre, une surface translucide
avec `argui_effects::Blur`, et une surface utilisant l'effet WGSL enregistré
`gallery.overlay.prism`. Les motifs derrière les panneaux rendent la différence
visible. Le texte du panneau reste net : ces exemples filtrent l'arrière-plan.

`Popover::layer` et `Tooltip::layer` acceptent une `LayerStyle` complète : filtres
du contenu, filtres d'arrière-plan, masque, ombres et opacité. Aucun preset
`argui-effects` n'est ajouté aux dépendances des widgets.

```rust
let layer = theme.overlay_layer(8.0, 0.0)
    .backdrop(argui_effects::Blur(6.0).filter());
let popover = Popover::new("settings", "Settings", open, trigger, content)
    .layer(layer)
    .build(&theme);
```

Pour voir le flou à travers le fond du panneau, fournir aussi une peinture
translucide avec `.paint(...)`. Une peinture opaque le masque. `.layer(...)`
remplace la couche entière, y compris le flou et les ombres par défaut.

Le flou et la teinte sont indépendants. `Blur(6.0)` règle l'intensité du flou ;
une valeur plus petite conserve davantage de détails derrière le panneau.
La couleur fournie à `QuadStyle::solid(...)` règle la teinte, et son
`.with_alpha(0.60)` règle l'opacité : `0.0` est transparent, `1.0` opaque.
La démo utilise un flou de `6.0` et une opacité de `0.60` pour une surface légère.
Ces réglages fonctionnent aussi bien pour Popover que pour Tooltip.

Les presets et les effets propres à l'application utilisent le même `Filter` :

```rust
let filter = Filter::Effect(EffectInstance::new(
    MY_EFFECT,
    [("strength", EffectValue::F32(0.4))],
));
let layer = theme.overlay_layer(8.0, 0.0).backdrop(filter);
let tooltip = Tooltip::new("help", "Preview your changes", open, trigger)
    .layer(layer)
    .build(&theme);
```

Enregistrer la définition avant le lancement avec
`RendererConfig::effects(argui_effects::registry()?.with_definition(definition)?)`.
Voir [effects.md](effects.md) et l'exemple complet dans
`crates/argui-widget-gallery/src/pages/overlay_effects.rs`.

## Features indépendantes

`argui` et `argui-widgets` ont `default = []`. Sans feature, la façade ne dépend
pas de `argui-widgets`. Chaque widget se sélectionne explicitement :

```toml
argui = { version = "0.1", default-features = false, features = ["widget-popover", "widget-tooltip"] }
# Ou directement :
argui-widgets = { version = "0.1", default-features = false, features = ["popover", "tooltip"] }
```

`argui/widgets-all` et `argui-widgets/all` activent les 43 features de widgets.
Les widgets composés activent leurs briques nécessaires : par exemple
`context-menu` et `menubar` utilisent `menu`. `TextArea` possède sa feature
`textarea`, indépendante de `input`. La façade expose leurs équivalents
`widget-context-menu`, `widget-menubar` et `widget-textarea`.

La vérification réutilisable compile chaque feature seule et importe son type
public depuis une application externe, puis contrôle les configurations vides
et complètes :

```sh
python3 scripts/check-widget-features.py
cargo nextest run -p argui-widgets -p argui-widget-gallery --all-features
```

Pour le contrôle graphique invisible, utiliser
`crates/argui-widget-gallery/tests/pages/overlay_effects.mjs` selon la procédure
de [linux_testing.md](linux_testing.md). Il vérifie les trois surfaces dans les
deux thèmes, la saisie, les fermetures, le survol, les passages rapides, Échap,
Tab et les relations d'accessibilité.
