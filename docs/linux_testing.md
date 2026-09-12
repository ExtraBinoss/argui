# Tests graphiques Linux sans fenêtre sur le bureau

Utiliser `scripts/linux-hidden-display.sh` pour chaque lancement graphique de
test. Il démarre Mutter avec un écran virtuel et un bus D-Bus privé. Le socket
Wayland vit dans un répertoire temporaire de mode 700. `DISPLAY`, le socket
Wayland hérité et le bus de la session utilisateur sont retirés de l'environnement.
Le compositeur s'arrête avec la commande ; le script démonte les éventuels
montages FUSE de ses portails puis supprime son répertoire temporaire.

Ce n'est pas un bureau GNOME supplémentaire auquel on bascule : ses fenêtres
sont invisibles sur les écrans physiques. Ne pas lancer un navigateur via
`xdg-open`, ni connecter l'automatisation au navigateur personnel déjà ouvert.
Un échec du lanceur ne justifie pas de revenir au bureau principal.

Prérequis : `mutter`, `dbus-run-session`, `mountpoint`, `fusermount3` et un pilote
EGL fonctionnel. Vérifier les options avec `mutter --help` : cette procédure a
été exercée le 11 septembre 2026 avec Mutter 50.4 sur Linux/GNOME et GPU Intel.
La documentation [GNOME sur les tests Wayland sans écran](https://gnome.pages.gitlab.gnome.org/pygobject/guide/testing.html)
décrit cette approche ; le script emploie les options de la version installée.

```sh
# Test de l'isolation, sans ouvrir de fenêtre.
./scripts/linux-hidden-display.sh sh -c \
  'test -z "${DISPLAY:-}" && test -S "$XDG_RUNTIME_DIR/$WAYLAND_DISPLAY"'

# Application native, dans son propre compositeur.
cargo build -p argui-widget-gallery --all-features
./scripts/linux-hidden-display.sh timeout 15s target/debug/argui-widget-gallery

# Tests ciblés ; ajouter ARGUI_NATIVE_TESTS=1 uniquement pour les tests natifs opt-in.
./scripts/linux-hidden-display.sh \
  cargo nextest run -p argui-widgets -p argui-widget-gallery --all-features
```

Un timeout volontaire renvoie un statut non nul. Ce lancement natif confirme
au plus le démarrage ; il ne remplace ni l'inspection de captures ni les tests
de comportement. La taille virtuelle se règle avec `ARGUI_TEST_MONITOR`, par
exemple `1920x1200@60` ; la valeur par défaut est `1600x1200@60`.

## X11 natif dans le même environnement privé

`ARGUI_TEST_BACKEND=x11` démarre un **Xwayland avec son propre framebuffer**
dans le compositeur Wayland privé. Le helper `scripts/linux-hidden-x11.sh`
choisit un numéro de display libre avec `-displayfd`, désactive TCP et retire
le socket Wayland de l’environnement de l’application testée. Il termine
uniquement le PID Xwayland qu’il a lui-même lancé. Ne pas lancer ce helper seul.

```sh
# Galerie native X11, sans fenêtre sur le bureau personnel.
ARGUI_TEST_BACKEND=x11 ./scripts/linux-hidden-display.sh \
  timeout 20s target/debug/argui-widget-gallery

# Ce test crée lui-même son compositeur et son serveur X privés.
ARGUI_NATIVE_TESTS=1 cargo nextest run -p argui-runtime --all-features \
  --test native_popups
```

Prérequis supplémentaires : `Xwayland`, Python avec `python-xlib` et Pillow.
Le driver vérifie le PID du serveur et son environnement privé avant d’injecter
les événements XTest. La souris et le clavier synthétiques restent dans ce
serveur X ; aucune capture d’entrée ni autorisation de bureau distant sur la
session utilisateur n’est nécessaire.

Le mode rootless de Xwayland ne fournit pas de capture globale exploitable et
peut demander un portail d’entrée pour XTest. Garder donc le serveur rootful
isolé du helper pour ces tests ; ne pas connecter le driver au Xwayland de la
session. Les commandes de déconnexion, redémarrage du bureau ou arrêt global
de processus ne font pas partie de cette procédure.

Le test `native_popups` vérifie le débordement, `WM_TRANSIENT_FOR` imbriqué,
la saisie et la sélection par raccourci, les clics, le scroll et la perte de
focus. Il enregistre le framebuffer réel dans `target/native-popups/` :
`outside-window.png`, `native-selection.png` et `native-scroll.png`.
Inspecter ces PNG ; le fond noir autour des trois fenêtres est l’espace vide
du serveur privé, pas une capture du bureau de l’utilisateur.

## Galerie WebGPU et captures

Construire et servir la galerie dans un terminal, puis lancer le scénario
dans un autre. Les builds utilisent toujours `--all-features`.

```sh
wasm-pack build crates/argui-widget-gallery --target web --dev \
  --out-dir ../../web/widgets/pkg --all-features
python3 scripts/dev_server.py 8793 --directory web --entry /widgets/
```

Le scénario utilise Puppeteer et un Chromium neuf, sans réutiliser le profil
de l'utilisateur. `PUPPETEER_MODULE` accepte le chemin absolu du module ESM
Puppeteer, ou un module exportant `puppeteer`. Si l'outil est déjà installé,
réutiliser ce module ; sinon l'installer dans un répertoire d'outillage séparé.

```sh
CHROME_PATH=/chemin/vers/chrome \
PUPPETEER_MODULE=/chemin/vers/puppeteer/lib/esm/puppeteer/puppeteer.js \
./scripts/linux-hidden-display.sh \
  node crates/argui-widget-gallery/tests/pages.mjs
```

Configuration vérifiée : Chromium 152.0.7977.54, `headless: false` **dans le
compositeur invisible**, `--ozone-platform=wayland`, `--enable-unsafe-webgpu`,
`--ignore-gpu-blocklist`, `--enable-features=Vulkan`, `--use-angle=vulkan`.
Ces options sont propres à la validation locale. Sur cette machine,
`--use-angle=swiftshader` a produit des captures blanches ou de 1 × 1 pixel,
alors que l'arbre accessible existait : ces captures ne prouvent rien sur le rendu.

Les PNG sont écrits dans `target/widget-interactions/` (`SCREENSHOT_DIR` permet
de changer la destination). `GALLERY_URL` règle l'adresse de la galerie.
Le scénario vérifie Label, Breadcrumb, Pagination, Skeleton, Collapsible,
Menubar, Calendar et Data table à 1220 × 780 et 800 × 720 dans les deux thèmes.
Il exerce la saisie des deux champs Label, les sections repliables, les actions
de File et View, la sélection de date et les passages rapides entre cellules,
puis vérifie Entrée, Espace, les limites de pagination et l'arrêt des Skeleton.

Pour les champs de texte et menus imbriqués, remplacer le scénario par
`node crates/argui-widget-gallery/tests/pages/inputs.mjs` dans la même commande.
Les captures vont dans `target/editor-interactions/`. Ce scénario compare les
raccourcis à des champs HTML de Chromium puis exerce de vrais doubles/triples
clics, le glisser par mots, le clic extérieur et Échap dans les sous-menus.
Il contrôle aussi la stabilité des dimensions et positions des champs pendant
la saisie et les changements de focus.

Pour Popover et Tooltip, utiliser
`node crates/argui-widget-gallery/tests/pages/overlay_effects.mjs` : les captures
dans `target/overlay-interactions/` montrent les versions opaques, floutées et
avec un effet WGSL enregistré, en clair et sombre. Le scénario exerce les
actions des panneaux, leurs fermetures et les infobulles au survol ou au clavier.
Il capture aussi les menus et sous-menus, Select, Date picker, Dialog et Toast
pour contrôler la lisibilité du style commun à toutes les surfaces flottantes.
Il vérifie aussi les infobulles par défaut de Button, leur retour après un clic
sans perte de focus, leur fermeture à l'ouverture d'un panneau, et capture les
deux états du switch dans le popover.
Il clique dans les titres, descriptions et marges des popovers, active les
téléchargements dans **Link options**, puis vérifie que les fermetures par
Échap ou clic extérieur respectent les deux niveaux imbriqués.

Ouvrir les fichiers de capture avec l'outil de lecture d'images de l'agent,
sans visionneuse graphique sur le bureau. Vérifier texte complet, contraste,
espacement, débordements et états désactivés. Compléter un test de layout avec
la préparation des glyphes : des boîtes correctes peuvent masquer un texte coupé.
Argui conserve souvent le focus DOM sur le canvas : vérifier son
`aria-activedescendant` et l'effet de la saisie, pas seulement `document.activeElement`.

Une fenêtre invisible peut cesser de présenter des images : des valeurs de
CPU faibles ne suffisent pas à prouver une bonne performance. Voir les limites
déjà observées dans [widget-gallery-performance.md](widget-gallery-performance.md).
Les tests de lecteurs d'écran natifs restent distincts des assertions DOM.

Pour la navigation et la recherche directe, utiliser
`node crates/argui-widget-gallery/tests/app/navigation.mjs`. Le scénario tape
dès le lancement, depuis un bouton puis depuis le fond de page, vérifie le
premier caractère, l'ordre du texte, le focus, Entrée et Échap, et conserve la
saisie dans Input, Textarea et Select. Les captures clair/sombre vont dans
`target/navigation-interactions/`. Les tests Rust vérifient aussi les couleurs
intermédiaires des libellés de boutons, les inversions rapides et le mouvement réduit.

Pour Liquid Glass, utiliser
`node crates/argui-widget-gallery/tests/pages/liquid_glass.mjs`. Le scénario
vérifie l'enregistrement des icônes, le défilement du contenu derrière la barre
fixe, les trois sections et les réglages. Les captures dans
`target/liquid-glass-web/` incluent la barre avant/après scroll et la page entière.
Les tests Rust comparent aussi la réfraction GPU à la formule du shader source,
la conservation de l'alpha et le layout à plusieurs largeurs.

## Contrôle final

Pour les ajouts du catalogue shadcn, utiliser
`node crates/argui-widget-gallery/tests/pages/catalogue.mjs` avec le même lanceur
et les mêmes variables Chromium/Puppeteer. Le scénario capture les 26 nouvelles
pages et Typography à 1220 × 780 et 800 × 720 en clair/sombre, puis exerce les
champs, toggles, sélecteurs, panneaux, focus, Hover Card et défilement clavier.
Inspecter les PNG de `target/catalogue-interactions/`. Le périmètre de chaque
API est décrit dans [shadcn-lib.md](shadcn-lib.md).

Pour les réglages de flou de bureau, utiliser
`node crates/argui-widget-gallery/tests/app/desktop_backdrop.mjs` avec le même
lanceur et les mêmes variables Chromium/Puppeteer. Captures :
`target/desktop-backdrop/`. Le scénario vérifie aussi que le canvas conserve
l'alpha. L'API et les limites par OS sont dans
[desktop-backdrops.md](desktop-backdrops.md).

Pour capturer une fenêtre **native Wayland**, le helper suivant utilise les
services ScreenCast/RemoteDesktop du **Mutter privé** et son propre PipeWire :

```sh
./scripts/linux-hidden-display.sh timeout --signal=INT --kill-after=3s 55s \
  python3 scripts/linux-wayland-capture.py \
  --output target/desktop-backdrop/wayland \
  --click settings:1110:260 --click glass:884:385 \
  --click transparency:884:664 --click sidebar:1110:260 \
  -- target/debug/argui-widget-gallery
```

Prérequis : Python/PyGObject/Pillow, GStreamer avec `pipewiresrc`, PipeWire et
WirePlumber. Le profil WirePlumber `policy` n'ouvre aucun périphérique audio,
Bluetooth ou caméra. Le socket, la configuration et l'état restent privés.
Le helper termine uniquement ses propres enfants. Il refuse un display non
privé et une capture sans contenu contrasté ; inspecter néanmoins chaque PNG.

Les clics sont exprimés en coordonnées du moniteur virtuel de 1600 × 1200 ;
contrôler d'abord `initial.png` si la taille ou la décoration a changé. Le helper
emploie des mouvements relatifs sur le moniteur privé : les mouvements absolus
du stream ScreenCast ont été ignorés sur Mutter headless 50.4. `--settle` règle
le délai initial (25 secondes par défaut, pour laisser démarrer les portails).
Ces services privés ne donnent aucun accès aux fenêtres du bureau personnel.

Conserver les règles de [code_quality.md](code_quality.md) : tests ciblés pendant
le développement, aucune couverture LLVM concurrente, puis une seule exécution
de `./scripts/quality.sh` quand l'implémentation est terminée, avant le commit.
Le lanceur peut envelopper ce script. Le contrôle mesure toutes les crates du
workspace, même celles non modifiées par le chantier ; ne pas changer les seuils
pour faire passer un contrôle.
