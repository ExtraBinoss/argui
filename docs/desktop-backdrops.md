# Desktop backdrops

Argui peut exposer le fond du bureau à travers certains éléments d'une fenêtre
et demander au système de le flouter. L'API s'utilise sur un `Element` : sidebar,
topbar, carte ou panneau flottant **dans la fenêtre**. Plusieurs régions sont
possibles, avec des teintes différentes.

## Activation

La feature `desktop-backdrop` est désactivée par défaut dans `argui`,
`argui-runtime`, `argui-platform` et la galerie. `--all-features` l'inclut.
Elle ne dépend pas de `native-popups`, des widgets ni des effets WGSL.

```toml
argui = { path = "../argui", features = ["desktop-backdrop"] }
```

Préparer la fenêtre à sa création, puis choisir les régions au rendu :

```rust
use argui::{
    core::{BackdropMaterial, Color},
    platform::WindowConfig,
    ui::{DesktopBackdrop, Element, length},
};

let window = WindowConfig {
    desktop_backdrop: Some(BackdropMaterial::Sidebar),
    ..WindowConfig::default()
};
let glass = DesktopBackdrop::new(
    Color::srgba(0.08, 0.09, 0.12, 0.72), // teinte + opacité active
    Color::srgb(0.08, 0.09, 0.12),       // repli sans support natif
).inactive_tint(Color::srgba(0.08, 0.09, 0.12, 0.90));

let sidebar = Element::column([Element::text("Navigation")])
    .width(length(260.0))
    .desktop_backdrop(glass);
let content = Element::column([Element::text("Contenu")])
    .grow(1.0)
    .background(Color::WHITE);
let root = Element::row([sidebar, content]);
```

Les ancêtres de la région doivent rester transparents : un fond opaque déjà
peint masque le bureau. Le texte et les contrôles gardent leur propre opacité.
La couleur `fallback` est utilisée sur les surfaces non compatibles, sur le web
et lorsque le contraste élevé désactive la demande de flou. Elle peut être
translucide si l'application accepte explicitement de la transparence sans flou.

Pour activer/désactiver à chaud, ajouter/retirer `.desktop_backdrop(...)` lors du
rendu et notifier le modèle. Retirer la dernière région désactive l'effet natif.
Le choix du matériau est une propriété de la fenêtre : `Glass`, `Sidebar` et
`Header` sont des indications sémantiques. Ils peuvent avoir le même rendu sur
certains systèmes. Une autre fenêtre peut choisir un autre matériau.

`WindowEnvironment::desktop_backdrop_available` indique que le backend natif et
la surface GPU permettent la demande. Le compositeur reste libre de réduire
l'effet selon ses préférences et l'état de la fenêtre. Les erreurs natives sont
signalées par `RuntimeEvent::DesktopBackdropUnavailable` (ou sa variante par
fenêtre) et provoquent un repli sans interrompre l'application.

## Réglages et limites

- Teinte RGBA, opacité et teinte inactive sont propres à chaque élément.
  `.inactive_fallback(color)` règle aussi le fond de la fenêtre inactive sans
  support natif ; par défaut il reprend `fallback`.
- Le fond de repli est explicite ; aucune capture d'écran n'est nécessaire.
- Les régions suivent le layout, le scroll, les transformations et les clips
  arrondis. Les masques natifs sont arrondis vers l'intérieur au pixel logique.
- La valeur alpha règle la quantité de fond visible, pas le rayon du flou.
  Les API publiques utilisées laissent le noyau du flou au système.
- Les filtres `argui-effects` restent utilisables sur le contenu Argui ; ils ne
  lisent pas les pixels des autres applications et ne remplacent pas cet effet.
- Les popovers détachés dans une fenêtre native gardent actuellement leur rendu
  opaque et utilisent `fallback`. Les popovers dans la fenêtre et les fenêtres
  applicatives configurées séparément acceptent cette API.

## Adaptateurs

