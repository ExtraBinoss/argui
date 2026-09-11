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

## Contrôle final

Conserver les règles de [code_quality.md](code_quality.md) : tests ciblés pendant
le développement, aucune couverture LLVM concurrente, puis une seule exécution
de `./scripts/quality.sh` quand l'implémentation est terminée, avant le commit.
Le lanceur peut envelopper ce script. Conserver le commit de départ du chantier
dans `ARGUI_COVERAGE_BASE` pour mesurer les crates effectivement modifiées par
ce chantier ; ne pas changer les seuils pour faire passer un contrôle.
