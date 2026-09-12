# Popovers natifs et adaptation à l’espace disponible

Argui propose une API commune pour présenter un panneau dans sa fenêtre ou
préférer une surface native qui peut en dépasser les bords. Le contenu, le
placement, le défilement et les interactions restent gérés par Argui. Les
adaptateurs OS ne s’occupent que des fenêtres, de leur propriétaire et de la
zone de travail. Un échec de création ou de rendu entraîne un repli interne.

Le flou du bureau derrière une fenêtre est une fonctionnalité distincte : voir
[desktop-backdrops.md](desktop-backdrops.md) pour l'API, les réglages et les
adaptateurs Linux/Windows/macOS.

## Activer la fonctionnalité

Aucune feature n’est activée par défaut. Choisir les widgets séparément ;
`native-popups` n’active aucun widget et `widgets-all` n’active pas le natif.

```toml
[dependencies]
argui = { path = "../argui/crates/argui", features = [
    "widget-popover", "widget-select", "widget-tooltip", "native-popups"
] }
```

La préférence reste utilisable sans la feature native et sur le web : elle
se résout alors dans la fenêtre. Elle ne contient aucun type propre à un OS.

```rust
use argui::ui::{Element, OverlaySurface};
use argui::widgets::Popover;

let panel = Popover::new(
    "sharing",
    "Sharing settings",
    open,
    trigger,
    Element::text("Options de partage"),
)
.surface(OverlaySurface::PreferNative)
.size(320.0, 400.0)
.build(theme);
```

`OverlaySurface::InWindow` impose la présentation interne.
`OverlaySurface::PreferNative` demande une surface native avec repli automatique.
Les builders `Popover`, `Tooltip`, `TooltipHost`, `Select`, `Menu`, `ContextMenu`
et `DatePicker` exposent tous `.surface(...)`. Pour `Menubar`, configurer les
`Menu` fournis au widget. Pour un composant personnalisé, appliquer
`.portal_surface(...)` **après** `.anchored_portal(...)` ou `.rect_portal(...)` ;
on peut aussi construire directement `Portal::new(...).surface(...)`.

En l’absence de préférence explicite, un portail hérite du portail ancêtre le
plus proche ; à la racine, la valeur est `InWindow`. Un sous-popover hérite donc
de `PreferNative`. Un enfant explicitement `InWindow` reste dans la surface de
son parent natif, avec ses contraintes et son scroll, sans créer de fenêtre.
Seuls les portails ancrés à un élément ou un rectangle sont candidats au natif.
Les couches modales ou plein écran restent dans leur fenêtre.

Les pages **Popover** et **Tooltip** de la galerie proposent le switch
**Allow outside this window**. Construire la galerie avec `--all-features`
pour inclure le backend natif ; la même démo dans le navigateur exerce le repli.

## Interface et fichiers par OS

Le contrat privé `PopupBackend` et l’interface runtime `NativePopup` vivent dans
[argui-platform/src/popup.rs](../crates/argui-platform/src/popup.rs).
La sélection de l’adaptateur se fait à la compilation avec `cfg(target_os)`.

| Backend de la fenêtre | Présentation actuelle | Adaptateur |
| --- | --- | --- |
| Linux X11 / Xwayland, hôte winit | Fenêtre popup native | [linux.rs](../crates/argui-platform/src/popup/linux.rs) |
| Windows, hôte winit | Fenêtre possédée native | [windows.rs](../crates/argui-platform/src/popup/windows.rs) |
| macOS, hôte winit | Fenêtre AppKit rattachée | [macos.rs](../crates/argui-platform/src/popup/macos.rs) |
| Linux Wayland ou hôte GTK | Repli `InWindow` | Refus explicite du backend |
| Web, autres OS, feature absente | `InWindow` | Aucun appel aux API natives |

