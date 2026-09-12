# Popover et Tooltip

La galerie expose **Popover** et **Tooltip** dans sa liste alphabétique, alignée
à gauche. Les interactions suivent les références
[Popover de shadcn/ui](https://ui.shadcn.com/docs/components/base/popover) et
[Tooltip de shadcn/ui](https://ui.shadcn.com/docs/components/base/tooltip).

Popover présente trois panneaux : renommer un projet, régler le partage et
choisir un accent. Le bouton ouvre ou ferme son panneau ; le clic extérieur et
Échap le ferment. Toute la boîte reçoit les clics, y compris le titre, la
description et les marges : ces clics ne ferment pas le panneau.
Le partage propose aussi **Link options**, un second popover qui permet
d'autoriser les téléchargements. Ses clics restent intérieurs au parent.
Échap ou un clic hors du panneau supérieur ferment ce niveau ; le parent
reste ouvert jusqu'à sa propre fermeture. L'exemple traite les événements
du popover enfant avant ceux du parent et réinitialise l'enfant avec le parent.

Tooltip fournit une description au survol ou au focus clavier. `TooltipState`
attend 350 ms au survol (délai configurable), annule les passages rapides,
permet de déplacer la souris dans la bulle et la ferme 100 ms après la sortie.
Le focus clavier ouvre immédiatement la bulle. Échap et l'activation du bouton
la ferment sans déplacer le focus ni consommer l'action du bouton. Un nouveau
survol peut la rouvrir après un clic, même si le bouton conserve le focus. Le contenu
porte le rôle `Tooltip`, relié au déclencheur par `described_by`.

L'application transmet ses événements à `TooltipState::update`, y compris
Échap reçu ailleurs dans la fenêtre lorsqu'une bulle est ouverte. Elle programme
`advance` à `next_deadline`, puis reconstruit si l'état change. Aucun polling ou
runtime de tâches n'est nécessaire à `TooltipState` lui-même. La galerie utilise `TaskSlot` et
annule les timers périmés ; voir `src/pages/tooltip.rs` dans la crate galerie.
Appeler `reset` et annuler le timer lorsque le déclencheur est retiré ou que sa
page est masquée, pour ne pas conserver un ancien survol lors de son retour.

## Infobulles automatiques des boutons

`Button` déclare par défaut une infobulle reprenant son libellé. Installer
`TooltipHost` une fois autour de l'application et activer la feature `tooltip`
pour les afficher automatiquement ; la galerie le fait déjà. `button` reste
utilisable seul, sans dépendance au runtime pour afficher ses boutons.

```rust
let application = TooltipHost::new(MyApplication::default());
let button = Button::new("save", "Save", theme.button())
    .tooltip("Save the current draft")
    .build();
let quiet = Button::new("cancel", "Cancel", theme.outline_button())
    .without_tooltip()
    .build();
```

La feature `tooltip` fournit le widget contrôlé, son état et le host ; elle
active le service de tâches du runtime pour programmer les délais sans polling.
`Element::tooltip(...)` déclare la même aide sur un élément personnalisé.
Les boutons désactivés ou occupés n'ouvrent pas d'infobulle automatique.
Un `Tooltip` contrôlé garde la main sur son déclencheur et ne reçoit pas une
seconde infobulle du host.

Le host ferme l'aide dès qu'un menu, popover ou dialogue apparaît, même si
l'ouverture vient du code. Les couches vides et les notifications n'empêchent
pas les infobulles. `TooltipHost::delay`, `paint` et `layer` personnalisent les
délais, la teinte, les ombres et les effets des infobulles automatiques.
Avec le menu de sélection de texte, utiliser
`TooltipHost::new(SelectionHost::new(application))`.

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
    .paint(PaintStyle::new(QuadStyle::solid(theme.popover.with_alpha(opacity))))
    .layer(layer)
    .build(&theme);
```

Pour voir le flou à travers le fond du panneau, fournir aussi une peinture
translucide avec `.paint(...)`. Une peinture opaque le masque. `.layer(...)`
remplace la couche entière, y compris le flou et les ombres par défaut.

Le flou et la teinte sont indépendants. `Blur(6.0)` règle l'intensité du flou ;
une valeur plus petite conserve davantage de détails derrière le panneau.
La couleur fournie à `QuadStyle::solid(...)` règle la teinte, et son
`.with_alpha(opacity)` règle l'opacité : `0.0` est transparent, `1.0` opaque.
La démo floutée utilise un rayon de `6.0`, une opacité de `0.45` en clair et
de `0.84` en sombre : une teinte blanche trop opaque masque rapidement le flou.
Les panneaux ordinaires
conservent `theme.overlay_blur` (`3.0`) et leur peinture opaque.
Un flou plus faible ne rend pas une surface plus lisible à lui seul : une
opacité suffisante évite que le texte derrière concurrence celui du panneau.
Ces réglages fonctionnent aussi bien pour Popover que pour Tooltip.
Ce sont uniquement des valeurs par défaut : `.layer(...)` peut fournir un
flou plus fort et `.paint(...)` une autre couleur ou opacité, sans limitation
supplémentaire imposée par le widget.

Les panneaux ordinaires conservent un fond opaque, une bordure visible dans
les deux thèmes et une ombre resserrée. Les menus, sous-menus, sélecteurs,
dialogues, notifications et infobulles partagent `theme.popover`,
`theme.popover_border` et `theme.overlay_shadows`. Les démos avec effets
conservent cette bordure ; leur transparence reste une personnalisation locale.

Le switch utilise une piste inactive distincte du fond, réglable avec
`theme.switch_unchecked`, et un curseur inactif réglable avec
`theme.switch_thumb`. À l'état actif, il reprend `theme.primary` et
`theme.primary_foreground` pour conserver le contraste avec l'accent choisi.

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
Tab et les relations d'accessibilité. Il capture aussi les menus et sous-menus,
les sélecteurs, le calendrier flottant, les dialogues et les notifications.