Tout passe par `argui_platform::desktop_backdrop::NativeBackdrop`. Le runtime
transmet les régions visibles et les préférences ; le choix OS reste dans
`desktop_backdrop/linux.rs`, `windows.rs` et `macos.rs`. La géométrie commune est
indépendante des API OS. Les handles propriétaires restent retenus jusqu'à la
destruction des effets.

| Plateforme | API et comportement |
| --- | --- |
| Linux Wayland | `ext-background-effect-v1`, détection de la capacité Blur ; protocole KDE `org_kde_kwin_blur_manager` si l'extension standard n'est pas proposée. Les régions sont engagées avec le prochain commit de la surface. |
| Linux X11 | Propriété `_KDE_NET_WM_BLUR_BEHIND_REGION`, seulement si le compositeur la publie sur la racine. Rectangles convertis en pixels physiques. |
| Windows | DWM `DWMWA_SYSTEMBACKDROP_TYPE` / `DWMSBT_TRANSIENTWINDOW` : Desktop Acrylic, Windows 11 build 22621 et suivants. Les trois indications de matériau emploient Acrylic. WGPU/DX12 possède le swapchain DirectComposition (`DxgiFromVisual`) pour conserver l'alpha, y compris entre fenêtres partageant un device. Les pixels opaques de l'UI masquent le matériau de la fenêtre. |
| macOS | `NSVisualEffectView`, `BehindWindow`, sous la vue GPU, avec masque de régions. Matériaux Popover, Sidebar et HeaderView ; l'état suit l'activité de la fenêtre. |
| Web / autre / API absente | Couleur de repli. Sur le web, un repli translucide laisse voir la page sous le canvas, jamais le bureau derrière le navigateur. |

Mutter **50.4** installé sur la machine de développement ne propose pas les deux
protocoles Wayland ci-dessus. Le support standard est annoncé dans Mutter
**51.beta**. Sur cette session, le flou de bureau reste donc indisponible ; le
repli fonctionne sans modifier, relancer ou remplacer le compositeur.

Références : [protocole Wayland](https://gitlab.freedesktop.org/wayland/wayland-protocols/-/blob/main/staging/ext-background-effect/ext-background-effect-v1.xml),
[KDE WindowEffects](https://github.com/KDE/kwindowsystem/blob/master/src/platforms/xcb/kwindoweffects.cpp),
[Mutter NEWS](https://github.com/GNOME/mutter/blob/main/NEWS),
[DWM backdrop types](https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/ne-dwmapi-dwm_systembackdrop_type),
[NSVisualEffectView](https://developer.apple.com/documentation/appkit/nsvisualeffectview),
[WGPU DXGI](https://docs.rs/wgpu/30.0.1/wgpu/enum.Dx12SwapchainKind.html).

## Galerie et validation

```sh
cargo run -p argui-widget-gallery --features desktop-backdrop
```

Dans **Appearance → Sidebar appearance**, activer **Desktop glass**. Le panneau
permet d'ajuster **Surface opacity**, **Accent tint**, **Inactive opacity**, et
d'autoriser **Allow transparency without blur**. Les deux switches sont éteints
au lancement. Le contenu principal et la barre du haut restent opaques.

Les tests de peinture et de layout vérifient l'invalidation, les clips, les
transformations, le repli et la conservation de l'opacité du contenu. Le test
opt-in `native_desktop_backdrop` vérifie les requêtes X11 sur un serveur privé,
y compris la disparition de la capacité ; il ne simule pas un rendu flouté.
Le scénario navigateur `tests/app/desktop_backdrop.mjs` vérifie les contrôles et
capture les thèmes clair/sombre et la fenêtre étroite. Suivre
[linux_testing.md](linux_testing.md) pour tous les lancements graphiques.

Les adaptateurs Windows et macOS sont vérifiés par compilation croisée ; leur
aspect natif doit encore être contrôlé sur ces systèmes.