**X11.** L’adaptateur utilise `override_redirect`, les types EWMH `TOOLTIP`,
`POPUP_MENU` et `COMBO`, puis `WM_TRANSIENT_FOR` pour rattacher chaque surface
à son parent réel, y compris les sous-popovers. La zone `_NET_WORKAREA` est
croisée avec l’écran du parent ; un serveur sans gestionnaire de fenêtres
utilise les limites de cet écran. `x11rb`, dépendance optionnelle de
`argui-platform`, porte ces appels.
[Spécification EWMH](https://specifications.freedesktop.org/wm/latest/ar01s05.html).

**Windows.** La fenêtre a un propriétaire winit, le style `WS_POPUP` et
`WS_EX_TOOLWINDOW`, sans `WS_CHILD`. Les infobulles ajoutent `WS_EX_NOACTIVATE`.
`MonitorFromWindow` et `GetMonitorInfoW.rcWork` fournissent les limites ; le
calcul du placement reste commun à tous les OS.
[Fenêtres possédées](https://learn.microsoft.com/en-us/windows/win32/winmsg/window-features#owned-windows),
[styles étendus](https://learn.microsoft.com/en-us/windows/win32/winmsg/extended-window-styles),
[GetMonitorInfoW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getmonitorinfow).

**macOS.** Le backend rattache une `NSWindow` sans décoration avec
`addChildWindow(..., Above)` et la détache avant destruction. Il utilise
`NSScreen.visibleFrame` en convertissant les coordonnées AppKit vers les pixels
physiques winit. Les appels AppKit s’exécutent sur le thread de la boucle OS.
[addChildWindow](https://developer.apple.com/documentation/appkit/nswindow/addchildwindow(_:ordered:)),
[visibleFrame](https://developer.apple.com/documentation/appkit/nsscreen/visibleframe).

**Wayland.** `winit 0.30.13` n’expose pas la création de `xdg_popup` ni le
contexte d’entrée nécessaire à son grab. L’adaptateur refuse donc la présentation
native sur ce backend. L’hôte GTK 3 du dépôt ne fournit pas encore cette
capacité non plus. Avec `webview`, le lancement Linux choisit cet hôte sur
Wayland ; sur X11, il utilise winit. Une future intégration Wayland devra porter
`xdg_positioner`, les configurations du compositeur et `popup_done` derrière
la même interface, sans modifier les widgets.
[API winit 0.30.13](https://docs.rs/winit/0.30.13/winit/window/struct.WindowAttributes.html),
[protocole xdg-shell](https://gitlab.freedesktop.org/wayland/wayland-protocols/-/blob/main/stable/xdg-shell/xdg-shell.xml).

## Placement et défilement partagés

Le layout conserve la taille souhaitée avant de contraindre le portail.
`FloatingPlacement` essaie les côtés autorisés, décale le rectangle et réduit
sa taille si nécessaire. La référence est le viewport en mode interne et la
zone de travail de l’écran du parent en mode natif. Le contenu conserve ses
limites explicites de largeur et hauteur.

Le runtime accepte la géométrie native en coordonnées logiques de l’arbre,
arrondie aux pixels physiques. Il effectue au plus deux passes de mise à jour
pour résoudre aussi les ancres imbriquées. La taille réellement reçue de l’OS
est prise en compte sans alterner avec la taille demandée aux DPI fractionnaires.
Le déplacement, le redimensionnement et le changement d’échelle du parent
invalident sa géométrie ; les mises à jour de l’arbre recalculent les ancres.
Une ancre absente ou masquée ne produit ni panneau ni cible
cliquable. La zone de travail choisie est celle de l’écran du parent, pas une
recherche du meilleur écran voisin.

`Popover`, `Select` et `Tooltip` utilisent `Overflow::Auto` sur les deux axes
et contiennent le scroll à leurs limites. Les menus et date pickers réutilisent
ces conteneurs. Une infobulle longue conserve sa description accessible complète
et reste survolable pour utiliser la molette. Un portail brut reste libre :
l’application doit lui donner son `ScrollConfig` et sa politique `Overflow`.

## État, événements et rendu

L’arbre logique et les `NodeId` restent uniques. La promotion en fenêtre native
ne recrée ni modèle, ni éditeur, ni sélection. Le layout sépare les listes de
dessin et traduit leurs commandes et clips dans les coordonnées de chaque
surface. Les glyphes, assets et le device GPU restent partagés. Les entrées de
chaque fenêtre sont reconverties vers le même arbre ; l’IME utilise la position
du curseur dans la fenêtre qui héberge l’éditeur.

Un clic dans un sous-popover reste intérieur à la chaîne. Les comportements
existants gèrent Échap et les clics extérieurs. Un transfert de focus entre
parent et enfant reste dans le même groupe. Une fermeture demandée par l’OS
ou une perte de focus du groupe émet `UiEventKind::DismissRequested`, via
`EventType::Dismiss`. Les composants contrôlés doivent écouter cet événement,
comme `PointerOutside`, puis appliquer leur `PopoverBehavior`, `SelectBehavior`
ou `TooltipState`. `TooltipHost` le fait automatiquement. Les enfants natifs
sont détruits avant leur parent.

En cas de refus ou de perte de surface GPU, le runtime restaure le rendu interne
et émet `RuntimeEvent::PopupFallback { node, reason }` (également disponible sur
`WindowRuntimeEvent`). Il ne retente pas la création à chaque frame : fermer
puis rouvrir le panneau permet une nouvelle tentative. Sans la feature ou sur
le web, il n’y a pas de tentative native ni de diagnostic de refus.

Les effets de contenu enregistrés restent disponibles dans le renderer natif.
Le **backdrop blur du bureau n’est pas implémenté** : une surface séparée ne
possède pas les pixels derrière la fenêtre. Le natif utilise actuellement un
fond opaque dérivé de la couleur du panneau, ou de `RendererConfig.clear_color`
si sa peinture n’a pas de fond uni. Les ombres Argui débordantes sont limitées
à la surface ; les coins extérieurs restent opaques. Le mode interne conserve
le flou et le rendu arrondi existants. L’arbre sémantique reste commun, mais un
pont d’accessibilité OS propre à chaque surface native reste à valider.

## Validation

Les tests Rust couvrent l’héritage et les overrides, la conversion DPI, le
placement contraint, les ancres absentes, la séparation des commandes et clips,
les cibles interactives hors viewport, le scroll et les demandes de fermeture.

Le test Linux `native_popups` lance son propre Xwayland dans le compositeur
privé décrit dans [linux_testing.md](linux_testing.md). Il vérifie la géométrie
au-delà de la fenêtre, le rattachement parent/enfant, la saisie, Ctrl+Maj+←,
les clics dans l’enfant et la molette, et enregistre de vraies captures de ce
serveur X. Le scénario WebGPU `overlay_effects.mjs` vérifie aussi la préférence
native et son repli pour Popover, sous-popover et Tooltip en clair et sombre.

Les adaptateurs Windows et macOS ont été vérifiés par compilation croisée de
`argui-platform --all-features`. Leurs interactions, l’IME et les configurations
multi-écrans doivent encore être exercés sur les OS concernés ; une compilation
ne constitue pas une validation graphique. Le natif Wayland reste indisponible.
